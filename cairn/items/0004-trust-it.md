---
id: 4
key: v0.2
title: Trust it
type: milestone
status: backlog
created: 2026-09-11
updated: 2026-09-11
due: 2026-10-24
---

## Ships

See why brainiac ranked something where it did, steer it when it is wrong, and know that a ranking change made results better rather than merely different.

This is the milestone that turns a ranking from a vibe into an engineering artifact. Every
tuning item here depends on the evaluation harness, because tuning without measurement is
how retrieval quality quietly rots.

## Done when

- [ ] An evaluation set of labelled queries runs over real repositories and prints a score.
- [ ] `scripts/task check` fails if ranking quality drops below the recorded baseline.
- [ ] `.brainiac.toml` lets a repo exclude paths and the file is documented.
- [ ] `--explain` shows the per-signal contribution behind any hit.
- [ ] `--json` output exists for the commands worth scripting.

## Explicitly not in this milestone

- New languages or new chunking strategies — they land in v0.3, measured by what is
  built here.
- Embeddings. Still out.
