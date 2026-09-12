//! Incremental indexing: gitignore-aware walk, content-addressed change
//! detection, parallel parse, single-writer commit.

use crate::{lang, parse, store};
use anyhow::{Context, Result};
use rayon::prelude::*;
use rusqlite::OptionalExtension;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

const MAX_BYTES: u64 = 1 << 20;

pub struct Stats {
    pub scanned: usize,
    pub reparsed: usize,
    pub pruned: usize,
    pub elapsed_ms: u128,
}

struct Unit {
    path: String,
    lang: lang::Lang,
    hash: String,
    size: u64,
    lines: u32,
    mtime: i64,
    parsed: Option<parse::Parsed>,
    chunks: Vec<parse::ChunkDraft>,
}

pub fn run(
    scope: &Path,
    git_root: Option<&Path>,
    st: &mut store::Store,
    force: bool,
) -> Result<Stats> {
    let root = scope;
    let t0 = Instant::now();
    let generation = st
        .get_meta("generation")?
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0)
        + 1;

    let mut known: HashMap<String, String> = HashMap::new();
    {
        let mut q = st.conn.prepare("SELECT path, hash FROM files")?;
        for row in q.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
            let (p, h) = row?;
            known.insert(p, h);
        }
    }

    let mut candidates: Vec<(String, lang::Lang, u64, i64)> = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .parents(true)
        .build();
    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Some(l) = lang::detect(entry.path()) else {
            continue;
        };
        let Ok(md) = entry.metadata() else { continue };
        if md.len() > MAX_BYTES || md.len() == 0 {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(root) else {
            continue;
        };
        let mtime = md
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        candidates.push((rel.to_string_lossy().to_string(), l, md.len(), mtime));
    }

    let scanned = candidates.len();

    let units: Vec<Unit> = candidates
        .par_iter()
        .filter_map(|(rel, l, size, mtime)| {
            let full = root.join(rel);
            let src = std::fs::read_to_string(&full).ok()?;
            let hash = blake3::hash(src.as_bytes()).to_hex().to_string();
            let unchanged = !force && known.get(rel).is_some_and(|h| *h == hash);
            let lines = src.lines().count() as u32;
            if unchanged {
                return Some(Unit {
                    path: rel.clone(),
                    lang: *l,
                    hash,
                    size: *size,
                    lines,
                    mtime: *mtime,
                    parsed: None,
                    chunks: Vec::new(),
                });
            }
            let parsed = parse::parse(*l, &src).unwrap_or_default();
            let chunks = parse::chunks(rel, &src, &parsed);
            Some(Unit {
                path: rel.clone(),
                lang: *l,
                hash,
                size: *size,
                lines,
                mtime: *mtime,
                parsed: Some(parsed),
                chunks,
            })
        })
        .collect();

    // Churn comes from the repository even when only a subtree is indexed, so its
    // repository-relative paths have to be rebased onto the scope.
    let churn = match git_root {
        Some(g) => rebase_churn(git_churn(g), scope, g),
        None => HashMap::new(),
    };

    let mut reparsed = 0usize;
    let pruned;
    {
        let db = st.path.clone();
        // One context for the whole write, because the failure can surface from any
        // statement inside it, not just from BEGIN.
        let mut write = |conn: &mut rusqlite::Connection| -> Result<usize> {
            let tx = store::begin_write(conn)?;
            for u in &units {
                let facts = store::FileFacts {
                    path: &u.path,
                    lang: u.lang.name(),
                    hash: &u.hash,
                    size: u.size,
                    lines: u.lines,
                    mtime: u.mtime,
                };
                let (id, changed) = store::upsert_file(&tx, &facts, generation, force)?;
                let Some(parsed) = &u.parsed else { continue };
                if !changed {
                    continue;
                }
                reparsed += 1;
                store::clear_payload(&tx, id)?;
                for d in &parsed.defs {
                    store::insert_symbol(
                        &tx,
                        id,
                        &d.name,
                        &d.kind,
                        d.start_line,
                        d.end_line,
                        &d.sig,
                    )?;
                }
                for (name, n) in &parsed.refs {
                    store::insert_ref(&tx, id, name, *n)?;
                }
                for c in &u.chunks {
                    store::insert_chunk(&tx, id, c)?;
                }
            }
            for (path, n) in &churn {
                store::set_churn(&tx, path, *n)?;
            }
            let pruned = store::prune(&tx, generation)?;
            tx.commit()?;
            Ok(pruned)
        };
        pruned = write(&mut st.conn)
            .with_context(|| format!("writing the index at {}", db.display()))?;
    }

    st.set_meta("generation", &generation.to_string())?;
    st.set_meta("root", &scope.to_string_lossy())?;
    if let Some(head) = git_root.and_then(git_head) {
        st.set_meta("head", &head)?;
    }
    st.conn.execute_batch("PRAGMA optimize;")?;

    Ok(Stats {
        scanned,
        reparsed,
        pruned,
        elapsed_ms: t0.elapsed().as_millis(),
    })
}

/// Move repository-relative churn paths onto a scope that may be a subtree.
///
/// A path outside the scope is dropped. A scope that is not inside the repository at
/// all yields nothing: applying every repository path to an unrelated directory would
/// attribute churn to files that do not exist there.
pub fn rebase_churn(
    churn: HashMap<String, u32>,
    scope: &Path,
    git_root: &Path,
) -> HashMap<String, u32> {
    let prefix = match scope.strip_prefix(git_root) {
        Ok(rel) if rel.as_os_str().is_empty() => return churn,
        Ok(rel) => rel.to_path_buf(),
        Err(_) => return HashMap::new(),
    };
    churn
        .into_iter()
        .filter_map(|(path, n)| {
            Path::new(&path)
                .strip_prefix(&prefix)
                .ok()
                .map(|r| (r.to_string_lossy().to_string(), n))
        })
        .collect()
}

/// Commits touching each file in the recent window. A cheap, honest recency
/// prior: what you have been editing is what you are about to ask about.
/// A `git` invocation pinned to `root`.
///
/// Git hooks export `GIT_DIR`, and it takes precedence over `-C`. Without scrubbing
/// the inherited environment, running brainiac from inside a hook would read history
/// from whatever repository invoked the hook rather than the one being indexed.
fn git_at(root: &Path) -> std::process::Command {
    let mut c = std::process::Command::new("git");
    for var in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_PREFIX",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        c.env_remove(var);
    }
    c.arg("-C").arg(root);
    c
}

fn git_churn(root: &Path) -> HashMap<String, u32> {
    let mut out = HashMap::new();
    let Ok(o) = git_at(root)
        .args([
            "log",
            "--since=120.days",
            "--name-only",
            "--pretty=format:",
            "--no-renames",
        ])
        .output()
    else {
        return out;
    };
    if !o.status.success() {
        return out;
    }
    for line in String::from_utf8_lossy(&o.stdout).lines() {
        let line = line.trim();
        if !line.is_empty() {
            *out.entry(line.to_string()).or_insert(0u32) += 1;
        }
    }
    out
}

fn git_head(root: &Path) -> Option<String> {
    let o = git_at(root)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    o.status
        .success()
        .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// True when the index has never been built for this repo.
pub fn is_empty(st: &store::Store) -> bool {
    st.conn
        .query_row("SELECT 1 FROM files LIMIT 1", [], |_| Ok(()))
        .optional()
        .map(|o| o.is_none())
        .unwrap_or(true)
}
