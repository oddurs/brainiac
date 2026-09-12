---
id: 19
title: Explain the per-signal contribution behind every hit
type: feature
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: rank
---

## Problem

`Hit::signals` prints `LSG`, which says *which* signals fired but not *how much* each
mattered. When a result is wrong, that is the difference between a diagnosis and a guess —
and it is the first thing anyone tuning the ranker needs.

## Proposal

`brainiac search --explain` prints, per hit, the rank contributed by each of the three
lists, the priors applied, and the fused total. Same data over MCP and in the TUI preview
header, since the TUI is where wrongness is usually noticed.

## Acceptance criteria

- [ ] `--explain` shows lexical rank, symbol rank, graph rank, each prior, and the total.
- [ ] The numbers reconcile: the parts sum to the printed score.
- [ ] A hit present only via the graph explains which seed file pulled it in.
- [ ] Default output is unchanged.
