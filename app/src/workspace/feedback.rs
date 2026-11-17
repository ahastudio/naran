//! Saving what the user changed, and telling them how things went.

use gpui::SharedString;

use super::Workspace;
use super::state::Outcome;
use crate::store;

impl Workspace {
    pub(super) fn save_selection(&mut self) {
        if let Err(error) = store::save(&self.files.selection, &self.selection) {
            self.report(
                format!("Could not save the selection: {error}"),
                Outcome::Failed,
            );
        }
    }

    pub(super) fn save_settings(&mut self) {
        if let Err(error) = store::save(&self.files.settings, &self.settings) {
            self.report(
                format!("Could not save the settings: {error}"),
                Outcome::Failed,
            );
        }
    }

    pub(super) fn save_paths(&mut self) {
        if let Err(error) = store::save(&self.files.repositories, &self.paths) {
            self.report(
                format!("Could not save the repository list: {error}"),
                Outcome::Failed,
            );
        }
    }

    /// Says what happened in the status bar. An error is also shown at once as
    /// a red notice at the top of the repository column, so it cannot be missed.
    pub(super) fn report(&mut self, message: impl Into<SharedString>, outcome: Outcome) {
        let message: SharedString = message.into();
        if outcome == Outcome::Failed {
            self.feedback.notice = Some(message.to_string());
        }
        self.feedback.message = Some((message, outcome));
    }
}
