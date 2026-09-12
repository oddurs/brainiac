---
id: 27
title: 'Index administration: where it is, how big, how to remove it'
type: feature
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: s
area: cli
---

## Problem

Indexes accumulate in the XDG data directory, one per repository, named by a hash. A
repository that is deleted or renamed leaves its index behind forever, and nothing lists
them or reclaims the space. The 471k-line reference repo alone is ~45MB.

## Proposal

`brainiac index --list` shows every index with its repo path, size and last use.
`brainiac index --clean` removes indexes whose repository no longer exists;
`--clean --all` removes every index.

## Acceptance criteria

- [ ] `--list` shows path, size and last-used for every index.
- [ ] `--clean` removes only indexes whose root is gone, and reports what it freed.
- [ ] `--clean --all` prompts unless `--yes` is given.
- [ ] Removing the index for the current repo is safe and it rebuilds on next use.
