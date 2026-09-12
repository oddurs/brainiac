---
id: 35
title: Ruby symbol graph
type: feature
status: backlog
milestone: v0.3
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: s
area: parse
---

## Problem

Ruby is missing and its tags query is well established upstream.

## Proposal

Add `tree-sitter-ruby` and verify on a real repository.

## Acceptance criteria

- [ ] `.rb` files produce modules, classes, methods and call references.
- [ ] Verified on a real repository.
- [ ] A fixture test covers module, class, method and a cross-file reference.
