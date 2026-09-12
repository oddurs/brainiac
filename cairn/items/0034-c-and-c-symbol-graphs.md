---
id: 34
title: C and C++ symbol graphs
type: feature
status: backlog
milestone: v0.3
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: parse
---

## Problem

C and C++ are missing, and they are the hardest of the set: the preprocessor means the
parsed tree is not the compiled tree, and headers separate declaration from definition in a
way the current one-name-one-definition model does not express.

## Proposal

Add both grammars. Accept that macro-heavy code will parse imperfectly and measure how
imperfectly rather than assuming. Decide explicitly how a header declaration and its
implementation relate in the graph — probably as two definitions of one name, which the
ubiquity cutoff already tolerates.

## Acceptance criteria

- [ ] `.c`, `.h`, `.cc`, `.cpp`, `.hpp` produce functions, structs, classes and references.
- [ ] Header/implementation pairs do not double-count in the repo map.
- [ ] Verified on a real C and a real C++ repository.
- [ ] Known preprocessor limitations are written down in the item and the docs.
