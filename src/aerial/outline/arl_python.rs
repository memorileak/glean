use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_python;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlPython;

impl ArlPython {
  const LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_python::LANGUAGE.into());
}

impl ArlLanguage for ArlPython {
  fn extensions(&self) -> &'static [&'static str] {
    &["py"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
