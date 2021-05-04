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
//! You can choose which return type you want [`cmd!`] to return,
//! as long as the chosen return type implements [`CmdOutput`].
//! For example you can use [`()`] if you don't want any result:
//!
//! ```
//! use stir::cmd;
//!
//! let () = cmd!("touch foo");
//! ```
//!
//! Or you can use e.g. [`String`] to collect what the child process
//! writes to `stdout`:
//!
//! ```
//! use stir::cmd;
//!
//! let output: String = cmd!("echo foo");
//! assert_eq!(output, "foo\n");
//! ```
//!
//! # Error Handling
//!
//! By default [`cmd!`] panics for a few reasons, e.g.:
//!
//! - when the child process exits with a non-zero exitcode,
//! - when the given executable cannot be found,
//! - when no strings are given as arguments to [`cmd!`].
//!
//! For example:
//!
//! ``` should_panic
//! use stir::cmd;
//!
//! // panics with "ls: exited with exit code: 1"
//! let () = cmd!("ls does-not-exist");
//! ```
//!
//! You can turn these panics into [`std::result::Result::Err`]s
//! by fixing the return type of [`cmd!`] to `Result<T>`, where
//! `T` is any type that implements [`CmdOutput`] and
//! [`Result`] is stir's custom result type, which uses [`Error`].
//! Here's some examples:
//!
//! ```
//! use stir::{cmd, Result};
//!
//! let result: Result<()> = cmd!("ls does-not-exist");
//! let error_message = format!("{}", result.unwrap_err());
//! assert_eq!(
//!     error_message,
//!     "ls does-not-exist:\n  exited with exit code: 2"
//! );
//!
//! let result: Result<String> = cmd!("echo foo");
//! assert_eq!(result, Ok("foo\n".to_string()));
//! ```
use std::process::{Command, Output};

mod error;

pub use error::{Error, Result};

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
pub trait CmdOutput: Sized {
    #[doc(hidden)]
    fn from_cmd_output(output: Result<Output>) -> Result<Self>;
}

impl CmdOutput for () {
    fn from_cmd_output(output: Result<Output>) -> Result<Self> {
        output?;
        Ok(())
    }
}

impl CmdOutput for String {
    fn from_cmd_output(output: Result<Output>) -> Result<Self> {
        let output = output?;
        String::from_utf8(output.stdout).map_err(|_| Error::InvalidUtf8ToStdout)
    }
}

impl<T> CmdOutput for Result<T>
where
    T: CmdOutput,
{
    fn from_cmd_output(output: Result<Output>) -> Result<Self> {
        Ok(match output {
            Ok(_) => T::from_cmd_output(output),
            Err(error) => Err(error),
        })
    }
}

#[doc(hidden)]
pub fn run_cmd<T: CmdOutput>(input: Vec<String>) -> T {
    let mut words = input.iter();
    let result = T::from_cmd_output({
        match words.next() {
            None => Err(Error::NoArgumentsGiven),
            Some(command) => {
                let output = Command::new(&command).args(words).output();
                match output {
                    Err(err) => Err(Error::CommandIoError {
                        message: format!("cmd!: {}: {}", command, err),
                    }),
                    Ok(output) => {
                        if output.status.success() {
                            Ok(output)
                        } else {
                            let full_command = input.join(" ");
                            Err(Error::NonZeroExitCode {
                                full_command,
                                exit_status: output.status,
                            })
                        }
                    }
                }
            }
        }
    });
    match result {
        Ok(result) => result,
        Err(error) => panic!("{}", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use executable_path::executable_path;
    use std::{
        env::{current_dir, set_current_dir},
        path::PathBuf,
        result,
    };
    use tempfile::TempDir;

    type R<T> = result::Result<T, Box<dyn std::error::Error>>;

    fn in_temporary_directory<F>(f: F) -> R<()>
    where
        F: FnOnce() -> R<()>,
    {
        let temp_dir = TempDir::new()?;
        let original_working_directory = current_dir()?;
        set_current_dir(&temp_dir)?;
        f()?;
        set_current_dir(original_working_directory)?;
        Ok(())
    }

    #[test]
    fn allows_to_execute_a_command() -> R<()> {
        in_temporary_directory(|| {
            let () = cmd!("touch foo");
            assert!(PathBuf::from("foo").exists());
            Ok(())
        })
    }

    mod errors {
        use super::*;

        mod panics_by_default {
            use super::*;

            #[test]
            #[should_panic(expected = "false:\n  exited with exit code: 1")]
            fn non_zero_exit_codes() {
                let () = cmd!("false");
            }

            #[test]
            #[should_panic(expected = "false:\n  exited with exit code: 1")]
            fn combine_panics_with_other_outputs() {
                let _: String = cmd!("false");
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
            #[should_panic(
                expected = "cmd!: does-not-exist: No such file or directory (os error 2)"
            )]
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

        mod result_types {
            use super::*;

            #[test]
            fn non_zero_exit_codes() {
                let result: Result<()> = cmd!("false");
                assert_eq!(
                    format!("{}", result.unwrap_err()),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn no_errors() {
                let result: Result<()> = cmd!("true");
                assert_eq!(result, Ok(()));
            }

            #[test]
            fn combine_ok_with_other_outputs() {
                let result: Result<String> = cmd!("echo foo");
                assert_eq!(result, Ok("foo\n".to_string()));
            }

            #[test]
            fn combine_err_with_other_outputs() {
                let result: Result<String> = cmd!("false");
                assert_eq!(
                    format!("{}", result.unwrap_err()),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_non_zero_exit_codes() {
                let result: Result<()> = cmd!("false foo bar");
                assert_eq!(
                    format!("{}", result.unwrap_err()),
                    "false foo bar:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn other_exit_codes() {
                let result: Result<()> = cmd!(
                    executable_path("stir_test_helper").to_str().unwrap(),
                    vec!["exit code 42"]
                );
                assert!(format!("{}", result.unwrap_err()).contains("exited with exit code: 42"));
            }

            #[test]
            fn executable_cannot_be_found() {
                let result: Result<()> = cmd!("does-not-exist");
                assert_eq!(
                    format!("{}", result.unwrap_err()),
                    "cmd!: does-not-exist: No such file or directory (os error 2)"
                );
            }

            #[test]
            fn no_executable() {
                let result: Result<()> = cmd!(vec![]);
                assert_eq!(
                    format!("{}", result.unwrap_err()),
                    "cmd!: no arguments given"
                );
            }

            #[test]
            fn invalid_utf8_stdout() {
                let result: Result<String> = cmd!(
                    executable_path("stir_test_helper").to_str().unwrap(),
                    vec!["invalid utf-8 stdout"]
                );
                assert_eq!(
                    format!("{}", result.unwrap_err()),
                    "cmd!: invalid utf-8 written to stdout"
                );
            }
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
