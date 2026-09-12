---
id: 13
title: Indexing must scope to the directory named, not the whole enclosing repo
type: bug
status: backlog
milestone: v0.1
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: index
---

## What happens

`config::discover` walks upward to the nearest `.git` and indexes from there. Running
`brainiac -C ~/Code/monorepo/packages/ui search "button"` indexes the entire monorepo —
every package, every vendored tree — when the user asked about one subtree. On a large
monorepo this turns a fast command into a very slow one and pollutes every result.

## What should happen

`-C` names the scope. The git root is still discovered, because churn and `HEAD` come from
it, but the walk and the index are confined to the directory given.

## Reproduction

1. `brainiac -C <monorepo>/packages/<one> index`
2. `brainiac -C <monorepo>/packages/<one> status` reports the whole monorepo's file count.

## Acceptance criteria

- [ ] The walk root is the directory passed to `-C`, defaulting to the git root when absent.
- [ ] Indexes are keyed by scope, so two subtrees do not share one database.
- [ ] `brainiac status` prints both the scope and the git root when they differ.
- [ ] A test indexes a subdirectory of a fixture repo and sees only that subtree.
