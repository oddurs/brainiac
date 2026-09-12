---
id: 40
title: Batch index writes so one huge transaction does not hold the database
type: chore
status: backlog
milestone: v0.3
depends_on:
- 14
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: store
---

## Problem

`index::run` opens a single transaction covering every insert for the whole repository. For
a small repo that is exactly right — it is atomic and fast. For a very large one it means a
long write lock, unbounded WAL growth, and no partial progress if it is interrupted.

The large-repository spike should say whether this is theoretical or real.

## Proposal

Commit in batches of a size chosen from the spike's measurements, keeping per-file work
atomic so an interrupted run leaves a consistent, if partial, index that the next run
completes.

## Acceptance criteria

- [ ] Batch size is one named constant justified by the spike's numbers.
- [ ] An interrupted index leaves a consistent index and the next run finishes the job.
- [ ] Peak WAL size on the largest test repository is bounded and recorded.
- [ ] Index time on the reference repo does not regress.
