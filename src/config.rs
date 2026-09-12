//! Repo root discovery and index location. The index lives outside the repo so
//! nothing has to be gitignored.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Repo {
    pub root: PathBuf,
    pub db: PathBuf,
}

pub fn discover(start: Option<&Path>) -> Result<Repo> {
    let start = match start {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir()?,
    };
    let start = std::fs::canonicalize(&start)
        .with_context(|| format!("no such path: {}", start.display()))?;

    let mut cur = start.as_path();
    let mut root = start.clone();
    loop {
        if cur.join(".git").exists() {
            root = cur.to_path_buf();
            break;
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }

    let key = blake3::hash(root.to_string_lossy().as_bytes()).to_hex();
    let slug = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".into());
    let base = directories::ProjectDirs::from("", "", "brainiac")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| root.join(".brainiac"));
    let db = base.join(format!("{slug}-{}.db", &key.as_str()[..12]));
    Ok(Repo { root, db })
}
