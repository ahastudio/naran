//! Runs the system `git`. Always with an argument list, never through a shell.

use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::Error;

const READ_TIMEOUT: Duration = Duration::from_secs(30);
const WRITE_TIMEOUT: Duration = Duration::from_secs(60);
const NETWORK_TIMEOUT: Duration = Duration::from_secs(180);
const FIRST_POLL_WAIT: Duration = Duration::from_millis(1);
const LAST_POLL_WAIT: Duration = Duration::from_millis(50);

struct Options<'a> {
    input: Option<&'a str>,
    timeout: Duration,
    /// Background read. Takes no index lock, so it never collides with the user's own Git work.
    background: bool,
}

/// Command that reads state in the background.
pub fn read(repository: &Path, arguments: &[&str]) -> Result<String, Error> {
    run(
        repository,
        arguments,
        &Options {
            input: None,
            timeout: READ_TIMEOUT,
            background: true,
        },
    )
}

/// Operation requested by the user. Changes the repository.
pub fn write(repository: &Path, arguments: &[&str]) -> Result<String, Error> {
    run(
        repository,
        arguments,
        &Options {
            input: None,
            timeout: WRITE_TIMEOUT,
            background: false,
        },
    )
}

/// Operation that passes standard input (commit messages, patches).
pub fn write_with_input(
    repository: &Path,
    arguments: &[&str],
    input: &str,
) -> Result<String, Error> {
    run(
        repository,
        arguments,
        &Options {
            input: Some(input),
            timeout: WRITE_TIMEOUT,
            background: false,
        },
    )
}

/// Operation that talks to a remote. Has a longer time limit.
pub fn network(repository: &Path, arguments: &[&str]) -> Result<String, Error> {
    run(
        repository,
        arguments,
        &Options {
            input: None,
            timeout: NETWORK_TIMEOUT,
            background: false,
        },
    )
}

fn run(repository: &Path, arguments: &[&str], options: &Options) -> Result<String, Error> {
    run_raw(repository, arguments, options)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// Like `run`, but the output is returned as it is, for files that are not text.
pub fn read_bytes(repository: &Path, arguments: &[&str]) -> Result<Vec<u8>, Error> {
    run_raw(
        repository,
        arguments,
        &Options {
            input: None,
            timeout: READ_TIMEOUT,
            background: true,
        },
    )
}

/// What a finished process left behind.
struct Finished {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// A program run with its arguments in a repository.
struct Invocation<'a> {
    program: &'a str,
    repository: &'a Path,
    arguments: &'a [&'a str],
}

fn run_raw(repository: &Path, arguments: &[&str], options: &Options) -> Result<Vec<u8>, Error> {
    let invocation = Invocation {
        program: "git",
        repository,
        arguments,
    };
    run_program(&invocation, options)
}

fn run_program(invocation: &Invocation, options: &Options) -> Result<Vec<u8>, Error> {
    let command = invocation.arguments.join(" ");
    let mut child = spawn(invocation, options).map_err(|source| spawn_error(&command, &source))?;
    let finished = communicate(&mut child, options, &command)?;
    finish(finished, command)
}

fn spawn(invocation: &Invocation, options: &Options) -> io::Result<Child> {
    let mut process = Command::new(invocation.program);
    process
        .arg("-C")
        .arg(invocation.repository)
        .args(invocation.arguments)
        // Never stop to ask for input. Authentication is left to credential helpers and SSH settings.
        .env("GIT_TERMINAL_PROMPT", "0")
        // Use the default merge message instead of opening an editor.
        .env("GIT_EDITOR", "true")
        .stdin(input_pipe(options.input))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if options.background {
        process.env("GIT_OPTIONAL_LOCKS", "0");
    }
    process.spawn()
}

fn input_pipe(input: Option<&str>) -> Stdio {
    match input {
        Some(_) => Stdio::piped(),
        None => Stdio::null(),
    }
}

fn spawn_error(command: &str, source: &io::Error) -> Error {
    Error::Spawn {
        command: command.to_owned(),
        message: source.to_string(),
    }
}

fn communicate(child: &mut Child, options: &Options, command: &str) -> Result<Finished, Error> {
    feed_input(child, options.input);
    let (stdout, stderr) = (collect(child.stdout.take()), collect(child.stderr.take()));
    let deadline = Instant::now() + options.timeout;
    let status = Running {
        child,
        deadline,
        command,
    }
    .wait()?;
    Ok(Finished {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

fn feed_input(child: &mut Child, input: Option<&str>) {
    if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
        let bytes = input.as_bytes().to_vec();
        thread::spawn(move || {
            let _ = stdin.write_all(&bytes);
        });
    }
}

/// A child process that must finish before its deadline.
struct Running<'a> {
    child: &'a mut Child,
    deadline: Instant,
    command: &'a str,
}

impl Running<'_> {
    fn wait(&mut self) -> Result<ExitStatus, Error> {
        let mut wait = FIRST_POLL_WAIT;
        loop {
            let polled = self.child.try_wait();
            match self.poll(polled)? {
                Some(status) => return Ok(status),
                None => wait = pause(wait),
            }
        }
    }

    /// Turns what `try_wait` gave into the exit status, or stops the child
    /// once the deadline has passed.
    fn poll(
        &mut self,
        polled: io::Result<Option<ExitStatus>>,
    ) -> Result<Option<ExitStatus>, Error> {
        match polled.map_err(|source| spawn_error(self.command, &source))? {
            Some(status) => Ok(Some(status)),
            None => self.expire(),
        }
    }

    fn expire(&mut self) -> Result<Option<ExitStatus>, Error> {
        if Instant::now() < self.deadline {
            return Ok(None);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        Err(Error::Timeout {
            command: self.command.to_owned(),
        })
    }
}

/// Sleeps, then returns the next wait. The wait doubles up to a limit.
fn pause(wait: Duration) -> Duration {
    thread::sleep(wait);
    (wait * 2).min(LAST_POLL_WAIT)
}

fn finish(finished: Finished, command: String) -> Result<Vec<u8>, Error> {
    if finished.status.success() {
        return Ok(finished.stdout);
    }
    Err(Error::Command {
        command,
        code: finished.status.code(),
        stderr: failure_message(&finished.stdout, &finished.stderr),
    })
}

/// Messages such as merge conflicts are printed to standard output.
fn failure_message(stdout: &[u8], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_owned();
    if !stderr.is_empty() {
        return stderr;
    }
    String::from_utf8_lossy(stdout).trim().to_owned()
}

fn collect(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buffer = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut buffer);
        }
        buffer
    })
}

#[cfg(test)]
mod tests;
