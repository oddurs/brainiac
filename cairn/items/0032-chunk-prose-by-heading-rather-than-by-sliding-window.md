---
id: 32
title: Chunk prose by heading rather than by sliding window
type: feature
status: backlog
milestone: v0.3
depends_on:
- 16
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: parse
---

## Problem

Markdown, and every other prose format, falls to fixed 60-line windows with 10 lines of
overlap and a chunk name that is just the file path. Documentation has structure — headings
are the natural retrieval unit and the natural name — and throwing it away is why prose
chunks need a 0.6 `kind_prior` to stop them crowding out code.

## Proposal

Parse headings with the markdown grammar, chunk by section, and name each chunk after its
heading so it participates in symbol matching. Long sections still split. Then check whether
the `kind_prior` penalty can be reduced, since it exists to compensate for a problem this
removes.

## Acceptance criteria

- [ ] Markdown chunks are sections, named by heading, verified by a test.
- [ ] A query matching a heading ranks that section above a sliding window over the same text.
- [ ] The evaluation score improves, or the change is reverted.
- [ ] Non-markdown prose still falls back to windows.
