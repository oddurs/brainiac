---
id: 9
title: Build and test on macOS and Linux in CI
type: chore
status: backlog
milestone: v0.1
depends_on:
- 6
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: build
---

## Problem

`.github/workflows/ci.yml` runs on `ubuntu-latest` only. macOS and Linux are both named
platforms, and they differ in exactly the places this project touches: path
canonicalisation, the XDG data directory, and whether `git` is on PATH.

## Proposal

Turn the `check` job into a matrix over `ubuntu-latest` and `macos-latest`, keeping the
single `required` job so branch protection still names one status check.

## Acceptance criteria

- [ ] `./scripts/task check` passes on both runners.
- [ ] `required` needs the whole matrix and fails if any leg fails.
- [ ] Branch protection still names exactly one check.
- [ ] The build cache is keyed per platform so the legs do not evict each other.
