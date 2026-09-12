//! SQLite index. One file per repo, WAL, FTS5 over chunk bodies with the text
//! held once in `chunks` and mirrored into an external-content FTS index.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::path::Path;

pub struct Store {
    pub conn: Connection,
}

#[derive(Debug, Clone)]
pub struct FileRow {
    pub id: i64,
    pub path: String,
    pub lang: String,
    pub lines: u32,
    pub churn: u32,
}

pub fn path_prior(path: &str) -> f64 {
    let p = path.to_ascii_lowercase();
    let has = |n: &str| p.contains(n);
    let mut w = 1.0f64;
    if has("/vendor/") || p.starts_with("vendor/") || has("/third_party/") || has("node_modules/") {
        w *= 0.2;
    }
    if has("/generated/") || has(".generated.") || has(".min.") || has("/dist/") || has("/build/") {
        w *= 0.25;
    }
    // Archived or version-pinned copies: the current one is what you want.
    if has("/versions/") || has("/archive/") || has("/legacy/") || has("/deprecated/") {
        w *= 0.25;
    }
    if has("/examples/") || has("/fixtures/") || has("/testdata/") || has("/snapshots/") {
        w *= 0.5;
    }
    if p.starts_with("tests/")
        || has("/tests/")
        || has("/test/")
        || has("_test.")
        || has(".test.")
        || has(".spec.")
        || has("_spec.")
    {
        w *= 0.7;
    }
    w
}

#[derive(Debug, Clone)]
pub struct SymbolRow {
    pub file_id: i64,
    pub name: String,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
    pub sig: String,
}

#[derive(Debug, Clone)]
pub struct ChunkRow {
    pub id: i64,
    pub file_id: i64,
    pub name: String,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
    pub body: String,
    /// Content hash of the body, used to collapse copy-pasted and vendored
    /// duplicates that would otherwise fill a result page with the same text.
    pub bhash: String,
}

/// Bumped whenever the tables change shape. A mismatch rebuilds from scratch,
/// which costs seconds and removes a whole class of migration bugs.
const SCHEMA_VERSION: &str = "3";

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS meta (k TEXT PRIMARY KEY, v TEXT NOT NULL);

CREATE TABLE IF NOT EXISTS files (
  id    INTEGER PRIMARY KEY,
  path  TEXT NOT NULL UNIQUE,
  lang  TEXT NOT NULL,
  hash  TEXT NOT NULL,
  size  INTEGER NOT NULL,
  lines INTEGER NOT NULL,
  mtime INTEGER NOT NULL,
  churn INTEGER NOT NULL DEFAULT 0,
  gen   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS symbols (
  id         INTEGER PRIMARY KEY,
  file_id    INTEGER NOT NULL,
  name       TEXT NOT NULL,
  kind       TEXT NOT NULL,
  start_line INTEGER NOT NULL,
  end_line   INTEGER NOT NULL,
  sig        TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS symbols_name ON symbols(name);
CREATE INDEX IF NOT EXISTS symbols_file ON symbols(file_id);

CREATE TABLE IF NOT EXISTS refs (
  file_id INTEGER NOT NULL,
  name    TEXT NOT NULL,
  n       INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS refs_name ON refs(name);
CREATE INDEX IF NOT EXISTS refs_file ON refs(file_id);

CREATE TABLE IF NOT EXISTS chunks (
  id         INTEGER PRIMARY KEY,
  file_id    INTEGER NOT NULL,
  name       TEXT NOT NULL,
  kind       TEXT NOT NULL,
  start_line INTEGER NOT NULL,
  end_line   INTEGER NOT NULL,
  body       TEXT NOT NULL,
  bhash      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS chunks_file ON chunks(file_id);
CREATE INDEX IF NOT EXISTS chunks_bhash ON chunks(bhash);

CREATE VIRTUAL TABLE IF NOT EXISTS chunk_fts USING fts5(
  name, body,
  content='chunks', content_rowid='id',
  tokenize="unicode61 tokenchars '_'"
);
"#;

impl Store {
    pub fn open(path: &Path) -> Result<Store> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("opening index at {}", path.display()))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (k TEXT PRIMARY KEY, v TEXT NOT NULL);",
        )?;
        let found: Option<String> = conn
            .query_row("SELECT v FROM meta WHERE k='schema'", [], |r| r.get(0))
            .optional()?;
        if found.as_deref() != Some(SCHEMA_VERSION) {
            conn.execute_batch(
                "DROP TABLE IF EXISTS chunk_fts;
                 DROP TABLE IF EXISTS chunks;
                 DROP TABLE IF EXISTS symbols;
                 DROP TABLE IF EXISTS refs;
                 DROP TABLE IF EXISTS files;
                 DELETE FROM meta;",
            )?;
        }
        conn.execute_batch(SCHEMA)?;
        let st = Store { conn };
        st.set_meta("schema", SCHEMA_VERSION)?;
        Ok(st)
    }

    pub fn set_meta(&self, k: &str, v: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta(k,v) VALUES(?1,?2) ON CONFLICT(k) DO UPDATE SET v=excluded.v",
            params![k, v],
        )?;
        Ok(())
    }

    pub fn get_meta(&self, k: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT v FROM meta WHERE k=?1", params![k], |r| r.get(0))
            .optional()?)
    }
}

/// A file as the walker found it on disk.
pub struct FileFacts<'a> {
    pub path: &'a str,
    pub lang: &'a str,
    pub hash: &'a str,
    pub size: u64,
    pub lines: u32,
    pub mtime: i64,
}

/// Returns (file_id, needs_reparse).
pub fn upsert_file(
    tx: &Transaction,
    f: &FileFacts,
    generation: i64,
    force: bool,
) -> Result<(i64, bool)> {
    let (path, lang, hash, size, lines, mtime) = (f.path, f.lang, f.hash, f.size, f.lines, f.mtime);
    let existing: Option<(i64, String)> = tx
        .query_row(
            "SELECT id, hash FROM files WHERE path=?1",
            params![path],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    match existing {
        Some((id, old_hash)) => {
            tx.execute(
                "UPDATE files SET lang=?2, hash=?3, size=?4, lines=?5, mtime=?6, gen=?7 WHERE id=?1",
                params![id, lang, hash, size as i64, lines, mtime, generation],
            )?;
            Ok((id, force || old_hash != hash))
        }
        None => {
            tx.execute(
                "INSERT INTO files(path,lang,hash,size,lines,mtime,gen) VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![path, lang, hash, size as i64, lines, mtime, generation],
            )?;
            Ok((tx.last_insert_rowid(), true))
        }
    }
}

/// Drop everything derived from a file, keeping the FTS index in sync. External
/// content tables need the old column values handed back on delete.
pub fn clear_payload(tx: &Transaction, file_id: i64) -> Result<()> {
    {
        let mut q = tx.prepare("SELECT id, name, body FROM chunks WHERE file_id=?1")?;
        let rows: Vec<(i64, String, String)> = q
            .query_map(params![file_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let mut del = tx.prepare(
            "INSERT INTO chunk_fts(chunk_fts, rowid, name, body) VALUES('delete', ?1, ?2, ?3)",
        )?;
        for (id, name, body) in rows {
            del.execute(params![id, name, body])?;
        }
    }
    tx.execute("DELETE FROM chunks WHERE file_id=?1", params![file_id])?;
    tx.execute("DELETE FROM symbols WHERE file_id=?1", params![file_id])?;
    tx.execute("DELETE FROM refs WHERE file_id=?1", params![file_id])?;
    Ok(())
}

pub fn insert_symbol(
    tx: &Transaction,
    file_id: i64,
    name: &str,
    kind: &str,
    start: u32,
    end: u32,
    sig: &str,
) -> Result<()> {
    tx.execute(
        "INSERT INTO symbols(file_id,name,kind,start_line,end_line,sig) VALUES(?1,?2,?3,?4,?5,?6)",
        params![file_id, name, kind, start, end, sig],
    )?;
    Ok(())
}

pub fn insert_ref(tx: &Transaction, file_id: i64, name: &str, n: u32) -> Result<()> {
    tx.execute(
        "INSERT INTO refs(file_id,name,n) VALUES(?1,?2,?3)",
        params![file_id, name, n],
    )?;
    Ok(())
}

pub fn insert_chunk(tx: &Transaction, file_id: i64, c: &crate::parse::ChunkDraft) -> Result<()> {
    // Content hash collapses vendored and version-pinned copies at query time.
    let bhash = blake3::hash(c.body.as_bytes()).to_hex()[..16].to_string();
    tx.execute(
        "INSERT INTO chunks(file_id,name,kind,start_line,end_line,body,bhash)
         VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![
            file_id,
            c.name,
            c.kind,
            c.start_line,
            c.end_line,
            c.body,
            bhash
        ],
    )?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO chunk_fts(rowid, name, body) VALUES(?1,?2,?3)",
        params![id, c.name, c.body],
    )?;
    Ok(())
}

pub fn set_churn(tx: &Transaction, path: &str, churn: u32) -> Result<()> {
    tx.execute(
        "UPDATE files SET churn=?2 WHERE path=?1",
        params![path, churn],
    )?;
    Ok(())
}

/// Remove files not visited in this generation.
pub fn prune(tx: &Transaction, generation: i64) -> Result<usize> {
    let stale: Vec<i64> = {
        let mut q = tx.prepare("SELECT id FROM files WHERE gen<>?1")?;
        q.query_map(params![generation], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    for id in &stale {
        clear_payload(tx, *id)?;
        tx.execute("DELETE FROM files WHERE id=?1", params![id])?;
    }
    Ok(stale.len())
}

pub fn all_files(conn: &Connection) -> Result<Vec<FileRow>> {
    let mut q = conn.prepare("SELECT id,path,lang,lines,churn FROM files ORDER BY path")?;
    let v = q
        .query_map([], |r| {
            Ok(FileRow {
                id: r.get(0)?,
                path: r.get(1)?,
                lang: r.get(2)?,
                lines: r.get(3)?,
                churn: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

pub fn file_by_id(conn: &Connection, id: i64) -> Result<Option<FileRow>> {
    Ok(conn
        .query_row(
            "SELECT id,path,lang,lines,churn FROM files WHERE id=?1",
            params![id],
            |r| {
                Ok(FileRow {
                    id: r.get(0)?,
                    path: r.get(1)?,
                    lang: r.get(2)?,
                    lines: r.get(3)?,
                    churn: r.get(4)?,
                })
            },
        )
        .optional()?)
}

pub fn symbols_of(conn: &Connection, file_id: i64) -> Result<Vec<SymbolRow>> {
    let mut q = conn.prepare(
        "SELECT file_id,name,kind,start_line,end_line,sig FROM symbols WHERE file_id=?1 ORDER BY start_line",
    )?;
    let v = q
        .query_map(params![file_id], map_symbol)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

pub fn symbols_named(conn: &Connection, name: &str) -> Result<Vec<SymbolRow>> {
    let mut q = conn.prepare(
        "SELECT file_id,name,kind,start_line,end_line,sig FROM symbols WHERE name=?1 LIMIT 64",
    )?;
    let v = q
        .query_map(params![name], map_symbol)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

fn map_symbol(r: &rusqlite::Row) -> rusqlite::Result<SymbolRow> {
    Ok(SymbolRow {
        file_id: r.get(0)?,
        name: r.get(1)?,
        kind: r.get(2)?,
        start_line: r.get(3)?,
        end_line: r.get(4)?,
        sig: r.get(5)?,
    })
}

pub fn chunk_by_id(conn: &Connection, id: i64) -> Result<Option<ChunkRow>> {
    Ok(conn
        .query_row(
            "SELECT id,file_id,name,kind,start_line,end_line,body,bhash FROM chunks WHERE id=?1",
            params![id],
            |r| {
                Ok(ChunkRow {
                    id: r.get(0)?,
                    file_id: r.get(1)?,
                    name: r.get(2)?,
                    kind: r.get(3)?,
                    start_line: r.get(4)?,
                    end_line: r.get(5)?,
                    body: r.get(6)?,
                    bhash: r.get(7)?,
                })
            },
        )
        .optional()?)
}

pub fn counts(conn: &Connection) -> Result<(i64, i64, i64, i64)> {
    let one = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    Ok((
        one("SELECT count(*) FROM files")?,
        one("SELECT count(*) FROM symbols")?,
        one("SELECT count(*) FROM chunks")?,
        one("SELECT coalesce(sum(lines),0) FROM files")?,
    ))
}
