use ast_grep_language::TypeScript;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstTypeScript;

impl AstLanguage for AstTypeScript {
  fn extensions(&self) -> &'static [&'static str] {
    &["ts", "mts", "cts"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(TypeScript, source, query)
  }
}
