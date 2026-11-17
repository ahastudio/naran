use super::*;

fn sleeping_child() -> Child {
    Command::new("sleep").arg("30").spawn().unwrap()
}

fn quick(timeout: Duration) -> Options<'static> {
    Options {
        input: None,
        timeout,
        background: false,
    }
}

#[test]
fn a_program_that_is_not_installed_cannot_run() {
    let invocation = Invocation {
        program: "naran-no-such-program",
        repository: Path::new("."),
        arguments: &["status"],
    };
    let error = run_program(&invocation, &quick(READ_TIMEOUT));
    let message = error.unwrap_err().to_string();
    assert!(
        message.starts_with("could not run git status: "),
        "{message}"
    );
}

#[test]
fn a_command_that_takes_too_long_is_stopped() {
    let directory = std::env::temp_dir();
    let arguments = ["-c", "alias.nap=!sleep 5", "nap"];
    let error = run_raw(&directory, &arguments, &quick(Duration::from_millis(50)));
    let command = arguments.join(" ");
    assert_eq!(error, Err(Error::Timeout { command }));
}

#[test]
fn a_child_that_cannot_be_waited_for_is_an_error() {
    let mut child = sleeping_child();
    let mut running = Running {
        child: &mut child,
        deadline: Instant::now() + Duration::from_secs(60),
        command: "sleep",
    };

    let error = running.poll(Err(io::Error::other("gone")));

    assert!(matches!(error, Err(Error::Spawn { message, .. }) if message == "gone"));
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn a_pipe_that_is_not_there_gives_no_output() {
    assert_eq!(collect(None::<io::Empty>).join().unwrap(), Vec::<u8>::new());
}

#[test]
fn a_spawn_failure_names_the_command() {
    let error = spawn_error("git status", &io::Error::other("no such program"));
    assert!(matches!(
        error,
        Error::Spawn { command, message } if command == "git status" && message == "no such program"
    ));
}

#[test]
fn a_child_still_running_after_the_deadline_is_killed() {
    let mut child = sleeping_child();
    let past = Instant::now()
        .checked_sub(Duration::from_millis(1))
        .unwrap();
    let mut running = Running {
        child: &mut child,
        deadline: past,
        command: "sleep",
    };
    let error = running.expire().unwrap_err();
    assert!(matches!(error, Error::Timeout { command } if command == "sleep"));
}

#[test]
fn a_child_before_the_deadline_is_left_alone() {
    let mut child = sleeping_child();
    let later = Instant::now() + Duration::from_secs(60);
    let mut running = Running {
        child: &mut child,
        deadline: later,
        command: "sleep",
    };
    assert_eq!(running.expire().unwrap(), None);
    child.kill().unwrap();
    child.wait().unwrap();
}
