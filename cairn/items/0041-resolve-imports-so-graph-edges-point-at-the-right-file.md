---
id: 41
title: Resolve imports so graph edges point at the right file
type: feature
status: backlog
milestone: v0.3
depends_on:
- 16
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: l
area: rank
---

## Problem

The reference graph matches purely on name: a reference to `parse` links to every file
defining anything called `parse`, with the weight split between them. The `UBIQUITY_CUTOFF`
exists to stop common names smearing rank across the repository — a blunt instrument
compensating for the absence of resolution.

Every supported language states its imports. Using them is the single largest available
improvement to graph quality.

## Proposal

Extract import and use statements per language, build a module-to-file map, and prefer the
imported definition when resolving a reference. Fall back to name matching when resolution
fails, so partial support degrades rather than breaks.

## Acceptance criteria

- [ ] Rust `use`, Python `import`, JS/TS `import`, and Go imports resolve to files.
- [ ] A name defined in five files but imported from one links only to that one — tested.
- [ ] `UBIQUITY_CUTOFF` can be relaxed; the new value is justified by the score.
- [ ] The evaluation score improves, or the change is reverted.
