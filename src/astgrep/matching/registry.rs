use std::path::Path;
use std::sync::LazyLock;

use super::AstLanguage;
use super::ast_angular::AstAngular;
use super::ast_css::AstCss;
use super::ast_html::AstHtml;
use super::ast_javascript::AstJavaScript;
use super::ast_json::AstJson;
use super::ast_python::AstPython;
use super::ast_rust::AstRust;
use super::ast_tsx::AstTsx;
use super::ast_typescript::AstTypeScript;
use super::ast_yaml::AstYaml;

pub struct LanguageRegistry {
  languages: Vec<Box<dyn AstLanguage>>,
}

impl LanguageRegistry {
  pub fn new() -> Self {
    Self {
      languages: Vec::new(),
    }
  }

  pub fn register(mut self, lang: impl AstLanguage + 'static) -> Self {
    self.languages.push(Box::new(lang));
    self
  }

  /// Detect language for a file path
  pub fn detect(&self, path: &Path) -> Option<&dyn AstLanguage> {
    let filename = path.file_name()?.to_str()?;

    // Sort by priority (descending) - done at registration time ideally
    // First: check filename patterns (Angular, etc.)
    for lang in self.languages.iter().filter(|l| l.priority() > 0) {
      if lang.matches_filename(filename) {
        return Some(lang.as_ref());
      }
    }

    // Then: check by extension
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    for lang in &self.languages {
      if lang.extensions().contains(&ext.as_str()) {
        return Some(lang.as_ref());
      }
    }

    None
  }
}

/// Global registry with all supported languages
pub static AST_LANGUAGES: LazyLock<LanguageRegistry> = LazyLock::new(|| {
  LanguageRegistry::new()
    .register(AstAngular) // Higher priority first
    .register(AstRust)
    .register(AstJavaScript)
    .register(AstTypeScript)
    .register(AstTsx)
    .register(AstPython)
    .register(AstJson)
    .register(AstYaml)
    .register(AstHtml)
    .register(AstCss)
});
