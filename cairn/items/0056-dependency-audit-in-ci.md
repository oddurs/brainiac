---
id: 56
title: Dependency audit in CI
type: chore
status: backlog
milestone: v1.0
depends_on:
- 9
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: s
area: build
---

## Problem

Sixteen direct dependencies, five of them compiling C. A personal tool in a public
repository should at least know when one of them has a published advisory.

## Proposal

`cargo-deny` in CI for advisories, licences and duplicate versions. Fail on advisories, warn
on the rest.

## Acceptance criteria

- [ ] `cargo deny check` runs on every pull request.
- [ ] Advisories fail the build; licence and duplicate findings warn.
- [ ] Every current dependency passes or has a recorded exception with a reason.
