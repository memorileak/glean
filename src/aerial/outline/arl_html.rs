use std::sync::LazyLock;

use tree_sitter::Language;
use tree_sitter_html;

use crate::types::Symbol;

use super::{ArlLanguage, Outliner};

pub struct ArlHtml;

impl ArlHtml {
  const LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_html::LANGUAGE.into());
}

impl ArlLanguage for ArlHtml {
  fn extensions(&self) -> &'static [&'static str] {
    &["html", "htm"]
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol> {
    Outliner::outline(&Self::LANGUAGE, source, query)
  }
}
