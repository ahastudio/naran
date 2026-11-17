//! The bytes of a file, read to show it as a picture.

use std::path::Path;

use crate::{Error, check_hash, git};

/// Where a file is read from when it is shown as a picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Side {
    /// The last commit.
    Head,
    /// The staging area.
    Index,
    /// A commit.
    Commit(String),
    /// The first parent of a commit.
    ParentOf(String),
}

/// Files bigger than this are not read as pictures.
pub const BLOB_LIMIT: u64 = 8 * 1024 * 1024;

/// The bytes of a file as git has it. `None` when git has no such file there
/// (a new file has no earlier version, a first commit has no parent).
pub fn blob(repository: &Path, side: &Side, path: &str) -> Result<Option<Vec<u8>>, Error> {
    let object = side.spec(path)?;
    let Ok(length) = git::read(repository, &["cat-file", "-s", &object]) else {
        return Ok(None);
    };
    check_size(path, length.trim().parse().unwrap_or(0))?;
    git::read_bytes(repository, &["cat-file", "blob", &object]).map(Some)
}

impl Side {
    /// The `<revision>:<path>` text that names the file for `git cat-file`.
    fn spec(&self, path: &str) -> Result<String, Error> {
        match self {
            Self::Head => Ok(format!("HEAD:{path}")),
            Self::Index => Ok(format!(":{path}")),
            Self::Commit(hash) => check_hash(hash).map(|()| format!("{hash}:{path}")),
            Self::ParentOf(hash) => check_hash(hash).map(|()| format!("{hash}^:{path}")),
        }
    }
}

fn check_size(path: &str, size: u64) -> Result<(), Error> {
    if size <= BLOB_LIMIT {
        return Ok(());
    }
    Err(Error::InvalidArgument {
        message: format!("{path} is {size} bytes, more than the {BLOB_LIMIT} shown"),
    })
}

/// The bytes of a file in the working directory. `None` when it is gone.
pub fn worktree_file(repository: &Path, path: &str) -> Result<Option<Vec<u8>>, Error> {
    let file = repository.join(path);
    let Ok(meta) = std::fs::metadata(&file) else {
        return Ok(None);
    };
    check_size(path, meta.len())?;
    std::fs::read(&file)
        .map(Some)
        .map_err(|source| Error::InvalidArgument {
            message: format!("{path}: {source}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_over_the_limit_is_refused() {
        assert!(check_size("small.bin", BLOB_LIMIT).is_ok());
        let error = check_size("big.bin", BLOB_LIMIT + 1).unwrap_err();
        assert!(matches!(error, Error::InvalidArgument { message } if message.contains("big.bin")));
    }
}
