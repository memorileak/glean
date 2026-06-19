use std::path::Path;
use std::sync::LazyLock;

use super::ArlLanguage;
use super::arl_css::ArlCss;
use super::arl_html::ArlHtml;
use super::arl_javascript::ArlJavascript;
use super::arl_json::ArlJson;
use super::arl_python::ArlPython;
use super::arl_rust::ArlRust;
use super::arl_tsx::ArlTsx;
use super::arl_typescript::ArlTypescript;
use super::arl_yaml::ArlYaml;

pub struct LanguageRegistry {
  languages: Vec<Box<dyn ArlLanguage>>,
}

impl LanguageRegistry {
  pub fn new() -> Self {
    Self {
      languages: Vec::new(),
    }
  }

  pub fn register(mut self, lang: impl ArlLanguage + 'static) -> Self {
    self.languages.push(Box::new(lang));
    self
  }

  /// Detect language for a file path
  pub fn detect(&self, path: &Path) -> Option<&dyn ArlLanguage> {
    let filename = path.file_name()?.to_str()?;

    // First: check filename patterns (higher priority languages)
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
pub static ARL_LANGUAGES: LazyLock<LanguageRegistry> = LazyLock::new(|| {
  LanguageRegistry::new()
    .register(ArlRust)
    .register(ArlJavascript)
    .register(ArlTypescript)
    .register(ArlTsx)
    .register(ArlPython)
    .register(ArlJson)
    .register(ArlYaml)
    .register(ArlHtml)
    .register(ArlCss)
});
