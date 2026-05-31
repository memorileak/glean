use anyhow::Result;
use jsonrpsee::RpcModule;

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

pub fn build_rpc_module() -> Result<RpcModule<()>> {
  let mut module = RpcModule::new(());

  module.register_method(REPO_STATS, |_, _, _| dummy_response(REPO_STATS))?;
  module.register_method(ADD_REPO, |_, _, _| dummy_response(ADD_REPO))?;
  module.register_method(REMOVE_REPO, |_, _, _| dummy_response(REMOVE_REPO))?;
  module.register_method(SEARCH_FILE, |_, _, _| dummy_response(SEARCH_FILE))?;
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
