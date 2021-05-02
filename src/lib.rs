use std::process::Command;

/// Runs the given command as a child process.
/// The string is split by whitespaces into words.
/// The first word is treated as the executable,
/// all following words are passed into the executable
/// as arguments.
///
/// Collects the `stdout` of the child process into a
/// [`String`] and returns it.
///
/// ```
/// use stir::cmd;
///
/// let stdout = cmd("echo -n foo");
/// assert_eq!(stdout, "foo");
/// ```
pub fn cmd(input: &str) -> String {
    let mut words = input.split_whitespace();
    let command = words.next().unwrap(); // todo
    let output = Command::new(dbg!(command)).args(words).output().unwrap();
    String::from_utf8(dbg!(output.stdout)).unwrap()
    // fixme
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::{
        env::{current_dir, set_current_dir},
        path::PathBuf,
    };
    use tempfile::TempDir;

    fn in_temporary_directory<F>(f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        let temp_dir = TempDir::new()?;
        let original_working_directory = current_dir()?;
        set_current_dir(&temp_dir)?;
        f()?;
        set_current_dir(original_working_directory)?;
        Ok(())
    }

    #[test]
    fn allows_to_execute_a_command() -> Result<()> {
        in_temporary_directory(|| {
            cmd("touch foo");
            assert!(PathBuf::from("foo").exists());
            Ok(())
        })
    }

    #[test]
    fn allows_to_retrieve_stdout() {
        assert_eq!(cmd("echo foo"), "foo\n");
    }
}
