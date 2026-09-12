//! End-to-end coverage of the index -> parse -> rank -> pack pipeline against a
//! throwaway repo on disk.

use brainiac::{index, lang, pack, parse, rank, store};
use std::fs;
use std::path::{Path, PathBuf};

struct Fixture {
    root: PathBuf,
    db: PathBuf,
    _tmp: tempfile::TempDir,
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(&root).unwrap();
    let db = tmp.path().join("index.db");
    Fixture {
        root,
        db,
        _tmp: tmp,
    }
}

impl Fixture {
    fn write(&self, rel: &str, body: &str) {
        let p = self.root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }
    fn rm(&self, rel: &str) {
        fs::remove_file(self.root.join(rel)).unwrap();
    }
    fn store(&self) -> store::Store {
        store::Store::open(&self.db).unwrap()
    }
    fn index(&self, st: &mut store::Store) -> index::Stats {
        index::run(&self.root, st, false).unwrap()
    }
}

const LIB_RS: &str = r#"
use crate::parser::parse_config;

/// Reads and validates the configuration file.
pub fn load_config(path: &str) -> Config {
    let raw = std::fs::read_to_string(path).unwrap();
    parse_config(&raw)
}

pub struct Config {
    pub retries: u32,
}

impl Config {
    pub fn retries(&self) -> u32 {
        self.retries
    }
}
"#;

const PARSER_RS: &str = r#"
use crate::Config;

/// Turns raw text into a Config.
pub fn parse_config(raw: &str) -> Config {
    let retries = raw.trim().parse().unwrap_or(3);
    Config { retries }
}
"#;

fn seed(fx: &Fixture) {
    fx.write("src/lib.rs", LIB_RS);
    fx.write("src/parser.rs", PARSER_RS);
    fx.write(
        "README.md",
        "# demo\n\nA configuration loader used in tests.\n",
    );
}

// --- parsing ---------------------------------------------------------------

#[test]
fn extracts_definitions_and_references() {
    let p = parse::parse(lang::Lang::Rust, LIB_RS).unwrap();
    let names: Vec<&str> = p.defs.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"load_config"), "got {names:?}");
    assert!(names.contains(&"Config"), "got {names:?}");
    // `parse_config` is used here but defined elsewhere: that is a graph edge.
    assert!(p.refs.contains_key("parse_config"), "refs {:?}", p.refs);
}

#[test]
fn a_definition_is_reported_once_per_span() {
    // Rust's tags query captures inherent methods as both `definition.method`
    // and `definition.function`; the index must not see two symbols.
    let p = parse::parse(lang::Lang::Rust, LIB_RS).unwrap();
    let retries: Vec<_> = p
        .defs
        .iter()
        .filter(|d| d.name == "retries" && d.kind != "field")
        .collect();
    assert_eq!(retries.len(), 1, "duplicated def: {retries:?}");
}

#[test]
fn one_line_definitions_are_not_chunked() {
    let src = "mod a;\nmod b;\npub fn real(x: u32) -> u32 {\n    x + 1\n}\n";
    let p = parse::parse(lang::Lang::Rust, src).unwrap();
    let chunks = parse::chunks("src/x.rs", src, &p);
    assert!(
        chunks.iter().all(|c| c.name != "a" && c.name != "b"),
        "module stubs became chunks: {:?}",
        chunks.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    assert!(chunks.iter().any(|c| c.name == "real"));
}

#[test]
fn files_without_a_grammar_still_chunk() {
    let src = (1..=200)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let p = parse::parse(lang::Lang::Prose, &src).unwrap();
    assert!(p.defs.is_empty());
    let chunks = parse::chunks("notes.md", &src, &p);
    assert!(
        chunks.len() > 1,
        "expected sliding windows, got {}",
        chunks.len()
    );
    assert!(chunks.iter().all(|c| c.kind == "file"));
}

// --- incremental indexing --------------------------------------------------

#[test]
fn reindex_is_incremental_and_prunes() {
    let fx = fixture();
    seed(&fx);
    let mut st = fx.store();

    let first = fx.index(&mut st);
    assert_eq!(first.scanned, 3);
    assert_eq!(first.reparsed, 3);

    // Nothing changed on disk: nothing should be reparsed.
    let second = fx.index(&mut st);
    assert_eq!(second.scanned, 3);
    assert_eq!(second.reparsed, 0, "unchanged repo was reparsed");

    // Touching one file reparses exactly that file.
    fx.write(
        "src/parser.rs",
        &format!("{PARSER_RS}\npub fn extra() -> u8 {{ 7 }}\n"),
    );
    let third = fx.index(&mut st);
    assert_eq!(third.reparsed, 1);
    assert!(!store::symbols_named(&st.conn, "extra").unwrap().is_empty());

    // Deleting a file removes it and everything derived from it.
    fx.rm("src/parser.rs");
    let fourth = fx.index(&mut st);
    assert_eq!(fourth.pruned, 1);
    assert!(
        store::symbols_named(&st.conn, "parse_config")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn deleting_a_file_keeps_the_search_index_consistent() {
    let fx = fixture();
    seed(&fx);
    let mut st = fx.store();
    fx.index(&mut st);
    assert!(
        !rank::search(&st.conn, "parse_config", 5)
            .unwrap()
            .is_empty()
    );

    fx.rm("src/parser.rs");
    fx.index(&mut st);

    // An external-content FTS table left out of sync would still return the
    // deleted rowid here, so this asserts the delete path, not just the table.
    let hits = rank::search(&st.conn, "parse_config", 5).unwrap();
    assert!(
        hits.iter().all(|h| h.path != "src/parser.rs"),
        "stale FTS rows survived deletion: {hits:?}"
    );
    let orphans: i64 = st
        .conn
        .query_row(
            "SELECT count(*) FROM chunk_fts WHERE rowid NOT IN (SELECT id FROM chunks)",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(orphans, 0);
}

#[test]
fn gitignored_files_are_never_indexed() {
    let fx = fixture();
    seed(&fx);
    fs::create_dir_all(fx.root.join(".git")).unwrap();
    fx.write(".gitignore", "target/\n");
    fx.write("target/junk.rs", "pub fn junk() {}\n");
    let mut st = fx.store();
    fx.index(&mut st);
    assert!(store::symbols_named(&st.conn, "junk").unwrap().is_empty());
}

#[test]
fn a_schema_change_rebuilds_rather_than_migrates() {
    let fx = fixture();
    seed(&fx);
    {
        let mut st = fx.store();
        fx.index(&mut st);
        st.set_meta("schema", "definitely-old").unwrap();
    }
    let st = fx.store();
    let (files, ..) = store::counts(&st.conn).unwrap();
    assert_eq!(files, 0, "stale schema was not dropped");
}

// --- ranking ---------------------------------------------------------------

#[test]
fn tokenize_splits_camel_case() {
    let t = rank::tokenize("parseConfigFile");
    assert!(t.contains(&"parseconfigfile".to_string()));
    assert!(t.contains(&"config".to_string()), "got {t:?}");
}

#[test]
fn search_finds_a_symbol_by_name() {
    let fx = fixture();
    seed(&fx);
    let mut st = fx.store();
    fx.index(&mut st);
    let hits = rank::search(&st.conn, "load_config", 10).unwrap();
    assert_eq!(hits[0].name, "load_config", "ranked: {hits:?}");
}

#[test]
fn identical_bodies_collapse_to_one_hit() {
    let fx = fixture();
    seed(&fx);
    // The same function, vendored into three version-pinned copies.
    for v in ["0.1", "0.2", "0.3"] {
        fx.write(
            &format!("docs/versions/{v}/sample.rs"),
            "pub fn duplicated_helper(x: u32) -> u32 {\n    x * 2\n}\n",
        );
    }
    let mut st = fx.store();
    fx.index(&mut st);
    let hits = rank::search(&st.conn, "duplicated_helper", 10).unwrap();
    let copies = hits
        .iter()
        .filter(|h| h.name == "duplicated_helper")
        .count();
    assert_eq!(copies, 1, "duplicate bodies not collapsed: {hits:?}");
}

#[test]
fn archived_and_vendored_paths_rank_below_live_source() {
    assert!(store::path_prior("src/pane.rs") > store::path_prior("vendor/pane.rs"));
    assert!(store::path_prior("src/pane.rs") > store::path_prior("docs/versions/0.1/pane.rs"));
    assert!(store::path_prior("src/pane.rs") > store::path_prior("tests/pane_test.rs"));
    assert!(store::path_prior("src/pane.rs") == 1.0);
}

#[test]
fn pagerank_seeding_moves_rank_toward_the_seed() {
    let fx = fixture();
    seed(&fx);
    let mut st = fx.store();
    fx.index(&mut st);
    let g = rank::build_graph(&st.conn).unwrap();

    let files = store::all_files(&st.conn).unwrap();
    let parser = files.iter().find(|f| f.path == "src/parser.rs").unwrap();

    let flat = rank::pagerank(&g, &Default::default());
    let seeded = rank::pagerank(&g, &[(parser.id, 1.0)].into_iter().collect());

    let share = |r: &rank::Ranked| {
        let total: f64 = r.files.values().sum();
        r.files[&parser.id] / total
    };
    assert!(
        share(&seeded) > share(&flat),
        "seeding did not bias rank: {} vs {}",
        share(&seeded),
        share(&flat)
    );
}

// --- packing ---------------------------------------------------------------

fn opts(query: &str, budget: usize, map_share: f64) -> pack::Opts {
    pack::Opts {
        query: query.into(),
        budget,
        map_share,
        head: None,
        root: "demo".into(),
    }
}

#[test]
fn a_pack_stays_inside_its_budget() {
    let fx = fixture();
    seed(&fx);
    for i in 0..40 {
        fx.write(
            &format!("src/mod{i}.rs"),
            &format!(
                "use crate::parser::parse_config;\n\
                 /// Handler number {i} for configuration work.\n\
                 pub fn handler_{i}(cfg: &str) -> u32 {{\n    \
                     let c = parse_config(cfg);\n    \
                     c.retries + {i}\n\
                 }}\n"
            ),
        );
    }
    let mut st = fx.store();
    fx.index(&mut st);

    for budget in [400usize, 1500, 6000] {
        let text = pack::build(&st.conn, &opts("configuration retries", budget, 0.3)).unwrap();
        let used = pack::est_tokens(&text);
        assert!(
            used <= budget + budget / 4,
            "budget {budget} overshot: {used} tokens"
        );
        assert!(used > 0);
    }
}

#[test]
fn a_pack_leads_with_code_that_answers_the_query() {
    let fx = fixture();
    seed(&fx);
    let mut st = fx.store();
    fx.index(&mut st);
    let text = pack::build(&st.conn, &opts("parse_config raw text", 4000, 0.2)).unwrap();
    assert!(text.contains("## Relevant code"), "{text}");
    assert!(text.contains("parse_config"), "{text}");
}

#[test]
fn an_empty_query_still_produces_a_usable_map() {
    let fx = fixture();
    seed(&fx);
    let mut st = fx.store();
    fx.index(&mut st);
    let text = pack::build(&st.conn, &opts("", 2000, 1.0)).unwrap();
    assert!(text.contains("## Repo map"));
    assert!(text.contains("src/lib.rs"), "{text}");
    assert!(!text.contains("## Relevant code"));
}

#[test]
fn token_estimate_tracks_length() {
    assert_eq!(pack::est_tokens(""), 0);
    let short = pack::est_tokens("fn main() {}");
    let long = pack::est_tokens(&"fn main() {}".repeat(10));
    assert!(long > short * 8, "{short} vs {long}");
}

// --- discovery -------------------------------------------------------------

#[test]
fn discovery_walks_up_to_the_git_root() {
    let fx = fixture();
    fs::create_dir_all(fx.root.join(".git")).unwrap();
    let deep = fx.root.join("src/deep/nested");
    fs::create_dir_all(&deep).unwrap();
    let repo = brainiac::config::discover(Some(&deep)).unwrap();
    assert_eq!(
        fs::canonicalize(&repo.root).unwrap(),
        fs::canonicalize(&fx.root).unwrap()
    );
    assert!(repo.db.extension().is_some_and(|e| e == "db"));
}

#[test]
fn language_detection_covers_the_supported_set() {
    let l = |p: &str| lang::detect(Path::new(p)).map(|l| l.name());
    assert_eq!(l("a/b.rs"), Some("rust"));
    assert_eq!(l("a/b.tsx"), Some("tsx"));
    assert_eq!(l("a/b.py"), Some("python"));
    assert_eq!(l("a/b.go"), Some("go"));
    assert_eq!(l("a/b.md"), Some("prose"));
    assert_eq!(l("a/b.png"), None);
    assert_eq!(l("Makefile"), None);
}

#[test]
fn typescript_gets_the_javascript_tags_too() {
    // TypeScript's shipped tags.scm only covers TS-specific nodes. Used alone it
    // finds the interface here and nothing else.
    let src = "\
export interface Props { id: string }\n\
export function loadUser(id: string): Props { return { id }; }\n\
export const saveUser = async (p: Props) => { await send(p); };\n\
export class UserStore { fetch(id: string) { return loadUser(id); } }\n";

    for lang in [lang::Lang::TypeScript, lang::Lang::Tsx] {
        let p = parse::parse(lang, src).unwrap();
        let names: Vec<&str> = p.defs.iter().map(|d| d.name.as_str()).collect();
        for want in ["Props", "loadUser", "saveUser", "UserStore"] {
            assert!(names.contains(&want), "{lang:?} missed {want}: {names:?}");
        }
        assert!(p.refs.contains_key("send"), "{lang:?} refs: {:?}", p.refs);
    }
}

/// Two indexers on one index — the MCP server refreshing while a CLI run arrives.
///
/// A smoke test, not a regression test: it drives the real write path under real
/// contention, but the interleaving that triggers SQLITE_BUSY_SNAPSHOT is not
/// guaranteed, and this passes on a deferred transaction often enough to be useless
/// as a guard. The guard is
/// `store::tests::the_index_transaction_takes_the_write_lock_at_begin`, which is
/// deterministic. This one is here to catch gross breakage: deadlock, corruption, or
/// a run that silently indexes nothing.
#[test]
fn two_concurrent_index_runs_both_succeed() {
    let fx = fixture();
    seed(&fx);
    for i in 0..60 {
        fx.write(
            &format!("src/mod{i}.rs"),
            &format!(
                "use crate::parser::parse_config;\npub fn handler_{i}(c: &str) -> u32 {{\n    parse_config(c).retries + {i}\n}}\n"
            ),
        );
    }
    fx.index(&mut fx.store());

    for round in 0..2 {
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let db = fx.db.clone();
                let root = fx.root.clone();
                std::thread::spawn(move || {
                    let mut st = store::Store::open(&db).unwrap();
                    index::run(&root, &mut st, true).map(|s| s.scanned)
                })
            })
            .collect();
        for h in handles {
            let scanned = h
                .join()
                .unwrap()
                .unwrap_or_else(|e| panic!("round {round}: concurrent index run failed: {e:#}"));
            assert_eq!(scanned, 63);
        }
    }
}
