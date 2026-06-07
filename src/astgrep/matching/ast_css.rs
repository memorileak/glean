use ast_grep_language::Css;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstCss;

impl AstLanguage for AstCss {
  fn extensions(&self) -> &'static [&'static str] {
    &["css", "scss", "less"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(Css, source, query)
  }
}
