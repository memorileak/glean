use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_rust;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlRust;

impl ArlRust {
  const LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_rust::LANGUAGE.into());
}

impl ArlLanguage for ArlRust {
  fn extensions(&self) -> &'static [&'static str] {
    &["rs"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
