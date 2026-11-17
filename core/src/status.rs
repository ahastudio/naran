//! Parsing of `git status --porcelain=v2 --branch -z` output.

use serde::Serialize;

/// The `git status` arguments every reader uses. Each untracked file is listed
/// on its own; git would otherwise show a new folder as one entry.
pub const ARGUMENTS: [&str; 4] = ["status", "--porcelain=v2", "-z", "--untracked-files=all"];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Current branch, or `None` for a detached HEAD.
    pub branch: Option<String>,
    /// HEAD commit, or `None` before the first commit.
    pub head: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
    pub conflicted: u32,
}

/// One changed file. `index` and `worktree` are the X and Y letters of `git status`; `.` means unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub path: String,
    /// Where a renamed or copied file came from.
    pub original: Option<String>,
    pub index: char,
    pub worktree: char,
    pub conflicted: bool,
}

/// Field counts of the records: 9 for ordinary, 10 for renames, 11 for conflicts.
/// The last field is the path and may contain spaces.
const FIELD_COUNTS: [(u8, usize); 4] = [(b'1', 9), (b'2', 10), (b'u', 11), (b'?', 2)];

#[must_use]
pub fn parse_changes(output: &str) -> Vec<Change> {
    entries(output)
        .filter_map(|(record, original)| change_of(record, original))
        .collect()
}

/// Records of the output, each with its original path. For renames and copies the original path
/// arrives as the next record, which is not an entry of its own.
fn entries(output: &str) -> impl Iterator<Item = (&str, Option<&str>)> {
    let mut records = output.split('\0').filter(|record| !record.is_empty());
    std::iter::from_fn(move || next_entry(&mut records))
}

fn next_entry<'a>(
    records: &mut impl Iterator<Item = &'a str>,
) -> Option<(&'a str, Option<&'a str>)> {
    let record = records.next()?;
    let original = if kind_of(record) == b'2' {
        records.next()
    } else {
        None
    };
    Some((record, original))
}

fn kind_of(record: &str) -> u8 {
    record.as_bytes()[0]
}

fn change_of(record: &str, original: Option<&str>) -> Option<Change> {
    let kind = kind_of(record);
    let (_, fields) = FIELD_COUNTS.iter().find(|(known, _)| *known == kind)?;
    let (index, worktree) = codes_of(kind, record);
    Some(Change {
        path: record
            .splitn(*fields, ' ')
            .last()
            .unwrap_or_default()
            .to_owned(),
        original: original.map(str::to_owned),
        index,
        worktree,
        conflicted: kind == b'u',
    })
}

fn codes_of(kind: u8, record: &str) -> (char, char) {
    if kind == b'?' {
        return ('?', '?');
    }
    let mut codes = record.chars().skip(2);
    (codes.next().unwrap_or('.'), codes.next().unwrap_or('.'))
}

#[must_use]
pub fn parse(output: &str) -> Status {
    let mut status = Status::default();
    entries(output).for_each(|(record, _)| status.add(record));
    status
}

impl Status {
    fn add(&mut self, record: &str) {
        match record.strip_prefix("# ") {
            Some(header) => parse_header(header, self),
            None => self.count(record),
        }
    }

    fn count(&mut self, record: &str) {
        match kind_of(record) {
            b'1' | b'2' => count_change(record, self),
            b'u' => self.conflicted += 1,
            b'?' => self.untracked += 1,
            _ => {}
        }
    }
}

fn parse_header(header: &str, status: &mut Status) {
    let Some((key, value)) = header.split_once(' ') else {
        return;
    };
    match key {
        "branch.oid" if value != "(initial)" => status.head = Some(value.to_owned()),
        "branch.head" if value != "(detached)" => status.branch = Some(value.to_owned()),
        _ => parse_tracking_header(key, value, status),
    }
}

fn parse_tracking_header(key: &str, value: &str, status: &mut Status) {
    match key {
        "branch.upstream" => status.upstream = Some(value.to_owned()),
        "branch.ab" => parse_ahead_behind(value, status),
        _ => {}
    }
}

fn parse_ahead_behind(value: &str, status: &mut Status) {
    for part in value.split(' ') {
        match part.split_at_checked(1) {
            Some(("+", number)) => status.ahead = number.parse().unwrap_or(0),
            Some(("-", number)) => status.behind = number.parse().unwrap_or(0),
            _ => {}
        }
    }
}

fn count_change(record: &str, status: &mut Status) {
    let mut codes = record.as_bytes()[2..4].iter();
    if codes.next() != Some(&b'.') {
        status.staged += 1;
    }
    if codes.next() != Some(&b'.') {
        status.unstaged += 1;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_ahead_behind_part_that_is_neither_is_skipped() {
        let mut status = Status::default();
        parse_ahead_behind("+3 ? -4  ", &mut status);
        assert_eq!((status.ahead, status.behind), (3, 4));
    }

    use super::*;

    #[test]
    fn empty_repository() {
        let status = parse("# branch.oid (initial)\0# branch.head main\0");

        assert_eq!(status.branch.as_deref(), Some("main"));
        assert_eq!(status.head, None);
        assert_eq!(status.upstream, None);
    }

    #[test]
    fn branch_with_upstream() {
        let status = parse(
            "# branch.oid abc123\0# branch.head main\0# branch.upstream origin/main\0# branch.ab +2 -3\0",
        );

        assert_eq!(status.head.as_deref(), Some("abc123"));
        assert_eq!(status.upstream.as_deref(), Some("origin/main"));
        assert_eq!((status.ahead, status.behind), (2, 3));
    }

    #[test]
    fn detached_head() {
        let status = parse("# branch.oid abc123\0# branch.head (detached)\0");

        assert_eq!(status.branch, None);
    }

    #[test]
    fn changes() {
        let output = [
            "1 M. N... 100644 100644 100644 a a 스테이지된 파일.txt",
            "1 .M N... 100644 100644 100644 a a 바뀐 파일.txt",
            "1 MM N... 100644 100644 100644 a a both.txt",
            "2 R. N... 100644 100644 100644 a a R100 new name.txt",
            "old name.txt",
            "u UU N... 100644 100644 100644 100644 a b c conflict.txt",
            "? untracked.txt",
            "! ignored.txt",
        ]
        .join("\0");

        let status = parse(&output);

        assert_eq!(status.staged, 3);
        assert_eq!(status.unstaged, 2);
        assert_eq!(status.conflicted, 1);
        assert_eq!(status.untracked, 1);
    }

    #[test]
    fn lists_changed_files() {
        let output = [
            "# branch.head main",
            "1 M. N... 100644 100644 100644 a a 스테이지 됨.txt",
            "2 R. N... 100644 100644 100644 a a R100 new name.txt",
            "old name.txt",
            "u UU N... 100644 100644 100644 100644 a b c conflict.txt",
            "? untracked.txt",
            "! ignored.txt",
        ]
        .join("\0");

        let changes = parse_changes(&output);

        let paths: Vec<_> = changes.iter().map(|change| change.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "스테이지 됨.txt",
                "new name.txt",
                "conflict.txt",
                "untracked.txt"
            ]
        );
        assert_eq!((changes[0].index, changes[0].worktree), ('M', '.'));
        assert_eq!(changes[1].original.as_deref(), Some("old name.txt"));
        assert_eq!(changes[0].original, None);
        assert_eq!(changes[3].original, None);
        assert!(changes[2].conflicted);
        assert_eq!((changes[3].index, changes[3].worktree), ('?', '?'));
    }

    #[test]
    fn malformed_headers_are_ignored() {
        let status = parse("# nonsense\0# branch.ab +x -y\0# other value\0");

        assert_eq!(status, Status::default());
    }
}
