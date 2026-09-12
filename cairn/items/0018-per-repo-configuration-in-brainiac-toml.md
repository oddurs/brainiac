---
id: 18
title: Per-repo configuration in .brainiac.toml
type: feature
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: config
---

## Problem

`store::path_prior` hardcodes what is probably uninteresting — `vendor/`, `/versions/`,
`/generated/`, test paths. Those heuristics were derived from a handful of repositories and
will be wrong somewhere. There is no way to tell brainiac that `docs/api-snapshots/` is
noise, or that this project's `tests/` really are the interesting part.

## Proposal

An optional `.brainiac.toml` at the scope root: gitignore-style `exclude` globs applied on
top of `.gitignore`, an `include` escape hatch, and per-repo defaults for budget and
map share. Use `ignore::overrides::OverrideBuilder` so the glob semantics match the walker
already in use and no new dependency is added.

## Acceptance criteria

- [ ] `exclude` globs remove paths from the index, verified by a test.
- [ ] `include` re-admits a path that `.gitignore` or `exclude` would have dropped.
- [ ] Default budget and map share can be set per repo and are overridden by CLI flags.
- [ ] A malformed config fails with the file, the line, and what was expected.
- [ ] Absent config behaves exactly as today.
