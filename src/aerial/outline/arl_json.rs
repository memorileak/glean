use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_json;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlJson;

impl ArlJson {
  const LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_json::LANGUAGE.into());
}

impl ArlLanguage for ArlJson {
  fn extensions(&self) -> &'static [&'static str] {
    &["json"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
