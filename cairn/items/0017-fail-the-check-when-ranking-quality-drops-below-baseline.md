---
id: 17
title: Fail the check when ranking quality drops below baseline
type: chore
status: backlog
milestone: v0.2
depends_on:
- 16
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: build
---

## Problem

A harness nobody runs is documentation. The three regressions already found were caught by
one person looking closely at one repository on one afternoon.

## Proposal

Add the evaluation run to `scripts/task check` — or to a `scripts/task eval` that CI runs
separately if it is too slow for the pre-push hook — and fail when the score drops more than
a stated tolerance below the committed baseline.

Improving the baseline is a deliberate commit, so a genuine improvement is visible in the
diff rather than silently absorbed.

## Acceptance criteria

- [ ] A drop beyond tolerance fails the build with the before and after scores.
- [ ] The tolerance is one named constant with a comment explaining the value.
- [ ] Raising the baseline requires editing a committed file.
- [ ] The gate runs in CI on every pull request.
