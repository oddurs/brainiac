---
id: 39
title: Cache git churn instead of spawning git on every index run
type: chore
status: backlog
milestone: v0.3
depends_on:
- 22
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: s
area: index
---

## Problem

`git_churn` runs `git log --since=120.days --name-only` on every index pass, including the
one every CLI command performs. On a repository with deep history that is a large process
spawn for data that changes only when HEAD does.

## Proposal

Key the churn map on HEAD and recompute only when HEAD moves. Store it with the index.

## Acceptance criteria

- [ ] `git` is not invoked when HEAD is unchanged since the last index run.
- [ ] Moving HEAD recomputes churn on the next run.
- [ ] A repository with no git, or with git absent from PATH, still indexes.
