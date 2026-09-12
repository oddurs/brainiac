---
id: 27
title: 'Index administration: where it is, how big, how to remove it'
type: feature
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-12
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

## 2026-09-12

Scoping (0013) makes this worse: anyone who ran 'brainiac -C <subdir>' before 0013 has an old index keyed on the git root holding a full-repo index, which that invocation will now never touch again. README quotes ~45 MB for a 471k-line repo. Orphan reclamation is this item's job.
