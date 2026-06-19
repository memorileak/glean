mod arl_css;
mod arl_html;
mod arl_javascript;
mod arl_json;
mod arl_python;
mod arl_rust;
mod arl_tsx;
mod arl_typescript;
mod arl_yaml;
mod registry;

pub use registry::ARL_LANGUAGES;

use tree_sitter::Language;

use crate::types::Symbol;

pub trait ArlLanguage: Send + Sync {
  fn extensions(&self) -> &'static [&'static str];

  #[allow(unused_variables)]
  fn matches_filename(&self, filename: &str) -> bool {
    false
  }

  fn priority(&self) -> u8 {
    0
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol>;
}

pub struct Outliner;

impl Outliner {
  fn outline(lang: &Language, source: &str, query: &str) -> Vec<Symbol> {
    vec![]
  }
}
