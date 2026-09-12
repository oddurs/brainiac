---
paths:
  - "tests/**/*.rs"
  - "src/tui.rs"
---

# Tests

- Integration tests build a throwaway repo on disk via the `Fixture` helper in
  `tests/pipeline.rs:1` and index it for real. No mocked SQLite.
- Name the behaviour, not the function: `deleting_a_file_keeps_the_search_index_consistent`.
- Ranking assertions check *order and presence*, never exact float scores — priors get
  retuned and brittle tests get deleted instead of fixed.
- TUI tests render through `ratatui::backend::TestBackend` and drive the keymap through
  `on_key`. A terminal is never required.
- A bug fix arrives with the test that would have caught it.
