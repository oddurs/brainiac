---
id: 29
title: Every error names the path and what to do about it
type: chore
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: cli
---

## Problem

`anyhow` context is applied where it was convenient, not where it is needed. A missing
repository, an unreadable index, a malformed query and an absent `git` all surface
differently, and some surface as a bare I/O message with no indication of which file.

## Proposal

Walk every `?` on a user-reachable path and add context naming the thing that failed and
the next action. Add a test per failure mode asserting the message, not just the exit code.

## Acceptance criteria

- [ ] A non-existent `-C` path names the path and exits 2.
- [ ] An unreadable or locked index says which file and suggests `--clean`.
- [ ] A query that is only punctuation says so rather than returning nothing.
- [ ] `git` missing degrades to no churn signal with one warning, not a failure.
- [ ] Each of the above has a test asserting the message text.
