use std::process::Command;

/// Runs the given command as a child process.
/// The string is split by whitespaces into words.
/// The first word is treated as the executable,
/// all following words are passed into the executable
/// as arguments.
///
/// ```
/// use stir::cmd;
/// cmd("ls -ls");
/// ```
pub fn cmd(input: &str) {
    let mut words = input.split_whitespace();
    let command = words.next().unwrap();
    let mut child_process = Command::new(dbg!(command)).args(words).spawn().unwrap();
    child_process.wait().unwrap();
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

    fn test<F>(f: F) -> Result<()>
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
        test(|| {
            cmd("touch foo");
            assert!(PathBuf::from("foo").exists());
            Ok(())
        })
    }
}
