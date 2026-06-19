use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_yaml;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlYaml;

impl ArlYaml {
  const LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_yaml::LANGUAGE.into());
}

impl ArlLanguage for ArlYaml {
  fn extensions(&self) -> &'static [&'static str] {
    &["yaml", "yml"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
