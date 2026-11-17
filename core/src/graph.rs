//! Lane (column) layout of the commit graph.
//!
//! Walks commits from top to bottom and remembers which commit each lane is waiting for.
//! A lane keeps its number once assigned, so lanes passing through stay in the same place.

use serde::Serialize;

use crate::log::Commit;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    /// Lane where this commit's dot sits.
    pub lane: usize,
    /// Lanes coming into this commit from above.
    pub incoming: Vec<usize>,
    /// Lanes passing by without touching this commit.
    pub through: Vec<usize>,
    /// Lanes leaving this commit downward toward its parents.
    pub outgoing: Vec<usize>,
}

#[must_use]
pub fn layout(commits: &[Commit]) -> Vec<Row> {
    let mut lanes = Lanes::default();
    commits.iter().map(|commit| lanes.step(commit)).collect()
}

/// For each lane, the commit it is waiting for. `None` is a free lane.
#[derive(Default)]
struct Lanes<'a>(Vec<Option<&'a str>>);

impl<'a> Lanes<'a> {
    fn step(&mut self, commit: &'a Commit) -> Row {
        let incoming = self.waiting_for(&commit.hash);
        let through = self.passing(&incoming);
        let lane = self.take_lane(&incoming);
        let outgoing = self.leave(lane, &commit.parents);
        Row {
            lane,
            incoming,
            through,
            outgoing,
        }
    }

    fn waiting_for(&self, hash: &str) -> Vec<usize> {
        let waiting = |lane: Option<&str>| lane == Some(hash);
        self.0
            .iter()
            .enumerate()
            .filter(|(_, lane)| waiting(**lane))
            .map(|(index, _)| index)
            .collect()
    }

    fn passing(&self, incoming: &[usize]) -> Vec<usize> {
        self.0
            .iter()
            .enumerate()
            .filter(|(index, lane)| lane.is_some() && !incoming.contains(index))
            .map(|(index, _)| index)
            .collect()
    }

    /// The commit's dot sits in its first incoming lane, or in a free one. Incoming lanes are freed.
    fn take_lane(&mut self, incoming: &[usize]) -> usize {
        let lane = incoming
            .first()
            .copied()
            .unwrap_or_else(|| self.free_slot());
        for &index in incoming {
            self.0[index] = None;
        }
        lane
    }

    /// Makes every parent wait in a lane and returns those lanes.
    /// The first parent continues in the commit's lane; the others get free lanes.
    fn leave(&mut self, lane: usize, parents: &'a [String]) -> Vec<usize> {
        let lanes = std::iter::once(Some(lane)).chain(std::iter::repeat(None));
        let outgoing = parents
            .iter()
            .zip(lanes)
            .map(|(parent, lane)| self.claim(parent, lane))
            .collect();
        self.trim();
        outgoing
    }

    /// A parent another lane already waits for stays there. Otherwise it goes
    /// to `lane`, or to a free lane when it has none.
    fn claim(&mut self, parent: &'a str, lane: Option<usize>) -> usize {
        if let Some(existing) = self.0.iter().position(|waiting| *waiting == Some(parent)) {
            return existing;
        }
        let slot = lane.unwrap_or_else(|| self.free_slot());
        self.0[slot] = Some(parent);
        slot
    }

    fn free_slot(&mut self) -> usize {
        self.0.iter().position(Option::is_none).unwrap_or_else(|| {
            self.0.push(None);
            self.0.len() - 1
        })
    }

    fn trim(&mut self) {
        while self.0.last() == Some(&None) {
            self.0.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(hash: &str, parents: &[&str]) -> Commit {
        Commit {
            hash: hash.to_owned(),
            parents: parents.iter().map(|parent| (*parent).to_owned()).collect(),
            author: String::new(),
            time: 0,
            refs: Vec::new(),
            subject: String::new(),
        }
    }

    #[test]
    fn linear_history_stays_in_one_lane() {
        let rows = layout(&[commit("c", &["b"]), commit("b", &["a"]), commit("a", &[])]);

        assert!(
            rows.iter()
                .all(|row| row.lane == 0 && row.through.is_empty())
        );
        assert_eq!(rows[0].outgoing, [0]);
        assert_eq!(rows[1].incoming, [0]);
        assert_eq!(rows[2].outgoing, Vec::<usize>::new());
    }

    #[test]
    fn merge_opens_and_closes_a_lane() {
        let rows = layout(&[
            commit("m", &["a", "b"]),
            commit("a", &["r"]),
            commit("b", &["r"]),
            commit("r", &[]),
        ]);

        assert_eq!(rows[0].outgoing, [0, 1]);
        assert_eq!((rows[1].lane, rows[1].through.clone()), (0, vec![1]));
        assert_eq!((rows[2].lane, rows[2].through.clone()), (1, vec![0]));
        assert_eq!(rows[2].outgoing, [0]);
        assert_eq!((rows[3].lane, rows[3].incoming.clone()), (0, vec![0]));
    }

    #[test]
    fn unrelated_tip_takes_a_free_lane() {
        let rows = layout(&[commit("x", &["r"]), commit("y", &["r"]), commit("r", &[])]);

        assert_eq!(rows[0].lane, 0);
        assert_eq!(rows[1].lane, 1);
        assert_eq!(rows[1].through, [0]);
        assert_eq!(rows[1].outgoing, [0]);
        assert_eq!(rows[2].incoming, [0]);
    }

    #[test]
    fn handles_missing_parents_in_a_truncated_log() {
        let rows = layout(&[commit("b", &["a"])]);

        assert_eq!(rows[0].outgoing, [0]);
    }
}
