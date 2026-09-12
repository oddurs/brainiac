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
        index::run(&self.root, None, st, false).unwrap()
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

/// `-C` names the scope. Asking about one package of a monorepo must not drag in
/// every other package, which is what walking up to the git root used to do.
#[test]
fn an_explicit_path_scopes_the_walk_to_that_subtree() {
    let fx = fixture();
    fs::create_dir_all(fx.root.join(".git")).unwrap();
    fx.write("packages/ui/lib.rs", "pub fn ui_button() {}\n");
    fx.write("packages/app/main.rs", "pub fn app_main() {}\n");
    fx.write("root_thing.rs", "pub fn root_thing() {}\n");

    let ui = fs::canonicalize(fx.root.join("packages/ui")).unwrap();
    let repo = brainiac::config::discover(Some(&ui)).unwrap();
    assert_eq!(repo.scope, ui);
    assert_eq!(
        repo.git_root
            .as_deref()
            .map(|p| fs::canonicalize(p).unwrap()),
        Some(fs::canonicalize(&fx.root).unwrap()),
        "the git root is still found, for churn and HEAD"
    );

    let mut st = store::Store::open(&fx.db).unwrap();
    let stats = index::run(&repo.scope, repo.git_root.as_deref(), &mut st, false).unwrap();
    assert_eq!(stats.scanned, 1, "only packages/ui should be walked");
    assert!(
        !store::symbols_named(&st.conn, "ui_button")
            .unwrap()
            .is_empty()
    );
    for outside in ["app_main", "root_thing"] {
        assert!(
            store::symbols_named(&st.conn, outside).unwrap().is_empty(),
            "{outside} is outside the scope but was indexed"
        );
    }
}

/// With no `-C`, the scope is the whole repository, so running from a subdirectory
/// still sees everything — the behaviour people rely on day to day.
#[test]
fn without_an_explicit_path_the_scope_is_the_git_root() {
    let fx = fixture();
    fs::create_dir_all(fx.root.join(".git")).unwrap();
    let deep = fx.root.join("src/deep/nested");
    fs::create_dir_all(&deep).unwrap();

    let repo = brainiac::config::discover_from(None, &deep).unwrap();
    assert_eq!(
        repo.scope,
        fs::canonicalize(&fx.root).unwrap(),
        "running from a subdirectory should still index the whole repository"
    );
    assert!(repo.db.extension().is_some_and(|e| e == "db"));
}

/// An explicit `-C` deep inside a repository is taken at face value.
#[test]
fn an_explicit_path_is_used_verbatim_however_deep_it_is() {
    let fx = fixture();
    fs::create_dir_all(fx.root.join(".git")).unwrap();
    let deep = fx.root.join("src/deep/nested");
    fs::create_dir_all(&deep).unwrap();
    let repo = brainiac::config::discover(Some(&deep)).unwrap();
    assert_eq!(repo.scope, fs::canonicalize(&deep).unwrap());
}

/// Outside a repository there is no git root, and the scope is just the directory.
#[test]
fn a_directory_outside_any_repository_still_indexes() {
    let fx = fixture();
    fx.write("a.rs", "pub fn only_thing() {}\n");
    let repo = brainiac::config::discover(Some(&fx.root)).unwrap();
    assert_ne!(
        repo.git_root.as_deref(),
        Some(repo.scope.as_path()),
        "the fixture directory is not itself a repository"
    );
    let mut st = store::Store::open(&fx.db).unwrap();
    let stats = index::run(&repo.scope, repo.git_root.as_deref(), &mut st, false).unwrap();
    assert_eq!(stats.scanned, 1);
}

/// Two subtrees of one repository must not collide in the index directory.
#[test]
fn each_scope_gets_its_own_index_file() {
    let fx = fixture();
    fs::create_dir_all(fx.root.join(".git")).unwrap();
    fx.write("packages/ui/lib.rs", "pub fn a() {}\n");
    fx.write("packages/app/main.rs", "pub fn b() {}\n");
    let ui = brainiac::config::discover(Some(&fx.root.join("packages/ui"))).unwrap();
    let app = brainiac::config::discover(Some(&fx.root.join("packages/app"))).unwrap();
    let whole = brainiac::config::discover(Some(&fx.root)).unwrap();
    assert_ne!(ui.db, app.db);
    assert_ne!(ui.db, whole.db);
    assert_ne!(app.db, whole.db);

    // Migration safety: a whole-repo scope must hash to the same path the old
    // git-root-keyed scheme produced, or every existing index is orphaned.
    let from_inside = brainiac::config::discover_from(None, &fx.root.join("packages/ui")).unwrap();
    assert_eq!(
        from_inside.db, whole.db,
        "whole-repo index path changed; existing indexes would be orphaned"
    );
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
                    index::run(&root, None, &mut st, true).map(|s| s.scanned)
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

// --- scoped indexing against a real repository -----------------------------

/// `git init` plus commits. The churn rebasing is the one genuinely new algorithm
/// in scoped indexing, and a fake `.git` directory cannot exercise it: `git log`
/// fails and the churn map comes back empty.
///
/// The environment scrubbing is not optional. Git hooks export `GIT_DIR`, and it
/// takes precedence over `-C` — so when the suite runs from `.githooks/pre-push`,
/// an unscrubbed `git commit` here lands in the real repository being pushed. That
/// happened, and it put four fixture commits on a feature branch.
fn git(fx: &Fixture, args: &[&str]) {
    let mut cmd = std::process::Command::new("git");
    for var in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_PREFIX",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        cmd.env_remove(var);
    }
    let out = cmd
        .args(["-C", &fx.root.to_string_lossy()])
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .output()
        .expect("git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn churn_of(conn: &rusqlite::Connection, path: &str) -> u32 {
    conn.query_row("SELECT churn FROM files WHERE path=?1", [path], |r| {
        r.get(0)
    })
    .unwrap_or(0)
}

#[test]
fn churn_is_rebased_onto_a_scoped_subtree() {
    let fx = fixture();
    git(&fx, &["init", "-q"]);
    fx.write("packages/ui/lib.rs", "pub fn ui_button() {}\n");
    fx.write("packages/app/main.rs", "pub fn app_main() {}\n");
    git(&fx, &["add", "-A"]);
    git(&fx, &["commit", "-qm", "one"]);
    // Three more commits touching only the ui package.
    for i in 0..3 {
        fx.write(
            "packages/ui/lib.rs",
            &format!("pub fn ui_button() {{ let _ = {i}; }}\n"),
        );
        git(&fx, &["add", "-A"]);
        git(&fx, &["commit", "-qm", "ui change"]);
    }

    let ui = brainiac::config::discover(Some(&fx.root.join("packages/ui"))).unwrap();
    assert!(ui.git_root.is_some(), "the real repository should be found");

    let mut st = store::Store::open(&fx.db).unwrap();
    index::run(&ui.scope, ui.git_root.as_deref(), &mut st, false).unwrap();

    // Repository-relative "packages/ui/lib.rs" must land on scope-relative "lib.rs".
    assert_eq!(
        churn_of(&st.conn, "lib.rs"),
        4,
        "churn did not rebase onto the scope"
    );
    // HEAD still comes from the repository even though only a subtree was indexed.
    assert!(
        st.get_meta("head").unwrap().is_some(),
        "head should come from the git root"
    );
}

#[test]
fn churn_for_a_whole_repo_scope_is_untouched() {
    let fx = fixture();
    git(&fx, &["init", "-q"]);
    fx.write("packages/ui/lib.rs", "pub fn ui_button() {}\n");
    fx.write("root_thing.rs", "pub fn root_thing() {}\n");
    git(&fx, &["add", "-A"]);
    git(&fx, &["commit", "-qm", "one"]);

    let whole = brainiac::config::discover(Some(&fx.root)).unwrap();
    let mut st = store::Store::open(&fx.db).unwrap();
    index::run(&whole.scope, whole.git_root.as_deref(), &mut st, false).unwrap();

    assert_eq!(churn_of(&st.conn, "packages/ui/lib.rs"), 1);
    assert_eq!(churn_of(&st.conn, "root_thing.rs"), 1);
}

#[test]
fn rebasing_drops_paths_outside_the_scope_rather_than_misattributing_them() {
    use std::collections::HashMap;
    use std::path::Path;
    let churn: HashMap<String, u32> = [
        ("packages/ui/lib.rs".to_string(), 4),
        ("packages/uix/other.rs".to_string(), 9), // shares a string prefix, not a path one
        ("root_thing.rs".to_string(), 2),
    ]
    .into_iter()
    .collect();

    let out = index::rebase_churn(churn.clone(), Path::new("/r/packages/ui"), Path::new("/r"));
    assert_eq!(out.get("lib.rs"), Some(&4));
    assert_eq!(out.len(), 1, "only paths inside the scope survive: {out:?}");

    // Whole-repo scope passes through untouched.
    let same = index::rebase_churn(churn.clone(), Path::new("/r"), Path::new("/r"));
    assert_eq!(same.len(), 3);

    // A scope outside the repository means no churn, never all of it applied raw.
    let unrelated = index::rebase_churn(churn, Path::new("/elsewhere"), Path::new("/r"));
    assert!(
        unrelated.is_empty(),
        "unrelated scope must not inherit churn"
    );
}

/// A file is not a scope. Before the directory guard this produced an index with
/// zero rows and no error on any surface: the walk yielded one entry, stripping the
/// root left an empty path, and reading `<file>/` failed ENOTDIR and was discarded.
#[test]
fn pointing_at_a_file_is_refused_rather_than_indexing_nothing() {
    let fx = fixture();
    seed(&fx);
    let msg = match brainiac::config::discover(Some(&fx.root.join("src/lib.rs"))) {
        Ok(r) => panic!("a file was accepted as a scope: {}", r.scope.display()),
        Err(e) => format!("{e:#}"),
    };
    assert!(msg.contains("is not a directory"), "{msg}");
    assert!(
        msg.contains("src"),
        "the message should suggest the directory: {msg}"
    );
}

/// Running with `GIT_DIR` set, exactly as a git hook does, must not read history
/// from — or write it to — the repository that set the variable.
///
/// This is the regression test for four fixture commits landing on a real feature
/// branch: `scripts/task check` ran from `.githooks/pre-push`, which exports
/// `GIT_DIR`, and `GIT_DIR` beats `-C`.
///
/// It drives the real binary in a child process rather than setting `GIT_DIR` in
/// this one: process environment is global, and mutating it would leak into every
/// other test running in parallel.
#[test]
fn an_inherited_git_dir_does_not_leak_into_indexing() {
    let victim = fixture();
    git(&victim, &["init", "-q"]);
    victim.write("only.rs", "pub fn victim_fn() {}\n");
    git(&victim, &["add", "-A"]);
    git(&victim, &["commit", "-qm", "victim"]);
    let victim_head = rev_parse(&victim);

    let subject = fixture();
    git(&subject, &["init", "-q"]);
    subject.write("a.rs", "pub fn subject_fn() {}\n");
    git(&subject, &["add", "-A"]);
    git(&subject, &["commit", "-qm", "subject"]);
    let subject_head = rev_parse(&subject);
    assert_ne!(victim_head, subject_head);

    // A private HOME keeps the child's index out of the real data directory.
    let home = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| -> String {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_brainiac"))
            .args(["-C", &subject.root.to_string_lossy()])
            .args(args)
            .env("GIT_DIR", victim.root.join(".git"))
            .env("HOME", home.path())
            .env("XDG_DATA_HOME", home.path())
            .output()
            .expect("run brainiac");
        assert!(
            out.status.success(),
            "brainiac {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    };

    run(&["index"]);
    let status = run(&["status"]);

    assert!(
        status.contains("files    1"),
        "the subject repo was not indexed:\n{status}"
    );
    // The commit reported must be the subject's, not the repository GIT_DIR names.
    assert!(
        status.contains(&subject_head[..7]),
        "status reported the wrong repository's commit:\n{status}"
    );
    assert!(
        !status.contains(&victim_head[..7]),
        "status leaked the GIT_DIR repository's commit:\n{status}"
    );
    // And indexing must not have written into the unrelated repository.
    assert_eq!(
        victim_head,
        rev_parse(&victim),
        "indexing moved the other repository's HEAD"
    );
}

fn rev_parse(fx: &Fixture) -> String {
    let mut cmd = std::process::Command::new("git");
    for var in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
    ] {
        cmd.env_remove(var);
    }
    let out = cmd
        .args(["-C", &fx.root.to_string_lossy(), "rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
