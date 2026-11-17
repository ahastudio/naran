//! Reading and writing the JSON files in the app config folder.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Bundle identifier; the saved files live in the config folder of this name.
const IDENTIFIER: &str = "dev.naran.app";

#[must_use]
pub fn config_directory() -> PathBuf {
    config_directory_in(dirs::config_dir())
}

/// The app folder inside the user's config folder, or inside the temporary
/// folder when the system has none.
fn config_directory_in(config: Option<PathBuf>) -> PathBuf {
    config.unwrap_or_else(std::env::temp_dir).join(IDENTIFIER)
}

/// Returns the default value when the file is missing or broken.
pub fn load<T: DeserializeOwned + Default>(file: &Path) -> T {
    fs::read(file)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Writes to a temporary file first and renames it, so an interrupted
/// write never leaves a broken file behind.
pub fn save<T: Serialize>(file: &Path, value: &T) -> io::Result<()> {
    if let Some(directory) = file.parent() {
        fs::create_dir_all(directory)?;
    }
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    fs::rename(temporary, file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_falls_back_to_default() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("a/b.json");
        assert_eq!(load::<Vec<u32>>(&file), Vec::<u32>::new());

        save(&file, &vec![1, 2]).unwrap();
        assert_eq!(load::<Vec<u32>>(&file), [1, 2]);

        fs::write(&file, "broken").unwrap();
        assert_eq!(load::<Vec<u32>>(&file), Vec::<u32>::new());
    }

    #[test]
    fn the_folder_is_inside_the_config_folder_or_the_temporary_one() {
        let named = |base: &Path| base.join("dev.naran.app");
        assert_eq!(
            config_directory_in(Some("/config".into())),
            named(Path::new("/config"))
        );
        assert_eq!(config_directory_in(None), named(&std::env::temp_dir()));
        assert!(config_directory().ends_with("dev.naran.app"));
    }

    #[test]
    fn a_value_that_is_not_json_cannot_be_saved() {
        let directory = tempfile::tempdir().unwrap();
        let keyed_by_pairs = std::collections::HashMap::from([((1, 2), 3)]);

        assert!(save(&directory.path().join("pairs.json"), &keyed_by_pairs).is_err());
    }

    #[test]
    fn a_file_inside_something_that_is_not_a_folder_cannot_be_saved() {
        let directory = tempfile::tempdir().unwrap();
        let blocker = directory.path().join("blocker");
        fs::write(&blocker, "a file").unwrap();

        assert!(save(&blocker.join("settings.json"), &vec![1]).is_err());
        assert!(save(Path::new(""), &vec![1]).is_err(), "no folder to make");
    }
}
