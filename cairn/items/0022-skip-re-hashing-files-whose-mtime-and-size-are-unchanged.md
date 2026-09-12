---
id: 22
title: Skip re-hashing files whose mtime and size are unchanged
type: feature
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: index
---

## Problem

Every command runs an incremental index pass, and that pass reads and blake3-hashes every
file in the repository — about 300ms on a 471k-line repo, paid on every single CLI
invocation. The content hash is the right correctness mechanism; reading every byte to
compute it is not the right fast path.

## Proposal

Compare `(mtime, size)` against the stored row first and skip the read when both match.
Keep the hash as the authority when either differs, and keep `--force` re-hashing
everything, so the fast path is an optimisation rather than a new source of truth.

## Acceptance criteria

- [ ] A no-change re-index does not read file contents, proven by measurement.
- [ ] Re-index time on the 471k-line reference repo drops below 100ms.
- [ ] Touching a file without changing it does not cause a reparse.
- [ ] Changing content while preserving mtime and size is still caught by `--force`, and
      the limitation is documented.
