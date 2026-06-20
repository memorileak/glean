use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_typescript;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlTypescript;

impl ArlTypescript {
  const LANGUAGE: LazyLock<Language> =
    LazyLock::new(|| tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into());
}

impl ArlLanguage for ArlTypescript {
  fn extensions(&self) -> &'static [&'static str] {
    &["ts", "mts", "cts"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
