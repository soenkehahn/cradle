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
//! (By default, the child's `stdout` is written to the parent's `stdout`.
//! Using `String` as the return type suppresses that.)
//!
//! See the implementations for [`CmdOutput`] for all the supported types.
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
//! let result: Result<()> = cmd!("false");
//! let error_message = format!("{}", result.unwrap_err());
//! assert_eq!(
//!     error_message,
//!     "false:\n  exited with exit code: 1"
//! );
//!
//! let result: Result<String> = cmd!("echo foo");
//! assert_eq!(result, Ok("foo\n".to_string()));
//! ```

mod cmd_output;
mod context;
mod error;

#[doc(hidden)]
pub use crate::context::Context;
pub use crate::{
    cmd_output::CmdOutput,
    error::{Error, Result},
};
use std::{
    io::Write,
    process::{Command, ExitStatus, Stdio},
};

/// Execute child processes. Please, see the module documentation on how to use it.
#[macro_export]
macro_rules! cmd {
    ($($args:expr),+) => {{
        let context = &mut $crate::Context::production();
        $crate::cmd_with_context!(context, $($args),+)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! cmd_with_context {
    ($context:expr, $($args:expr),+) => {{
        let mut args = vec![];
        $($crate::CmdArgument::add_as_argument($args, &mut args);)+
        $crate::run_cmd($context, args)
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

#[doc(hidden)]
pub fn run_cmd<Stdout, T>(context: &mut Context<Stdout>, input: Vec<String>) -> T
where
    Stdout: Write + Clone + Send + 'static,
    T: CmdOutput,
{
    T::prepare_context(context);
    match T::from_run_result(run_cmd_safe(context, input)) {
        Ok(result) => result,
        Err(error) => panic!("{}", error),
    }
}

#[doc(hidden)]
pub struct RunResult {
    stdout: Vec<u8>,
}

fn run_cmd_safe<Stdout>(context: &mut Context<Stdout>, input: Vec<String>) -> Result<RunResult>
where
    Stdout: Write + Clone + Send + 'static,
{
    let (command, arguments) = parse_input(input.clone())?;
    let mut child = Command::new(&command)
        .args(arguments)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| Error::command_io_error(&command, error))?;
    let collected_stdout = context
        .clone()
        .spawn_stdout_relaying(child.stdout.take().unwrap());
    let exit_status = child.wait().unwrap();
    let collected_stdout = collected_stdout.join().unwrap();
    check_exit_status(input, exit_status)?;
    Ok(RunResult {
        stdout: collected_stdout,
    })
}

fn parse_input(input: Vec<String>) -> Result<(String, impl Iterator<Item = String>)> {
    let mut words = input.into_iter();
    {
        match words.next() {
            None => Err(Error::NoArgumentsGiven),
            Some(command) => Ok((command, words)),
        }
    }
}

fn check_exit_status(input: Vec<String>, exit_status: ExitStatus) -> Result<()> {
    if !exit_status.success() {
        let full_command = input.join(" ");
        Err(Error::NonZeroExitCode {
            full_command,
            exit_status,
        })
    } else {
        Ok(())
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
            #[cfg_attr(
                target_family = "unix",
                should_panic(
                    expected = "cmd!: does-not-exist: No such file or directory (os error 2)"
                )
            )]
            #[cfg_attr(
                target_family = "windows",
                should_panic(
                    expected = "cmd!: does-not-exist: The system cannot find the file specified. (os error 2)"
                )
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
                    result.unwrap_err().to_string(),
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
                let result: Result<String> = cmd!("echo -n foo");
                assert_eq!(result, Ok("foo".to_string()));
            }

            #[test]
            fn combine_err_with_other_outputs() {
                let result: Result<String> = cmd!("false");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_non_zero_exit_codes() {
                let result: Result<()> = cmd!("false foo bar");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false foo bar:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn other_exit_codes() {
                let result: Result<()> = cmd!(
                    executable_path("stir_test_helper").to_str().unwrap(),
                    vec!["exit code 42"]
                );
                assert!(result
                    .unwrap_err()
                    .to_string()
                    .contains("exited with exit code: 42"));
            }

            #[test]
            fn executable_cannot_be_found() {
                let result: Result<()> = cmd!("does-not-exist");
                let expected = if cfg!(target_os = "windows") {
                    "cmd!: does-not-exist: The system cannot find the file specified. (os error 2)"
                } else {
                    "cmd!: does-not-exist: No such file or directory (os error 2)"
                };
                assert_eq!(result.unwrap_err().to_string(), expected);
            }

            #[test]
            fn no_executable() {
                let result: Result<()> = cmd!(vec![]);
                assert_eq!(result.unwrap_err().to_string(), "cmd!: no arguments given");
            }

            #[test]
            fn invalid_utf8_stdout() {
                let result: Result<String> = cmd!(
                    executable_path("stir_test_helper").to_str().unwrap(),
                    vec!["invalid utf-8 stdout"]
                );
                assert_eq!(
                    result.unwrap_err().to_string(),
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

    mod stdout {
        use super::*;
        use std::{thread, time::Duration};

        #[test]
        fn inherits_stdout_by_default() {
            let context = &mut Context::test();
            let () = cmd_with_context!(context, "echo foo");
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn streams_stdout_for_non_zero_exit_codes() {
            let context = &mut Context::test();
            let _: Result<()> = cmd_with_context!(
                context,
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["output foo and exit with 42"]
            );
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn streams_stdout() {
            in_temporary_directory(|| {
                let context = Context::test();
                let mut context_clone = context.clone();
                let thread = thread::spawn(move || {
                    let () = cmd_with_context!(
                        &mut context_clone,
                        executable_path("stir_test_helper").to_str().unwrap(),
                        vec!["stream chunk then wait for file"]
                    );
                });
                while (context.stdout()) != "foo\n" {
                    thread::sleep(Duration::from_secs_f32(0.05));
                }
                let () = cmd!("touch file");
                thread.join().unwrap();
                Ok(())
            })
            .unwrap()
        }

        #[test]
        fn suppress_output_when_collecting_stdout_into_string() {
            let context = Context::test();
            let _: String = cmd_with_context!(&mut context.clone(), "echo foo");
            assert_eq!(context.stdout(), "");
        }

        #[test]
        fn suppress_output_when_collecting_stdout_into_result_of_string() {
            let context = Context::test();
            let _: Result<String> = cmd_with_context!(&mut context.clone(), "echo foo");
            assert_eq!(context.stdout(), "");
        }
    }
}
