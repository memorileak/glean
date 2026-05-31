use std::path::PathBuf;
use std::result::Result;

use jsonrpsee::types::ErrorObjectOwned;
use serde::Deserialize;

use crate::database::{RepositoryNew, RepositoryRepo};

use super::invalid_params;

#[derive(Deserialize)]
pub struct AddRepoParams {
  id: String,
  path: String,
}

pub fn handle_add_repo(repos: Vec<AddRepoParams>) -> Result<(), ErrorObjectOwned> {
  let new_repos = repos
    .into_iter()
    .map(|repo| {
      let path = PathBuf::from(&repo.path);
      if !path.is_absolute() {
        return Err(invalid_params(format!(
          "repository path must be absolute for id '{}'",
          repo.id
        )));
      }

      Ok(RepositoryNew { id: repo.id, path })
    })
    .collect::<Result<Vec<_>, _>>()?;

  let mut repo_repo = RepositoryRepo::new();
  repo_repo
    .add_repositories(&new_repos)
    .map_err(invalid_params)?;

  Ok(())
}
