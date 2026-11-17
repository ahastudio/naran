//! Parsing of `git blame --porcelain` output.

use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlameLine {
    pub hash: String,
    pub author: String,
    pub text: String,
}

#[must_use]
pub fn parse(output: &str) -> Vec<BlameLine> {
    let mut parser = Parser::default();
    output.split('\n').for_each(|line| parser.feed(line));
    parser.lines
}

/// Remembers the commit of the last header and the author of each commit seen so far.
#[derive(Default)]
struct Parser<'a> {
    authors: HashMap<&'a str, &'a str>,
    hash: &'a str,
    lines: Vec<BlameLine>,
}

impl<'a> Parser<'a> {
    fn feed(&mut self, line: &'a str) {
        match Line::of(line) {
            Line::Text(text) => self.push(text),
            Line::Author(author) => drop(self.authors.insert(self.hash, author)),
            Line::Header(hash) => self.hash = hash,
            Line::Other => {}
        }
    }

    fn push(&mut self, text: &str) {
        self.lines.push(BlameLine {
            hash: self.hash.to_owned(),
            author: self
                .authors
                .get(self.hash)
                .copied()
                .unwrap_or_default()
                .to_owned(),
            text: text.to_owned(),
        });
    }
}

enum Line<'a> {
    Text(&'a str),
    Author(&'a str),
    Header(&'a str),
    Other,
}

impl<'a> Line<'a> {
    fn of(line: &'a str) -> Self {
        line.strip_prefix('\t')
            .map(Line::Text)
            .or_else(|| line.strip_prefix("author ").map(Line::Author))
            .or_else(|| is_header(line).then(|| Line::Header(&line[..40])))
            .unwrap_or(Line::Other)
    }
}

/// `<40-character hash> <original line> <final line> [<line count>]`
fn is_header(line: &str) -> bool {
    let mut parts = line.split(' ');
    parts
        .next()
        .is_some_and(|hash| hash.len() == 40 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
        && parts.next().is_some_and(|part| part.parse::<u32>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grouped_and_repeated_commits() {
        let a = "a".repeat(40);
        let b = "b".repeat(40);
        let output = format!(
            "{a} 1 1 2\nauthor 홍길동\nsummary first\nfilename f\n\t첫 줄\n\
             {a} 2 2\n\t둘째 줄\n\
             {b} 1 3 1\nauthor Ada\nauthor-mail <a@b>\nfilename f\n\t셋째 줄\n"
        );

        let lines = parse(&output);

        assert_eq!(lines.len(), 3);
        assert_eq!(
            (lines[0].hash.as_str(), lines[0].author.as_str()),
            (a.as_str(), "홍길동")
        );
        assert_eq!(lines[1].author, "홍길동");
        assert_eq!(lines[1].text, "둘째 줄");
        assert_eq!(lines[2].author, "Ada");
    }

    #[test]
    fn ignores_everything_else() {
        assert_eq!(parse("summary x\nnot a header\n"), Vec::<BlameLine>::new());
    }
}
