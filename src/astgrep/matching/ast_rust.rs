use ast_grep_language::Rust;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstRust;

impl AstLanguage for AstRust {
  fn extensions(&self) -> &'static [&'static str] {
    &["rs"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(Rust, source, query)
  }
}
