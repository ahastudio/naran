//! Fetching, pulling and pushing.

use std::path::Path;

use serde::Serialize;

use crate::{Error, Summary, git, summarize};

pub fn fetch(repository: &Path) -> Result<(), Error> {
    git::network(repository, &["fetch", "--prune"]).map(drop)
}

pub fn pull(repository: &Path) -> Result<(), Error> {
    git::network(repository, &["pull"]).map(drop)
}

pub fn push(repository: &Path) -> Result<(), Error> {
    push_with(repository, None)
}

/// Overwrites the remote branch, but only while it is where the last fetch
/// saw it, so commits someone else pushed since are not lost.
pub fn force_push(repository: &Path) -> Result<(), Error> {
    push_with(repository, Some("--force-with-lease"))
}

/// Without an upstream, pushes to `origin` and sets it as the upstream.
fn push_with(repository: &Path, flag: Option<&str>) -> Result<(), Error> {
    let tracking = git::read(repository, &["rev-parse", "--abbrev-ref", "@{upstream}"]).is_ok();
    let upstream_flags = (!tracking).then_some(["-u", "origin", "HEAD"]);
    let arguments: Vec<&str> = ["push"]
        .into_iter()
        .chain(flag)
        .chain(upstream_flags.into_iter().flatten())
        .collect();
    git::network(repository, &arguments).map(drop)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum Outcome {
    Done,
    Skipped { reason: String },
}

/// Pulls only when there is no risk of conflict. Used for batch pulls.
pub fn pull_safe(repository: &Path) -> Result<Outcome, Error> {
    let summary = summarize(repository)?;
    if let Some(reason) = skip_reason(&summary) {
        return Ok(Outcome::Skipped {
            reason: reason.to_owned(),
        });
    }
    git::network(repository, &["pull", "--ff-only"])?;
    Ok(Outcome::Done)
}

/// Why a safe pull must not run. The first reason that applies wins.
fn skip_reason(summary: &Summary) -> Option<&'static str> {
    let state = &summary.status;
    let reasons = [
        (summary.operation.is_some(), "an operation is in progress"),
        (state.conflicted > 0, "there are conflicts"),
        (state.staged + state.unstaged > 0, "there are local changes"),
        (state.upstream.is_none(), "no upstream branch"),
        (state.behind == 0, "not behind"),
        (state.ahead > 0, "local commits would need a merge"),
    ];
    reasons
        .into_iter()
        .find(|(applies, _)| *applies)
        .map(|(_, reason)| reason)
}
