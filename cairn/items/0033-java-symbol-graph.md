---
id: 33
title: Java symbol graph
type: feature
status: backlog
milestone: v0.3
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: parse
---

## Problem

Java is absent. It is also the language whose tags queries are most likely to behave, being
heavily classed and explicitly imported.

## Proposal

Add `tree-sitter-java`, wire it into the registry, and verify against a real Java repository
rather than a fixture — the TypeScript gap was only found that way.

## Acceptance criteria

- [ ] `.java` files produce classes, interfaces, methods and call references.
- [ ] Verified on a real repository: symbol count is plausible for its size.
- [ ] A fixture test covers class, interface, method and a cross-file reference.
- [ ] Binary size and cold build time impact are recorded.
