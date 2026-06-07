use ast_grep_language::Tsx;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstTsx;

impl AstLanguage for AstTsx {
  fn extensions(&self) -> &'static [&'static str] {
    &["tsx"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(Tsx, source, query)
  }
}
