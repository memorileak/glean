use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_css;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlCss;

impl ArlCss {
  const LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_css::LANGUAGE.into());
}

impl ArlLanguage for ArlCss {
  fn extensions(&self) -> &'static [&'static str] {
    &["css", "scss", "less"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
