//! Which list a changed file shows in, and the letter on its badge.

use naran_core::{Change, Staging};

/// Where a changed file stands, from the `git status` letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileState {
    pub kind: FileKind,
    /// Has changes in the index.
    pub staged: bool,
    /// Has changes in the working directory that are not staged.
    pub unstaged: bool,
}

/// Whether git follows a changed file, does not know it yet, or is stuck
/// merging it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Tracked,
    Untracked,
    Conflicted,
}

impl FileKind {
    fn of(change: &Change) -> Self {
        if change.conflicted {
            return Self::Conflicted;
        }
        if change.index == '?' {
            return Self::Untracked;
        }
        Self::Tracked
    }
}

impl FileState {
    /// A conflicted file waits in the working directory list until it is
    /// staged; its letters only describe the two sides.
    pub fn of(change: &Change) -> Self {
        let kind = FileKind::of(change);
        let tracked = kind == FileKind::Tracked;
        let staged = tracked && change.index != '.';
        let unstaged = !tracked || change.worktree != '.';
        Self {
            kind,
            staged,
            unstaged,
        }
    }

    pub fn untracked(self) -> bool {
        self.kind == FileKind::Untracked
    }

    /// The side an open diff stays on: its own while the file still has
    /// changes there, otherwise the other one, or none.
    pub fn open_side(self, side: Staging) -> Option<Staging> {
        [side, other_side(side)]
            .into_iter()
            .find(|&side| self.listed_in(side))
    }

    /// Whether the file shows in the Staged list or in the Unstaged list.
    pub fn listed_in(self, side: Staging) -> bool {
        match side {
            Staging::Staged => self.staged,
            Staging::Unstaged => self.unstaged,
        }
    }
}

fn other_side(side: Staging) -> Staging {
    match side {
        Staging::Staged => Staging::Unstaged,
        Staging::Unstaged => Staging::Staged,
    }
}

/// One letter for the badge before a changed file: M modified, A added,
/// D deleted, R renamed, C copied, T type changed, ? untracked, ! conflict.
pub fn change_badge(change: &Change, side: Staging) -> char {
    if change.conflicted {
        return '!';
    }
    let letter = match side {
        Staging::Staged => change.index,
        Staging::Unstaged => change.worktree,
    };
    match letter {
        letter @ ('?' | 'A' | 'D' | 'R' | 'C' | 'T') => letter,
        _ => 'M',
    }
}

/// The text of a changed file's row: its path, and for a file renamed or
/// copied on that side the new path before the path it came from
/// (`new <- old`).
pub fn change_label(change: &Change, side: Staging) -> String {
    match (&change.original, change_badge(change, side)) {
        (Some(original), 'R' | 'C') => format!("{} <- {original}", change.path),
        _ => change.path.clone(),
    }
}
