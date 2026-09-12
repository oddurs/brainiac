---
id: 31
title: Index a monorepo once and query it per package
type: feature
status: backlog
milestone: v0.3
depends_on:
- 13
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: l
area: index
---

## Problem

Scoping (v0.1) fixed the wrong-results problem by confining the walk, at the cost of
indexing the same monorepo many times — once per subtree — with no sharing. For a large
monorepo that is both slow and wasteful, and cross-package references become invisible
exactly where they matter most.

## Proposal

One index for the repository, with a package dimension on files. Queries scope by package
but the reference graph stays whole, so a hit in `packages/ui` can still be pulled in by a
seed in `packages/app`. Detect package roots from the obvious manifests
(`Cargo.toml` workspaces, `package.json` workspaces, `go.work`, `pyproject.toml`).

## Acceptance criteria

- [ ] One index covers the monorepo; `--path` and package scoping filter it.
- [ ] Graph edges cross package boundaries and a test proves a cross-package pull-in.
- [ ] Package roots are detected from at least Cargo, npm and Go workspaces.
- [ ] Indexing time is materially better than indexing each package separately — measured.
