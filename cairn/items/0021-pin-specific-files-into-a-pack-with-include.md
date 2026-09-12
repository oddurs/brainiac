---
id: 21
title: Pin specific files into a pack with --include
type: feature
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: pack
---

## Problem

Sometimes you already know two files are relevant and want the ranker to fill in around
them. Today the only way to get a specific file into a pack is to hope it ranks.

## Proposal

`pack --include <path>` forces a file in, spends its cost from the budget first, and seeds
the PageRank teleport vector with it so the rest of the pack is chosen *relative* to what
was pinned. That second half is the interesting part and the reason this is not just
concatenation.

## Acceptance criteria

- [ ] Pinned files always appear, in full where the budget allows.
- [ ] Pinned files seed the graph, changing what else is selected — shown by a test.
- [ ] Pinning more than the budget allows fails with a clear message, not a silent truncation.
- [ ] The pack header lists what was pinned.
