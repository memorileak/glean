use ast_grep_language::Yaml;

use crate::types::Match;

use super::{AstLanguage, collect_matches};

pub struct AstYaml;

impl AstLanguage for AstYaml {
  fn extensions(&self) -> &'static [&'static str] {
    &["yaml", "yml"]
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(Yaml, source, query)
  }
}
