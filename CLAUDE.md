# brainiac

A local context manager for repositories: index a repo, rank it against a question,
emit a token-budgeted context pack. Ships a CLI, a TUI browser, and an MCP server
from one binary.

## Commands

```
scripts/task check          # fmt:check + lint + test + build — must pass before a PR
scripts/task test           # cargo test --all-features
cargo test <name>           # a single test
cargo run -- search "q"     # against the enclosing repo
cargo run -- -C ~/other map # against another repo
```

## Architecture

Four stages, each usable alone: `index` (walk + hash + reparse only what changed) →
`parse` (tree-sitter tags → definitions, references, chunks) → `rank` (BM25 + symbol
match + personalised PageRank, fused with RRF) → `pack` (fit to a token budget).

Retrieval is deliberately embedding-free. Lexical, symbolic, and graph evidence cover
the cases embeddings would, with no model download, no network, and millisecond
queries. Do not add a vector store without a measured reason.

## Traps

- `chunk_fts` is an **external-content** FTS5 table. Deleting a chunk needs the
  `'delete'` command carrying the *old* column values — `store::clear_payload` does
  this. A bare `DELETE FROM chunks` leaves the search index lying.
- Tag queries capture one node under several roles: Rust reports an inherent method
  as both `definition.method` and `definition.function`. `parse::dedupe_defs` collapses
  them. Skipping it doubles every symbol count.
- Schema changes bump `SCHEMA_VERSION` in `src/store.rs:1` and drop the tables. There
  are no migrations; a rebuild costs seconds and removes a class of bugs.
- In `mcp` mode stdout carries the protocol. Every diagnostic goes to stderr.
- The index is not in the repo. It lives in the XDG data dir, named from a hash of the
  repo root, so nothing needs gitignoring.

## Ranking changes

Any edit to scoring — priors, fusion weights, chunking — is a behaviour change, not a
refactor. Follow `.claude/skills/tune-retrieval` and show before/after output on a real
repo, not only on the fixtures.

## Workflow

- Never commit to `main`. It advances only through a merged pull request.
- One unit of work, one worktree, one branch, one PR. The slug starts with the cairn id:
  `scripts/agent start fix/0013-scope-indexing`.
- `scripts/task check` must pass before a PR. The hooks enforce it; never `--no-verify`.
- Conventional Commits, subject ≤ 72 chars, imperative, no trailing period.
- No AI or assistant attribution in commits, PRs, comments, or docs.
- `ROADMAP.md` is generated. Edit `cairn/items/`, never the rendered file.

<!-- cairn:begin -->
## Roadmap and issues

This project tracks its roadmap and issues with `cairn`. Every item is a Markdown file under `cairn/items`, described by the schema in `cairn.toml`.

**Do not create ad-hoc TODO, PLAN or NOTES files.** Create a cairn item instead, so the work appears on the board and in the generated roadmap.

### The loop

1. `cairn next` — what is ready to start. It excludes anything blocked by unfinished dependencies and puts work already in progress first.
2. `cairn claim <ID>` — take it before you start, so no one duplicates the work. `cairn claim --next` picks and claims the top-ranked unclaimed item in one step, and prints its body so you can begin immediately.
3. Do the work. Record what you learn: `cairn set <ID> <field>=<value>` for fields, `cairn note <ID> "<TEXT>"` for anything that needs a sentence — why you chose something, what you tried, what to watch for.
4. `cairn close <ID>` when it is done, or `cairn release <ID>` to hand it back.
5. `cairn check` before you report finished. It must pass.

### Commands

```sh
cairn next --json                 # ready work, ranked
cairn claim --next                # take the next ready item
cairn search <TEXT> --json        # titles, bodies and labels
cairn list --json                 # all open items
cairn list --filter 'blocked=false,priority=p0'
cairn show <ID> --json            # one item, including its body
cairn new "<TITLE>" --type <TYPE> --milestone <MILESTONE>
cairn set <ID> status=<STATUS>    # also labels+=x, or any field below
cairn note <ID> "<TEXT>"          # append reasoning; never replaces
cairn close <ID>
cairn check                       # validate; run before finishing
cairn render                      # regenerate ROADMAP.md
```

### Schema

- **Types**: `feature`, `bug`, `spike`, `chore`, `docs`, `milestone`
- **Statuses**: `backlog` (open), `planned` (open), `doing` (active), `blocked` (active), `done` (done), `dropped` (dropped)
- **`due`**: date, YYYY-MM-DD — when a milestone is meant to land
- **`part_of`**: names any items, by id, several allowed — a larger piece of work this belongs to
- **`priority`**: one of p0, p1, p2, p3 — p0 is a release blocker
- **`effort`**: one of s, m, l, xl — Rough size, not an estimate
- **`area`**: one of index, parse, rank, pack, store, cli, tui, mcp, config, build, docs — Subsystem this touches
- **Milestones**: `v0.1` (due 2026-09-26), `v0.2` (due 2026-10-24), `v0.3` (due 2026-11-21), `v1.0` (due 2027-01-16), `later`
- **Saved views** (`cairn list --view NAME`): `now`, `next`, `blockers`, `unknowns`, `triage`

### Rules

1. Before starting work, find or create the item and set it to an active status.
2. Use the fields above rather than inventing new ones; add new fields to `cairn.toml` first.
3. Never hand-edit the generated roadmap file — change items and run `cairn render`.
4. `cairn check` must pass before the work is considered done.

<!-- cairn:end -->
