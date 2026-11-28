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
const GIT_ENTRIES: [&str; 9] = [
    "HEAD",
    "index",
    "refs",
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "rebase-merge",
    "rebase-apply",
    // The Git entries of the linked worktrees: their HEAD and index.
    "worktrees",
];

struct Root {
    /// The path as registered, which is what gets reported.
    path: PathBuf,
    /// The folder given to the watcher; not `path` for a worktree's folder.
    watched: PathBuf,
    /// The path with links resolved. Events carry this or `watched`, so
    /// matching accepts both.
    resolved: PathBuf,
    ignore: Gitignore,
}

impl Root {
    fn new(path: &Path) -> Self {
        Self::linked(path, path)
    }

    /// The folder of a worktree, which reports `owner`: its changes are
    /// changes of the repository that lists it.
    fn linked(folder: &Path, owner: &Path) -> Self {
        Self {
            path: owner.to_path_buf(),
            watched: folder.to_path_buf(),
            resolved: folder
                .canonicalize()
                .unwrap_or_else(|_| folder.to_path_buf()),
            ignore: Gitignore::new(folder.join(".gitignore")).0,
        }
    }
}

pub struct Watching {
    watcher: RecommendedWatcher,
    /// Dropped with `Watching`, so the periodic tick wakes up and ends at
    /// once instead of leaking a thread that sleeps for `SAFETY_NET`.
    _stop: mpsc::Sender<()>,
    roots: Arc<Mutex<Vec<Root>>>,
}

impl Watching {
    /// Sends the set of changed repositories to `changes` after each burst.
    pub fn start(changes: UnboundedSender<Vec<PathBuf>>) -> notify::Result<Self> {
        let roots = Roots::default();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || debounce(&receiver, &changes));
        let (stop, stop_signal) = mpsc::channel();
        file_watcher(&roots, sender, stop_signal).map(|watcher| Self {
            watcher,
            _stop: stop,
            roots,
        })
    }

    pub fn watch(&mut self, path: &Path) {
        let watcher = &mut self.watcher;
        add_root(&self.roots, path, || {
            watcher.watch(path, RecursiveMode::Recursive).is_ok()
        });
    }

    /// Watches the folder of a worktree. Its changes are reported as changes
    /// of `owner`, the repository that lists it.
    pub fn watch_linked(&mut self, folder: &Path, owner: &Path) {
        let watcher = &mut self.watcher;
        add_linked(&self.roots, Root::linked(folder, owner), || {
            watcher.watch(folder, RecursiveMode::Recursive).is_ok()
        });
    }

    /// Stops watching the worktree folders not in `folders`, such as those of
    /// removed worktrees. A folder that is a repository of its own stays.
    pub fn keep_linked(&mut self, folders: &[PathBuf]) {
        let gone = stale_folders(&self.roots, folders);
        let own_or_kept = |root: &Root| root.watched == root.path || !gone.contains(&root.watched);
        locked(&self.roots).retain(own_or_kept);
        for folder in &gone {
            self.unwatch_unused(folder);
        }
    }

    fn unwatch_unused(&mut self, folder: &Path) {
        let resolved = folder.canonicalize().unwrap_or_else(|_| folder.into());
        if !locked(&self.roots).iter().any(|r| r.resolved == resolved) {
            let _ = self.watcher.unwatch(folder);
        }
    }

    pub fn unwatch(&mut self, path: &Path) {
        let _ = self.watcher.unwatch(path);
        locked(&self.roots).retain(|root| root.path != path);
    }
}

/// The watched worktree folders that `folders` no longer lists. The lock is
/// let go before they are unwatched (see `add_root`).
fn stale_folders(roots: &Mutex<Vec<Root>>, folders: &[PathBuf]) -> Vec<PathBuf> {
    let roots = locked(roots);
    let stale = roots
        .iter()
        .filter(|root| root.watched != root.path && !folders.contains(&root.watched));
    stale.map(|root| root.watched.clone()).collect()
}

/// Registers a worktree folder once `add` has started watching it. A folder
/// that is watched already is left alone, whichever repository it reports.
fn add_linked(roots: &Mutex<Vec<Root>>, root: Root, add: impl FnOnce() -> bool) {
    let known = locked(roots)
        .iter()
        .any(|known| known.resolved == root.resolved);
    if known {
        return;
    }
    if add() {
        locked(roots).push(root);
    }
}

/// Registers `path` as a root once `add` has started watching it.
/// The lock must not be held while `add` runs: notify waits for its event
/// thread, which waits for the lock in `route`, and both would hang.
fn add_root(roots: &Mutex<Vec<Root>>, path: &Path, add: impl FnOnce() -> bool) {
    if locked(roots).iter().any(|root| root.path == path) {
        return;
    }
    if add() {
        let root = Root::new(path);
        locked(roots).push(root);
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
    let root = roots.iter().find(|root| relative(root, path).is_some())?;
    is_relevant(root, path).then_some(root)
}

/// The repositories being watched.
type Roots = Arc<Mutex<Vec<Root>>>;

/// Watches files and starts the periodic tick, both feeding `sender`.
fn file_watcher(
    roots: &Roots,
    sender: mpsc::Sender<PathBuf>,
    stop: Receiver<()>,
) -> notify::Result<RecommendedWatcher> {
    let (shared, ticker) = (Arc::clone(roots), sender.clone());
    let ticking = Ticking {
        roots: Arc::clone(roots),
        sender: ticker,
        stop,
    };
    notify::recommended_watcher(move |result: notify::Result<Event>| {
        route(&shared, &sender, result);
    })
    .inspect(|_watcher| {
        thread::spawn(move || ticking.run());
    })
}

/// Reports every repository again at `SAFETY_NET`, until nobody listens
/// or `stop` is dropped.
struct Ticking {
    roots: Arc<Mutex<Vec<Root>>>,
    sender: mpsc::Sender<PathBuf>,
    stop: Receiver<()>,
}

impl Ticking {
    fn run(&self) {
        self.run_every(SAFETY_NET);
    }

    fn run_every(&self, every: Duration) {
        loop {
            if self.stop.recv_timeout(every) != Err(RecvTimeoutError::Timeout) {
                return;
            }
            if !report_all(&self.roots, &self.sender) {
                return;
            }
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

/// `path` inside the folder, as registered or with links resolved.
fn relative<'a>(root: &Root, path: &'a Path) -> Option<&'a Path> {
    let resolved = path.strip_prefix(&root.resolved);
    resolved.or_else(|_| path.strip_prefix(&root.watched)).ok()
}

fn is_relevant(root: &Root, path: &Path) -> bool {
    let Some(relative) = relative(root, path) else {
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
