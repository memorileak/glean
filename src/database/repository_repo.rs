use std::error::Error;
use std::path::PathBuf;

use redb::{Database, ReadableTable, TypeName, Value};

use super::tables::{DB_PATH, TBL_REPOSITORIES, TBL_REPOSITORY_SEQ};

#[derive(Debug)]
pub struct RepositoryVal {
  // First 4 bytes, little-endian u32.
  pub sequence: u32,
  // Remaining bytes, parsed as PathBuf.
  pub path: PathBuf,
}

impl Value for RepositoryVal {
  type SelfType<'a> = RepositoryVal;
  type AsBytes<'a> = Vec<u8>;

  fn type_name() -> TypeName {
    TypeName::new("RepositoryVal")
  }

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    let sequence = u32::from_le_bytes(data[..4].try_into().unwrap());
    let path = PathBuf::from(String::from_utf8(data[4..].to_vec()).unwrap());
    RepositoryVal { sequence, path }
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'b,
  {
    let sequence_bytes = value.sequence.to_le_bytes();
    let path_bytes = value.path.to_str().unwrap().as_bytes();
    let mut buf = Vec::with_capacity(4 + path_bytes.len());
    buf.extend_from_slice(&sequence_bytes);
    buf.extend_from_slice(path_bytes);
    buf
  }
}

#[derive(Debug)]
pub struct RepositoryRec {
  id: String,
  sequence: u32,
  path: PathBuf,
}

#[derive(Debug)]
pub struct RepositoryNew {
  id: String,
  path: PathBuf,
}

pub struct RepositoryRepo {
  db: Option<Database>,
}

impl RepositoryRepo {
  pub fn new() -> Self {
    RepositoryRepo { db: None }
  }

  fn get_db(&mut self) -> Result<&Database, Box<dyn Error>> {
    if self.db.is_some() {
      return Ok(self.db.as_ref().unwrap());
    }
    let database = Database::create(PathBuf::from(DB_PATH).as_path())?;
    self.db = Some(database);
    Ok(self.db.as_ref().unwrap())
  }

  pub fn add_repositories(&mut self, new_repos: &[RepositoryNew]) -> Result<(), Box<dyn Error>> {
    let db = self.get_db()?;
    let rw_txn = db.begin_write()?;

    {
      let mut tbl_seq = rw_txn.open_table(TBL_REPOSITORY_SEQ)?;
      let mut tbl_repos = rw_txn.open_table(TBL_REPOSITORIES)?;

      let mut seq: u32 = if let Some(seq_guard) = tbl_seq.get(&0)? {
        seq_guard.value()
      } else {
        1
      };

      for rp in new_repos {
        tbl_repos.insert(
          &rp.id,
          &RepositoryVal {
            sequence: seq,
            path: rp.path.clone(),
          },
        )?;
        seq += 1;
      }

      tbl_seq.insert(&0, &seq)?;
    }

    rw_txn.commit()?;

    Ok(())
  }
}
