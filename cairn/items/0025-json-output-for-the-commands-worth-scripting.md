---
id: 25
title: JSON output for the commands worth scripting
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

Every command prints for humans. Anything built on top — an editor integration, a shell
function, an eval harness — has to parse a table that was never meant to be parsed.

## Proposal

`--json` on `search`, `status` and `symbol`, plus pack metadata (token count, files
included, budget used) as a JSON sidecar so a caller can check a pack fit without parsing
markdown.

## Acceptance criteria

- [ ] Each supported command emits one well-formed JSON document.
- [ ] The schema is documented and covered by a test that parses the output.
- [ ] Errors also emit JSON when `--json` is given, rather than plain text on stderr.
- [ ] Human output is byte-identical to today when the flag is absent.
