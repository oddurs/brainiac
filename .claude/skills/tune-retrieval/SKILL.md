---
name: tune-retrieval
description: Change and verify ranking quality — priors, fusion weights, chunking, or graph construction. Use when search returns the wrong thing, when adding a scoring signal, or when a change to src/rank.rs, src/pack.rs, or src/parse.rs could move results.
---

# Tuning retrieval

Ranking has no compiler. A change that looks obviously right routinely makes results
worse on real repositories, so the only evidence that counts is before/after output on
a repo big enough to have noise in it.

## 1. Capture the baseline first

Pick two or three repos you know well and a handful of queries whose right answer you
can name. Then, **before touching any code**:

```sh
cargo build --release
for q in "the thing you changed" "another query"; do
  echo "== $q"
  ./target/release/brainiac -C ~/Code/<repo> search "$q" -n 10
done > /tmp/before.txt
```

Include at least one repo with vendored or version-pinned directories, and one query
whose answer lives in a test file. Those are where priors bite.

## 2. Make the change

Read `.claude/rules/retrieval.md` first — it holds the invariants that are easy to
break: RRF over ranks rather than raw scores, priors applied after the power iteration,
every signal visible in `Hit::signals`.

If chunk boundaries move, bump `SCHEMA_VERSION` in `src/store.rs` so existing indexes
rebuild instead of silently mixing shapes.

## 3. Diff the results, not the code

```sh
cargo build --release
# regenerate into /tmp/after.txt with the identical loop, then:
diff /tmp/before.txt /tmp/after.txt
```

Read every moved line and say why it moved. A change you cannot explain is a change
that will be reverted later by someone who cannot explain it either.

Check the three failure modes this ranker has had:

- **Duplicate bodies** — vendored or version-pinned copies filling a page.
  `brainiac -C <repo> search <symbol in a vendored tree> -n 10` should show it once.
- **Trivial symbols** — one-line accessors crowding a repo map.
  `brainiac -C <repo> map -b 800` should lead with substantive definitions.
- **Prose drowning code** — documentation outranking the implementation for a code
  question. Watch the `L`/`S`/`G` letters: an all-`L` result page means the graph
  contributed nothing and the query probably needed it to.

## 4. Cost the pack

Budget overshoot is a correctness bug — it silently truncates someone's context window.

```sh
./target/release/brainiac -C ~/Code/<repo> pack "a real question" -b 4000 | tail -1
```

The trailing token comment must be at or under the budget.

## 5. Lock it in

Add the case to `tests/pipeline.rs` as an order-and-presence assertion — never an exact
float. Then `scripts/task check`.
