use ast_grep_language::Python;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstPython;

impl AstLanguage for AstPython {
  fn extensions(&self) -> &'static [&'static str] {
    &["py"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(Python, source, query)
  }
}
