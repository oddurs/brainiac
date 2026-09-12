//! Hybrid retrieval: BM25 over chunk text, fuzzy symbol matching, and a
//! personalised PageRank over the definition/reference graph, fused with
//! reciprocal rank fusion. No embeddings, no model download, no network.

use crate::store::{self, ChunkRow};
use anyhow::Result;
use rusqlite::{Connection, params};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Hit {
    pub chunk_id: i64,
    pub file_id: i64,
    pub path: String,
    pub name: String,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
    pub score: f64,
    pub signals: String,
    pub bhash: String,
}

/// Reciprocal rank fusion constant. 60 is the value from the original paper and
/// behaves well when the input lists have very different score scales.
const RRF_K: f64 = 60.0;
const DAMPING: f64 = 0.85;
const ITERATIONS: usize = 24;
/// Names defined in more than this share of files are language noise (`new`,
/// `get`, `main`) and would smear rank across the whole repo.
const UBIQUITY_CUTOFF: f64 = 0.04;

pub fn tokenize(q: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in q.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
        if raw.len() < 2 {
            continue;
        }
        let lower = raw.to_lowercase();
        if !out.contains(&lower) {
            out.push(lower);
        }
        // Split camelCase / PascalCase so `parseConfig` also matches `config`.
        for part in split_camel(raw) {
            if part.len() >= 3 && !out.contains(&part) {
                out.push(part);
            }
        }
    }
    out.truncate(12);
    out
}

fn split_camel(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_uppercase() && !cur.is_empty() {
            parts.push(std::mem::take(&mut cur).to_lowercase());
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        parts.push(cur.to_lowercase());
    }
    if parts.len() < 2 { Vec::new() } else { parts }
}

fn fts_expr(tokens: &[String]) -> String {
    let mut parts: Vec<String> = tokens
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect();
    if let Some(last) = tokens.last() {
        parts.push(format!("\"{}\"*", last.replace('"', "")));
    }
    parts.join(" OR ")
}

// ---------------------------------------------------------------------------
// Graph

pub struct Graph {
    pub file_ids: Vec<i64>,
    index: HashMap<i64, usize>,
    /// Normalised out-edges: from -> [(to, weight)] summing to 1.
    out: Vec<Vec<(usize, f64)>>,
    /// Edge provenance, used to push file rank down onto individual symbols.
    credit: Vec<(usize, usize, String, f64)>,
    churn: Vec<f64>,
    /// Static per-file multiplier from path shape and size. Applied after the
    /// power iteration so it biases the answer without distorting the walk.
    prior: Vec<f64>,
}

pub fn build_graph(conn: &Connection) -> Result<Graph> {
    let files = store::all_files(conn)?;
    let n = files.len();
    let mut index = HashMap::with_capacity(n);
    let mut churn = vec![0.0; n];
    let mut prior = vec![1.0; n];
    for (i, f) in files.iter().enumerate() {
        index.insert(f.id, i);
        churn[i] = f.churn as f64;
        prior[i] = store::path_prior(&f.path) * (1.0 + 0.12 * (1.0 + f.lines as f64).ln());
    }

    let mut definers: HashMap<String, Vec<usize>> = HashMap::new();
    {
        let mut q = conn.prepare("SELECT name, file_id FROM symbols")?;
        let mut rows = q.query([])?;
        while let Some(r) = rows.next()? {
            let name: String = r.get(0)?;
            let fid: i64 = r.get(1)?;
            if let Some(&i) = index.get(&fid) {
                let e = definers.entry(name).or_default();
                if !e.contains(&i) {
                    e.push(i);
                }
            }
        }
    }
    let cutoff = ((n as f64 * UBIQUITY_CUTOFF).ceil() as usize).max(3);
    definers.retain(|_, v| v.len() <= cutoff);

    let mut raw: Vec<HashMap<usize, f64>> = vec![HashMap::new(); n];
    let mut credit = Vec::new();
    {
        let mut q = conn.prepare("SELECT file_id, name, n FROM refs")?;
        let mut rows = q.query([])?;
        while let Some(r) = rows.next()? {
            let fid: i64 = r.get(0)?;
            let name: String = r.get(1)?;
            let cnt: i64 = r.get(2)?;
            let (Some(&i), Some(ds)) = (index.get(&fid), definers.get(&name)) else {
                continue;
            };
            // A reference is evidence proportional to sqrt(count), split across
            // every file that could satisfy it.
            let w = (cnt as f64).sqrt() / ds.len() as f64;
            for &j in ds {
                if i == j {
                    continue;
                }
                *raw[i].entry(j).or_insert(0.0) += w;
                credit.push((i, j, name.clone(), w));
            }
        }
    }

    let out: Vec<Vec<(usize, f64)>> = raw
        .iter()
        .map(|m| {
            let total: f64 = m.values().sum();
            if total <= 0.0 {
                Vec::new()
            } else {
                m.iter().map(|(&j, &w)| (j, w / total)).collect()
            }
        })
        .collect();

    Ok(Graph {
        file_ids: files.iter().map(|f| f.id).collect(),
        index,
        out,
        credit,
        churn,
        prior,
    })
}

pub struct Ranked {
    pub files: HashMap<i64, f64>,
    pub symbols: HashMap<(i64, String), f64>,
}

/// Personalised PageRank. `seeds` is a file_id -> weight teleport vector; an
/// empty map gives the unbiased repo-wide importance ranking.
pub fn pagerank(g: &Graph, seeds: &HashMap<i64, f64>) -> Ranked {
    let n = g.file_ids.len();
    let mut p = vec![0.0; n];
    let total: f64 = seeds.values().sum();
    if total > 0.0 {
        for (fid, w) in seeds {
            if let Some(&i) = g.index.get(fid) {
                p[i] += w / total;
            }
        }
        let s: f64 = p.iter().sum();
        if s <= 0.0 {
            p = vec![1.0 / n.max(1) as f64; n];
        }
    } else {
        p = vec![1.0 / n.max(1) as f64; n];
    }

    let mut r = p.clone();
    for _ in 0..ITERATIONS {
        let mut next = vec![0.0; n];
        let mut dangling = 0.0;
        for (i, &mass) in r.iter().enumerate() {
            if g.out[i].is_empty() {
                dangling += mass;
                continue;
            }
            for &(j, w) in &g.out[i] {
                next[j] += mass * w;
            }
        }
        for (j, slot) in next.iter_mut().enumerate() {
            *slot = (1.0 - DAMPING) * p[j] + DAMPING * (*slot + dangling * p[j]);
        }
        r = next;
    }

    // Priors: recently touched files are likelier to be the subject, vendored and
    // archived copies almost never are, and bigger files carry more to say.
    for (i, mass) in r.iter_mut().enumerate() {
        *mass *= g.prior[i] * (1.0 + 0.18 * (1.0 + g.churn[i]).ln());
    }

    let mut symbols: HashMap<(i64, String), f64> = HashMap::new();
    for (i, j, name, w) in &g.credit {
        let contrib = r[*i] * w;
        *symbols.entry((g.file_ids[*j], name.clone())).or_insert(0.0) += contrib;
    }

    let files = g
        .file_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, r[i]))
        .collect();
    Ranked { files, symbols }
}

// ---------------------------------------------------------------------------
// Fusion

/// Prose windows and whole-file fallbacks are weaker evidence than a match
/// inside a named definition.
fn kind_prior(kind: &str) -> f64 {
    match kind {
        "file" => 0.6,
        "header" => 0.8,
        _ => 1.0,
    }
}

struct Candidate {
    chunk: ChunkRow,
    path: String,
    lex: Option<usize>,
    sym: Option<usize>,
    gph: Option<usize>,
}

pub fn search(conn: &Connection, query: &str, limit: usize) -> Result<Vec<Hit>> {
    let g = build_graph(conn)?;
    search_with_graph(conn, &g, query, limit)
}

pub fn search_with_graph(
    conn: &Connection,
    g: &Graph,
    query: &str,
    limit: usize,
) -> Result<Vec<Hit>> {
    let tokens = tokenize(query);
    let mut seeds: HashMap<i64, f64> = HashMap::new();

    // 1. Lexical.
    let mut lex: Vec<i64> = Vec::new();
    if !tokens.is_empty() {
        let expr = fts_expr(&tokens);
        let mut q = conn.prepare(
            "SELECT rowid, bm25(chunk_fts, 4.0, 1.0) AS s FROM chunk_fts
             WHERE chunk_fts MATCH ?1 ORDER BY s LIMIT 400",
        )?;
        let rows = q.query_map(params![expr], |r| r.get::<_, i64>(0));
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                lex.push(row);
            }
        }
    }

    // 2. Symbol name matching, ranked exact > prefix > substring.
    let mut sym_scored: Vec<(f64, i64, String)> = Vec::new();
    for t in &tokens {
        let like = format!("%{t}%");
        let mut q =
            conn.prepare("SELECT file_id, name FROM symbols WHERE lower(name) LIKE ?1 LIMIT 200")?;
        for row in q
            .query_map(params![like], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })?
            .flatten()
        {
            let lower = row.1.to_lowercase();
            let s = if lower == *t {
                3.0
            } else if lower.starts_with(t.as_str()) {
                2.0
            } else {
                1.0
            };
            sym_scored.push((s, row.0, row.1));
        }
    }
    sym_scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    sym_scored.truncate(200);

    // 3. Seed the graph from both evidence streams.
    for (rank, cid) in lex.iter().enumerate() {
        if let Some(c) = store::chunk_by_id(conn, *cid)? {
            *seeds.entry(c.file_id).or_insert(0.0) += 1.0 / (10.0 + rank as f64);
        }
    }
    for (s, fid, _) in &sym_scored {
        *seeds.entry(*fid).or_insert(0.0) += s / 4.0;
    }

    let ranked = pagerank(g, &seeds);

    // 4. Candidate pool: lexical hits, symbol-matched chunks, and the chunks of
    //    the highest-ranked files even when they never matched a token.
    let mut pool: HashMap<i64, Candidate> = HashMap::new();
    let add = |conn: &Connection, pool: &mut HashMap<i64, Candidate>, c: ChunkRow| -> Result<()> {
        if pool.contains_key(&c.id) {
            return Ok(());
        }
        let path = store::file_by_id(conn, c.file_id)?
            .map(|f| f.path)
            .unwrap_or_default();
        pool.insert(
            c.id,
            Candidate {
                chunk: c,
                path,
                lex: None,
                sym: None,
                gph: None,
            },
        );
        Ok(())
    };

    for (rank, cid) in lex.iter().enumerate() {
        if let Some(c) = store::chunk_by_id(conn, *cid)? {
            add(conn, &mut pool, c)?;
            if let Some(e) = pool.get_mut(cid) {
                e.lex = Some(rank);
            }
        }
    }

    for (rank, (_, fid, name)) in sym_scored.iter().enumerate() {
        let mut q = conn.prepare(
            "SELECT id,file_id,name,kind,start_line,end_line,body,bhash FROM chunks
             WHERE file_id=?1 AND name=?2 LIMIT 4",
        )?;
        let rows: Vec<ChunkRow> = q
            .query_map(params![fid, name], row_to_chunk)?
            .collect::<rusqlite::Result<_>>()?;
        for c in rows {
            let id = c.id;
            add(conn, &mut pool, c)?;
            let e = pool.get_mut(&id).unwrap();
            e.sym = Some(e.sym.map_or(rank, |r| r.min(rank)));
        }
    }

    let mut top_files: Vec<(i64, f64)> = ranked.files.iter().map(|(&k, &v)| (k, v)).collect();
    top_files.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut graph_order: Vec<(f64, i64)> = Vec::new();
    for (fid, fscore) in top_files.iter().take(40) {
        let mut q = conn.prepare(
            "SELECT id,file_id,name,kind,start_line,end_line,body,bhash FROM chunks WHERE file_id=?1",
        )?;
        let rows: Vec<ChunkRow> = q
            .query_map(params![fid], row_to_chunk)?
            .collect::<rusqlite::Result<_>>()?;
        for c in rows {
            let s = fscore
                * (1.0
                    + ranked
                        .symbols
                        .get(&(*fid, c.name.clone()))
                        .copied()
                        .unwrap_or(0.0)
                        * 40.0);
            let id = c.id;
            add(conn, &mut pool, c)?;
            graph_order.push((s, id));
        }
    }
    graph_order.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (rank, (_, id)) in graph_order.iter().enumerate() {
        if let Some(e) = pool.get_mut(id) {
            e.gph = Some(rank);
        }
    }

    // 5. Fuse. Weights favour direct lexical evidence, then symbol identity,
    //    then structural importance — which mostly breaks ties.
    let mut hits: Vec<Hit> = pool
        .into_values()
        .map(|c| {
            let mut score = 0.0;
            let mut signals = String::new();
            let bias = kind_prior(&c.chunk.kind) * store::path_prior(&c.path);
            if let Some(r) = c.lex {
                score += bias / (RRF_K + r as f64);
                signals.push('L');
            }
            if let Some(r) = c.sym {
                score += 0.8 * bias / (RRF_K + r as f64);
                signals.push('S');
            }
            if let Some(r) = c.gph {
                score += 0.45 / (RRF_K + r as f64);
                signals.push('G');
            }
            Hit {
                bhash: c.chunk.bhash.clone(),
                chunk_id: c.chunk.id,
                file_id: c.chunk.file_id,
                path: c.path,
                name: c.chunk.name,
                kind: c.chunk.kind,
                start_line: c.chunk.start_line,
                end_line: c.chunk.end_line,
                score,
                signals,
            }
        })
        .collect();

    hits.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.path.cmp(&b.path))
            .then(a.start_line.cmp(&b.start_line))
    });
    // Vendored trees and versioned doc snapshots hold byte-identical copies.
    // Keep the best-ranked instance of each body and drop the rest.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    hits.retain(|h| seen.insert(h.bhash.clone()));
    hits.truncate(limit);
    Ok(hits)
}

fn row_to_chunk(r: &rusqlite::Row) -> rusqlite::Result<ChunkRow> {
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
}
