//! Real Git repositories in a temporary folder, and the settings that list
//! them.

use super::*;

pub(in crate::workspace) fn git(directory: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .env("GIT_AUTHOR_NAME", "Naran")
        .env("GIT_AUTHOR_EMAIL", "naran@example.com")
        .env("GIT_COMMITTER_NAME", "Naran")
        .env("GIT_COMMITTER_EMAIL", "naran@example.com")
        .status()
        .expect("git runs");
    assert!(status.success(), "git {arguments:?}");
}

/// A repository with one commit, plus one untracked file when `dirty`.
pub(super) fn repository(root: &Path, name: &str, dirty: bool) -> PathBuf {
    let path = root.join(name);
    git(root, &["init", "-q", "-b", "main", name]);
    commit(&path, "README.md", name, "first");
    dirty.then(|| std::fs::write(path.join("draft.txt"), "draft").unwrap());
    path
}

/// Writes `content` to `file` and commits it.
pub(super) fn commit(path: &Path, file: &str, content: &str, message: &str) {
    std::fs::write(path.join(file), content).unwrap();
    git(path, &["add", file]);
    git(path, &["commit", "-q", "-m", message]);
}

pub(in crate::workspace) struct Fixture {
    _root: tempfile::TempDir,
    pub(in crate::workspace) config: PathBuf,
    pub(in crate::workspace) alpha: PathBuf,
    pub(in crate::workspace) beta: PathBuf,
    pub(in crate::workspace) gamma: PathBuf,
}

/// Without groups every repository goes into one group named 「기본」,
/// because a repository outside every group is dropped.
pub(in crate::workspace) fn fixture(groups: &[(&str, &[usize])]) -> Fixture {
    let default: [(&str, &[usize]); 1] = [("기본", &[0, 1, 2])];
    let groups = if groups.is_empty() {
        &default[..]
    } else {
        groups
    };
    fixture_with(groups)
}

pub(super) fn fixture_with(groups: &[(&str, &[usize])]) -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let [alpha, beta, gamma] = [("alpha", false), ("beta", true), ("gamma", false)]
        .map(|(name, dirty)| repository(root.path(), name, dirty));
    let config = root.path().join("config");
    save_config(
        &config,
        &[alpha.clone(), beta.clone(), gamma.clone()],
        groups,
    );
    Fixture {
        _root: root,
        config,
        alpha,
        beta,
        gamma,
    }
}

/// The repository list and the groups, as the app saves them.
fn save_config(config: &Path, paths: &[PathBuf], groups: &[(&str, &[usize])]) {
    let group = |(name, members): &(&str, &[usize])| model::Group {
        name: name.to_string(),
        paths: members.iter().map(|index| paths[*index].clone()).collect(),
    };
    let settings = Settings {
        groups: groups.iter().map(group).collect(),
    };
    store::save(&config.join("repositories.json"), &paths).unwrap();
    store::save(&config.join("settings.json"), &settings).unwrap();
}

/// Output of a `git` command in `directory`.
pub(in crate::workspace) fn git_output(directory: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("git runs");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The smallest valid PNG: one pixel.
pub(in crate::workspace) const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0xf0,
    0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

/// Adds `count` commits on top of the current branch with `git fast-import`,
/// which is much faster than committing one by one.
pub(super) fn add_commits(path: &Path, count: usize) {
    add_commits_to(path, "main", count);
}

pub(super) fn add_commits_to(path: &Path, branch: &str, count: usize) {
    use std::io::Write;
    let stream: Vec<String> = (0..count)
        .map(|index| fast_import_commit(branch, index))
        .collect();
    let mut child = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["fast-import", "--quiet", "--force"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stream.concat().as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
}

/// One commit for `git fast-import`. The first one starts from `main`.
fn fast_import_commit(branch: &str, index: usize) -> String {
    let time = 1_700_000_000 + index;
    let from = if index == 0 {
        "from refs/heads/main^0\n"
    } else {
        ""
    };
    format!(
        "commit refs/heads/{branch}\ncommitter n <a@b.c> {time} +0000\ndata 6\nc{index:04}\n{from}\n"
    )
}

pub(super) fn four_untracked_files(fixture: &Fixture) {
    for name in ["a.txt", "b.txt", "c.txt", "d.txt"] {
        std::fs::write(fixture.alpha.join(name), format!("{name}\n")).unwrap();
    }
}

/// `count` lines, `line <number>`, numbered from `first`.
pub(super) fn many_lines_from(first: usize, count: usize) -> String {
    let lines: Vec<String> = (first..first + count)
        .map(|number| format!("line {number}\n"))
        .collect();
    lines.concat()
}
