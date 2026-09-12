---
id: 30
title: Document .brainiac.toml
type: docs
status: backlog
milestone: v0.2
depends_on:
- 18
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: docs
---

## Problem

A configuration file nobody can find is a configuration file nobody uses, and the defaults
it overrides are heuristics that will be wrong for some repository.

## Proposal

A README section and a commented example covering every key, why the built-in path priors
exist, and the two cases that actually come up: a vendored tree the walker cannot infer,
and a test directory that is the point rather than the noise.

## Acceptance criteria

- [ ] Every key is documented with its default and its effect.
- [ ] A worked example shows tuning for a repo where tests are the interesting code.
- [ ] The example file is committed and parses — checked by a test.
