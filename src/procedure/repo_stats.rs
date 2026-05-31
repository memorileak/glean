use std::result::Result;

use jsonrpsee::types::ErrorObjectOwned;
use serde::Serialize;

use crate::database::RepositoryRepo;
use crate::file_scanner::FileScanner;

use super::internal_error;

#[derive(Clone, Serialize)]
pub struct RepoStats {
  id: String,
  sequence: u32,
  path: String,
  file_count: usize,
  status: &'static str,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<String>,
}

pub fn handle_repo_stats() -> Result<Vec<RepoStats>, ErrorObjectOwned> {
  let mut repo_repo = RepositoryRepo::new();
  let file_scanner = FileScanner::new();
  let repositories = repo_repo.list_repositories().map_err(internal_error)?;

  Ok(
    repositories
      .into_iter()
      .map(|repo| match file_scanner.scan_files(&repo.path) {
        Ok(files) => RepoStats {
          id: repo.id,
          sequence: repo.sequence,
          path: repo.path.display().to_string(),
          file_count: files.len(),
          status: "ok",
          error: None,
        },
        Err(err) => RepoStats {
          id: repo.id,
          sequence: repo.sequence,
          path: repo.path.display().to_string(),
          file_count: 0,
          status: "error",
          error: Some(err.to_string()),
        },
      })
      .collect(),
  )
}
