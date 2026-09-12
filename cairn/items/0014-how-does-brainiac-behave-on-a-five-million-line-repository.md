---
id: 14
title: How does brainiac behave on a five-million-line repository?
type: spike
status: backlog
milestone: v0.2
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: index
---

## Question

Everything is measured on repositories of roughly half a million lines. Index time, index
size, graph construction, and PageRank all scale differently, and one of them will
dominate first. Which, and at what size does it stop being usable?

`rank::build_graph` loads every symbol and every reference into memory on **each CLI
invocation**. That is the suspect.

## Timebox

One day. Measure, write the numbers down, stop.

## How to answer it

- [ ] Find or assemble a repository of 3-10M lines (chromium subset, linux, rust-lang/rust).
- [ ] Record: cold index, warm re-index, database size, graph build, query latency, peak RSS.
- [ ] Identify which stage dominates and at what repository size it crosses one second.
- [ ] Note whether one transaction for the whole index run becomes a problem.

## Answer

_Unanswered._
