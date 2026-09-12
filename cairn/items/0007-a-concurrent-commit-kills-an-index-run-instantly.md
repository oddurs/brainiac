---
id: 7
title: A concurrent commit kills an index run instantly
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-09-11
updated: 2026-09-12
priority: p0
effort: s
area: store
---

## What happens

An MCP server holds the index open for a whole agent session. A `brainiac` run in another
terminal starts its own index pass. One of them dies with `database is locked`.

The filed cause was wrong — rusqlite already sets `busy_timeout` to 5000ms, measured with
`pragma_busy_timeout`. The real cause is the transaction mode. `index::run` used
`Connection::transaction()`, which is **DEFERRED**, and `upsert_file` issues
`SELECT id, hash FROM files WHERE path=?1` as its first statement. So the transaction takes
a read snapshot before it takes the write lock. If anyone commits in between, SQLite
answers the upgrade with `SQLITE_BUSY_SNAPSHOT` (extended code 517) — and the busy handler
is **never consulted** for that code. The run fails instantly however long the timeout is.

Reproduced directly against this branch:

    DEFERRED: write FAILED -> SqliteFailure(DatabaseBusy, extended_code: 517)
    IMMEDIATE: the second writer waits for the lock, as intended

## What should happen

A second indexer waits its turn. Taking the write lock at BEGIN puts the contention where
the busy handler applies.

## Reproduction

1. Open a transaction with `Connection::transaction()` and read from `files`.
2. Commit anything from a second connection.
3. Write from the first. It fails immediately with extended code 517.

## Acceptance criteria

- [x] The index write transaction begins IMMEDIATE, through one `store::begin_write` helper.
- [x] A deterministic test fails if it ever begins deferred again.
- [x] `busy_timeout` is set explicitly, in one named constant, with a comment saying why.
- [x] Index write failures name the index path, with the context covering the whole write.

## Notes

I twice reached the wrong conclusion here: first implementing against the filed premise,
then declaring there was no bug at all when my test passed with the fix reverted. The
review caught the real mechanism. The lesson is in the test that survives — the guard is
the deterministic `transaction_state` assertion, not the concurrent-runs test, which passes
on a deferred transaction often enough to be worthless as a guard and is now labelled a
smoke test.

## 2026-09-12

Review found the real mechanism: SQLITE_BUSY_SNAPSHOT (517) on a deferred transaction's read-to-write upgrade, which no busy timeout covers. Fix is TransactionBehavior::Immediate via store::begin_write. Verified the deterministic guard fails when reverted to Deferred.
