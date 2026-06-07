use crate::types::Match;

use super::super::morelangs::Angular;
use super::{AstLanguage, collect_matches};

pub struct AstAngular;

impl AstLanguage for AstAngular {
  fn extensions(&self) -> &'static [&'static str] {
    &[] // Uses filename matching instead
  }

  fn matches_filename(&self, filename: &str) -> bool {
    filename.ends_with(".angular.html") || filename.ends_with(".component.html")
  }

  fn priority(&self) -> u8 {
    10 // Higher priority than HtmlLang
  }

  fn find_matches(&self, source: &str, query: &str) -> Vec<Match> {
    collect_matches(Angular, source, query)
  }
}
