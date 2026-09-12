//! The MCP surface, driven over stdio exactly as an agent drives it.
//!
//! These run the real binary rather than calling into the library: the protocol
//! contract includes that stdout carries nothing but JSON-RPC, and that is only
//! observable from outside the process.

use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

struct Session {
    out: Vec<Value>,
    stderr: String,
}

/// Feed a batch of requests to `brainiac mcp` and collect the replies.
fn talk(repo: &std::path::Path, home: &std::path::Path, requests: &[Value]) -> Session {
    let mut child = Command::new(env!("CARGO_BIN_EXE_brainiac"))
        .args(["-C", &repo.to_string_lossy(), "mcp"])
        .env("HOME", home)
        .env("XDG_DATA_HOME", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn brainiac mcp");
    {
        let stdin = child.stdin.as_mut().unwrap();
        for r in requests {
            writeln!(stdin, "{r}").unwrap();
        }
    }
    let out = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l)
                .unwrap_or_else(|e| panic!("stdout line is not JSON-RPC ({e}): {l}"))
        })
        .collect();
    Session {
        out: parsed,
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

fn init() -> Value {
    json!({"jsonrpc":"2.0","id":1,"method":"initialize",
           "params":{"protocolVersion":"2025-06-18","capabilities":{}}})
}

fn call(id: u32, name: &str, args: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
           "params":{"name":name,"arguments":args}})
}

fn fixture_repo() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let home = tmp.path().join("home");
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(
        repo.join("src/lib.rs"),
        "use crate::parser::parse_config;\n\
         /// Reads and validates the configuration file.\n\
         pub fn load_config(path: &str) -> u32 {\n    \
             parse_config(path)\n\
         }\n",
    )
    .unwrap();
    std::fs::write(
        repo.join("src/parser.rs"),
        "/// Turns raw text into a retry count.\n\
         pub fn parse_config(raw: &str) -> u32 {\n    \
             raw.trim().parse().unwrap_or(3)\n\
         }\n",
    )
    .unwrap();
    (tmp, repo, home)
}

#[test]
fn the_handshake_reports_the_protocol_and_the_five_tools() {
    let (_t, repo, home) = fixture_repo();
    let s = talk(
        &repo,
        &home,
        &[
            init(),
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        ],
    );
    // The notification carries no id and must not be answered.
    assert_eq!(s.out.len(), 2, "a notification was answered: {:?}", s.out);

    let r = &s.out[0]["result"];
    assert_eq!(r["protocolVersion"], "2025-06-18");
    assert_eq!(r["serverInfo"]["name"], "brainiac");

    let tools = s.out[1]["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        [
            "context_pack",
            "search_code",
            "repo_map",
            "read_symbol",
            "reindex"
        ]
    );
    for t in tools {
        assert!(
            t["description"].as_str().is_some_and(|d| d.len() > 30),
            "{} needs a description saying when to use it",
            t["name"]
        );
        assert!(t["inputSchema"]["type"] == "object", "{} schema", t["name"]);
    }
}

#[test]
fn every_tool_returns_usable_content() {
    let (_t, repo, home) = fixture_repo();
    let s = talk(
        &repo,
        &home,
        &[
            init(),
            call(2, "repo_map", json!({"budget": 400})),
            call(
                3,
                "search_code",
                json!({"query": "parse_config", "limit": 3}),
            ),
            call(
                4,
                "context_pack",
                json!({"query": "how is config loaded", "budget": 600}),
            ),
            call(5, "read_symbol", json!({"name": "parse_config"})),
            call(6, "reindex", json!({})),
        ],
    );
    assert_eq!(s.out.len(), 6);
    for (i, reply) in s.out.iter().enumerate().skip(1) {
        let text = reply["result"]["content"][0]["text"].as_str().unwrap_or("");
        assert!(!text.trim().is_empty(), "tool {i} returned nothing");
        assert_eq!(
            reply["result"]["isError"], false,
            "tool {i} reported an error: {text}"
        );
    }
    // The ones that should name the code actually do.
    assert!(
        s.out[2]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("parse_config")
    );
    assert!(
        s.out[4]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("parse_config")
    );
}

#[test]
fn an_unknown_tool_is_reported_as_an_error() {
    let (_t, repo, home) = fixture_repo();
    let s = talk(
        &repo,
        &home,
        &[init(), call(2, "definitely_not_a_tool", json!({}))],
    );
    assert_eq!(
        s.out[1]["result"]["isError"], true,
        "an unknown tool must not look like a successful answer: {:?}",
        s.out[1]
    );
}

#[test]
fn progress_goes_to_stderr_and_never_into_the_protocol_stream() {
    let (_t, repo, home) = fixture_repo();
    let s = talk(&repo, &home, &[init()]);
    // talk() already asserts every stdout line parses as JSON-RPC.
    assert!(
        s.stderr.contains("brainiac:") && s.stderr.contains("indexed"),
        "indexing progress should be on stderr: {:?}",
        s.stderr
    );
    assert!(
        s.stderr.contains(&repo.to_string_lossy().to_string()) || s.stderr.contains("scope"),
        "stderr should name the scope so a misdirected registration is visible: {:?}",
        s.stderr
    );
}

#[test]
fn a_registration_pointed_at_a_missing_directory_fails_loudly() {
    let out = Command::new(env!("CARGO_BIN_EXE_brainiac"))
        .args(["-C", "/no/such/directory", "mcp"])
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    assert!(!out.status.success(), "a bad scope must exit non-zero");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("/no/such/directory"),
        "the path should be named: {err}"
    );
}
