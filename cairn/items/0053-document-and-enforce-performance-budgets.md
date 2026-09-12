---
id: 53
title: Document and enforce performance budgets
type: chore
status: backlog
milestone: v1.0
depends_on:
- 14
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: build
---

## Problem

The README quotes numbers from one laptop on one repository at one moment. They are already
one refactor away from being fiction, and performance is a feature here — the whole design
trades semantic recall for speed.

## Proposal

Pick the handful of numbers worth promising — cold index, warm re-index, query latency,
index size per thousand lines — and check them in CI against thresholds loose enough to
survive runner variance and tight enough to catch a tenfold regression.

## Acceptance criteria

- [ ] Budgets are stated for index, re-index, query and index size.
- [ ] CI fails when a budget is exceeded, with the measurement in the output.
- [ ] Thresholds tolerate runner variance without hiding a real regression.
- [ ] README figures are generated from the same measurement, not typed by hand.
