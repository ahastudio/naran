//! Reading `git diff` output into the rows of the diff view.

use std::ops::Range;

/// Kind of one line in `git diff` output, which decides its color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Added,
    Removed,
    Hunk,
    Meta,
    Context,
}

/// The first prefix that matches decides the kind; a line with none is context.
const LINE_KINDS: [(&str, LineKind); 14] = [
    ("+++", LineKind::Meta),
    ("---", LineKind::Meta),
    ("+", LineKind::Added),
    ("-", LineKind::Removed),
    ("@@", LineKind::Hunk),
    ("diff ", LineKind::Meta),
    ("index ", LineKind::Meta),
    ("new file", LineKind::Meta),
    ("deleted file", LineKind::Meta),
    ("old mode", LineKind::Meta),
    ("new mode", LineKind::Meta),
    ("similarity", LineKind::Meta),
    ("rename ", LineKind::Meta),
    ("\\", LineKind::Meta),
];

pub fn classify(line: &str) -> LineKind {
    let found = LINE_KINDS
        .iter()
        .find(|(prefix, _)| line.starts_with(prefix));
    found.map_or(LineKind::Context, |(_, kind)| *kind)
}

/// Line ranges of each hunk: from its `@@` header up to the next header or
/// the end.
pub fn hunks(lines: &[String]) -> Vec<Range<usize>> {
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| classify(line) == LineKind::Hunk)
        .map(|(index, _)| index)
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(position, start)| *start..starts.get(position + 1).copied().unwrap_or(lines.len()))
        .collect()
}

/// Indexes of the added and removed lines inside `range`.
pub fn changed_lines(lines: &[String], range: Range<usize>) -> Vec<usize> {
    range
        .filter(|index| {
            matches!(
                classify(&lines[*index]),
                LineKind::Added | LineKind::Removed
            )
        })
        .collect()
}

/// Shows the whole content of a file Git does not track yet as added lines.
pub fn untracked_as_diff(content: &str) -> Vec<String> {
    content.lines().map(|line| format!("+{line}")).collect()
}

/// One row of the diff view. `index` is the line in the `git diff` output it
/// comes from, which the staging code uses to name hunks and lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffRow {
    Hunk {
        index: usize,
        title: String,
    },
    Line {
        index: usize,
        kind: LineKind,
        old: Option<usize>,
        new: Option<usize>,
        text: String,
    },
    Note {
        index: usize,
        text: String,
    },
}

impl DiffRow {
    pub fn index(&self) -> usize {
        match self {
            Self::Hunk { index, .. } | Self::Line { index, .. } | Self::Note { index, .. } => {
                *index
            }
        }
    }
}

/// Turns `git diff` output into what the diff view shows: the file header
/// lines (`diff --git`, `index`, `---`, `+++`) are dropped, every hunk gets a
/// title, and every line gets its old and new line numbers and loses its
/// `+`, `-` or space prefix.
pub fn diff_rows(lines: &[String]) -> Vec<DiffRow> {
    let mut builder = DiffBuilder::default();
    lines
        .iter()
        .enumerate()
        .for_each(|(index, line)| builder.feed(index, line));
    builder.rows
}

/// Rows made so far, and the line numbers the next line will get.
struct DiffBuilder {
    rows: Vec<DiffRow>,
    old: usize,
    new: usize,
    hunk: usize,
    in_hunk: bool,
}

impl Default for DiffBuilder {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            old: 1,
            new: 1,
            hunk: 0,
            in_hunk: false,
        }
    }
}

impl DiffBuilder {
    fn feed(&mut self, index: usize, line: &str) {
        match (classify(line) == LineKind::Hunk, self.in_hunk) {
            (true, _) => self.start_hunk(index, line),
            (false, false) => self.before_hunk(index, line),
            (false, true) => self.in_hunk_line(index, line),
        }
    }

    fn start_hunk(&mut self, index: usize, line: &str) {
        self.hunk += 1;
        self.in_hunk = true;
        let title = self.title(line);
        self.rows.push(DiffRow::Hunk { index, title });
    }

    /// Restarts the line numbers at the header's numbers, when it has them.
    fn title(&mut self, line: &str) -> String {
        let Some(ranges) = hunk_header(line) else {
            return format!("Hunk {}", self.hunk);
        };
        (self.old, self.new) = (ranges.0.0, ranges.1.0);
        hunk_title(self.hunk, ranges)
    }

    /// A file Git does not track yet comes as added lines without a header;
    /// anything else here is a message such as "Binary files differ".
    fn before_hunk(&mut self, index: usize, line: &str) {
        match classify(line) {
            LineKind::Meta => {}
            LineKind::Added => self.add(index, line),
            _ => self.note(index, line),
        }
    }

    fn in_hunk_line(&mut self, index: usize, line: &str) {
        match line.chars().next() {
            Some('+') => self.add(index, line),
            Some('-') => self.remove(index, line),
            Some('\\') => self.note(index, line.trim_start_matches('\\').trim()),
            Some(' ') => self.context(index, unprefixed(line)),
            _ => self.context(index, with_spaces(line)),
        }
    }

    fn add(&mut self, index: usize, line: &str) {
        let (kind, old, new) = (LineKind::Added, None, Some(self.new));
        self.rows.push(DiffRow::Line {
            index,
            kind,
            old,
            new,
            text: unprefixed(line),
        });
        self.new += 1;
    }

    fn remove(&mut self, index: usize, line: &str) {
        let (kind, old, new) = (LineKind::Removed, Some(self.old), None);
        self.rows.push(DiffRow::Line {
            index,
            kind,
            old,
            new,
            text: unprefixed(line),
        });
        self.old += 1;
    }

    /// A context line has a space prefix, except when git leaves it out.
    fn context(&mut self, index: usize, text: String) {
        let (kind, old, new) = (LineKind::Context, Some(self.old), Some(self.new));
        self.rows.push(DiffRow::Line {
            index,
            kind,
            old,
            new,
            text,
        });
        (self.old, self.new) = (self.old + 1, self.new + 1);
    }

    fn note(&mut self, index: usize, text: &str) {
        self.rows.push(DiffRow::Note {
            index,
            text: text.to_string(),
        });
    }
}

/// The line without its `+`, `-` or space prefix, with tabs as four spaces.
fn unprefixed(line: &str) -> String {
    with_spaces(line.get(1..).unwrap_or_default())
}

/// Tabs as four spaces, the way the diff view shows them.
fn with_spaces(text: &str) -> String {
    text.replace('\t', "    ")
}

/// The new range, or the old one when nothing is added.
fn shown_range(old: (usize, usize), new: (usize, usize)) -> (usize, usize) {
    if new.1 > 0 { new } else { old }
}

/// `Hunk 2 : Lines 10-14`, from the new range, or the old one when nothing is added.
fn hunk_title(hunk: usize, (old, new): ((usize, usize), (usize, usize))) -> String {
    let (start, length) = shown_range(old, new);
    match length {
        0 | 1 => format!("Hunk {hunk} : Line {start}"),
        _ => format!("Hunk {hunk} : Lines {start}-{}", start + length - 1),
    }
}

/// `@@ -78,5 +78,7 @@ context` as `((78, 5), (78, 7))`.
fn hunk_header(line: &str) -> Option<((usize, usize), (usize, usize))> {
    fn range(text: &str) -> Option<(usize, usize)> {
        match text.split_once(',') {
            Some((start, length)) => Some((start.parse().ok()?, length.parse().ok()?)),
            None => Some((text.parse().ok()?, 1)),
        }
    }
    let rest = line.strip_prefix("@@ -")?;
    let (old, rest) = rest.split_once(" +")?;
    let new = rest.split(" @@").next().unwrap_or_default();
    Some((range(old)?, range(new)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hunk_header_is_read_with_or_without_counts_and_a_broken_one_is_refused() {
        assert_eq!(
            hunk_header("@@ -78,5 +78,7 @@ context"),
            Some(((78, 5), (78, 7)))
        );
        assert_eq!(hunk_header("@@ -3 +4 @@"), Some(((3, 1), (4, 1))));
        for broken in [
            "x",
            "@@ -1",
            "@@ -a +1 @@",
            "@@ -a,1 +1 @@",
            "@@ -1,b +1 @@",
            "@@ -1 +c @@",
            "@@ -1 +1,d @@",
        ] {
            assert_eq!(hunk_header(broken), None, "{broken}");
        }
    }
}
