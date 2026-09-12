//! What to index, and where its index lives.
//!
//! Two paths matter and they are not the same. The **scope** is the directory
//! actually walked. The **git root** is the enclosing repository, which is where
//! churn and `HEAD` come from even when only a subtree is being indexed.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Repo {
    /// The directory to index: the one named on the command line, or the git root.
    pub scope: PathBuf,
    /// The enclosing git repository, if there is one.
    pub git_root: Option<PathBuf>,
    pub db: PathBuf,
}

/// `start` is the `-C` argument. Given explicitly it *is* the scope; absent, the
/// scope is the enclosing repository, so running from `src/` still sees the repo.
pub fn discover(start: Option<&Path>) -> Result<Repo> {
    discover_from(start, &std::env::current_dir()?)
}

/// The body of [`discover`] with the working directory passed in, so the
/// no-argument branch is testable without mutating process-global state.
pub fn discover_from(start: Option<&Path>, cwd: &Path) -> Result<Repo> {
    let explicit = start.is_some();
    let from = match start {
        Some(p) => p.to_path_buf(),
        None => cwd.to_path_buf(),
    };
    let from = std::fs::canonicalize(&from)
        .with_context(|| format!("no such path: {}", from.display()))?;
    if !from.is_dir() {
        anyhow::bail!(
            "{} is not a directory — pass the directory to index, such as {}",
            from.display(),
            from.parent().unwrap_or(Path::new(".")).display()
        );
    }

    let git_root = find_git_root(&from);
    let scope = if explicit {
        from
    } else {
        git_root.clone().unwrap_or(from)
    };

    Ok(Repo {
        db: index_path(&scope),
        scope,
        git_root,
    })
}

fn find_git_root(from: &Path) -> Option<PathBuf> {
    let mut cur = Some(from);
    while let Some(dir) = cur {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    None
}

/// Keyed on the scope, so two subtrees of one repository do not share an index.
fn index_path(scope: &Path) -> PathBuf {
    let key = blake3::hash(scope.to_string_lossy().as_bytes()).to_hex();
    let slug = scope
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".into());
    let base = directories::ProjectDirs::from("", "", "brainiac")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| scope.join(".brainiac"));
    base.join(format!("{slug}-{}.db", &key.as_str()[..12]))
}
