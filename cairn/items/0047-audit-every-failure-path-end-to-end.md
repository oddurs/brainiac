---
id: 47
title: Audit every failure path end to end
type: chore
status: backlog
milestone: v1.0
depends_on:
- 29
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: cli
---

## Problem

v0.2 fixed the errors that were known to be bad. v1.0 needs the ones nobody has hit:
a full disk mid-index, a repository deleted between walk and read, a file that changes while
being parsed, an interrupted index, a `$HOME` that is not writable.

## Proposal

Enumerate the failure modes deliberately rather than waiting to meet them. Fault-inject
where practical, reason carefully where not, and make every one exit non-zero with a message
naming the cause.

## Acceptance criteria

- [ ] A written list of failure modes with the observed behaviour of each.
- [ ] No failure mode panics, aborts, or exits zero.
- [ ] An interrupted index leaves the next run able to complete the work.
- [ ] An unwritable data directory is reported with the path and a suggested fix.
