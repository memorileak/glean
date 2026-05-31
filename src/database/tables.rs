use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::result::Result;

use redb::{Database, TableDefinition};

use super::repository_repo::RepositoryVal;
use crate::glean_path;

pub const DB_PATH: &'static str = glean_path!("/database/glean.redb");

pub const TBL_REPOSITORY_SEQ: TableDefinition<u8, u32> = TableDefinition::new("TBL_REPOSITORY_SEQ");
pub const TBL_REPOSITORIES: TableDefinition<String, RepositoryVal> =
  TableDefinition::new("TBL_REPOSITORIES");

pub struct DatabaseInitializer;

impl DatabaseInitializer {
  pub fn initialize() -> Result<(), Box<dyn Error>> {
    let db_path = PathBuf::from(DB_PATH);
    if let Some(parent) = db_path.parent() {
      fs::create_dir_all(parent)?;
    }
    let db = Database::create(db_path.as_path())?;
    let rw_txn = db.begin_write()?;
    rw_txn.open_table(TBL_REPOSITORY_SEQ)?;
    rw_txn.open_table(TBL_REPOSITORIES)?;
    rw_txn.commit()?;
    Ok(())
  }
}
