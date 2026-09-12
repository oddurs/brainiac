---
id: 9
title: Build and test on macOS and Linux in CI
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 6
created: 2026-09-11
updated: 2026-09-12
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

- [x] `./scripts/task check` passes on both runners.
- [x] `required` needs the whole matrix and fails if any leg fails.
- [x] Branch protection still names exactly one check.
- [x] The build cache is keyed per platform so the legs do not evict each other.

## 2026-09-12

CI's first run warns that actions/checkout and Swatinem/rust-cache target Node.js 20, which GitHub now forces onto Node 24. Bump both pins while adding the platform matrix — same file, same change.
