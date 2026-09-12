---
id: 45
title: Freeze and document the CLI surface
type: docs
status: backlog
milestone: v1.0
depends_on:
- 20
- 21
- 24
- 25
- 27
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: cli
---

## Problem

Flags have accumulated one at a time. v1.0 promises a surface worth depending on, and that
means someone deciding, once, what every command and flag is called and what it guarantees —
before a shell function somewhere depends on the accident of what happened to be convenient.

## Proposal

Review every command and flag together. Rename what is wrong now, while it is still free.
Document each one with its default, and state which parts are covered by the 1.x stability
promise and which are explicitly not.

## Acceptance criteria

- [ ] Every command and flag is documented with its default and effect.
- [ ] Naming is consistent across commands — one word for one concept.
- [ ] The stability promise states what may change in a 1.x release.
- [ ] `--help` for every subcommand is complete and accurate, checked by a test.
