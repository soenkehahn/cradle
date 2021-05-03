//! `stir` provides the [`cmd!`] macro, that makes
//! it easy to run commands from rust programs.
//!
//! ```
//! use stir::cmd;
//!
//! let stdout: String = cmd!("echo -n foo");
//! assert_eq!(stdout, "foo");
//! ```
//!
//! # Arguments
//!
//! You can pass in multiple arguments (of different types) to [`cmd!`]
//! to specify arguments:
//!
//! ```
//! use stir::cmd;
//!
//! let stdout: String = cmd!("echo", "foo", "bar");
//! assert_eq!(stdout, "foo bar\n");
//! ```
//!
//! Arguments of type [`&str`] will be split by whitespace into words.
//! You can also pass in values of type [`Vec<&str>`]. All elements will
//! be used as arguments:
//!
//! ```
//! use stir::cmd;
//!
//! let stdout: String = cmd!("echo", vec!["foo", "bar"]);
//! assert_eq!(stdout, "foo bar\n");
//! ```
//!
//! Elements of vectors are **not** being split by whitespace, so you can
//! use that to avoid whitespace splitting:
//!
//! ```
//! use std::path::PathBuf;
//! use stir::cmd;
//!
//! let () = cmd!("touch", vec!["filename with spaces"]);
//! assert!(PathBuf::from("filename with spaces").exists());
//! ```
//!
//! # Output
//!
//! [`cmd!`] collects the `stdout` of the child process into a
//! [`String`] and returns it.
//!
//! # Error Handling
//!
//! [`cmd!`] panics when the child process exits with a non-zero exitcode:
//!
//! ``` should_panic
//! use stir::cmd;
//!
//! // panics with "ls: exited with exit code: 1"
//! let () = cmd!("ls does-not-exist");
//! ```
use std::{
    io,
    process::{self, Command},
};

/// Execute child processes. Please, see the module documentation on how to use it.
#[macro_export]
macro_rules! cmd {
    ($($args:expr),+) => {{
        let mut args = vec![];
        $($crate::CmdArgument::add_as_argument($args, &mut args);)+
        $crate::run_cmd(args)
    }}
}

/// All types that are possible arguments to [`cmd!`] have to implement this trait.
pub trait CmdArgument {
    #[doc(hidden)]
    fn add_as_argument(self, accumulator: &mut Vec<String>);
}

impl CmdArgument for &str {
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        for argument in self.split_whitespace() {
            accumulator.push(argument.to_string());
        }
    }
}

impl CmdArgument for Vec<&str> {
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        for argument in self {
            accumulator.push(argument.to_string());
        }
    }
}

/// All possible return types of [`cmd!`] have to implement this trait.
pub trait CmdOutput {
    #[doc(hidden)]
    fn from_cmd_output(output: Output) -> Self;
}

impl CmdOutput for () {
    fn from_cmd_output(_output: Output) -> Self {
        ()
    }
}

impl CmdOutput for String {
    fn from_cmd_output(output: Output) -> Self {
        match String::from_utf8(output.output.stdout) {
            Ok(stdout) => stdout,
            Err(_err) => panic!("cmd!: invalid utf-8 written to stdout"),
        }
    }
}

#[doc(hidden)]
pub struct Output {
    output: process::Output,
}

#[doc(hidden)]
pub fn run_cmd<Output: CmdOutput>(input: Vec<String>) -> Output {
    let mut words = input.iter();
    let command = words.next().expect("cmd!: no arguments given");
    let output = Command::new(&command).args(words).output();
    match output {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            panic!("cmd!: {}: command not found", command);
        }
        Err(err) => panic!("cmd!: {}", err),
        Ok(output) if !output.status.success() => {
            let full_command = input.join(" ");
            panic!("{}:\n  exited with {}", full_command, output.status);
        }
        Ok(output) => Output::from_cmd_output(crate::Output { output }),
    }
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
            let () = cmd!("touch foo");
            assert!(PathBuf::from("foo").exists());
            Ok(())
        })
    }

    mod panics_by_default {
        use executable_path::executable_path;

        #[test]
        #[should_panic(expected = "false:\n  exited with exit code: 1")]
        fn non_zero_exit_codes() {
            let () = cmd!("false");
        }

        #[test]
        #[should_panic(expected = "false foo bar:\n  exited with exit code: 1")]
        fn includes_full_command_on_non_zero_exit_codes() {
            let () = cmd!("false foo bar");
        }

        #[test]
        #[should_panic(expected = "exited with exit code: 42")]
        fn other_exit_codes() {
            let () = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["exit code 42"]
            );
        }

        #[test]
        #[should_panic(expected = "cmd!: does-not-exist: command not found")]
        fn executable_cannot_be_found() {
            let () = cmd!("does-not-exist");
        }

        #[test]
        #[should_panic(expected = "cmd!: no arguments given")]
        fn no_executable() {
            let () = cmd!(vec![]);
        }

        #[test]
        #[should_panic(expected = "cmd!: invalid utf-8 written to stdout")]
        fn invalid_utf8_stdout() {
            let _: String = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["invalid utf-8 stdout"]
            );
        }
    }

    #[test]
    fn allows_to_retrieve_stdout() {
        let stdout: String = cmd!("echo foo");
        assert_eq!(stdout, "foo\n");
    }

    #[test]
    fn command_and_argument_as_separate_ref_str() {
        let stdout: String = cmd!("echo", "foo");
        assert_eq!(stdout, "foo\n");
    }

    #[test]
    fn multiple_arguments_as_ref_str() {
        let stdout: String = cmd!("echo", "foo", "bar");
        assert_eq!(stdout, "foo bar\n");
    }

    #[test]
    fn allows_to_pass_in_arguments_as_a_vec_of_ref_str() {
        let args: Vec<&str> = vec!["foo"];
        let stdout: String = cmd!("echo", args);
        assert_eq!(stdout, "foo\n");
    }
}
