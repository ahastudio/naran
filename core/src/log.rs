//! Parsing of `git log` output.

use serde::Serialize;

pub const FORMAT: &str = "--format=%H%x1f%P%x1f%an%x1f%at%x1f%D%x1f%s";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub time: i64,
    pub refs: Vec<String>,
    pub subject: String,
}

/// Output read with `-z`: commits are separated by NUL, fields by `\x1f`.
pub fn parse(output: &str) -> Vec<Commit> {
    output.split('\0').filter_map(parse_record).collect()
}

fn parse_record(record: &str) -> Option<Commit> {
    let fields: Vec<&str> = record.trim_start_matches('\n').splitn(6, '\x1f').collect();
    let [hash, parents, author, time, refs, subject] = fields.try_into().ok()?;
    if hash.is_empty() {
        return None;
    }
    Some(Commit {
        hash: hash.to_owned(),
        parents: parents.split_whitespace().map(str::to_owned).collect(),
        author: author.to_owned(),
        time: time.parse().unwrap_or(0),
        refs: refs
            .split(", ")
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .collect(),
        subject: subject.to_owned(),
    })
}

/// Everything `git show` says about one commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    pub hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub author_email: String,
    /// Local time, `YYYY-MM-DD HH:MM:SS`.
    pub author_date: String,
    pub committer: String,
    pub committer_email: String,
    pub committer_date: String,
    /// The whole message: subject, a blank line, then the body.
    pub message: String,
}

/// Arguments after `git show -s`; the message comes last so it may hold anything.
pub const DETAIL_FORMAT: &str = "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%ad%x1f%cn%x1f%ce%x1f%cd%x1f%B";
pub const DETAIL_DATE: &str = "--date=format-local:%Y-%m-%d %H:%M:%S";

#[must_use]
pub fn parse_detail(output: &str) -> Option<CommitDetail> {
    let fields: Vec<&str> = output.splitn(9, '\x1f').collect();
    let fields: [&str; 9] = fields.try_into().ok()?;
    let hash = Some(fields[0].trim()).filter(|hash| !hash.is_empty())?;
    Some(CommitDetail::from_fields(hash, &fields))
}

impl CommitDetail {
    /// The fields of [`DETAIL_FORMAT`], in its order.
    fn from_fields(hash: &str, fields: &[&str; 9]) -> Self {
        let field = |index: usize| fields[index].to_owned();
        Self {
            hash: hash.to_owned(),
            parents: fields[1].split_whitespace().map(str::to_owned).collect(),
            author: field(2),
            author_email: field(3),
            author_date: field(4),
            committer: field(5),
            committer_email: field(6),
            committer_date: field(7),
            message: fields[8].trim_end().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_detail_needs_all_nine_fields_and_a_hash() {
        let detail = "a1\x1fp1 p2\x1fAda\x1fada@x\x1f2026-11-16\x1fBob\x1fbob@x\x1f2026-11-16\x1fsubject\n\nbody\n";
        let parsed = parse_detail(detail).unwrap();
        assert_eq!((parsed.hash.as_str(), parsed.parents.len()), ("a1", 2));
        assert_eq!(parsed.message, "subject\n\nbody");
        assert_eq!(parse_detail("a1\x1fp1\x1fAda"), None, "too few fields");
        assert_eq!(
            parse_detail(&detail.replacen("a1", "  ", 1)),
            None,
            "no hash"
        );
    }

    #[test]
    fn parses_commits() {
        let output = "b\x1fa c\x1f홍길동\x1f1700000000\x1fHEAD -> main, tag: v1\x1f병합\0\
                      a\x1f\x1fAda\x1f1600000000\x1f\x1f처음 | 하나\0";

        let commits = parse(output);

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].parents, ["a", "c"]);
        assert_eq!(commits[0].author, "홍길동");
        assert_eq!(commits[0].time, 1_700_000_000);
        assert_eq!(commits[0].refs, ["HEAD -> main", "tag: v1"]);
        assert_eq!(commits[1].parents, Vec::<String>::new());
        assert_eq!(commits[1].refs, Vec::<String>::new());
        assert_eq!(commits[1].subject, "처음 | 하나");
    }

    #[test]
    fn parses_a_commit_detail_with_a_body() {
        let output = "abc\x1fp1 p2\x1f홍길동\x1fa@x.kr\x1f2026-11-16 21:05:00\x1f나\x1fc@x.kr\x1f2026-11-16 09:00:00\x1f제목\n\n본문 첫 줄\n본문 \x1f 둘째\n\n";
        let detail = parse_detail(output).unwrap();
        assert_eq!(detail.hash, "abc");
        assert_eq!(detail.parents, ["p1", "p2"]);
        assert_eq!(detail.author_email, "a@x.kr");
        assert_eq!(detail.committer_date, "2026-11-16 09:00:00");
        assert_eq!(detail.message, "제목\n\n본문 첫 줄\n본문 \x1f 둘째");
        assert!(parse_detail("").is_none());
        assert!(parse_detail("abc\x1fonly").is_none());
    }

    #[test]
    fn skips_incomplete_records() {
        assert_eq!(parse(""), Vec::<Commit>::new());
        assert_eq!(parse("only-hash\0"), Vec::<Commit>::new());
        assert_eq!(parse("\x1fx\x1fy\x1f1\x1f\x1fz\0"), Vec::<Commit>::new());
    }

    #[test]
    fn tolerates_bad_time() {
        assert_eq!(parse("a\x1f\x1fn\x1fnope\x1f\x1fs")[0].time, 0);
    }
}
