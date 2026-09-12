---
id: 10
title: cargo install from a clean clone works with no extra steps
type: chore
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-09-12
depends_on:
- 6
created: 2026-09-11
updated: 2026-09-12
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

- [x] `git clone` to a temp dir, `cargo install --path .`, `brainiac --version` — all work.
- [x] The installed binary indexes and packs an unrelated repo.
- [x] No dev-dependency is required to build the binary.
- [x] Cold build time is recorded here, so later dependency growth is visible.

## 2026-09-12

Verified from a cold clean clone of the public repo: 'cargo install --path . --locked' succeeded in 22.3s wall (82s user, parallel), producing a working binary that indexed and packed an unrelated 204k-line repo it had never seen. No dev-dependency is built for the binary. Recorded here so later dependency growth is visible.
