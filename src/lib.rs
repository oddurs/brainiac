//! brainiac — a local context manager for your own repositories.
//!
//! The pipeline is four stages, each usable on its own:
//!
//! 1. [`index`] walks the repo (gitignore-aware), hashes every file, and only
//!    reparses what changed.
//! 2. [`parse`] turns source into definitions and references via tree-sitter
//!    tag queries, plus the retrieval chunks those spans imply.
//! 3. [`rank`] answers a query by fusing BM25, symbol matching, and a
//!    personalised PageRank over the reference graph.
//! 4. [`pack`] fits the answer into a stated token budget.

pub mod config;
pub mod index;
pub mod lang;
pub mod mcp;
pub mod pack;
pub mod parse;
pub mod rank;
pub mod store;
pub mod tui;
