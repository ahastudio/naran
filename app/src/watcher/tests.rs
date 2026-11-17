use futures::StreamExt;

use super::*;

fn root(directory: &Path) -> Root {
    std::fs::write(directory.join(".gitignore"), "target/\n*.log\n").unwrap();
    Root::new(directory)
}

#[test]
fn a_folder_that_cannot_be_watched_is_left_out_and_keeps_its_own_name() {
    let (sender, _receiver) = futures::channel::mpsc::unbounded();
    let mut watching = Watching::start(sender).unwrap();
    let missing = Path::new("/there/is/no/such/folder");

    watching.watch(missing);

    assert!(watching.roots.lock().unwrap().is_empty());
    assert_eq!(Root::new(missing).resolved, missing);
}

#[test]
fn watches_worktree_files_and_git_state() {
    let directory = tempfile::tempdir().unwrap();
    let root = root(directory.path());
    let path = &root.resolved.clone();

    assert!(is_relevant(&root, &path.join("src/main.rs")));
    assert!(is_relevant(&root, &path.join(".git/HEAD")));
    assert!(is_relevant(&root, &path.join(".git/index")));
    assert!(is_relevant(&root, &path.join(".git/refs/heads/main")));
}

#[test]
fn ignores_noise() {
    let directory = tempfile::tempdir().unwrap();
    let root = root(directory.path());
    let path = &root.resolved.clone();

    assert!(!is_relevant(&root, &path.join("target/debug/app")));
    assert!(!is_relevant(&root, &path.join("build.log")));
    assert!(!is_relevant(&root, &path.join(".git/objects/ab/cdef")));
    assert!(!is_relevant(&root, &path.join(".git/index.lock")));
    assert!(!is_relevant(&root, &path.join(".git")));
    assert!(!is_relevant(&root, Path::new("/elsewhere/file")));
}

#[test]
fn reports_a_burst_of_writes_once_with_the_registered_path() {
    // The temporary folder sits behind a symbolic link on macOS, which
    // is exactly the case the registered path must survive.
    let directory = tempfile::tempdir().unwrap();
    let repository = directory.path().to_path_buf();
    let (sender, mut receiver) = futures::channel::mpsc::unbounded();
    let mut watching = Watching::start(sender).unwrap();
    watching.watch(&repository);
    thread::sleep(Duration::from_millis(300));

    for number in 0..5 {
        std::fs::write(repository.join(format!("file{number}.txt")), "x").unwrap();
    }

    // Waits on another thread, so a missing report fails the test
    // after five seconds instead of hanging it.
    let (found, waiting) = mpsc::channel();
    thread::spawn(move || {
        let _ = found.send(futures::executor::block_on(receiver.next()));
    });
    let changed = waiting.recv_timeout(Duration::from_secs(5)).ok().flatten();
    assert_eq!(changed, Some(vec![repository.clone()]));

    // Watching the same folder twice adds nothing; unwatching stops it.
    watching.watch(&repository);
    assert_eq!(watching.roots.lock().unwrap().len(), 1);
    watching.unwatch(&repository);
    assert!(watching.roots.lock().unwrap().is_empty());
}

fn roots_of(directory: &Path) -> Mutex<Vec<Root>> {
    Mutex::new(vec![Root::new(directory)])
}

#[test]
fn routes_relevant_events_to_their_repository_and_skips_failed_ones() {
    let directory = tempfile::tempdir().unwrap();
    let roots = roots_of(directory.path());
    let resolved = directory.path().canonicalize().unwrap();
    let (sender, receiver) = mpsc::channel();

    route(&roots, &sender, Err(notify::Error::generic("lost")));
    let event = Event::default()
        .add_path(resolved.join("a.txt"))
        .add_path(resolved.join(".git/objects/x"))
        .add_path(PathBuf::from("/elsewhere/b.txt"));
    route(&roots, &sender, Ok(event));
    drop(sender);

    let sent: Vec<PathBuf> = receiver.iter().collect();
    assert_eq!(
        sent,
        [directory.path().to_path_buf()],
        "only the file in the work folder counts"
    );
}

#[test]
fn the_tick_reports_every_repository_until_nobody_listens() {
    let directory = tempfile::tempdir().unwrap();
    let roots = roots_of(directory.path());
    let (sender, receiver) = mpsc::channel();
    let first = thread::scope(|scope| {
        let ticking = scope.spawn(|| tick(&roots, &sender, Duration::from_millis(1)));
        let first = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        drop(receiver);
        ticking.join().unwrap(); // returns once the report cannot be sent
        first
    });
    assert_eq!(first, directory.path());
}

#[test]
fn debounce_groups_a_burst_and_stops_when_either_side_is_gone() {
    let (sender, receiver) = mpsc::channel();
    let (changes, mut reports) = futures::channel::mpsc::unbounded();
    sender.send(PathBuf::from("/a")).unwrap();
    sender.send(PathBuf::from("/a")).unwrap();
    sender.send(PathBuf::from("/b")).unwrap();
    let debouncing = thread::spawn(move || debounce(&receiver, &changes));
    thread::sleep(DEBOUNCE * 2);
    drop(sender); // nothing more comes: the loop ends
    debouncing.join().unwrap();
    let mut burst = futures::executor::block_on(reports.next()).unwrap();
    burst.sort();
    assert_eq!(burst, [PathBuf::from("/a"), PathBuf::from("/b")]);

    // A burst cut short by the sender going away is not reported.
    let (sender, receiver) = mpsc::channel();
    let (changes, mut reports) = futures::channel::mpsc::unbounded();
    sender.send(PathBuf::from("/c")).unwrap();
    drop(sender);
    debounce(&receiver, &changes);
    drop(changes);
    assert_eq!(futures::executor::block_on(reports.next()), None);

    // Nobody listens for reports any more: the loop ends after the first.
    let (sender, receiver) = mpsc::channel();
    let (changes, reports) = futures::channel::mpsc::unbounded::<Vec<PathBuf>>();
    drop(reports);
    sender.send(PathBuf::from("/d")).unwrap();
    let debouncing = thread::spawn(move || debounce(&receiver, &changes));
    debouncing.join().unwrap();
    drop(sender);
}
