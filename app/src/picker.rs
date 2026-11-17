//! The system folder picker, run in another process.
//!
//! GPUI opens the macOS panel on the main thread and waits there for the
//! "Open and Save Panel Service" to answer. When that service is slow (here it
//! took many seconds) the whole window stops. `osascript` shows the same kind
//! of panel from its own process, so Naran only waits in the background and
//! stays alive.

use std::path::PathBuf;
use std::process::{Command, Output};

/// A line of every picker, so it can be found and stopped (Cancel).
const MARKER: &str = "Naran: add repositories";

/// The program that shows the picker and the line that marks it. Tests use
/// another program and marker, so they never open a panel on the screen or
/// stop a picker that is really open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picker {
    program: String,
    marker: String,
}

impl Default for Picker {
    fn default() -> Self {
        Self::new("osascript")
    }
}

/// How a pick ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickOutcome {
    Picked(Vec<PathBuf>),
    /// The user closed the picker, or Cancel was pressed in Naran.
    Canceled,
    /// The picker could not be shown or failed; the text says why.
    Failed(String),
}

impl Picker {
    /// A picker shown by `program`, marked with Naran's own line.
    pub fn new(program: &str) -> Self {
        Self {
            program: program.into(),
            marker: MARKER.into(),
        }
    }

    /// The same picker marked with another line, so stopping it leaves other
    /// pickers alone. Only tests need it.
    #[cfg(test)]
    #[must_use]
    pub fn marked_by(self, marker: &str) -> Self {
        Self {
            marker: marker.into(),
            ..self
        }
    }

    /// Shows the picker and waits for the answer. Call it away from the main thread.
    pub fn choose(&self) -> PickOutcome {
        let mut command = Command::new(&self.program);
        command.args(script_arguments(&self.marker));
        match command.output() {
            Ok(output) => classify(&output),
            Err(error) => PickOutcome::Failed(format!("could not start {}: {error}", self.program)),
        }
    }

    /// Stops a picker that is waiting. Its answer then arrives as `Canceled`. A
    /// plain TERM does not end `osascript` while its panel is open, so it is killed.
    pub fn cancel(&self) {
        let _ = Command::new("pkill")
            .args(["-KILL", "-f", &self.marker])
            .status();
    }
}

/// The `-e` options that make `osascript` show the picker, one script line each.
fn script_arguments(marker: &str) -> Vec<String> {
    let choose = format!(
        "set chosen to choose folder with prompt \"{marker}\" with multiple selections allowed"
    );
    let lines = [
        "tell me to activate",
        &choose,
        "set output to \"\"",
        "repeat with item_ in chosen",
        "set output to output & POSIX path of item_ & linefeed",
        "end repeat",
        "return output",
    ];
    lines
        .iter()
        .flat_map(|line| ["-e", line])
        .map(str::to_string)
        .collect()
}

/// Reads what `osascript` printed. The folders come one per line, each with a
/// slash at the end.
pub fn classify(output: &Output) -> PickOutcome {
    if output.status.success() {
        return picked(&String::from_utf8_lossy(&output.stdout));
    }
    failed(
        output.status.code(),
        &String::from_utf8_lossy(&output.stderr),
    )
}

/// A closed picker is not a failure, only a pick of nothing.
fn failed(code: Option<i32>, stderr: &str) -> PickOutcome {
    if was_closed(code, stderr) {
        return PickOutcome::Canceled;
    }
    PickOutcome::Failed(failure_message(code, stderr))
}

fn picked(stdout: &str) -> PickOutcome {
    let lines = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let paths: Vec<PathBuf> = lines.map(folder_path).collect();
    if paths.is_empty() {
        PickOutcome::Canceled
    } else {
        PickOutcome::Picked(paths)
    }
}

/// The slash at the end is dropped, except for the root folder.
fn folder_path(line: &str) -> PathBuf {
    PathBuf::from(if line.len() > 1 {
        line.trim_end_matches('/')
    } else {
        line
    })
}

/// `AppleScript` reports a closed picker as error -128; a stopped process has no exit code.
fn was_closed(code: Option<i32>, stderr: &str) -> bool {
    ["-128", "User canceled"]
        .iter()
        .any(|text| stderr.contains(text))
        || code.is_none()
}

fn failure_message(code: Option<i32>, stderr: &str) -> String {
    let message = stderr.trim();
    if message.is_empty() {
        format!("osascript exited with code {code:?}")
    } else {
        message.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    use super::*;

    /// What a program that exited with `code` printed. No code means a signal
    /// stopped it.
    fn ended(code: Option<i32>, stdout: &str, stderr: &str) -> PickOutcome {
        let status = ExitStatus::from_raw(code.map_or(9, |code| code << 8));
        classify(&Output {
            status,
            stdout: stdout.into(),
            stderr: stderr.into(),
        })
    }

    #[test]
    fn several_folders_come_back_without_the_trailing_slash() {
        let outcome = ended(Some(0), "/Users/me/work/app/\n/Users/me/docs/\n", "");
        assert_eq!(
            outcome,
            PickOutcome::Picked(vec![
                PathBuf::from("/Users/me/work/app"),
                PathBuf::from("/Users/me/docs")
            ])
        );
    }

    #[test]
    fn the_root_folder_keeps_its_slash() {
        assert_eq!(
            ended(Some(0), "/\n", ""),
            PickOutcome::Picked(vec![PathBuf::from("/")])
        );
    }

    #[test]
    fn a_closed_picker_is_a_cancel_not_an_error() {
        let closed = "execution error: User canceled. (-128)";
        assert_eq!(ended(Some(1), "", closed), PickOutcome::Canceled);
        assert_eq!(
            ended(None, "", ""),
            PickOutcome::Canceled,
            "a stopped process"
        );
        assert_eq!(
            ended(Some(0), "", ""),
            PickOutcome::Canceled,
            "nothing chosen"
        );
    }

    #[test]
    fn any_other_failure_says_why() {
        assert_eq!(
            ended(Some(1), "", "execution error: boom (-1)\n"),
            PickOutcome::Failed("execution error: boom (-1)".into())
        );
        assert!(matches!(ended(Some(2), "", ""), PickOutcome::Failed(text) if text.contains('2')));
    }

    #[test]
    fn the_system_picker_is_osascript() {
        assert_eq!(Picker::default(), Picker::new("osascript"));
    }

    #[test]
    fn runs_the_picker_program_and_reads_its_answer() {
        assert_eq!(
            Picker::new("true").marked_by("m").choose(),
            PickOutcome::Canceled,
            "nothing printed"
        );
        assert_eq!(
            Picker::new("false").marked_by("m").choose(),
            PickOutcome::Failed("osascript exited with code Some(1)".into())
        );
        let missing = Picker::new("naran-no-such-program").marked_by("m").choose();
        assert!(
            matches!(&missing, PickOutcome::Failed(text) if text.starts_with("could not start naran-no-such-program")),
            "{missing:?}"
        );
    }

    #[test]
    fn cancel_stops_only_the_process_with_the_marker() {
        let marker = format!("naran-picker-test-{}", std::process::id());
        let mut waiting = Command::new("sh")
            .args(["-c", "sleep 30; true", &marker])
            .spawn()
            .unwrap();
        Picker::new("osascript").marked_by(&marker).cancel();
        let status = waiting.wait().unwrap();
        assert_eq!(status.code(), None, "stopped by a signal: {status:?}");
    }
}
