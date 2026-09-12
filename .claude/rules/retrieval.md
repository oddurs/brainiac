---
paths:
  - "src/rank.rs"
  - "src/pack.rs"
  - "src/parse.rs"
---

# Retrieval

- Scores from different sources are not comparable. Combine ranks with RRF, never raw
  BM25 against PageRank mass.
- Priors multiply after the power iteration, never inside it — biasing the walk itself
  makes the graph mean something other than reference structure.
- A new signal needs a letter in `Hit::signals` so `brainiac search` shows why a result
  is there. Silent scoring is unfixable scoring.
- Chunks are the retrieval unit and their ids are FTS rowids. Changing how chunks are
  cut means bumping `SCHEMA_VERSION`.
- Budgets are honoured by measurement, not by estimate-and-hope: every block is costed
  with `pack::est_tokens` before it is appended.
