---
id: 61
title: Search across several repositories at once
type: feature
status: backlog
milestone: later
created: 2026-09-11
updated: 2026-09-11
priority: p3
effort: l
area: cli
---

## Problem

Questions sometimes span repositories — a client and its server, a library and the app that
consumes it. Today that means two invocations and manual assembly.

## Proposal

A named set of repositories queried together, with results attributed by repo and the graph
kept per-repo rather than merged, since cross-repo name collisions would be severe.

## Acceptance criteria

- [ ] Decide whether this is genuinely useful or just appealing.
- [ ] If useful, decompose with a milestone.
