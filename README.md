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
cargo install --path . --locked
```

Requires a Rust toolchain. `--locked` builds against the committed `Cargo.lock`.
From a cold clone that takes about 25 seconds and produces a 9.5 MiB binary — most of
which is the six compiled-in tree-sitter grammars.

Indexes live in your XDG data directory, keyed by a hash of the indexed directory, so
nothing lands in the repo.

## What gets indexed

`-C` names the directory to index. Without it, the scope is the enclosing git
repository, so running from a subdirectory still sees everything:

```sh
cd src && brainiac search "…"     # the whole repository
brainiac -C packages/ui search "…"  # only that package
```

The two therefore differ: `brainiac -C .` inside `src/` indexes `src/` alone, while a
bare `brainiac` there indexes the repository. Each scope gets its own index file.

The enclosing repository is still located either way — recency and the commit id come
from it even when only a subtree is indexed.

One sharp edge: `.gitignore` rules do not apply to the scope root itself, so
`-C node_modules/foo` will index it. Rules *below* the scope are honoured normally.

## Use it from an agent

```sh
claude mcp add brainiac -- brainiac -C /path/to/repo mcp
```

**`-C` is the scope, and it is not optional in practice.** The server indexes the
directory you name — point it at a package inside a monorepo and it will only ever see
that package. Without `-C` the scope is whatever directory the agent happened to launch
the server from, which is rarely what you want. Use an absolute path.

Five tools:

| | |
|---|---|
| `context_pack` | a token-budgeted map plus the code most likely to answer a question |
| `search_code` | ranked hybrid search, returning spans with their source |
| `repo_map` | the skeleton, for orienting in unfamiliar code |
| `read_symbol` | every definition with an exact name |
| `reindex` | re-scan, if files changed during the session |

The index is brought up to date when the server starts and on demand, so a long agent
session does not go stale.

Progress and diagnostics go to **stderr**; stdout carries nothing but JSON-RPC. The
startup line names the scope, which is the one place a misdirected registration becomes
visible:

```
brainiac: scope /path/to/repo — indexed 82 files (82 reparsed) in 45ms
```

A registration pointing at a directory that does not exist exits non-zero naming the
path, rather than starting and answering every question with nothing.

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
