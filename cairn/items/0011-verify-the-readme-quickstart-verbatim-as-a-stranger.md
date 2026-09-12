---
id: 11
title: Verify the README quickstart verbatim, as a stranger
type: docs
status: backlog
milestone: v0.1
depends_on:
- 10
created: 2026-09-11
updated: 2026-09-11
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

- [ ] Every command block in `README.md` runs as written and produces the described output.
- [ ] The performance table matches a fresh measurement, or is corrected.
- [ ] A first-time user reaches a useful context pack without reading the source.
