mod ast_angular;
mod ast_css;
mod ast_html;
mod ast_javascript;
mod ast_json;
mod ast_python;
mod ast_rust;
mod ast_tsx;
mod ast_typescript;
mod ast_yaml;
mod registry;

pub use registry::AST_LANGUAGES;

use ast_grep_core::matcher::Pattern;
use ast_grep_core::tree_sitter::{LanguageExt, StrDoc};
use ast_grep_core::{Doc, Node};

use crate::types::{Match, Position};

/// Trait for languages that support AST-based pattern matching
pub trait AstLanguage: Send + Sync {
  /// File extensions this language handles (lowercase, without dot)
  fn extensions(&self) -> &'static [&'static str];

  /// Optional special filename matching (e.g., ".angular.html")
  #[allow(unused_variables)]
  fn matches_filename(&self, filename: &str) -> bool {
    false
  }

  /// Priority for detection (higher = checked first). Useful for Angular vs HTML.
  fn priority(&self) -> u8 {
    0
  }

  /// Find pattern matches in source code
  fn find_matches(&self, source: &str, query: &str) -> Vec<Match>;
}

impl<'tree, D> From<Node<'tree, D>> for Match
where
  D: Doc,
{
  fn from(node: Node<'_, D>) -> Self {
    let start = node.start_pos();
    let end = node.end_pos();
    Match {
      start: Position(start.line(), start.column(&node)),
      end: Position(end.line(), end.column(&node)),
    }
  }
}

pub fn collect_matches<L>(lang: L, source: &str, query: &str) -> Vec<Match>
where
  L: LanguageExt + Clone,
{
  let Ok(pattern) = Pattern::try_new(query, lang.clone()) else {
    return Vec::new();
  };

  let root = lang.ast_grep(source);
  root
    .root()
    .find_all(pattern)
    .map(|matched| Match::from(Into::<Node<'_, StrDoc<L>>>::into(matched)))
    .collect()
}
