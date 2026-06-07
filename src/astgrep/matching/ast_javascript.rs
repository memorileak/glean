use ast_grep_language::JavaScript;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstJavaScript;

impl AstLanguage for AstJavaScript {
  fn extensions(&self) -> &'static [&'static str] {
    &["js", "mjs", "cjs", "jsx"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(JavaScript, source, query)
  }
}
