---
id: 59
title: LSP-backed symbol resolution
type: feature
status: backlog
milestone: later
depends_on:
- 41
created: 2026-09-11
updated: 2026-09-11
priority: p3
effort: xl
area: rank
---

## Problem

Import resolution gets edges mostly right. A language server gets them exactly right — it
knows types, generics, re-exports, and which overload was meant.

## Proposal

Optionally consult a running language server for resolution when one is available, falling
back to the static graph when it is not.

This is deliberately left `xl` and undecomposed: it should only be broken down if import
resolution proves insufficient in practice, and it may never be.

## Acceptance criteria

- [ ] Decide whether static resolution is in fact insufficient, with evidence.
- [ ] If so, decompose this into real items and close it.
