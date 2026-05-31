use std::error::Error;
use std::path::PathBuf;

use ignore::WalkBuilder;
use path_clean::PathClean;

pub struct FileScanner;

impl FileScanner {
  pub fn new() -> Self {
    FileScanner
  }

  pub fn scan_files(&self, root_dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files: Vec<PathBuf> = Vec::new();

    for result in WalkBuilder::new(root_dir.as_path()).hidden(false).build() {
      let entry = result?;
      let path = entry.path();
      if path.is_file() {
        files.push(path.clean());
      }
    }

    Ok(files)
  }
}
