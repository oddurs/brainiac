---
id: 6
title: Publish the repository and get the first CI run green
type: chore
status: backlog
milestone: v0.1
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: s
area: build
---

## Problem

The whole tree is written, tested and staged, but nothing is committed and there is no
remote. Every other item in v0.1 assumes a repository that exists.

## Proposal

Make the initial commit on `main` — the only commit that may land there directly, because
a pull request needs a base — then create the GitHub repository, push, and turn on the
branch protection that makes every later change go through a PR.

## Acceptance criteria

- [ ] `git log` shows one initial commit containing the full tree.
- [ ] `github.com/oddurs/brainiac` exists, public, with the MIT licence detected.
- [ ] The `ci` workflow has run once and is green.
- [ ] Branch protection on `main` requires a PR and the `required` status check.
- [ ] A direct push to `main` is refused by `.githooks/pre-push`.
