//! Builds patches that contain only the selected lines. Used to stage hunks and lines.

use std::ops::Range;

/// `diff` is the `git diff` output of one file. `selected` holds line numbers of `diff.split('\n')`;
/// only added (`+`) and removed (`-`) lines matter.
///
/// A forward patch moves working tree changes into the index; a reverse one
/// takes part of the staged changes back out (for `git apply --reverse`).
/// Unselected lines become context lines or are dropped, so they keep their current state.
#[must_use]
pub fn build(diff: &str, selected: &[usize], direction: Direction) -> Option<String> {
    let lines: Vec<&str> = diff.split('\n').collect();
    let first_hunk = lines.iter().position(|line| line.starts_with("@@"))?;
    let selection = Selection {
        selected,
        direction,
    };
    let hunks: String = hunk_starts(&lines, first_hunk)
        .filter_map(|start| build_hunk(&lines, start..hunk_end(&lines, start), &selection))
        .collect();
    (!hunks.is_empty()).then(|| format!("{}\n{hunks}", plain_header(&lines[..first_hunk])))
}

/// The header of the patch. A rename header would make `git apply` move the
/// file, but only some lines move, so the patch names the new path alone.
fn plain_header(header: &[&str]) -> String {
    let renamed = header
        .iter()
        .find_map(|line| line.strip_prefix("rename to "));
    match renamed {
        Some(path) => renamed_header(path),
        None => header.join("\n"),
    }
}

/// Git quotes a name with a tab, a quote or a backslash in it as a whole, so
/// `a/` and `b/` go inside the quotes.
fn renamed_header(path: &str) -> String {
    let inner = path.trim_matches('"');
    let quoted = path.starts_with('"');
    let name = |side: &str| {
        if quoted {
            format!("\"{side}/{inner}\"")
        } else {
            format!("{side}/{path}")
        }
    };
    let (old, new) = (name("a"), name("b"));
    format!("diff --git {old} {new}\n--- {old}\n+++ {new}")
}

/// Which way a patch is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Applies the diff as it is.
    Forward,
    /// Undoes the diff.
    Reverse,
}

impl Direction {
    /// An unselected change stays as context when undoing it would be a
    /// change of its own: a line added in a reverse patch, or removed in a
    /// forward one. `change` is [`Fate::Added`] or [`Fate::Removed`].
    fn unselected(self, change: &Fate) -> Fate {
        match (self, change) {
            (Self::Reverse, Fate::Added) | (Self::Forward, Fate::Removed) => Fate::Demoted,
            _ => Fate::Dropped,
        }
    }
}

struct Selection<'a> {
    selected: &'a [usize],
    direction: Direction,
}

impl Selection<'_> {
    /// None for the trailing empty line, which is not part of the patch.
    fn fate_of(&self, index: usize, line: &str) -> Option<Fate> {
        let fate = match line.chars().next()? {
            '+' => self.change_fate(index, Fate::Added),
            '-' => self.change_fate(index, Fate::Removed),
            '\\' => Fate::Marker,
            _ => Fate::Unchanged,
        };
        Some(fate)
    }

    fn change_fate(&self, index: usize, change: Fate) -> Fate {
        if self.selected.contains(&index) {
            return change;
        }
        self.direction.unselected(&change)
    }
}

/// What happens to one line of a hunk.
enum Fate {
    Added,
    Removed,
    /// An unselected change that keeps its current state, so it becomes context.
    Demoted,
    Unchanged,
    Dropped,
    /// The "No newline at end of file" marker. It stays only after a kept line.
    Marker,
}

impl Fate {
    fn counts(&self) -> (usize, usize) {
        match self {
            Self::Added => (0, 1),
            Self::Removed => (1, 0),
            Self::Demoted | Self::Unchanged => (1, 1),
            Self::Dropped | Self::Marker => (0, 0),
        }
    }

    fn is_change(&self) -> bool {
        matches!(self, Self::Added | Self::Removed)
    }
}

/// Line numbers where a hunk starts, from the first hunk on.
fn hunk_starts<'a>(lines: &'a [&str], first: usize) -> impl Iterator<Item = usize> + 'a {
    std::iter::successors(Some(first), |&start| {
        Some(hunk_end(lines, start)).filter(|&end| end < lines.len())
    })
}

/// A hunk ends where the next hunk or the next file starts.
fn hunk_end(lines: &[&str], start: usize) -> usize {
    (start + 1..lines.len())
        .find(|&index| starts_section(lines[index]))
        .unwrap_or(lines.len())
}

fn starts_section(line: &str) -> bool {
    ["@@", "diff "]
        .iter()
        .any(|prefix| line.starts_with(prefix))
}

fn build_hunk(lines: &[&str], range: Range<usize>, selection: &Selection) -> Option<String> {
    let start = parse_header(lines[range.start])?;
    let fated = (range.start + 1..range.end).filter_map(|index| {
        Some((
            lines[index].to_owned(),
            selection.fate_of(index, lines[index])?,
        ))
    });
    let hunk = reopen_last_line(fated.collect())
        .into_iter()
        .fold(Hunk::default(), Hunk::with);
    hunk.render(start)
}

/// The last line of a file without a newline cannot stay context when lines
/// are added after it, because the new file has a newline there. It is
/// removed and added again instead.
fn reopen_last_line(mut fated: Vec<(String, Fate)>) -> Vec<(String, Fate)> {
    if let Some(at) = unterminated_context(&fated) {
        let added = format!("+{}", &fated[at].0[1..]);
        fated[at].1 = Fate::Removed;
        fated.insert(at + 2, (added, Fate::Added));
    }
    fated
}

/// Where a context line is followed by the "No newline" marker and then by
/// added lines.
fn unterminated_context(fated: &[(String, Fate)]) -> Option<usize> {
    let ends_file =
        |pair: &[(String, Fate)]| matches!(pair, [(_, Fate::Demoted), (_, Fate::Marker)]);
    let at = fated.windows(2).position(ends_file)?;
    let added_after = fated[at + 2..]
        .iter()
        .any(|(_, fate)| matches!(fate, Fate::Added));
    added_after.then_some(at)
}

#[derive(Default)]
struct Hunk {
    body: Vec<String>,
    old: usize,
    new: usize,
    changed: bool,
    previous_kept: bool,
}

impl Hunk {
    fn with(mut self, (line, fate): (String, Fate)) -> Self {
        self.tally(&fate);
        self.keep(self.text(&fate, &line));
        self
    }

    fn text(&self, fate: &Fate, line: &str) -> Option<String> {
        match fate {
            Fate::Added | Fate::Removed | Fate::Unchanged => Some(line.to_owned()),
            Fate::Demoted => Some(format!(" {}", &line[1..])),
            Fate::Dropped => None,
            Fate::Marker => self.previous_kept.then(|| line.to_owned()),
        }
    }

    fn tally(&mut self, fate: &Fate) {
        let (old, new) = fate.counts();
        self.old += old;
        self.new += new;
        self.changed |= fate.is_change();
    }

    fn keep(&mut self, kept: Option<String>) {
        self.previous_kept = kept.is_some();
        self.body.extend(kept);
    }

    /// None when the selection changes nothing in this hunk.
    fn render(self, start: Start) -> Option<String> {
        let lines: String = self.body.iter().flat_map(|line| [line, "\n"]).collect();
        let (old, new) = (self.old, self.new);
        let (old_start, new_start) = (start.old, start.new);
        self.changed
            .then(|| format!("@@ -{old_start},{old} +{new_start},{new} @@\n{lines}"))
    }
}

/// The first line of a hunk in the old and in the new file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Start {
    old: usize,
    new: usize,
}

/// Reads a and c from `@@ -a[,b] +c[,d] @@ ...`.
fn parse_header(header: &str) -> Option<Start> {
    let mut parts = header.split(' ').skip(1);
    let number = |part: &str, sign: char| {
        let digits = part.strip_prefix(sign)?;
        digits
            .split(',')
            .next()
            .unwrap_or_default()
            .parse::<usize>()
            .ok()
    };
    let old = number(parts.next()?, '-')?;
    let new = number(parts.next()?, '+')?;
    Some(Start { old, new })
}

#[cfg(test)]
mod tests;
