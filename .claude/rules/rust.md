---
paths:
  - "**/*.rs"
---

# Rust

- `clippy` runs with `-D warnings`. A lint you genuinely need to break gets
  `#[allow(...)]` on the narrowest item with a comment saying why.
- Propagate with `anyhow::Result` and add context at the boundary where a path or name
  is still in scope: `.with_context(|| format!("opening index at {}", p.display()))`.
  Never `unwrap()` outside tests.
- Prefer `let ... else { continue }` over nested `if let` in loops; the parser and
  ranker are full of partial matches and the flat form stays readable.
- SQLite is single-writer. Reads take `&Connection`, writes take `&Transaction`, and a
  whole index pass commits once.
- Parallelism is rayon over pure parse work only. Nothing touching the connection runs
  off the main thread.
