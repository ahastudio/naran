//! Watches repository folders and reports which repositories changed.
//! Events are grouped per repository and debounced, so a burst of file
//! writes becomes one report. A slow periodic tick catches missed events.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use ignore::gitignore::Gitignore;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

const DEBOUNCE: Duration = Duration::from_millis(200);
/// Every repository is reported again at this interval in case an event
/// was lost. Unchanged repositories cost one `git status` each.
const SAFETY_NET: Duration = Duration::from_secs(60);
/// Entries inside `.git` that change the state shown on screen. Large
/// folders such as `objects` and `logs` are ignored.
const GIT_ENTRIES: [&str; 8] = [
    "HEAD",
    "index",
    "refs",
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "rebase-merge",
    "rebase-apply",
];

struct Root {
    /// The path as registered, which is what gets reported.
    path: PathBuf,
    /// The path with symbolic links resolved. File events arrive with
    /// resolved paths (on macOS `/var` becomes `/private/var`, and a linked
    /// work folder becomes its target), so matching uses this one.
    resolved: PathBuf,
    ignore: Gitignore,
}

impl Root {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            resolved: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
            ignore: Gitignore::new(path.join(".gitignore")).0,
        }
    }
}

pub struct Watching {
    watcher: RecommendedWatcher,
    roots: Arc<Mutex<Vec<Root>>>,
}

impl Watching {
    /// Sends the set of changed repositories to `changes` after each burst.
    pub fn start(changes: UnboundedSender<Vec<PathBuf>>) -> notify::Result<Self> {
        let roots = Roots::default();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || debounce(&receiver, &changes));
        file_watcher(&roots, sender).map(|watcher| Self { watcher, roots })
    }

    pub fn watch(&mut self, path: &Path) {
        let mut roots = locked(&self.roots);
        if roots.iter().any(|root| root.path == path) {
            return;
        }
        if self.watcher.watch(path, RecursiveMode::Recursive).is_ok() {
            roots.push(Root::new(path));
        }
    }

    pub fn unwatch(&mut self, path: &Path) {
        let _ = self.watcher.unwatch(path);
        locked(&self.roots).retain(|root| root.path != path);
    }
}

/// Passes on the repositories a file event touched. A failed event carries
/// no paths and is skipped; the periodic tick makes up for it.
fn route(roots: &Mutex<Vec<Root>>, sender: &mpsc::Sender<PathBuf>, result: notify::Result<Event>) {
    if let Ok(event) = result {
        send_roots(&locked(roots), sender, &event.paths);
    }
}

fn send_roots(roots: &[Root], sender: &mpsc::Sender<PathBuf>, paths: &[PathBuf]) {
    for root in paths.iter().filter_map(|path| root_of(roots, path)) {
        let _ = sender.send(root.path.clone());
    }
}

/// The watched repository a changed path belongs to, when the change matters.
fn root_of<'a>(roots: &'a [Root], path: &Path) -> Option<&'a Root> {
    let root = roots.iter().find(|root| path.starts_with(&root.resolved))?;
    is_relevant(root, path).then_some(root)
}

/// The repositories being watched.
type Roots = Arc<Mutex<Vec<Root>>>;

/// Watches files and starts the periodic tick, both feeding `sender`.
fn file_watcher(
    roots: &Roots,
    sender: mpsc::Sender<PathBuf>,
) -> notify::Result<RecommendedWatcher> {
    let (shared, tracked, ticker) = (Arc::clone(roots), Arc::clone(roots), sender.clone());
    notify::recommended_watcher(move |result: notify::Result<Event>| {
        route(&shared, &sender, result);
    })
    .inspect(|_watcher| {
        thread::spawn(move || tick(&tracked, &ticker, SAFETY_NET));
    })
}

/// Reports every repository again at each interval, until nobody listens.
fn tick(roots: &Mutex<Vec<Root>>, ticker: &mpsc::Sender<PathBuf>, every: Duration) {
    loop {
        thread::sleep(every);
        if !report_all(roots, ticker) {
            return;
        }
    }
}

/// False when nobody listens any more.
fn report_all(roots: &Mutex<Vec<Root>>, ticker: &mpsc::Sender<PathBuf>) -> bool {
    let roots = locked(roots);
    roots
        .iter()
        .all(|root| ticker.send(root.path.clone()).is_ok())
}

/// The roots, even when a thread panicked while holding them: the list itself
/// stays whole.
fn locked(roots: &Mutex<Vec<Root>>) -> MutexGuard<'_, Vec<Root>> {
    roots.lock().unwrap_or_else(PoisonError::into_inner)
}

fn is_relevant(root: &Root, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(&root.resolved) else {
        return false;
    };
    let mut components = relative.components();
    match components.next() {
        Some(first) if first.as_os_str() == ".git" => components
            .next()
            .is_some_and(|entry| GIT_ENTRIES.iter().any(|name| entry.as_os_str() == *name)),
        _ => !root
            .ignore
            .matched_path_or_any_parents(relative, path.is_dir())
            .is_ignore(),
    }
}

fn debounce(receiver: &Receiver<PathBuf>, changes: &UnboundedSender<Vec<PathBuf>>) {
    while let Ok(first) = receiver.recv() {
        if !deliver(receiver, changes, first) {
            return;
        }
    }
}

/// Sends the burst that starts with `first`. False when a side is gone.
fn deliver(
    receiver: &Receiver<PathBuf>,
    changes: &UnboundedSender<Vec<PathBuf>>,
    first: PathBuf,
) -> bool {
    let mut pending = HashSet::from([first]);
    collect_burst(receiver, &mut pending)
        && changes
            .unbounded_send(pending.into_iter().collect())
            .is_ok()
}

/// Adds paths until the events stop for `DEBOUNCE`. False when the sender is gone.
fn collect_burst(receiver: &Receiver<PathBuf>, pending: &mut HashSet<PathBuf>) -> bool {
    loop {
        match receiver.recv_timeout(DEBOUNCE) {
            Ok(path) => drop(pending.insert(path)),
            Err(error) => return error == RecvTimeoutError::Timeout,
        }
    }
}

#[cfg(test)]
mod tests;
