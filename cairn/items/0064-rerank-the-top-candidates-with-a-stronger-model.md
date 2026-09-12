---
id: 64
title: Rerank the top candidates with a stronger model
type: feature
status: backlog
milestone: later
depends_on:
- 58
created: 2026-09-11
updated: 2026-09-11
priority: p3
effort: l
area: rank
---

## Problem

RRF fusion is cheap and order-only. A cross-encoder or a small language model scoring the
top fifty candidates against the query would likely order them better.

## Proposal

An optional reranking stage over the fused top-N. Costs a model and a great deal of latency,
which is exactly the trade this project was built to avoid — so it belongs here, not on the
path.

## Acceptance criteria

- [ ] The embeddings spike is answered first, since it settles whether semantic signal helps
      at all on these corpora.
- [ ] If it does, measure reranking against the same baseline.
