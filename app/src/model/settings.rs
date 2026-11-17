//! The groups and the selection the app keeps between runs.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::Entry;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    pub paths: Vec<PathBuf>,
}

impl Group {
    /// Puts `path` into the slot `target` has. Both must be in the group.
    fn move_path(&mut self, path: &Path, target: &Path) -> bool {
        match (self.index_of(path), self.index_of(target)) {
            (Some(from), Some(to)) if from != to => self.shift(from, to),
            _ => false,
        }
    }

    fn holds(&self, path: &Path) -> bool {
        self.paths.iter().any(|held| held == path)
    }

    fn index_of(&self, path: &Path) -> Option<usize> {
        self.paths.iter().position(|existing| existing == path)
    }

    fn shift(&mut self, from: usize, to: usize) -> bool {
        let moved = self.paths.remove(from);
        self.paths.insert(to, moved);
        true
    }
}

/// Missing fields fall back to their defaults.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub groups: Vec<Group>,
}

impl Settings {
    pub fn group(&self, name: &str) -> Option<&Group> {
        self.groups.iter().find(|group| group.name == name)
    }

    /// Does nothing when the name is empty or already taken.
    pub fn create_group(&mut self, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() || self.group(name).is_some() {
            return false;
        }
        self.groups.push(Group {
            name: name.to_string(),
            paths: Vec::new(),
        });
        true
    }

    /// Does nothing when the new name is empty or used by another group.
    pub fn rename_group(&mut self, old: &str, new: &str) -> bool {
        let new = new.trim();
        let taken = new.is_empty() || self.group(new).is_some();
        let found = self.groups.iter_mut().find(|group| group.name == old);
        found
            .filter(|_| !taken)
            .map(|group| group.name = new.to_string())
            .is_some()
    }

    /// Whether any group holds the repository.
    pub fn in_any_group(&self, path: &Path) -> bool {
        self.groups.iter().any(|group| group.holds(path))
    }

    pub fn delete_group(&mut self, name: &str) {
        self.groups.retain(|group| group.name != name);
    }

    pub fn add_to_group(&mut self, name: &str, path: &Path) {
        let found = self.groups.iter_mut().find(|group| group.name == name);
        if let Some(group) = found.filter(|group| !group.holds(path)) {
            group.paths.push(path.to_path_buf());
        }
    }

    /// Puts `path` into the slot `target` has in the group. Both must be in
    /// the group. Returns whether the order changed.
    pub fn move_in_group(&mut self, name: &str, path: &Path, target: &Path) -> bool {
        let found = self.groups.iter_mut().find(|group| group.name == name);
        found.is_some_and(|group| group.move_path(path, target))
    }

    /// Puts the group `from` into the slot the group `to` has. Both must
    /// exist. Returns whether the order changed.
    pub fn move_group(&mut self, from: &str, to: &str) -> bool {
        match (self.group_index(from), self.group_index(to)) {
            (Some(source), Some(slot)) if source != slot => {
                let moved = self.groups.remove(source);
                self.groups.insert(slot, moved);
                true
            }
            _ => false,
        }
    }

    fn group_index(&self, name: &str) -> Option<usize> {
        self.groups.iter().position(|group| group.name == name)
    }

    pub fn remove_from_group(&mut self, name: &str, path: &Path) {
        if let Some(group) = self.groups.iter_mut().find(|group| group.name == name) {
            group.paths.retain(|existing| existing != path);
        }
    }
}

/// Entries of a group in the group's order. Paths that have not been read
/// yet are skipped.
pub fn members<'a>(entries: &'a [Entry], group: &Group) -> Vec<&'a Entry> {
    group
        .paths
        .iter()
        .filter_map(|path| entries.iter().find(|entry| entry.path() == path))
        .collect()
}

/// What the user had selected when the app last closed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Selection {
    pub group: Option<String>,
    pub repository: Option<PathBuf>,
}
