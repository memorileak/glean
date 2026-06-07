use ast_grep_language::Json;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstJson;

impl AstLanguage for AstJson {
  fn extensions(&self) -> &'static [&'static str] {
    &["json"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(Json, source, query)
  }
}
