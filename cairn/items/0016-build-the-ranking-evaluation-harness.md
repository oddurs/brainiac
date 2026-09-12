---
id: 16
title: Build the ranking evaluation harness
type: feature
status: backlog
milestone: v0.2
depends_on:
- 15
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: l
area: rank
---

## Problem

`.claude/skills/tune-retrieval` tells a contributor to capture before/after output and read
every moved line by hand. That is the right instinct and the wrong mechanism: it does not
produce a number, so it cannot gate anything.

## Proposal

Implement whatever the evaluation spike decided: a labelled query set stored in the repo, a
runner that indexes the pinned corpora and scores the ranker, and a single summary line
that a human or CI can compare.

Keep it out of the main binary. This is a development tool.

## Acceptance criteria

- [ ] `scripts/task eval` (or an equivalent verb) runs the set and prints one score per repo.
- [ ] The labelled queries live in this repository as plain text, reviewable in a PR.
- [ ] A baseline score is recorded and committed.
- [ ] Running it twice on unchanged code produces the identical score.
- [ ] It completes fast enough to run before a PR, or it is split into quick and full modes.
