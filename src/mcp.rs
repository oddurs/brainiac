//! Minimal MCP server over stdio. Hand-rolled JSON-RPC keeps the dependency
//! surface at serde_json and the protocol visible in one file.

use crate::{config::Repo, index, pack, rank, store::Store};
use anyhow::Result;
use serde_json::{Value, json};
use std::io::{BufRead, Write};

const PROTOCOL: &str = "2025-06-18";

pub fn serve(repo: &Repo, st: &mut Store) -> Result<()> {
    // Bring the index up to date before answering anything; stdout is reserved
    // for the protocol, so progress goes to stderr.
    let s = index::run(&repo.scope, repo.git_root.as_deref(), st, false)?;
    eprintln!(
        "brainiac: scope {} — indexed {} files ({} reparsed) in {}ms",
        repo.scope.display(),
        s.scanned,
        s.reparsed,
        s.elapsed_ms
    );

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(req): Result<Value, _> = serde_json::from_str(&line) else {
            continue;
        };
        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(json!({}));

        let result = dispatch(repo, st, method, &params);
        // Notifications carry no id and must not be answered.
        let Some(id) = id else { continue };

        let body = match result {
            Ok(Some(v)) => json!({"jsonrpc":"2.0","id":id,"result":v}),
            Ok(None) => continue,
            Err(e) => json!({
                "jsonrpc":"2.0","id":id,
                "error":{"code":-32603,"message":e.to_string()}
            }),
        };
        writeln!(stdout, "{body}")?;
        stdout.flush()?;
    }
    Ok(())
}

fn dispatch(repo: &Repo, st: &mut Store, method: &str, params: &Value) -> Result<Option<Value>> {
    match method {
        "initialize" => Ok(Some(json!({
            "protocolVersion": PROTOCOL,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "brainiac", "version": env!("CARGO_PKG_VERSION")}
        }))),
        "ping" => Ok(Some(json!({}))),
        "tools/list" => Ok(Some(json!({"tools": tools()}))),
        "tools/call" => {
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let text = call(repo, st, name, &args)?;
            Ok(Some(
                json!({"content":[{"type":"text","text":text}],"isError":false}),
            ))
        }
        m if m.starts_with("notifications/") => Ok(None),
        _ => Ok(Some(json!({}))),
    }
}

fn tools() -> Value {
    json!([
        {
            "name": "context_pack",
            "description": "Assemble a token-budgeted context pack for a question about this repo: a reference-ranked repo map plus the code most likely to answer it. Prefer this over reading files one by one.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "What you are trying to find or change."},
                    "budget": {"type": "integer", "description": "Approximate token budget. Default 6000."}
                },
                "required": ["query"]
            }
        },
        {
            "name": "search_code",
            "description": "Ranked hybrid search over the repo (BM25 + symbol match + reference-graph PageRank). Returns file:line spans with code.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "description": "Default 12."}
                },
                "required": ["query"]
            }
        },
        {
            "name": "repo_map",
            "description": "Skeleton of the repository: the most structurally central files with their definition signatures. Use to orient before touching unfamiliar code.",
            "inputSchema": {
                "type": "object",
                "properties": {"budget": {"type": "integer", "description": "Default 3000."}}
            }
        },
        {
            "name": "read_symbol",
            "description": "Full source of every definition with the given exact name.",
            "inputSchema": {
                "type": "object",
                "properties": {"name": {"type": "string"}},
                "required": ["name"]
            }
        },
        {
            "name": "reindex",
            "description": "Re-scan the repository. Only needed if files changed on disk during this session.",
            "inputSchema": {"type": "object", "properties": {}}
        }
    ])
}

fn call(repo: &Repo, st: &mut Store, name: &str, args: &Value) -> Result<String> {
    let s = |k: &str| {
        args.get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let n = |k: &str, d: usize| {
        args.get(k)
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(d)
    };
    let head = st.get_meta("head")?;
    let root = repo.scope.to_string_lossy().to_string();

    match name {
        "context_pack" => pack::build(
            &st.conn,
            &pack::Opts {
                query: s("query"),
                budget: n("budget", 6000),
                map_share: 0.3,
                head,
                root,
            },
        ),
        "repo_map" => pack::build(
            &st.conn,
            &pack::Opts {
                query: String::new(),
                budget: n("budget", 3000),
                map_share: 1.0,
                head,
                root,
            },
        ),
        "search_code" => {
            let hits = rank::search(&st.conn, &s("query"), n("limit", 12))?;
            if hits.is_empty() {
                return Ok("no matches".into());
            }
            let mut out = String::new();
            for h in hits {
                let body: String = st.conn.query_row(
                    "SELECT body FROM chunks WHERE id=?1",
                    rusqlite::params![h.chunk_id],
                    |r| r.get(0),
                )?;
                out.push_str(&format!(
                    "### {}:{}-{} — {} [{}]\n```\n{}\n```\n\n",
                    h.path, h.start_line, h.end_line, h.name, h.signals, body
                ));
            }
            Ok(out)
        }
        "read_symbol" => {
            let want = s("name");
            let syms = crate::store::symbols_named(&st.conn, &want)?;
            if syms.is_empty() {
                return Ok(format!("no definition named `{want}`"));
            }
            let mut out = String::new();
            for sym in syms {
                let Some(f) = crate::store::file_by_id(&st.conn, sym.file_id)? else {
                    continue;
                };
                let full = repo.scope.join(&f.path);
                let src = std::fs::read_to_string(&full).unwrap_or_default();
                let lines: Vec<&str> = src.lines().collect();
                let a = sym.start_line.saturating_sub(1) as usize;
                let b = (sym.end_line as usize).min(lines.len());
                out.push_str(&format!(
                    "### {}:{}-{} — {} {}\n```\n{}\n```\n\n",
                    f.path,
                    sym.start_line,
                    sym.end_line,
                    sym.kind,
                    sym.name,
                    lines[a.min(b)..b].join("\n")
                ));
            }
            Ok(out)
        }
        "reindex" => {
            let s = index::run(&repo.scope, repo.git_root.as_deref(), st, false)?;
            Ok(format!(
                "scanned {} files, reparsed {}, pruned {} in {}ms",
                s.scanned, s.reparsed, s.pruned, s.elapsed_ms
            ))
        }
        other => Ok(format!("unknown tool `{other}`")),
    }
}
