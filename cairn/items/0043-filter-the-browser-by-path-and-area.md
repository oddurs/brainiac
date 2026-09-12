---
id: 43
title: Filter the browser by path and area
type: feature
status: backlog
milestone: v0.3
depends_on:
- 20
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: m
area: tui
---

## Problem

The TUI searches the whole scope with no way to narrow. Once `--path` exists on the CLI the
browser is the interface that most needs it, because narrowing is an iterative act and the
browser is where iteration happens.

## Proposal

A filter line below the query, toggled by a key, applying the same path filter as the CLI
and re-ranking live.

## Acceptance criteria

- [ ] A path filter narrows results and updates as it is typed.
- [ ] The active filter is visible and dismissible.
- [ ] Filtering happens before truncation, as on the CLI.
- [ ] The binding appears in the help overlay.
