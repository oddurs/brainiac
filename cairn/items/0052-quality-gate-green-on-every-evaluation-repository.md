---
id: 52
title: Quality gate green on every evaluation repository
type: chore
status: backlog
milestone: v1.0
depends_on:
- 17
- 36
- 41
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: s
area: rank
---

## Problem

The gate lands in v0.2 against whatever corpora exist then. By v1.0 the ranker has taken
import resolution, prose restructuring and four new languages — each of which could improve
one corpus while quietly damaging another.

## Proposal

Run the full set, confirm every repository is at or above its committed baseline, and
investigate any that regressed rather than re-baselining it.

## Acceptance criteria

- [ ] Every evaluation repository is at or above baseline.
- [ ] Any repository that regressed has a written explanation and a decision.
- [ ] Baselines are re-committed with the v1.0 scores.
