---
id: 2
key: v1.0
title: The promise
type: milestone
status: backlog
depends_on:
- 1
created: 2026-09-11
due: 2027-01-16
---

## Ships

Ask a question about a repository; get back a token-budgeted pack of the code most likely to answer it. True on macOS and Linux, documented, and stable enough that breaking it is a bug.

No new features. If something here is exciting, it is in the wrong milestone.

## Done when

- [ ] Every CLI flag and MCP tool schema is documented and frozen for the 1.x line.
- [ ] Every failure exits non-zero with a message naming the path and the fix.
- [ ] A corrupt or locked index recovers by rebuilding instead of failing.
- [ ] CI is green on macOS and Linux, x86_64 and arm64.
- [ ] The ranking quality gate passes on every evaluation repository.
- [ ] Documented performance budgets are enforced in CI.

## Explicitly not in this milestone

- Embeddings or vector search. They live in `later` and stay there unless a spike shows
  the L/S/G fusion actually loses.
- Writing to a repository. brainiac reads and ranks; it never edits.
- Anything hosted, multi-user, or remote.
- Windows.

## The promise, stated plainly

This is a personal tool in a public repository. v1.0 means it works for me, on the
platforms named, and that breaking it counts as a bug. It does not promise support,
backwards compatibility beyond the 1.x CLI surface, or that it fits anyone else's
workflow.
