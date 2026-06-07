use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use jsonrpsee::types::ErrorObjectOwned;
use serde::{Deserialize, Serialize};

use crate::astgrep::matching::{AST_LANGUAGES, AstLanguage};
use crate::database::RepositoryRepo;
use crate::file_scanner::FileScanner;
use crate::types::Match;

use super::{internal_error, invalid_params};

#[derive(Deserialize)]
pub struct SearchPatternParams {
  pub query: String,
}

#[derive(Clone, Serialize)]
pub struct SearchPatternResult {
  repo_id: String,
  file_path: String,
  matches: Vec<Match>,
}

#[derive(Clone)]
struct ScannedFile<'a> {
  repo_id: String,
  file_path: String,
  abs_path: PathBuf,
  ast_language: &'a dyn AstLanguage,
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
          let Some(ast_language) = AST_LANGUAGES.detect(&abs_path) else {
            continue;
          };

          if let Ok(rel_path) = abs_path.strip_prefix(&repo.path) {
            all_files.push(ScannedFile {
              repo_id: repo.id.clone(),
              file_path: rel_path.to_string_lossy().into_owned(),
              abs_path,
              ast_language,
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
  let chunk_size = (all_files.len() + threads - 1) / threads;
  let query = Arc::new(query);
  let mut handles = Vec::with_capacity(threads);

  for chunk in all_files.chunks(chunk_size) {
    let partition = chunk.to_vec();
    let query = Arc::clone(&query);

    handles.push(thread::spawn(move || {
      let mut results = Vec::new();
      for file in partition {
        let Ok(source) = fs::read_to_string(&file.abs_path) else {
          continue;
        };

        let matches = file.ast_language.find_matches(&source, &query);
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
