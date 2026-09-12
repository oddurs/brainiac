---
id: 46
title: Freeze and version the MCP tool schemas
type: chore
status: backlog
milestone: v1.0
depends_on:
- 44
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: s
area: mcp
---

## Problem

The five tool schemas are hand-written JSON literals in `mcp.rs`. Agents build prompts
around tool names and argument shapes; changing one silently breaks every session that
learned the old shape.

## Proposal

Fix the schemas, declare the protocol version supported, and add a test that fails when a
schema changes without a deliberate version bump.

## Acceptance criteria

- [ ] Tool names and argument schemas are settled and documented.
- [ ] A snapshot test fails on any unintended schema change.
- [ ] The supported protocol version is asserted in the handshake test.
- [ ] Tool descriptions say when to use each tool, not just what it does.
