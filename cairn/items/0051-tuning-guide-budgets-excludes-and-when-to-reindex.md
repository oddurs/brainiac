---
id: 51
title: 'Tuning guide: budgets, excludes, and when to reindex'
type: docs
status: backlog
milestone: v1.0
depends_on:
- 23
- 30
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: docs
---

## Problem

Once configuration exists, the questions become practical: what budget for what model, when
to exclude rather than tolerate, what the map share trades away, and whether a reindex is
ever needed by hand.

## Proposal

A short guide answering those from measurement rather than intuition, using the evaluation
corpora as worked examples.

## Acceptance criteria

- [ ] Budget guidance is backed by measured pack contents at several sizes.
- [ ] The map-share trade-off is shown with a worked before and after.
- [ ] Says plainly when a manual reindex is needed — ideally never.
- [ ] Every command in the guide runs as written.
