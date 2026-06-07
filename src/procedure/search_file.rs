use std::result::Result;
use std::sync::Arc;
use std::thread;

use jsonrpsee::types::ErrorObjectOwned;
use serde::{Deserialize, Serialize};
use skim::fuzzy_matcher::FuzzyMatcher;
use skim::prelude::SkimMatcherV2;

use crate::database::RepositoryRepo;
use crate::file_scanner::FileScanner;

use super::internal_error;

#[derive(Deserialize)]
pub struct SearchFileParams {
  pub query: String,
  pub limit: Option<usize>,
}

#[derive(Clone, Serialize)]
pub struct FileSearchResult {
  repo_id: String,
  file_path: String,
  score: i64,
}

pub fn handle_search_file(
  params: SearchFileParams,
) -> Result<Vec<FileSearchResult>, ErrorObjectOwned> {
  let threads = 4;

  let query = params.query.trim().to_string();
  let limit = params.limit.unwrap_or(100);

  let mut repo_repo = RepositoryRepo::new();
  let repositories = repo_repo.list_repositories().map_err(internal_error)?;

  let file_scanner = FileScanner::new();
  let mut all_files: Vec<(String, String)> = Vec::new();

  for repo in repositories {
    match file_scanner.scan_files(&repo.path) {
      Ok(files) => {
        for abs_path in files {
          if let Ok(rel_path) = abs_path.strip_prefix(&repo.path) {
            all_files.push((repo.id.clone(), rel_path.to_string_lossy().into_owned()));
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
  let mut handles = Vec::new();

  for chunk in all_files.chunks(chunk_size) {
    let chunk: Vec<(String, String)> = chunk.to_vec();
    let query = Arc::clone(&query);

    handles.push(thread::spawn(move || {
      let matcher = SkimMatcherV2::default();
      let mut results = Vec::new();
      for (repo_id, file_path) in chunk {
        if let Some(score) = matcher.fuzzy_match(&file_path, &query) {
          results.push(FileSearchResult {
            repo_id,
            file_path,
            score,
          });
        }
      }
      results
    }));
  }

  let mut all_results: Vec<FileSearchResult> = Vec::new();
  for handle in handles {
    let results = handle
      .join()
      .map_err(|_| internal_error("thread panicked"))?;
    all_results.extend(results);
  }

  all_results.sort_unstable_by(|a, b| b.score.cmp(&a.score));

  Ok(all_results.into_iter().take(limit).collect())
}
