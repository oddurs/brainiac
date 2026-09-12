# brainiac

Ranked, token-budgeted context for your own repositories. One Rust binary: a CLI, a
TUI browser, and an MCP server.

Point it at a repo and ask a question. It answers with a reference-ranked map of the
codebase plus the code most likely to matter — sized to a token budget you choose.

```sh
brainiac index                       # build or refresh the index
brainiac search "how does auth work" # ranked file:line spans
brainiac pack "add a retry to the client" -b 8000 | pbcopy
brainiac map -b 3000                 # the repo's skeleton, ranked
brainiac browse                      # interactive
brainiac mcp                         # serve to an agent over stdio
```

## How it ranks

Three independent kinds of evidence, fused with reciprocal rank fusion:

| Signal | What it knows |
|---|---|
| `L` lexical | BM25 over chunk text, via SQLite FTS5 |
| `S` symbol | fuzzy match against definition names, camelCase-aware |
| `G` graph | personalised PageRank over the definition/reference graph |

The graph is what makes it more than grep: a file that never mentions your query but
is referenced by everything that does still surfaces. Seeded by the lexical and symbol
hits, the walk answers "what else would I need open to work on this?"

Priors then demote what is rarely the subject — vendored trees, version-pinned doc
copies, generated output — and promote what you have been editing lately, read from
`git log`. Byte-identical bodies collapse to one hit.

No embeddings, no model download, no network. The whole thing works on a plane.

## Install

```sh
cargo install --path .
```

Requires a Rust toolchain. Indexes live in your XDG data directory, keyed by a hash of
the repo root, so nothing lands in the repo.

## Use it from an agent

```sh
claude mcp add brainiac -- brainiac -C /path/to/repo mcp
```

Five tools: `context_pack`, `search_code`, `repo_map`, `read_symbol`, `reindex`. The
server brings the index up to date on start and on demand.

## Languages

Rust, Python, JavaScript, TypeScript, TSX, and Go get a full symbol graph through
tree-sitter tag queries. Markdown, config, and other text are indexed for search with
sliding windows. Anything else is skipped.

## Performance

Measured on a 471k-line, 1560-file repository on an M-series laptop:

| | |
|---|---|
| full index from cold | ~2 s |
| re-index, nothing changed | ~300 ms |
| `search` end to end, including the freshness check | ~0.6 s |
| keystroke in the TUI, where the graph is held open | 60–200 ms |

Every command re-indexes incrementally before answering, because a stale answer costs
more than the wait. The index is ~45 MB for that repo.

## Development

```sh
scripts/setup        # wire up git hooks
scripts/task check   # fmt, lint, test, build
scripts/agent start feat/the-thing
```

## Licence

MIT
