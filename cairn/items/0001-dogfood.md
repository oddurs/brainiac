---
id: 1
key: v0.1
title: Dogfood
type: milestone
status: backlog
created: 2026-09-11
due: 2026-09-26
---

## Ships

Install brainiac, point it at a repo you own, and get a context pack worth pasting — from the CLI, the TUI, or an agent over MCP.

## Done when

- [ ] The repository exists on GitHub with the full tree committed and CI green.
- [ ] `cargo install --path .` works from a clean clone with no extra steps.
- [ ] The MCP server is registered in a real agent and answers `context_pack`.
- [ ] Two processes can touch one index concurrently without an error.
- [ ] Indexing scopes to the directory you name, not silently to the whole monorepo.
- [ ] `README.md` quickstart works verbatim for someone who has never seen it.

## Explicitly not in this milestone

- Configuration files. Defaults only.
- Any measurement of ranking quality — that is v0.2's whole point.
- Languages beyond the six already supported.
