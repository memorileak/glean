use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_javascript;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlJavascript;

impl ArlJavascript {
  const LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_javascript::LANGUAGE.into());
}

impl ArlLanguage for ArlJavascript {
  fn extensions(&self) -> &'static [&'static str] {
    &["js", "mjs", "cjs", "jsx"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
