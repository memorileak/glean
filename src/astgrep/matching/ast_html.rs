use ast_grep_language::Html;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstHtml;

impl AstLanguage for AstHtml {
  fn extensions(&self) -> &'static [&'static str] {
    &["html", "htm"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(Html, source, query)
  }
}
