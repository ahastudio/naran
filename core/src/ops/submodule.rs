//! Listing submodules and their state.

use std::path::Path;

use serde::Serialize;

use crate::{Error, git};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Submodule {
    pub path: String,
    pub hash: String,
    /// `ok`, `uninitialized`, `modified`, `conflict`
    pub state: String,
}

pub fn submodules(repository: &Path) -> Result<Vec<Submodule>, Error> {
    let output = git::read(repository, &["submodule", "status"])?;
    Ok(output.lines().filter_map(parse_submodule).collect())
}

fn parse_submodule(line: &str) -> Option<Submodule> {
    let state = submodule_state(line.chars().next()?)?;
    match line[1..].split(' ').collect::<Vec<_>>()[..] {
        [hash, path, ..] => Some(Submodule {
            hash: hash.to_owned(),
            path: path.to_owned(),
            state: state.to_owned(),
        }),
        _ => None,
    }
}

fn submodule_state(mark: char) -> Option<&'static str> {
    [
        (' ', "ok"),
        ('-', "uninitialized"),
        ('+', "modified"),
        ('U', "conflict"),
    ]
    .into_iter()
    .find_map(|(known, state)| (known == mark).then_some(state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_that_are_cut_short_are_skipped() {
        assert!(parse_submodule("").is_none());
        assert!(parse_submodule("?abc path").is_none(), "unknown mark");
        assert!(parse_submodule(" abc").is_none(), "no path");
        assert_eq!(parse_submodule(" abc path (v1)").unwrap().path, "path");
    }

    #[test]
    fn parses_submodules() {
        let modified = parse_submodule("+abc123 libs/a (heads/main)").unwrap();
        assert_eq!(
            (modified.state.as_str(), modified.path.as_str()),
            ("modified", "libs/a")
        );
        assert_eq!(
            parse_submodule("-abc libs/b").unwrap().state,
            "uninitialized"
        );
        assert_eq!(parse_submodule(" abc libs/c (v1)").unwrap().state, "ok");
        assert_eq!(parse_submodule("Uabc libs/d").unwrap().state, "conflict");
        assert_eq!(parse_submodule("?? nope"), None);
        assert_eq!(parse_submodule(""), None);
    }
}
