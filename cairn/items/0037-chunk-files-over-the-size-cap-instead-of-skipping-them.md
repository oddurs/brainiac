---
id: 37
title: Chunk files over the size cap instead of skipping them
type: feature
status: backlog
milestone: v0.3
depends_on:
- 18
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: m
area: index
---

## Problem

`MAX_BYTES` is 1MiB and anything larger is dropped entirely. Generated clients, large
schema files and long changelogs are exactly the files somebody asks about, and they vanish
without trace.

## Proposal

Parse up to a cap and window the remainder, or window the whole file when parsing would be
too slow. Record that the file was truncated so it can be reported.

## Acceptance criteria

- [ ] A 10MB source file is indexed, with its first section fully parsed.
- [ ] Truncated files are marked and reported by `brainiac status`.
- [ ] Index time on the reference repo does not regress measurably.
- [ ] The cap is configurable in `.brainiac.toml`.
