---
id: 7
title: Concurrent CLI and MCP runs must block, not fail
type: bug
status: backlog
milestone: v0.1
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: s
area: store
---

## What happens

`Store::open` sets WAL and `synchronous=NORMAL` but never sets a busy timeout. SQLite
therefore returns `SQLITE_BUSY` immediately when a second writer arrives. With the MCP
server holding the index open in an agent session and a `brainiac search` run in a
terminal — the normal way this tool gets used — the second process fails outright.

## What should happen

A writer waits for the lock and succeeds. Only a genuinely stuck lock surfaces an error,
and that error says which other process holds it.

## Reproduction

1. `brainiac mcp` in one terminal against a large repo.
2. `brainiac index -f` in another while the server's own index pass is running.
3. Observe the failure rather than a wait.

## Acceptance criteria

- [ ] `busy_timeout` is set on every connection, with the value in one named constant.
- [ ] A test opens two connections to one index and writes from both without error.
- [ ] The failure that remains after the timeout names the index path.
