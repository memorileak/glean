use redb::TableDefinition;

use super::repository_repo::RepositoryVal;
use crate::glean_path;

pub const DB_PATH: &'static str = glean_path!("/database/glean.redb");

pub const TBL_REPOSITORY_SEQ: TableDefinition<u8, u32> = TableDefinition::new("TBL_REPOSITORY_SEQ");
pub const TBL_REPOSITORIES: TableDefinition<String, RepositoryVal> =
  TableDefinition::new("TBL_REPOSITORIES");
