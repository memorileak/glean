mod add_repo;
mod remove_repo;
mod repo_stats;
mod search_file;

use std::result::Result as StdResult;

use anyhow::Result as AnyhowResult;
use jsonrpsee::RpcModule;
use jsonrpsee::types::ErrorObjectOwned;

use add_repo::{AddRepoParams, handle_add_repo};
use remove_repo::{RemoveRepoParams, handle_remove_repo};
use repo_stats::{RepoStats, handle_repo_stats};
use search_file::{FileSearchResult, SearchFileParams, handle_search_file};

const REPO_STATS: &str = "repo_stats";
const ADD_REPO: &str = "add_repo";
const REMOVE_REPO: &str = "remove_repo";
const SEARCH_FILE: &str = "search_file";
const SEARCH_PATTERN: &str = "search_pattern";
const GET_FILE_OUTLINE: &str = "get_file_outline";
const GET_FILE_CONTENT: &str = "get_file_content";
const GET_MATCHES_CONTENT: &str = "get_matches_content";
const GET_UI_CONFIG: &str = "get_ui_config";
const SET_UI_CONFIG: &str = "set_ui_config";

pub fn build_rpc_module() -> AnyhowResult<RpcModule<()>> {
  let mut module = RpcModule::new(());

  module
    .register_method::<StdResult<Vec<RepoStats>, ErrorObjectOwned>, _>(REPO_STATS, |_, _, _| {
      handle_repo_stats()
    })?;

  module.register_method::<StdResult<(), ErrorObjectOwned>, _>(ADD_REPO, |params, _, _| {
    let repos: Vec<AddRepoParams> = params.parse()?;
    handle_add_repo(repos)
  })?;

  module.register_method::<StdResult<(), ErrorObjectOwned>, _>(REMOVE_REPO, |params, _, _| {
    let repos: Vec<RemoveRepoParams> = params.parse()?;
    handle_remove_repo(repos)
  })?;

  module.register_method::<StdResult<Vec<FileSearchResult>, ErrorObjectOwned>, _>(
    SEARCH_FILE,
    |params, _, _| {
      let params: SearchFileParams = params.parse()?;
      handle_search_file(params)
    },
  )?;

  module.register_method(SEARCH_PATTERN, |_, _, _| dummy_response(SEARCH_PATTERN))?;
  module.register_method(GET_FILE_OUTLINE, |_, _, _| dummy_response(GET_FILE_OUTLINE))?;
  module.register_method(GET_FILE_CONTENT, |_, _, _| dummy_response(GET_FILE_CONTENT))?;
  module.register_method(GET_MATCHES_CONTENT, |_, _, _| {
    dummy_response(GET_MATCHES_CONTENT)
  })?;
  module.register_method(GET_UI_CONFIG, |_, _, _| dummy_response(GET_UI_CONFIG))?;
  module.register_method(SET_UI_CONFIG, |_, _, _| dummy_response(SET_UI_CONFIG))?;

  Ok(module)
}

fn dummy_response(method: &str) -> String {
  format!("{method}: dummy response")
}

fn invalid_params(err: impl std::fmt::Display) -> ErrorObjectOwned {
  ErrorObjectOwned::owned(-32602, err.to_string(), None::<()>)
}

fn internal_error(err: impl std::fmt::Display) -> ErrorObjectOwned {
  ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>)
}
