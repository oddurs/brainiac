---
id: 44
title: Return structured results from the MCP tools
type: feature
status: backlog
milestone: v0.3
depends_on:
- 25
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: m
area: mcp
---

## Problem

Every MCP tool returns one markdown blob. An agent that wants the third hit's path must
parse prose, and an agent that wants to know a pack was truncated cannot find out at all.

## Proposal

Return structured content alongside the text — paths, spans, scores, signals, token counts
and whether the budget bound — so an agent can act on the result without parsing.

## Acceptance criteria

- [ ] Each tool returns structured content with a documented schema.
- [ ] Text output remains for agents that only read text.
- [ ] `context_pack` reports tokens used, budget, and whether it was truncated.
- [ ] A test drives the server over stdio and validates the structure.
