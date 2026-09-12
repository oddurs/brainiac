---
id: 26
title: Golden-file tests for pack layout
type: chore
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: m
area: pack
---

## Problem

`pack::build` decides section order, headings, fences and the token comment. Tests cover
the budget and that the right code appears, but nothing pins the *shape* — so a formatting
change that breaks a downstream consumer passes.

## Proposal

Commit golden packs generated from the existing fixture repo at several budgets, and diff
against them. Regenerate deliberately with an environment variable, never automatically.

## Acceptance criteria

- [ ] Goldens exist for a map-only pack, a code-heavy pack, and a tiny budget.
- [ ] A layout change fails with a readable diff.
- [ ] Regeneration is one documented command.
- [ ] Goldens are small enough to review in a pull request.
