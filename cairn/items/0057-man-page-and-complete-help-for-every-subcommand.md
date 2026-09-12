---
id: 57
title: Man page and complete help for every subcommand
type: docs
status: backlog
milestone: v1.0
depends_on:
- 45
created: 2026-09-11
updated: 2026-09-11
priority: p2
effort: s
area: cli
---

## Problem

`--help` is whatever clap derived from the struct. Some flags have no description and none
has an example.

## Proposal

Write real help text with examples, generate a man page from it, and ship shell completions.

## Acceptance criteria

- [ ] Every command and flag has a description and at least one example.
- [ ] A man page is generated and installed by the release workflow.
- [ ] Shell completions are generated for zsh and bash.
- [ ] A test asserts no flag ships without a description.
