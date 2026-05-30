//! Angular template language adapter for ast-grep, backed by the patched
//! `tree-sitter-angular` crate in `patches/tree-sitter-angular`.
//!
//! The patch bumps the tree-sitter dependency from `~0.25` to `0.26` and
//! updates the bundled C headers and `lib.rs` to match the 0.26 API.
//! All C compilation is handled by the crate's own `build.rs`.

use ast_grep_core::{
  language::Language,
  matcher::{Pattern, PatternBuilder, PatternError},
  tree_sitter::{LanguageExt, StrDoc, TSLanguage},
};

/// ast-grep language for Angular HTML templates.
#[derive(Clone, Copy, Debug)]
pub struct Angular;

impl Language for Angular {
  fn kind_to_id(&self, kind: &str) -> u16 {
    self
      .get_ts_language()
      .id_for_node_kind(kind, /* named */ true)
  }

  fn field_to_id(&self, field: &str) -> Option<u16> {
    self
      .get_ts_language()
      .field_id_for_name(field)
      .map(|f| f.get())
  }

  fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
    builder.build(|src| StrDoc::try_new(src, *self))
  }
}

impl LanguageExt for Angular {
  fn get_ts_language(&self) -> TSLanguage {
    tree_sitter_angular::language()
  }
}
