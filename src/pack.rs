//! Token-budgeted context packs: a ranked skeleton of the repo plus the code
//! that actually answers the query, assembled to fit a stated budget.

use crate::lang::Lang;
use crate::rank::{self, Hit};
use crate::store;
use anyhow::Result;
use rusqlite::{Connection, params};
use std::collections::HashMap;

pub struct Opts {
    pub query: String,
    pub budget: usize,
    /// Share of the budget spent on the repo map rather than on code bodies.
    pub map_share: f64,
    pub head: Option<String>,
    pub root: String,
}

/// Byte-per-token ratio for source text. Measured against BPE tokenizers this
/// runs within a few percent on code, and costs nothing to compute.
pub fn est_tokens(s: &str) -> usize {
    (s.len() as f64 / 3.6).ceil() as usize
}

pub fn build(conn: &Connection, opts: &Opts) -> Result<String> {
    let g = rank::build_graph(conn)?;
    let hits = if opts.query.trim().is_empty() {
        Vec::new()
    } else {
        rank::search_with_graph(conn, &g, &opts.query, 120)?
    };

    let mut seeds: HashMap<i64, f64> = HashMap::new();
    for (i, h) in hits.iter().enumerate() {
        *seeds.entry(h.file_id).or_insert(0.0) += 1.0 / (10.0 + i as f64);
    }
    let ranked = rank::pagerank(&g, &seeds);

    let map_budget = (opts.budget as f64 * opts.map_share) as usize;
    let mut out = String::new();
    let (nf, ns, _, nl) = store::counts(conn)?;

    out.push_str(&format!("# Context pack — {}\n", opts.root));
    if let Some(h) = &opts.head {
        out.push_str(&format!("commit: {h}\n"));
    }
    if !opts.query.trim().is_empty() {
        out.push_str(&format!("query: {}\n", opts.query.trim()));
    }
    out.push_str(&format!("index: {nf} files, {nl} lines, {ns} symbols\n\n"));

    let map = render_map(conn, &ranked, map_budget)?;
    if !map.is_empty() {
        out.push_str("## Repo map\n\nRanked by reference centrality. Signatures only.\n\n");
        out.push_str(&map);
        out.push('\n');
    }

    let spent = est_tokens(&out);
    let code_budget = opts.budget.saturating_sub(spent);
    if code_budget > 0 && !hits.is_empty() {
        let code = render_code(conn, &hits, code_budget)?;
        if !code.is_empty() {
            out.push_str("## Relevant code\n\n");
            out.push_str(&code);
        }
    }

    out.push_str(&format!("\n<!-- ~{} tokens -->\n", est_tokens(&out)));
    Ok(out)
}

fn render_map(conn: &Connection, ranked: &rank::Ranked, budget: usize) -> Result<String> {
    if budget == 0 {
        return Ok(String::new());
    }
    let mut files: Vec<(i64, f64)> = ranked.files.iter().map(|(&k, &v)| (k, v)).collect();
    files.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut out = String::new();
    let mut used = 0usize;
    for (fid, _) in files {
        let Some(f) = store::file_by_id(conn, fid)? else {
            continue;
        };
        let syms = store::symbols_of(conn, fid)?;
        if syms.is_empty() {
            continue;
        }
        // Within a file: graph importance first, then span, so one-line accessors
        // never crowd out the definitions that carry the file's meaning.
        let many = syms.len() > 12;
        let mut scored: Vec<_> = syms
            .into_iter()
            .filter(|s| !(many && s.end_line.saturating_sub(s.start_line) < 3))
            .map(|s| {
                let g = ranked
                    .symbols
                    .get(&(fid, s.name.clone()))
                    .copied()
                    .unwrap_or(0.0);
                let span = (1.0 + s.end_line.saturating_sub(s.start_line) as f64).ln();
                (g * 1000.0 + span, s)
            })
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        if scored.is_empty() {
            continue;
        }

        let mut block = format!("`{}` ({} lines)\n", f.path, f.lines);
        let mut shown: Vec<String> = Vec::new();
        for (_, s) in scored.iter().take(14) {
            let sig = s.sig.trim().trim_end_matches('{').trim_end().to_string();
            if shown.contains(&sig) {
                continue;
            }
            block.push_str(&format!("  {:>5}  {}\n", s.start_line, sig));
            shown.push(sig);
        }
        block.push('\n');
        let cost = est_tokens(&block);
        if used + cost > budget {
            if used == 0 {
                out.push_str(&block);
            }
            break;
        }
        used += cost;
        out.push_str(&block);
    }
    Ok(out)
}

fn render_code(conn: &Connection, hits: &[Hit], budget: usize) -> Result<String> {
    let mut out = String::new();
    let mut used = 0usize;
    // Merge overlapping spans per file so nothing is shown twice.
    let mut taken: HashMap<i64, Vec<(u32, u32)>> = HashMap::new();

    for h in hits {
        let spans = taken.entry(h.file_id).or_default();
        if spans
            .iter()
            .any(|(a, b)| h.start_line >= *a && h.end_line <= *b)
        {
            continue;
        }
        let Some(c) = conn
            .query_row(
                "SELECT body FROM chunks WHERE id=?1",
                params![h.chunk_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
        else {
            continue;
        };
        let lang = store::file_by_id(conn, h.file_id)?
            .map(|f| Lang::from_name(&f.lang))
            .unwrap_or(Lang::Prose);
        let block = format!(
            "### {}:{}-{} — {} [{}]\n```{}\n{}\n```\n\n",
            h.path,
            h.start_line,
            h.end_line,
            h.name,
            h.signals,
            lang.fence(),
            c
        );
        let cost = est_tokens(&block);
        if used + cost > budget {
            if used == 0 && cost < budget * 2 {
                out.push_str(&block);
            }
            break;
        }
        used += cost;
        spans.push((h.start_line, h.end_line));
        out.push_str(&block);
    }
    Ok(out)
}
