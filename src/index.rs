//! Incremental indexing: gitignore-aware walk, content-addressed change
//! detection, parallel parse, single-writer commit.

use crate::{lang, parse, store};
use anyhow::Result;
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

pub fn run(root: &Path, st: &mut store::Store, force: bool) -> Result<Stats> {
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

    let churn = git_churn(root);

    let mut reparsed = 0usize;
    let pruned;
    {
        let tx = st.conn.transaction()?;
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
                store::insert_symbol(&tx, id, &d.name, &d.kind, d.start_line, d.end_line, &d.sig)?;
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
        pruned = store::prune(&tx, generation)?;
        tx.commit()?;
    }

    st.set_meta("generation", &generation.to_string())?;
    st.set_meta("root", &root.to_string_lossy())?;
    if let Some(head) = git_head(root) {
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

/// Commits touching each file in the recent window. A cheap, honest recency
/// prior: what you have been editing is what you are about to ask about.
fn git_churn(root: &Path) -> HashMap<String, u32> {
    let mut out = HashMap::new();
    let Ok(o) = std::process::Command::new("git")
        .args([
            "-C",
            &root.to_string_lossy(),
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
    let o = std::process::Command::new("git")
        .args([
            "-C",
            &root.to_string_lossy(),
            "rev-parse",
            "--short",
            "HEAD",
        ])
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
