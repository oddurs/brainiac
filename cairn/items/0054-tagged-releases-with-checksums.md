---
id: 54
title: Tagged releases with checksums
type: chore
status: backlog
milestone: v1.0
depends_on:
- 9
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: build
---

## Problem

Install is `cargo install --path .` from a clone. That is acceptable for a personal tool and
makes it impossible to say which version produced a given pack, or to go back to one that
worked.

## Proposal

A tag-triggered workflow that builds macOS and Linux binaries, attaches them with checksums
to a GitHub release, and generates notes from the commit history. No crates.io, no package
managers — that is a distribution promise this project deliberately does not make.

## Acceptance criteria

- [ ] Pushing a `v*` tag produces a release with macOS and Linux binaries.
- [ ] Checksums are attached and verifiable.
- [ ] `brainiac --version` matches the tag.
- [ ] The README documents installing a released binary as well as from source.
