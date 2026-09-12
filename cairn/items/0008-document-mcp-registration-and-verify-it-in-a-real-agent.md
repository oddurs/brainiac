---
id: 8
title: Document MCP registration and verify it in a real agent
type: docs
status: backlog
milestone: v0.1
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: s
area: mcp
---

## Problem

The README gives one `claude mcp add` line. Nobody has run it and confirmed an agent can
actually call `context_pack` and get something useful back. An MCP server that fails
handshake in practice is worse than no MCP server, because the failure is silent.

## Proposal

Register the server in a real agent session, exercise all five tools, and write down what
a working registration looks like — including the `-C` argument, which is easy to forget
and causes the server to index the wrong repository.

## Acceptance criteria

- [ ] All five tools are called from an agent and return usable output.
- [ ] The README documents registration including the repo-scoping argument.
- [ ] Startup indexing progress appears on stderr and never corrupts the protocol stream.
- [ ] A registration pointed at a non-existent path fails with a clear message.
