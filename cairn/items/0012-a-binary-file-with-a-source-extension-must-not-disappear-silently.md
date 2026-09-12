---
id: 12
title: A binary file with a source extension must not disappear silently
type: bug
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: index
---

## What happens

`index::run` reads each candidate with `read_to_string` and drops the file on error via
`.ok()?`. A minified bundle, a generated `.go` fixture holding raw bytes, or a UTF-16 file
is skipped with no record anywhere that it existed.

## What should happen

Unreadable files are skipped — that part is right — but the count is reported, so a
surprisingly small index is diagnosable instead of mysterious.

## Reproduction

1. Write a file of random bytes to `src/blob.rs` in a test repo.
2. `brainiac index` — the file count is one lower than the file count on disk.
3. Nothing says why.

## Acceptance criteria

- [ ] Skipped-unreadable files are counted and shown in the index summary.
- [ ] A test writes invalid UTF-8 to a source extension and asserts the run succeeds.
- [ ] The skipped count is reachable from `brainiac status`.
