---
id: 23
title: Retune the priors against the evaluation set
type: chore
status: backlog
milestone: v0.2
depends_on:
- 16
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: rank
---

## Problem

Every constant in `rank.rs` — `RRF_K`, the 0.8 and 0.45 fusion weights, the 0.12 size
exponent, the 0.18 churn exponent, `UBIQUITY_CUTOFF`, `kind_prior` — was set by judgement
on one repository. Several are probably wrong and none has ever been varied to find out.

## Proposal

With the harness in place, sweep each constant independently, record the score curve, and
choose values with evidence. Where a constant turns out not to matter, say so in a comment
and stop pretending it is tuned.

## Acceptance criteria

- [ ] Each constant has a recorded score curve in the item or a committed note.
- [ ] Values that changed are justified by a number, not a preference.
- [ ] Constants that do not move the score are documented as insensitive.
- [ ] The baseline score improves, or the item closes explaining why it could not.
