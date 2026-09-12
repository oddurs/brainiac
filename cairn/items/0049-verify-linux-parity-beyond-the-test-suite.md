---
id: 49
title: Verify Linux parity beyond the test suite
type: chore
status: backlog
milestone: v1.0
depends_on:
- 9
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: build
---

## Problem

CI green on Linux proves the tests pass, not that the tool is pleasant there. The parts
likeliest to differ are the ones tests do not cover: the XDG data directory on a machine
with `XDG_DATA_HOME` set, terminal behaviour in the TUI across common emulators, locale and
non-UTF-8 filenames, and case-sensitive path handling.

## Proposal

Drive the whole tool by hand on Linux, including the TUI, across a couple of terminals.
Write down what differs from macOS.

## Acceptance criteria

- [ ] The TUI renders and restores correctly in at least two Linux terminals.
- [ ] `XDG_DATA_HOME` is honoured; the fallback path is correct when it is unset.
- [ ] Non-UTF-8 filenames do not abort an index run.
- [ ] Case-sensitive filesystem behaviour matches macOS for path filters.
- [ ] Any platform difference that remains is documented.
