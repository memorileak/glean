use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use ast_grep_core::Node;
use ast_grep_core::matcher::Pattern;
use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::{Css, Html, JavaScript, Json, Python, Rust, Tsx, TypeScript, Yaml};
use jsonrpsee::types::ErrorObjectOwned;
use serde::{Deserialize, Serialize};

use crate::astgrep::morelangs::angular::Angular;
use crate::database::RepositoryRepo;
use crate::file_scanner::FileScanner;

use super::{internal_error, invalid_params};

#[derive(Deserialize)]
pub struct SearchPatternParams {
  pub query: String,
}

#[derive(Clone, Serialize)]
pub struct Position(
  usize, // line
  usize, // column
);

#[derive(Clone, Serialize)]
pub struct Match {
  start: Position,
  end: Position,
}

#[derive(Clone, Serialize)]
pub struct SearchPatternResult {
  repo_id: String,
  file_path: String,
  matches: Vec<Match>,
}

#[derive(Clone)]
struct ScannedFile {
  repo_id: String,
  file_path: String,
  abs_path: PathBuf,
}

#[derive(Clone, Copy)]
enum PatternLanguage {
  Rust,
  JavaScript,
  TypeScript,
  Tsx,
  Html,
  Css,
  Json,
  Yaml,
  Python,
  Angular,
}

pub fn handle_search_pattern(
  params: SearchPatternParams,
) -> Result<Vec<SearchPatternResult>, ErrorObjectOwned> {
  let threads = 4;
  let query = params.query.trim().to_string();

  if query.is_empty() {
    return Err(invalid_params("query must not be empty"));
  }

  let mut repo_repo = RepositoryRepo::new();
  let repositories = repo_repo.list_repositories().map_err(internal_error)?;

  let file_scanner = FileScanner::new();
  let mut all_files: Vec<ScannedFile> = Vec::new();

  for repo in repositories {
    match file_scanner.scan_files(&repo.path) {
      Ok(files) => {
        for abs_path in files {
          if detect_language(&abs_path).is_none() {
            continue;
          }

          if let Ok(rel_path) = abs_path.strip_prefix(&repo.path) {
            all_files.push(ScannedFile {
              repo_id: repo.id.clone(),
              file_path: rel_path.to_string_lossy().into_owned(),
              abs_path,
            });
          }
        }
      }
      Err(_) => continue,
    }
  }

  if all_files.is_empty() {
    return Ok(Vec::new());
  }

  let threads = threads.max(1);
  let query = Arc::new(query);
  let mut handles = Vec::with_capacity(threads);

  for partition_index in 0..threads {
    let start = partition_index * all_files.len() / threads;
    let end = (partition_index + 1) * all_files.len() / threads;
    let partition = all_files[start..end].to_vec();
    let query = Arc::clone(&query);

    handles.push(thread::spawn(move || {
      let mut results = Vec::new();

      for file in partition {
        let Ok(source) = fs::read_to_string(&file.abs_path) else {
          continue;
        };

        let Some(language) = detect_language(&file.abs_path) else {
          continue;
        };

        let matches = search_matches(language, &source, &query);
        if matches.is_empty() {
          continue;
        }

        results.push(SearchPatternResult {
          repo_id: file.repo_id,
          file_path: file.file_path,
          matches,
        });
      }

      results
    }));
  }

  let mut all_results = Vec::new();
  for handle in handles {
    let results = handle
      .join()
      .map_err(|_| internal_error("thread panicked"))?;
    all_results.extend(results);
  }

  all_results.sort_unstable_by(|a, b| {
    a.repo_id
      .cmp(&b.repo_id)
      .then_with(|| a.file_path.cmp(&b.file_path))
  });

  Ok(all_results)
}

fn detect_language(path: &Path) -> Option<PatternLanguage> {
  let file_name = path.file_name()?.to_str()?;
  if file_name.ends_with(".angular.html") || file_name.ends_with(".component.html") {
    return Some(PatternLanguage::Angular);
  }

  match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
    "rs" => Some(PatternLanguage::Rust),
    "js" | "jsx" => Some(PatternLanguage::JavaScript),
    "ts" => Some(PatternLanguage::TypeScript),
    "tsx" => Some(PatternLanguage::Tsx),
    "html" => Some(PatternLanguage::Html),
    "css" => Some(PatternLanguage::Css),
    "json" => Some(PatternLanguage::Json),
    "yaml" | "yml" => Some(PatternLanguage::Yaml),
    "py" => Some(PatternLanguage::Python),
    _ => None,
  }
}

fn search_matches(language: PatternLanguage, source: &str, query: &str) -> Vec<Match> {
  match language {
    PatternLanguage::Rust => collect_matches(Rust, source, query),
    PatternLanguage::JavaScript => collect_matches(JavaScript, source, query),
    PatternLanguage::TypeScript => collect_matches(TypeScript, source, query),
    PatternLanguage::Tsx => collect_matches(Tsx, source, query),
    PatternLanguage::Html => collect_matches(Html, source, query),
    PatternLanguage::Css => collect_matches(Css, source, query),
    PatternLanguage::Json => collect_matches(Json, source, query),
    PatternLanguage::Yaml => collect_matches(Yaml, source, query),
    PatternLanguage::Python => collect_matches(Python, source, query),
    PatternLanguage::Angular => collect_matches(Angular, source, query),
  }
}

fn collect_matches<L>(lang: L, source: &str, query: &str) -> Vec<Match>
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
    .map(|matched| to_match(matched.into()))
    .collect()
}

fn to_match<D>(node: Node<'_, D>) -> Match
where
  D: ast_grep_core::Doc,
{
  let start = node.start_pos();
  let end = node.end_pos();

  Match {
    start: Position(start.line() + 1, start.column(&node) + 1),
    end: Position(end.line() + 1, end.column(&node) + 1),
  }
}
