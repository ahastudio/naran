//! Runs the same work on many repositories at once, with a limit on concurrency. Results keep the input order.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;

/// The paths still to work on, and the results found so far, in input order.
struct Queue<'a, T> {
    paths: &'a [PathBuf],
    next: AtomicUsize,
    results: Mutex<Vec<Option<T>>>,
}

impl<'a, T> Queue<'a, T> {
    fn new(paths: &'a [PathBuf]) -> Self {
        Self {
            paths,
            next: AtomicUsize::new(0),
            results: Mutex::new(paths.iter().map(|_| None).collect()),
        }
    }

    fn next_path(&self) -> Option<(usize, &'a PathBuf)> {
        let index = self.next.fetch_add(1, Ordering::Relaxed);
        self.paths.get(index).map(|path| (index, path))
    }

    fn store(&self, index: usize, value: T) {
        let mut results = self.results.lock().unwrap_or_else(PoisonError::into_inner);
        results[index] = Some(value);
    }

    fn work_through(&self, work: &impl Fn(&PathBuf) -> T) {
        while let Some((index, path)) = self.next_path() {
            self.store(index, work(path));
        }
    }

    fn into_results(self) -> Vec<(PathBuf, T)> {
        let results = self
            .results
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner);
        let found = results.into_iter().flatten();
        self.paths.iter().cloned().zip(found).collect()
    }
}

pub fn parallel<T, F>(paths: &[PathBuf], limit: usize, work: F) -> Vec<(PathBuf, T)>
where
    T: Send,
    F: Fn(&PathBuf) -> T + Sync,
{
    let queue = Queue::new(paths);
    thread::scope(|scope| {
        for _ in 0..limit.clamp(1, paths.len().max(1)) {
            scope.spawn(|| queue.work_through(&work));
        }
    });
    queue.into_results()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_input_order_and_runs_everything() {
        let paths: Vec<PathBuf> = (0..20)
            .map(|number| PathBuf::from(number.to_string()))
            .collect();

        let results = parallel(&paths, 4, |path| path.to_string_lossy().len());

        assert_eq!(results.len(), 20);
        assert!(
            results
                .iter()
                .zip(&paths)
                .all(|((path, _), expected)| path == expected)
        );
    }

    #[test]
    fn handles_no_paths() {
        assert_eq!(parallel(&[], 4, |_| 0), Vec::<(PathBuf, i32)>::new());
    }
}
