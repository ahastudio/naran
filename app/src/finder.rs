//! Shows a folder in Finder with `open -R`, in another process, so Naran
//! never waits for it. Tests use another program, so no Finder window opens.

use std::io;
use std::path::Path;
use std::process::Command;
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finder {
    program: String,
}

impl Default for Finder {
    fn default() -> Self {
        Self::new("open")
    }
}

impl Finder {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.into(),
        }
    }

    /// Starts the program and leaves it to finish on its own.
    pub fn reveal(&self, path: &Path) -> io::Result<()> {
        let mut child = Command::new(&self.program).arg("-R").arg(path).spawn()?;
        thread::spawn(move || child.wait());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finder_is_opened_with_open() {
        assert_eq!(Finder::default(), Finder::new("open"));
    }

    #[test]
    fn a_program_that_starts_reveals_and_a_missing_one_is_an_error() {
        assert!(Finder::new("true").reveal(Path::new("/")).is_ok());
        assert!(
            Finder::new("naran-no-such-program")
                .reveal(Path::new("/"))
                .is_err()
        );
    }
}
