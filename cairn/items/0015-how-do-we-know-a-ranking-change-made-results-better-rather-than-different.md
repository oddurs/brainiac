---
id: 15
title: How do we know a ranking change made results better rather than different?
type: spike
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: l
area: rank
---

## Question

Every ranking decision so far — the RRF weights, the path priors, the size and churn
multipliers, the `kind_prior` on prose — was chosen by reading output and judging it. That
worked to get started and does not scale past one person's memory of one afternoon.

What is the cheapest measurement that would have caught the three real regressions already
found by hand: doubled symbols, vendored copies dominating, trivial accessors filling the
map?

## Timebox

Two days. The output is a decision and a schema, not the harness itself.

## How to answer it

- [ ] Choose a metric. MRR and recall@10 over labelled queries are the obvious candidates;
      say why the chosen one suits a tool whose output is a *set* of context, not a ranking.
- [ ] Decide how ground truth is captured without it becoming a second job: probably
      `query -> the file:symbol that answers it`, 20-40 per repo, written by hand once.
- [ ] Decide which repositories form the evaluation set and how they are pinned, given they
      are private working repos that change under you.
- [ ] Decide the storage format so the labels live in this repo and survive.
- [ ] Prove the metric moves the right way by replaying a known past regression against it.

## Answer

_Unanswered. Until this closes, every tuning item below is blocked, on purpose._
