---
id: 8
title: Document MCP registration and verify it in a real agent
type: docs
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-09-12
created: 2026-09-11
updated: 2026-09-12
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

- [x] All five tools are called from an agent and return usable output.
- [x] The README documents registration including the repo-scoping argument.
- [x] Startup indexing progress appears on stderr and never corrupts the protocol stream.
- [x] A registration pointed at a non-existent path fails with a clear message.

## 2026-09-12

Verified: all five tools driven over stdio return usable content; stdout is pure JSON-RPC across every reply; a notification gets no response; stderr carries the scope. Registered with 'claude mcp add' and a real agent client reported Connected — that proves the handshake; the five tool calls were exercised by driving the protocol directly. Found and fixed: an unknown tool name returned isError:false, which an agent would read as a successful answer. Registration removed again because it pointed at the worktree binary; the README documents the stable form.
