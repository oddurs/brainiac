---
id: 28
title: Open the selected hit in $EDITOR from the browser
type: feature
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: s
area: tui
---

## Problem

The browser finds the right span and then the workflow stops: you read the path off the
screen and retype it somewhere else.

## Proposal

A key that suspends the TUI, opens `$EDITOR` at the file and line, and restores the browser
when the editor exits. Use the conventional `+LINE` / `--line` argument shapes for the
common editors and fall back to opening the file plainly.

## Acceptance criteria

- [ ] The editor opens at the correct line for the selected hit.
- [ ] The terminal is restored correctly on return, including after the editor crashes.
- [ ] `$EDITOR` unset gives a clear message instead of a panic.
- [ ] The binding appears in the help overlay.
