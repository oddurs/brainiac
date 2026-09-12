---
id: 50
title: 'Architecture page: what L, S and G actually do'
type: docs
status: backlog
milestone: v1.0
depends_on:
- 19
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: docs
---

## Problem

The README says three signals are fused with RRF. That is enough to use the tool and not
enough to trust it, tune it, or contribute to it — and the reasoning behind the design
currently lives in one person's head and a handful of code comments.

## Proposal

One page: the four stages, why retrieval is embedding-free, what personalised PageRank is
computing over this graph, why priors multiply after the power iteration rather than inside
it, and what each constant controls. Diagram the seed-and-walk, which is the part that is
hard to explain in prose.

## Acceptance criteria

- [ ] Each stage is explained with its inputs, outputs and the decision behind it.
- [ ] The seeding and walk are illustrated.
- [ ] Every tunable constant is named with what it controls.
- [ ] Someone who has not seen the code can explain why a result ranked where it did.
