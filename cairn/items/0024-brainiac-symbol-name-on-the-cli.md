---
id: 24
title: brainiac symbol <name> on the CLI
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

`read_symbol` exists over MCP but not on the command line, so the CLI and the agent
interface have drifted apart. The CLI is also the fastest way to check what the indexer
actually captured for a given name.

## Proposal

`brainiac symbol <name>` prints every definition with that exact name, with path, span and
source. `--kind` narrows by definition kind.

## Acceptance criteria

- [ ] Output matches what the MCP tool returns for the same name.
- [ ] A name with no definition exits non-zero saying so.
- [ ] `--json` emits structured output.
- [ ] Names defined in many files list all of them, capped with a count of the remainder.
