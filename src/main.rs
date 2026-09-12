//! brainiac — a local context manager for your own repositories.

use anyhow::Result;
use brainiac::{config, index, mcp, pack, rank, store, tui};
use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "brainiac",
    version,
    about = "Ranked, token-budgeted context for your repos — CLI, TUI, and MCP."
)]
struct Cli {
    /// Repository to operate on. Defaults to the enclosing git repo.
    #[arg(long, short = 'C', global = true)]
    repo: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan the repo and update the index.
    Index {
        /// Reparse every file, ignoring content hashes.
        #[arg(long, short)]
        force: bool,
    },
    /// Ranked search. Prints file:line spans.
    Search {
        query: Vec<String>,
        #[arg(long, short = 'n', default_value_t = 12)]
        limit: usize,
        /// Include the matching source, not just locations.
        #[arg(long, short)]
        body: bool,
    },
    /// Build a context pack for a question.
    Pack {
        query: Vec<String>,
        #[arg(long, short, default_value_t = 6000)]
        budget: usize,
        /// Fraction of the budget spent on the repo map.
        #[arg(long, default_value_t = 0.3)]
        map_share: f64,
        /// Write to a file instead of stdout.
        #[arg(long, short)]
        out: Option<PathBuf>,
    },
    /// Print the repo skeleton, ranked by reference centrality.
    Map {
        #[arg(long, short, default_value_t = 3000)]
        budget: usize,
    },
    /// Interactive browser.
    Browse { query: Vec<String> },
    /// Serve over MCP on stdio.
    Mcp,
    /// Index statistics.
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo = config::discover(cli.repo.as_deref())?;
    let mut st = store::Store::open(&repo.db)?;

    match cli.cmd {
        Cmd::Index { force } => {
            let s = index::run(&repo.root, &mut st, force)?;
            let (nf, ns, nc, nl) = store::counts(&st.conn)?;
            eprintln!(
                "{} · {} files, {} lines, {} symbols, {} chunks",
                repo.root.display(),
                nf,
                nl,
                ns,
                nc
            );
            eprintln!(
                "scanned {} · reparsed {} · pruned {} · {}ms",
                s.scanned, s.reparsed, s.pruned, s.elapsed_ms
            );
        }
        Cmd::Status => {
            let (nf, ns, nc, nl) = store::counts(&st.conn)?;
            let size = std::fs::metadata(&repo.db).map(|m| m.len()).unwrap_or(0);
            println!("root     {}", repo.root.display());
            println!("index    {}", repo.db.display());
            println!("size     {:.1} MiB", size as f64 / 1048576.0);
            println!("commit   {}", st.get_meta("head")?.unwrap_or("—".into()));
            println!("files    {nf}");
            println!("lines    {nl}");
            println!("symbols  {ns}");
            println!("chunks   {nc}");
        }
        Cmd::Search { query, limit, body } => {
            ensure_indexed(&repo, &mut st)?;
            let q = query.join(" ");
            let hits = rank::search(&st.conn, &q, limit)?;
            if hits.is_empty() {
                eprintln!("no matches for {q:?}");
                return Ok(());
            }
            for h in &hits {
                println!(
                    "{:<3} {:>6.3}  {}:{}-{}  {} {}",
                    h.signals,
                    h.score * 1000.0,
                    h.path,
                    h.start_line,
                    h.end_line,
                    h.kind,
                    h.name
                );
                if body && let Some(c) = store::chunk_by_id(&st.conn, h.chunk_id)? {
                    for l in c.body.lines() {
                        println!("      {l}");
                    }
                    println!();
                }
            }
        }
        Cmd::Pack {
            query,
            budget,
            map_share,
            out,
        } => {
            ensure_indexed(&repo, &mut st)?;
            let text = pack::build(
                &st.conn,
                &pack::Opts {
                    query: query.join(" "),
                    budget,
                    map_share: map_share.clamp(0.0, 1.0),
                    head: st.get_meta("head")?,
                    root: repo.root.to_string_lossy().to_string(),
                },
            )?;
            emit(&text, out)?;
        }
        Cmd::Map { budget } => {
            ensure_indexed(&repo, &mut st)?;
            let text = pack::build(
                &st.conn,
                &pack::Opts {
                    query: String::new(),
                    budget,
                    map_share: 1.0,
                    head: st.get_meta("head")?,
                    root: repo.root.to_string_lossy().to_string(),
                },
            )?;
            print!("{text}");
        }
        Cmd::Browse { query } => {
            ensure_indexed(&repo, &mut st)?;
            match tui::run(&st.conn, &query.join(" "))? {
                tui::Outcome::Quit => {}
                tui::Outcome::Pack { query, marked } => {
                    let text = if marked.is_empty() {
                        pack::build(
                            &st.conn,
                            &pack::Opts {
                                query,
                                budget: 6000,
                                map_share: 0.25,
                                head: st.get_meta("head")?,
                                root: repo.root.to_string_lossy().to_string(),
                            },
                        )?
                    } else {
                        tui::pack_marked(&st.conn, &marked)?
                    };
                    print!("{text}");
                }
            }
        }
        Cmd::Mcp => mcp::serve(&repo, &mut st)?,
    }
    Ok(())
}

/// Every read path indexes first. An incremental pass over an unchanged repo is
/// cheap, and a stale answer is worse than a short wait.
fn ensure_indexed(repo: &config::Repo, st: &mut store::Store) -> Result<()> {
    let cold = index::is_empty(st);
    if cold {
        eprintln!("brainiac: building index for {} …", repo.root.display());
    }
    let s = index::run(&repo.root, st, false)?;
    if cold || s.reparsed > 0 {
        eprintln!(
            "brainiac: {} files, {} reparsed, {}ms",
            s.scanned, s.reparsed, s.elapsed_ms
        );
    }
    Ok(())
}

fn emit(text: &str, out: Option<PathBuf>) -> Result<()> {
    match out {
        Some(p) => {
            std::fs::write(&p, text)?;
            eprintln!("wrote {} ({} bytes)", p.display(), text.len());
        }
        None => {
            let mut o = std::io::stdout();
            o.write_all(text.as_bytes())?;
            o.flush()?;
        }
    }
    Ok(())
}
