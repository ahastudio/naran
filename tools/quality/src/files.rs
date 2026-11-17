//! Which files are checked.

use std::fs;
use std::path::{Path, PathBuf};

/// The Rust files in the files and folders given.
pub fn sources(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths.iter().flat_map(|path| rust_files_in(path)).collect()
}

fn rust_files_in(path: &Path) -> Vec<PathBuf> {
    if path.is_dir() {
        return rust_files(path);
    }
    let rust = path.extension().is_some_and(|extension| extension == "rs");
    rust.then(|| path.to_path_buf()).into_iter().collect()
}

fn rust_files(folder: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(folder).into_iter().flatten().flatten();
    let mut paths: Vec<PathBuf> = entries.map(|entry| entry.path()).collect();
    paths.sort();
    paths.iter().flat_map(|path| rust_files_in(path)).collect()
}

/// A file named `tests.rs`, or one inside a `tests` folder.
pub fn is_test(file: &Path) -> bool {
    file.components()
        .any(|component| component.as_os_str() == "tests")
        || file.file_stem().is_some_and(|stem| stem == "tests")
}
