use jsonrpsee::types::ErrorObjectOwned;
use serde::Deserialize;

use crate::database::RepositoryRepo;

use super::internal_error;

#[derive(Deserialize)]
pub struct RemoveRepoParams {
  id: String,
}

pub fn handle_remove_repo(
  repos: Vec<RemoveRepoParams>,
) -> std::result::Result<(), ErrorObjectOwned> {
  let ids = repos.into_iter().map(|repo| repo.id).collect::<Vec<_>>();
  let mut repo_repo = RepositoryRepo::new();
  repo_repo
    .remove_repositories(&ids)
    .map_err(internal_error)?;
  Ok(())
}
