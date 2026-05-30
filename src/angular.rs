//! Custom Angular template language backed by the vendored tree-sitter-angular
//! C grammar (compatible with tree-sitter 0.26.x).
//!
//! The official `tree-sitter-angular` crate on crates.io (0.9.2) requires
//! tree-sitter ~0.25, which conflicts with our 0.26.9 dependency.  We work
//! around this by compiling the upstream C grammar ourselves via `build.rs`
//! and binding to it here with a thin unsafe FFI shim.

use ast_grep_core::{
    language::Language,
    matcher::{Pattern, PatternBuilder, PatternError},
    tree_sitter::{LanguageExt, StrDoc, TSLanguage},
};
use tree_sitter_language::LanguageFn;

// Link to the C symbol produced by build.rs.
unsafe extern "C" {
    fn tree_sitter_angular() -> *const ();
}

/// The tree-sitter-angular LanguageFn.
///
/// # Safety
/// `tree_sitter_angular` is a valid C function produced by the grammar generator
/// and compiled by `build.rs`.
const ANGULAR_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_angular) };

/// ast-grep language for Angular HTML templates.
#[derive(Clone, Copy, Debug)]
pub struct Angular;

impl Language for Angular {
    fn kind_to_id(&self, kind: &str) -> u16 {
        self.get_ts_language()
            .id_for_node_kind(kind, /* named */ true)
    }

    fn field_to_id(&self, field: &str) -> Option<u16> {
        self.get_ts_language()
            .field_id_for_name(field)
            .map(|f| f.get())
    }

    fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
        builder.build(|src| StrDoc::try_new(src, *self))
    }
}

impl LanguageExt for Angular {
    fn get_ts_language(&self) -> TSLanguage {
        ANGULAR_LANGUAGE.into()
    }
}
