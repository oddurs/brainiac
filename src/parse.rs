//! Tree-sitter tag extraction: definitions, references, and retrieval chunks.

use crate::lang::Lang;
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

thread_local! {
    /// Compiling a tag query costs more than parsing a small file, and indexing
    /// runs one rayon worker per core over thousands of files.
    static COMPILED: RefCell<HashMap<&'static str, (Parser, Query)>> =
        RefCell::new(HashMap::new());
}

#[derive(Debug)]
pub struct Def {
    pub name: String,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
    pub sig: String,
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub defs: Vec<Def>,
    pub refs: HashMap<String, u32>,
}

/// Longest single chunk we will store, in lines. Bodies past this are truncated;
/// the symbol graph still knows the real span.
const MAX_CHUNK_LINES: u32 = 140;

pub fn parse(lang: Lang, src: &str) -> Result<Parsed> {
    if lang.grammar().is_none() {
        return Ok(Parsed::default());
    }
    COMPILED.with(|cache| {
        let mut cache = cache.borrow_mut();
        if !cache.contains_key(lang.name()) {
            let (language, tags) = lang.grammar().expect("checked above");
            let mut parser = Parser::new();
            parser.set_language(&language)?;
            let query = Query::new(&language, &tags)?;
            cache.insert(lang.name(), (parser, query));
        }
        let (parser, query) = cache.get_mut(lang.name()).expect("just inserted");
        parse_with(parser, query, src)
    })
}

fn parse_with(parser: &mut Parser, query: &Query, src: &str) -> Result<Parsed> {
    let Some(tree) = parser.parse(src, None) else {
        return Ok(Parsed::default());
    };
    let names: Vec<&str> = query.capture_names().to_vec();

    let mut out = Parsed::default();
    let mut cursor = QueryCursor::new();
    let bytes = src.as_bytes();
    let mut it = cursor.matches(query, tree.root_node(), bytes);

    while let Some(m) = it.next() {
        // A tags match pairs one `@definition.*` / `@reference.*` node with a `@name`.
        let mut role: Option<(&str, tree_sitter::Node)> = None;
        let mut name: Option<&str> = None;
        for c in m.captures() {
            let cap = names[c.index as usize];
            if let Some(kind) = cap.strip_prefix("definition.") {
                role = Some((kind, c.node));
            } else if cap.starts_with("reference.") {
                role = Some(("@ref", c.node));
            } else if cap == "name" {
                name = c.node.utf8_text(bytes).ok();
            }
        }
        let (Some((kind, node)), Some(name)) = (role, name) else {
            continue;
        };
        if name.is_empty() || name.len() > 128 {
            continue;
        }
        if kind == "@ref" {
            *out.refs.entry(name.to_string()).or_insert(0) += 1;
        } else {
            let start = node.start_position().row as u32;
            let end = node.end_position().row as u32;
            let sig = src
                .lines()
                .nth(start as usize)
                .unwrap_or("")
                .trim_end()
                .to_string();
            out.defs.push(Def {
                name: name.to_string(),
                kind: kind.to_string(),
                start_line: start + 1,
                end_line: end + 1,
                sig: truncate(&sig, 200),
            });
        }
    }

    dedupe_defs(&mut out.defs);

    // A definition that is also referenced inside its own file is noise for the graph.
    for d in &out.defs {
        out.refs.remove(&d.name);
    }
    Ok(out)
}

/// Tag queries routinely capture one node under several roles — Rust emits both
/// `definition.method` and `definition.function` for an inherent method. Keep the
/// most specific kind per span.
fn dedupe_defs(defs: &mut Vec<Def>) {
    fn specificity(kind: &str) -> u8 {
        match kind {
            "method" => 5,
            "class" | "struct" | "trait" | "interface" | "enum" | "type" => 4,
            "function" => 3,
            "constant" | "variable" | "field" => 2,
            _ => 1,
        }
    }
    defs.sort_by(|a, b| {
        (a.start_line, a.end_line, &a.name)
            .cmp(&(b.start_line, b.end_line, &b.name))
            .then(specificity(&b.kind).cmp(&specificity(&a.kind)))
    });
    defs.dedup_by(|a, b| {
        a.start_line == b.start_line && a.end_line == b.end_line && a.name == b.name
    });
}

pub struct ChunkDraft {
    pub name: String,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
    pub body: String,
}

/// Retrieval units. One per definition, plus a header chunk, plus sliding
/// windows for files with no symbol structure.
pub fn chunks(path: &str, src: &str, parsed: &Parsed) -> Vec<ChunkDraft> {
    let lines: Vec<&str> = src.lines().collect();
    let total = lines.len() as u32;
    let mut out = Vec::new();
    let slice = |a: u32, b: u32| -> String {
        let a = a.saturating_sub(1) as usize;
        let b = (b as usize).min(lines.len());
        lines[a.min(b)..b].join("\n")
    };

    if parsed.defs.is_empty() {
        let win = 60u32;
        let step = 50u32;
        let mut start = 1u32;
        while start <= total && out.len() < 40 {
            let end = (start + win - 1).min(total);
            out.push(ChunkDraft {
                name: path.to_string(),
                kind: "file".into(),
                start_line: start,
                end_line: end,
                body: slice(start, end),
            });
            if end == total {
                break;
            }
            start += step;
        }
        return out;
    }

    // Header: imports and module docs carry a lot of routing signal.
    let head_end = parsed
        .defs
        .iter()
        .map(|d| d.start_line)
        .min()
        .unwrap_or(1)
        .saturating_sub(1)
        .min(40);
    if head_end > 2 {
        out.push(ChunkDraft {
            name: path.to_string(),
            kind: "header".into(),
            start_line: 1,
            end_line: head_end,
            body: slice(1, head_end),
        });
    }

    for d in &parsed.defs {
        if d.end_line.saturating_sub(d.start_line) < 2 {
            continue;
        }
        let end = d.end_line.min(d.start_line + MAX_CHUNK_LINES);
        out.push(ChunkDraft {
            name: d.name.clone(),
            kind: d.kind.clone(),
            start_line: d.start_line,
            end_line: end,
            body: slice(d.start_line, end),
        });
    }
    out
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        return s.to_string();
    }
    let mut cut = n;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…", &s[..cut])
}
