---
id: 38
title: Report what was skipped and why
type: feature
status: backlog
milestone: v0.3
depends_on:
- 12
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: s
area: cli
---

## Problem

Three separate rules silently drop files — unknown extension, over the size cap, unreadable
bytes — and none leaves a record. A user whose file never appears in results has no way to
learn which rule ate it.

## Proposal

Count skips by reason during the walk and surface them in `brainiac status`, with
`--skipped` listing the paths.

## Acceptance criteria

- [ ] `status` shows skip counts by reason.
- [ ] `status --skipped` lists paths with the rule that dropped each.
- [ ] Counts are stored with the index, not recomputed by a second walk.
