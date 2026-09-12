//! Language registry: extension -> tree-sitter grammar + tags query.

use tree_sitter::Language;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Tsx,
    Go,
    /// Indexed for text search, but no symbol graph.
    Prose,
}

impl Lang {
    pub fn name(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::TypeScript => "typescript",
            Lang::Tsx => "tsx",
            Lang::Go => "go",
            Lang::Prose => "prose",
        }
    }

    pub fn from_name(s: &str) -> Lang {
        match s {
            "rust" => Lang::Rust,
            "python" => Lang::Python,
            "javascript" => Lang::JavaScript,
            "typescript" => Lang::TypeScript,
            "tsx" => Lang::Tsx,
            "go" => Lang::Go,
            _ => Lang::Prose,
        }
    }

    /// Fence hint for markdown output.
    pub fn fence(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::TypeScript => "typescript",
            Lang::Tsx => "tsx",
            Lang::Go => "go",
            Lang::Prose => "",
        }
    }

    /// Grammar plus the tag query to run against it.
    ///
    /// TypeScript's shipped `tags.scm` only covers TypeScript-specific nodes —
    /// it is meant to be concatenated with the JavaScript one, since the grammar
    /// extends it. Used alone it misses every function, class, and call site.
    pub fn grammar(self) -> Option<(Language, String)> {
        let js = tree_sitter_javascript::TAGS_QUERY;
        let (f, q) = match self {
            Lang::Rust => (
                tree_sitter_rust::LANGUAGE,
                tree_sitter_rust::TAGS_QUERY.to_string(),
            ),
            Lang::Python => (
                tree_sitter_python::LANGUAGE,
                tree_sitter_python::TAGS_QUERY.to_string(),
            ),
            Lang::JavaScript => (tree_sitter_javascript::LANGUAGE, js.to_string()),
            Lang::TypeScript => (
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
                format!("{js}\n{}", tree_sitter_typescript::TAGS_QUERY),
            ),
            Lang::Tsx => (
                tree_sitter_typescript::LANGUAGE_TSX,
                format!("{js}\n{}", tree_sitter_typescript::TAGS_QUERY),
            ),
            Lang::Go => (
                tree_sitter_go::LANGUAGE,
                tree_sitter_go::TAGS_QUERY.to_string(),
            ),
            Lang::Prose => return None,
        };
        Some((f.into(), q))
    }
}

/// Files worth indexing at all. Everything else is skipped before it is read.
pub fn detect(path: &std::path::Path) -> Option<Lang> {
    let ext = path.extension()?.to_str()?;
    Some(match ext {
        "rs" => Lang::Rust,
        "py" | "pyi" => Lang::Python,
        "js" | "mjs" | "cjs" | "jsx" => Lang::JavaScript,
        "ts" | "mts" | "cts" => Lang::TypeScript,
        "tsx" => Lang::Tsx,
        "go" => Lang::Go,
        "md" | "mdx" | "markdown" | "txt" | "rst" | "adoc" => Lang::Prose,
        "toml" | "yaml" | "yml" | "json" | "sql" | "sh" | "bash" | "zsh" | "nix" | "proto" => {
            Lang::Prose
        }
        _ => return None,
    })
}
