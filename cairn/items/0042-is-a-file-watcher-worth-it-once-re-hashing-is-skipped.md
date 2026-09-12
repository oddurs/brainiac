---
id: 42
title: Is a file watcher worth it once re-hashing is skipped?
type: spike
status: backlog
milestone: v0.3
depends_on:
- 22
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: m
area: index
---

## Question

Every command re-indexes before answering, which costs a full directory walk. The mtime fast
path removes the read cost but not the walk. A watcher would remove the walk too — at the
price of a daemon, platform-specific APIs, and a whole class of staleness bugs when the
watcher misses an event.

Is the remaining walk actually slow enough to justify that?

## Timebox

One day, after the mtime fast path lands, and not before.

## How to answer it

- [ ] Measure walk-only time on repos of 10k, 100k and 1M files.
- [ ] Establish the latency a user actually notices for a `search` invocation.
- [ ] Cost the alternatives: watcher daemon, a staleness TTL, or an explicit `--no-index` flag.
- [ ] State what the failure mode of a missed event would be and whether it is acceptable.

## Answer

_Unanswered._
