---
id: 20
title: Filter search and pack by path prefix
type: feature
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: cli
---

## Problem

There is no way to ask a question about one part of a repository. On anything large the
answer is often "in `src/server`, how does X work" and the only recourse is to read past
the hits from everywhere else.

## Proposal

`--path <prefix>` on `search` and `pack`, repeatable, applied as a filter on candidates
before fusion so the ranking is computed over the filtered set rather than filtered after
truncation.

## Acceptance criteria

- [ ] `--path src/server` restricts results to that prefix.
- [ ] Repeating the flag unions the prefixes.
- [ ] Filtering happens before truncation, verified by a test.
- [ ] A prefix matching nothing says so rather than printing an empty list.
