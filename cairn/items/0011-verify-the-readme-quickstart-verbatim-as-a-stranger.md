---
id: 11
title: Verify the README quickstart verbatim, as a stranger
type: docs
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 10
created: 2026-09-11
updated: 2026-09-12
priority: p0
effort: s
area: docs
---

## Problem

The README's commands were written alongside the code and have never been run in order,
from scratch, by someone without the author's context. Quickstarts rot silently.

## Proposal

Follow the README top to bottom on a fresh machine or container: install, index, search,
pack, browse, register over MCP. Fix the document wherever reality disagrees, rather than
fixing reality to match the document.

## Acceptance criteria

- [x] Every command block in `README.md` runs as written and produces the described output.
- [x] The performance table matches a fresh measurement, or is corrected.
- [x] A first-time user reaches a useful context pack without reading the source.

## 2026-09-12

Ran every README command verbatim from a fresh clone using a binary installed from a clean clone, with HOME/XDG_DATA_HOME redirected so nothing touched real state: index, search, map, pack (7617 tokens against a 8000 budget), pack | pbcopy, browse (pty, renders and exits 0), mcp (handshake). Performance table was stale and is now measured: full index 1.0s (was 2s), re-index 80ms (was 300ms), search 0.11s end to end (was 0.6s), index 55 MiB. The old TUI keystroke figure was removed rather than restated, because I could not measure it reliably in a harness.
