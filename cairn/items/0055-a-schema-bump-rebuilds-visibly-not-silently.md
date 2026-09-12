---
id: 55
title: A schema bump rebuilds visibly, not silently
type: feature
status: backlog
milestone: v1.0
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: store
---

## Problem

`SCHEMA_VERSION` mismatch drops every table and rebuilds — deliberately, and correctly. But
it happens without a word, so a user who upgrades and then waits six seconds for what is
usually a 300ms command has no idea why.

## Proposal

Say it: one line naming the old and new schema versions and the fact that a rebuild is
happening. Keep the behaviour exactly as it is.

## Acceptance criteria

- [ ] A version mismatch prints one line explaining the rebuild before it starts.
- [ ] The message goes to stderr and never corrupts MCP stdout.
- [ ] A test asserts the message on a forced version change.
