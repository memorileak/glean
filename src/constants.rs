// Define constants used across the project.

// The path to the Glean data directory.
// Store all data to the ".glean" directory
// in the working directory where the application is run.
#[macro_export]
macro_rules! glean_path {
  () => {
    ".glean"
  };
  ($suffix:literal) => {
    concat!(".glean", $suffix)
  };
}
