//! A file stays small enough to read in one sitting.

use std::path::Path;

use crate::Finding;

/// The most lines a file may have, tests included.
pub const LIMIT: usize = 300;

/// A finding when the file is longer than the limit.
pub fn check(path: &Path, source: &str) -> Option<Finding> {
    let lines = source.lines().count();
    (lines > LIMIT).then(|| Finding {
        path: path.to_path_buf(),
        line: 1,
        message: format!("the file has {lines} lines"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(source: &str) -> Option<String> {
        check(Path::new("a.rs"), source).map(|finding| finding.message)
    }

    #[test]
    fn a_file_over_the_limit_is_found_with_its_tests() {
        let code = "fn a() {}\n".repeat(LIMIT);
        assert_eq!(lines(&code), None);
        let tests = format!("{code}#[cfg(test)]\nmod tests {{}}\n");
        assert_eq!(lines(&tests).as_deref(), Some("the file has 302 lines"));
    }
}
