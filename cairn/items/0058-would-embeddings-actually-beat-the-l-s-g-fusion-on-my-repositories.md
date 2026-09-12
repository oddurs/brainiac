---
id: 58
title: Would embeddings actually beat the L/S/G fusion on my repositories?
type: spike
status: backlog
milestone: later
depends_on:
- 16
created: 2026-09-11
updated: 2026-09-11
priority: p3
effort: l
area: rank
---

## Question

The whole design bets that lexical, symbolic and graph evidence together cover what
embeddings would, without a model download, a vector store, or a network call. That bet has
never been tested — it was an architectural preference, defensible but unmeasured.

Does a hybrid with a local embedding model score better on the evaluation set, and by
enough to justify what it costs?

## Timebox

Three days, and only once the evaluation harness is trusted. Before that there is no way to
answer this and the question should stay closed.

## How to answer it

- [ ] Pick a small local embedding model that runs on CPU at acceptable speed.
- [ ] Index one evaluation corpus, add embeddings as a fourth RRF list.
- [ ] Score against the committed baseline.
- [ ] Record the cost: index time, index size, binary size, first-run download.
- [ ] Identify which query classes improve, if any. Conceptual questions with no shared
      vocabulary are the plausible win; if those do not improve, nothing will.

## Answer

_Unanswered. Explicitly out of scope for v1.0 — this exists so the question can be settled
with evidence rather than re-argued._
