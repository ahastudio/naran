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
    assert!(is_relevant(
        &root,
        &path.join(".git/worktrees/feature/index")
    ));
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
    let roots = Arc::new(roots_of(directory.path()));
    let (sender, receiver) = mpsc::channel();
    let (_stop, stop_signal) = mpsc::channel();
    let ticking = Ticking {
        roots: Arc::clone(&roots),
        sender,
        stop: stop_signal,
    };
    let first = thread::scope(|scope| {
        let running = scope.spawn(move || ticking.run_every(Duration::from_millis(1)));
        let first = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        drop(receiver);
        running.join().unwrap(); // returns once the report cannot be sent
        first
    });
    assert_eq!(first, directory.path());
}

#[test]
fn dropping_watching_ends_its_threads_instead_of_leaking_them() {
    let directory = tempfile::tempdir().unwrap();
    let (sender, mut receiver) = futures::channel::mpsc::unbounded::<Vec<PathBuf>>();
    let mut watching = Watching::start(sender).unwrap();
    watching.watch(directory.path());

    drop(watching);

    // The tick and debounce threads hold the sending side of the reports.
    // The channel closes only once both ended; a tick thread that sleeps for
    // `SAFETY_NET` would keep it open. Waits on another thread, so a leak
    // fails the test after five seconds instead of hanging it.
    let (closed, waiting) = mpsc::channel();
    thread::spawn(move || {
        while futures::executor::block_on(receiver.next()).is_some() {}
        let _ = closed.send(());
    });
    assert!(
        waiting.recv_timeout(Duration::from_secs(5)).is_ok(),
        "a thread of the dropped Watching is still running"
    );
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

#[test]
fn a_folder_is_added_while_the_roots_are_unlocked() {
    // notify waits for its event thread while it adds a path, and that
    // thread waits for the roots in `route` when a file event arrives, so
    // the roots must be free while the path is added.
    let directory = tempfile::tempdir().unwrap();
    let roots: Mutex<Vec<Root>> = Mutex::default();
    let mut unlocked = None;

    add_root(&roots, directory.path(), || {
        unlocked = Some(roots.try_lock().is_ok());
        true
    });

    assert_eq!(unlocked, Some(true), "the roots are locked while adding");
    assert_eq!(locked(&roots).len(), 1);

    // A path that is already there is not added again, and one that could
    // not be watched is not registered.
    add_root(&roots, directory.path(), || panic!("added twice"));
    let other = tempfile::tempdir().unwrap();
    add_root(&roots, other.path(), || false);
    assert_eq!(locked(&roots).len(), 1);
}

#[test]
fn reports_the_repository_for_a_change_in_its_worktree() {
    let directory = tempfile::tempdir().unwrap();
    let repository = directory.path().join("repository");
    let folder = directory.path().join("repository-feature");
    std::fs::create_dir_all(&repository).unwrap();
    std::fs::create_dir_all(&folder).unwrap();
    let (sender, mut receiver) = futures::channel::mpsc::unbounded();
    let mut watching = Watching::start(sender).unwrap();
    watching.watch(&repository);
    watching.watch_linked(&folder, &repository);
    watching.watch_linked(&folder, &directory.path().join("other"));
    thread::sleep(Duration::from_millis(300));
    assert_eq!(watching.roots.lock().unwrap().len(), 2);

    std::fs::write(folder.join("file.txt"), "x").unwrap();

    let (found, waiting) = mpsc::channel();
    thread::spawn(move || {
        let _ = found.send(futures::executor::block_on(receiver.next()));
    });
    let changed = waiting.recv_timeout(Duration::from_secs(5)).ok().flatten();
    assert_eq!(changed, Some(vec![repository]));
}

#[test]
fn a_worktree_folder_that_is_no_longer_listed_is_not_watched_any_more() {
    let directory = tempfile::tempdir().unwrap();
    let repository = directory.path().join("repository");
    let (kept, gone) = (directory.path().join("kept"), directory.path().join("gone"));
    for folder in [&repository, &kept, &gone] {
        std::fs::create_dir_all(folder).unwrap();
    }
    let (sender, _receiver) = futures::channel::mpsc::unbounded();
    let mut watching = Watching::start(sender).unwrap();
    watching.watch(&repository);
    watching.watch_linked(&kept, &repository);
    watching.watch_linked(&gone, &repository);
    assert_eq!(watching.roots.lock().unwrap().len(), 3);

    watching.keep_linked(std::slice::from_ref(&kept));

    let roots = watching.roots.lock().unwrap();
    let watched: Vec<_> = roots.iter().map(|root| root.watched.clone()).collect();
    assert_eq!(watched, [repository, kept]);
}

#[test]
fn a_folder_that_is_also_a_repository_stays_watched_when_it_is_no_longer_a_worktree() {
    let directory = tempfile::tempdir().unwrap();
    let (repository, other) = (
        directory.path().join("repository"),
        directory.path().join("other"),
    );
    for folder in [&repository, &other] {
        std::fs::create_dir_all(folder).unwrap();
    }
    let (sender, _receiver) = futures::channel::mpsc::unbounded();
    let mut watching = Watching::start(sender).unwrap();
    watching.watch(&repository);
    watching.watch_linked(&other, &repository);
    watching.watch(&other);

    watching.keep_linked(&[]);

    let roots = watching.roots.lock().unwrap();
    let watched: Vec<_> = roots.iter().map(|root| root.watched.clone()).collect();
    assert_eq!(watched, [repository, other]);
}
