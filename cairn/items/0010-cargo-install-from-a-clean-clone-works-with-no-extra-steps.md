---
id: 10
title: cargo install from a clean clone works with no extra steps
type: chore
status: backlog
milestone: v0.1
depends_on:
- 6
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: s
area: build
---

## Problem

Install has only been exercised from the development tree, where `target/` is warm and
dev-dependencies are present. The README tells a stranger to run `cargo install --path .`
and that path has never been tested cold.

## Proposal

Clone into a temporary directory, install, and run the resulting binary from
`~/.cargo/bin` against a repository it has never seen.

## Acceptance criteria

- [ ] `git clone` to a temp dir, `cargo install --path .`, `brainiac --version` — all work.
- [ ] The installed binary indexes and packs an unrelated repo.
- [ ] No dev-dependency is required to build the binary.
- [ ] Cold build time is recorded here, so later dependency growth is visible.
