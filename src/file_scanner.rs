use std::error::Error;
use std::path::PathBuf;

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use path_clean::PathClean;

pub struct FileScanner;

impl FileScanner {
  pub fn new() -> Self {
    FileScanner
  }

  pub fn scan_files(&self, root_dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files: Vec<PathBuf> = Vec::new();

    let mut override_builder = OverrideBuilder::new(root_dir);
    override_builder.add("!.git/**")?; // Ignore .git directory

    let overrides = override_builder.build().unwrap();

    for result in WalkBuilder::new(root_dir.as_path())
      .hidden(false)
      .overrides(overrides)
      .build()
    {
      let entry = result?;
      let path = entry.path();
      if path.is_file() {
        files.push(path.clean());
      }
    }

    Ok(files)
  }
}
