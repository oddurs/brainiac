---
id: 36
title: Verify every language against a real repository, not a fixture
type: chore
status: backlog
milestone: v0.3
depends_on:
- 16
- 33
- 34
- 35
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: parse
---

## Problem

TypeScript shipped half-blind — its bundled tags query is a supplement to the JavaScript
one, and used alone it misses every function, class and call site. A fixture test would not
have caught it; a real repository did, immediately, via an implausible symbol count.

This is a class of bug, not an incident, and every added language can have it.

## Proposal

A check that indexes one real repository per supported language and asserts symbols-per-line
falls in a plausible band, plus a report of definitions by kind so a missing category is
visible. Run it in the evaluation harness rather than the unit suite, since it needs corpora.

## Acceptance criteria

- [ ] Every supported language has a named reference repository.
- [ ] Symbols-per-line outside the expected band fails with the numbers.
- [ ] Definition kinds are reported per language, so a missing kind is obvious.
- [ ] Adding a language without adding its reference repository fails the check.
