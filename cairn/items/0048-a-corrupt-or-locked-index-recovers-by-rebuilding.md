---
id: 48
title: A corrupt or locked index recovers by rebuilding
type: feature
status: backlog
milestone: v1.0
depends_on:
- 7
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: store
---

## Problem

`Store::open` handles a schema mismatch by dropping and rebuilding, which is the right
instinct. It does not handle a *corrupt* database — a truncated file, a bad page, a WAL from
a killed process — and those surface as raw SQLite errors on a path the user cannot act on.

## Proposal

Run an integrity check when opening fails or reports corruption, and rebuild from scratch
with one line of explanation. The index is a cache; losing it costs seconds and should never
require the user to know where it lives.

## Acceptance criteria

- [ ] A deliberately truncated index file rebuilds automatically, with a message.
- [ ] A stale WAL from a killed process recovers without intervention.
- [ ] Rebuilding never deletes anything outside the index directory.
- [ ] Tests cover truncation and a corrupted page.
