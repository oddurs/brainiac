---
id: 65
title: Scoping into an ignored directory indexes ignored content
type: bug
status: backlog
milestone: v0.2
created: 2026-09-12
updated: 2026-09-12
priority: p2
effort: s
area: index
---

## What happens

The `ignore` crate does not apply gitignore rules to the walk root itself, only below it.
So `brainiac -C node_modules/foo` or `-C target/debug` indexes content the whole-repo walk
would never touch. Measured: a `.gitignore` containing `ignored/` does not stop
`-C ignored` from indexing `secret.rs`.

Found while reviewing 0013, which introduced scoping. Narrow: rules below the scope work
correctly, so this is confined to the scope root.

## What should happen

Probably a warning rather than a refusal — naming a directory explicitly is a reasonable
way to say you mean it — but silently indexing ignored content is wrong either way. Decide
which, then do it.

## Reproduction

1. `.gitignore` containing `ignored/`, with `ignored/secret.rs` present.
2. `brainiac -C ignored index` — indexes it.
3. `brainiac index` from the root — does not.

## Acceptance criteria

- [ ] Decide warn vs refuse, and record why in this item.
- [ ] A scope that its own repository ignores is handled per that decision.
- [ ] A test covers it.
- [ ] The README's sharp-edge note is updated or removed to match.
