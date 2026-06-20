use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_typescript;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlTsx;

impl ArlTsx {
  const LANGUAGE: LazyLock<Language> =
    LazyLock::new(|| tree_sitter_typescript::LANGUAGE_TSX.into());
}

impl ArlLanguage for ArlTsx {
  fn extensions(&self) -> &'static [&'static str] {
    &["tsx"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
