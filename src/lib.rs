//! `stir` provides the [`cmd!`] macro, that makes
//! it easy to run commands from rust programs.
//!
//! ```
//! use stir::*;
//!
//! let stdout: String = cmd!("echo -n foo");
//! assert_eq!(stdout, "foo");
//! ```
//!
//! # Arguments
//!
//! You can pass in multiple arguments (of different types) to [`cmd!`]
//! to specify arguments, as long as they implement the [`CmdArgument`]
//! trait:
//!
//! ```
//! use stir::*;
//!
//! let stdout: String = cmd!("echo", "foo", "bar");
//! assert_eq!(stdout, "foo bar\n");
//! ```
//!
//! Arguments of type [`&str`] will be split by whitespace into words.
//! You can also pass in arrays of type [`[&str]`]. All elements will
//! be used as arguments:
//!
//! ```
//! use stir::*;
//!
//! # #[rustversion::since(1.51)]
//! # fn test() {
//! let stdout: String = cmd!("echo", ["foo", "bar"]);
//! assert_eq!(stdout, "foo bar\n");
//! # }
//! # #[rustversion::before(1.51)]
//! # fn test() {}
//! # test();
//! ```
//!
//! Elements of arrays are **not** being split by whitespace, so you can
//! use that to avoid whitespace splitting:
//!
//! ```
//! use std::path::PathBuf;
//! use stir::*;
//!
//! # #[rustversion::since(1.51)]
//! # fn test() {
//! let _: String = cmd!("touch", ["filename with spaces"]);
//! assert!(PathBuf::from("filename with spaces").exists());
//! # }
//! # #[rustversion::before(1.51)]
//! # fn test() {}
//! # test();
//! ```
//!
//! Before rust version `1.51`, instead of arrays, please use [`Vec<&str>`]:
//!
//! ```
//! use std::path::PathBuf;
//! use stir::*;
//!
//! let _: String = cmd!("touch", vec!["filename with spaces"]);
//! assert!(PathBuf::from("filename with spaces").exists());
//! ```
//!
//! For all possible inputs to [`cmd!`], see [`CmdArgument`].
//!
//! # Output
//!
//! You can choose which return type you want [`cmd!`] to return,
//! as long as the chosen return type implements [`CmdOutput`].
//! For example you can use e.g. [`String`] to collect what the
//! child process writes to `stdout`:
//!
//! ```
//! use stir::*;
//!
//! let output: String = cmd!("echo foo");
//! assert_eq!(output, "foo\n");
//! ```
//!
//! (By default, the child's `stdout` is written to the parent's `stdout`.
//! Using `String` as the return type suppresses that.)
//!
//! If you don't want any result from [`cmd!`], you can use `()`
//! as the return value:
//!
//! ```
//! use stir::*;
//!
//! let () = cmd!("touch foo");
//! ```
//!
//! Since that's a very common case, `stir` provides the [`cmd_unit!`]
//! shortcut, that behaves exactly like [`cmd!`], but pins the return
//! type down to `()`:
//!
//! ```
//! use stir::*;
//!
//! cmd_unit!("touch foo");
//! ```
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
//! use stir::*;
//!
//! // panics with "false:\n  exited with exit code: 1"
//! cmd_unit!("false");
//! ```
//!
//! You can suppress these panics by using the [`Exit`] type as
//! a return type of [`cmd!`]:
//!
//! ```
//! use stir::*;
//!
//! let Exit(exit_status) = cmd!("false");
//! assert_eq!(exit_status.code(), Some(1));
//! ```
//!
//! You can also turn all panics into [`std::result::Result::Err`]s
//! by fixing the return type of [`cmd!`] to `Result<T, stir::Error>`, where
//! `T` is any type that implements [`CmdOutput`].
//! Here's some examples:
//!
//! ```
//! use stir::*;
//!
//! let result: Result<(), stir::Error> = cmd!("false");
//! let error_message = format!("{}", result.unwrap_err());
//! assert_eq!(
//!     error_message,
//!     "false:\n  exited with exit code: 1"
//! );
//!
//! let result: Result<String, stir::Error> = cmd!("echo foo");
//! assert_eq!(result, Ok("foo\n".to_string()));
//! ```

mod cmd_argument;
mod cmd_output;
mod collected_output;
mod config;
mod context;
mod error;

use collected_output::CollectedOutput;

use crate::collected_output::Waiter;
pub use crate::{
    cmd_argument::{CmdArgument, LogCommand},
    cmd_output::{CmdOutput, Exit, Stderr},
    error::Error,
};
#[doc(hidden)]
pub use crate::{config::Config, context::Context};
use std::{
    io::Write,
    process::{Command, ExitStatus, Stdio},
};

/// Execute child processes. Please, see the module documentation on how to use it.
#[macro_export]
macro_rules! cmd {
    ($($args:expr),+) => {{
        let context = $crate::Context::production();
        $crate::cmd_with_context!(context, $($args),+)
    }}
}

/// Like [`cmd!`], but fixes the return type to `()`.
#[macro_export]
macro_rules! cmd_unit {
    ($($args:expr),+) => {{
        let () = $crate::cmd!($($args),+);
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! cmd_with_context {
    ($context:expr, $($args:expr),+) => {{
        let mut config = $crate::Config::default();
        $($crate::CmdArgument::prepare_config($args, &mut config);)+
        $crate::run_cmd($context, config)
    }}
}

// fixme: rename to collapse
fn conflate<T>(result: Result<T, T>) -> T {
    match result {
        Ok(result) => result,
        Err(result) => result,
    }
}

#[doc(hidden)]
pub fn run_cmd<Stdout, Stderr, T>(context: Context<Stdout, Stderr>, mut config: Config) -> T
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
    T: CmdOutput,
{
    <T as CmdOutput>::prepare_config(&mut config);
    let run_result = conflate(run_cmd_safe(context, &config));
    if config.should_panic {
        match &run_result {
            RunResult::EarlyError(error) | RunResult::LaterError { error, .. } => {
                panic!("cmd!: {}", error)
            }
            RunResult::Success { .. } => {}
        }
    }
    match T::from_run_result(&config, run_result) {
        Ok(result) => result,
        Err(error) => panic!("cmd!: {}", error),
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum RunResult {
    EarlyError(Error),
    LaterError {
        collected_output: CollectedOutput,
        error: Error,
    },
    Success {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_status: ExitStatus,
    },
}

fn run_cmd_safe<Stdout, Stderr>(
    mut context: Context<Stdout, Stderr>,
    config: &Config,
) -> Result<RunResult, RunResult>
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
{
    let (command, arguments) =
        parse_input(config.arguments.clone()).map_err(RunResult::EarlyError)?;
    if config.log_command {
        writeln!(context.stderr, "+ {}", config.full_command()).map_err(|io_error| {
            RunResult::EarlyError(Error::command_io_error(&config, io_error))
        })?;
    }
    let mut child = Command::new(&command)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| RunResult::EarlyError(Error::command_io_error(&config, error)))?;
    let waiter = Waiter::spawn_standard_stream_relaying(
        &context,
        config.clone(),
        child
            .stdout
            .take()
            .expect("child process should have stdout"),
        child
            .stderr
            .take()
            .expect("child process should have stderr"),
    );
    let collected_output = waiter
        .join()
        .map_err(|error| RunResult::EarlyError(Error::command_io_error(&config, error)))?;
    let exit_status = child.wait().map_err(|error| RunResult::LaterError {
        collected_output: collected_output.clone(), // fixme: shouldn't be necessary! :(
        error: Error::command_io_error(&config, error),
    })?;
    check_exit_status(&config, exit_status).map_err(|error| RunResult::LaterError {
        collected_output: collected_output.clone(), // fixme: don't clone
        error,
    })?;
    Ok(RunResult::Success {
        // fixme: collapse fields
        stdout: collected_output.stdout,
        stderr: collected_output.stderr,
        exit_status,
    })
}

fn parse_input(input: Vec<String>) -> Result<(String, impl Iterator<Item = String>), Error> {
    let mut words = input.into_iter();
    {
        match words.next() {
            None => Err(Error::NoArgumentsGiven),
            Some(command) => Ok((command, words)),
        }
    }
}

fn check_exit_status(config: &Config, exit_status: ExitStatus) -> Result<(), Error> {
    if config.error_on_non_zero_exit_code && !exit_status.success() {
        Err(Error::NonZeroExitCode {
            full_command: config.full_command(),
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
    };
    use tempfile::TempDir;

    fn in_temporary_directory<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        let temp_dir = TempDir::new().unwrap();
        let original_working_directory = current_dir().unwrap();
        set_current_dir(&temp_dir).unwrap();
        let result = std::panic::catch_unwind(|| {
            f();
        });
        set_current_dir(original_working_directory).unwrap();
        result.unwrap();
    }

    macro_rules! cmd_with_context_unit {
        ($context:expr, $($args:expr),+) => {{
            let () = $crate::cmd_with_context!($context, $($args),+);
        }}
    }

    #[test]
    fn allows_to_execute_a_command() {
        in_temporary_directory(|| {
            cmd_unit!("touch foo");
            assert!(PathBuf::from("foo").exists());
        })
    }

    mod errors {
        use super::*;

        mod panics_by_default {
            use super::*;

            #[test]
            #[should_panic(expected = "cmd!: false:\n  exited with exit code: 1")]
            fn non_zero_exit_codes() {
                cmd_unit!("false");
            }

            #[test]
            #[should_panic(expected = "cmd!: false:\n  exited with exit code: 1")]
            fn combine_panics_with_other_outputs() {
                let _: String = cmd!("false");
            }

            #[test]
            #[should_panic(expected = "cmd!: false foo bar:\n  exited with exit code: 1")]
            fn includes_full_command_on_non_zero_exit_codes() {
                cmd_unit!("false foo bar");
            }

            #[test]
            #[should_panic(expected = "exited with exit code: 42")]
            fn other_exit_codes() {
                cmd_unit!(
                    executable_path("stir_test_helper").to_str().unwrap(),
                    vec!["exit code 42"]
                );
            }

            #[test]
            #[cfg_attr(
                target_family = "unix",
                should_panic(
                    expected = "cmd!: does-not-exist:\n  No such file or directory (os error 2)"
                )
            )]
            #[cfg_attr(
                target_family = "windows",
                should_panic(
                    expected = "cmd!: does-not-exist:\n  The system cannot find the file specified. (os error 2)"
                )
            )]
            fn executable_cannot_be_found() {
                cmd_unit!("does-not-exist");
            }

            #[test]
            #[cfg_attr(
                target_family = "unix",
                should_panic(
                    expected = "cmd!: does-not-exist foo bar:\n  No such file or directory (os error 2)"
                )
            )]
            #[cfg_attr(
                target_family = "windows",
                should_panic(
                    expected = "cmd!: does-not-exist foo bar:\n  The system cannot find the file specified. (os error 2)"
                )
            )]
            fn includes_full_command_on_missing_executables() {
                cmd_unit!("does-not-exist foo bar");
            }

            #[test]
            #[should_panic(expected = "cmd!: no arguments given")]
            fn no_executable() {
                cmd_unit!("");
            }

            #[test]
            #[should_panic(expected = "invalid utf-8 written to stdout")]
            fn invalid_utf8_stdout() {
                let _: String = cmd!(
                    executable_path("stir_test_helper").to_str().unwrap(),
                    vec!["invalid utf-8 stdout"]
                );
            }

            #[test]
            fn invalid_utf8_to_stdout_is_allowed_when_not_captured() {
                cmd_unit!(
                    executable_path("stir_test_helper").to_str().unwrap(),
                    vec!["invalid utf-8 stdout"]
                );
            }
        }

        mod result_types {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn non_zero_exit_codes() {
                let result: Result<(), Error> = cmd!("false");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn no_errors() {
                let result: Result<(), Error> = cmd!("true");
                assert_eq!(result, Ok(()));
            }

            #[test]
            fn combine_ok_with_other_outputs() {
                let result: Result<String, Error> = cmd!("echo -n foo");
                assert_eq!(result, Ok("foo".to_string()));
            }

            #[test]
            fn combine_err_with_other_outputs() {
                let result: Result<String, Error> = cmd!("false");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_non_zero_exit_codes() {
                let result: Result<(), Error> = cmd!("false foo bar");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false foo bar:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_missing_executables() {
                let result: Result<(), Error> = cmd!("does-not-exist foo bar");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    if cfg!(target_os = "windows") {
                        "does-not-exist foo bar:\n  The system cannot find the file specified. (os error 2)"
                    } else {
                        "does-not-exist foo bar:\n  No such file or directory (os error 2)"
                    }
                );
            }

            #[test]
            fn other_exit_codes() {
                let result: Result<(), Error> = cmd!(
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
                let result: Result<(), Error> = cmd!("does-not-exist");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    if cfg!(target_os = "windows") {
                        "does-not-exist:\n  The system cannot find the file specified. (os error 2)"
                    } else {
                        "does-not-exist:\n  No such file or directory (os error 2)"
                    }
                );
            }

            #[test]
            fn no_executable() {
                let result: Result<(), Error> = cmd!("");
                assert_eq!(result.unwrap_err().to_string(), "no arguments given");
            }

            #[test]
            fn invalid_utf8_stdout() {
                let test_helper = executable_path("stir_test_helper");
                let test_helper = test_helper.to_str().unwrap();
                let result: Result<String, Error> = cmd!(test_helper, vec!["invalid utf-8 stdout"]);
                assert_eq!(
                    result.unwrap_err().to_string(),
                    format!(
                        "{} 'invalid utf-8 stdout':\n  invalid utf-8 written to stdout",
                        test_helper
                    )
                );
            }

            #[test]
            fn result_and_exit_does_not_panic_on_missing_executable() {
                let (result, Exit(status)): (Result<(), Error>, Exit) = cmd!("does-not-exist");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "does-not-exist:\n  No such file or directory (os error 2)".to_string()
                );
                assert!(!status.success());
                assert_eq!(status.code(), Some(127));
            }

            #[test]
            fn result_and_exit_does_not_panic_for_missing_arguments() {
                let (result, Exit(status)): (Result<(), Error>, Exit) = cmd!("");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "no arguments given".to_string()
                );
                assert!(!status.success());
                assert_eq!(status.code(), Some(1));
            }

            #[test]
            fn capturing_stdout_on_errors() {
                let (result, output): (Result<(), Error>, String) =
                    cmd!(test_helper(), vec!["output foo and exit with 42"]);
                assert_eq!(
                    result.unwrap_err().to_string(),
                    format!(
                        "{} 'output foo and exit with 42':\n  exited with exit code: 42",
                        test_helper()
                    )
                );
                assert_eq!(output, "output to stdout\n");
            }

            // fixme: use everywhere?
            fn test_helper() -> String {
                executable_path("stir_test_helper")
                    .to_str()
                    .unwrap()
                    .to_string()
            }

            #[test]
            fn capturing_stderr_on_errors() {
                let (result, Stderr(output)): (Result<(), Error>, Stderr) =
                    cmd!(test_helper(), vec!["write to stderr and exit with 42"]);
                assert_eq!(
                    result.unwrap_err().to_string(),
                    format!(
                        "{} 'write to stderr and exit with 42':\n  exited with exit code: 42",
                        test_helper()
                    )
                );
                assert_eq!(output, "output to stderr\n");
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

    #[rustversion::since(1.51)]
    #[test]
    fn arrays_as_arguments() {
        let args: [&str; 2] = ["echo", "foo"];
        let stdout: String = cmd!(args);
        assert_eq!(stdout, "foo\n");
    }

    #[rustversion::since(1.51)]
    #[test]
    fn elements_in_arrays_are_not_split_by_whitespace() {
        in_temporary_directory(|| {
            let args: [&str; 1] = ["foo bar"];
            cmd_unit!("touch", args);
            assert!(PathBuf::from("foo bar").exists());
        });
    }

    #[rustversion::since(1.51)]
    #[test]
    fn array_refs_as_arguments() {
        let args: &[&str; 2] = &["echo", "foo"];
        let stdout: String = cmd!(args);
        assert_eq!(stdout, "foo\n");
    }

    #[rustversion::since(1.51)]
    #[test]
    fn elements_in_array_refs_are_not_split_by_whitespace() {
        in_temporary_directory(|| {
            let args: &[&str; 1] = &["foo bar"];
            cmd_unit!("touch", args);
            assert!(PathBuf::from("foo bar").exists());
        });
    }

    #[test]
    fn slices_as_arguments() {
        let args: &[&str] = &["echo", "foo"];
        let stdout: String = cmd!(args);
        assert_eq!(stdout, "foo\n");
    }

    #[test]
    fn elements_in_slices_are_not_split_by_whitespace() {
        in_temporary_directory(|| {
            let args: &[&str] = &["foo bar"];
            cmd_unit!("touch", args);
            assert!(PathBuf::from("foo bar").exists());
        });
    }

    mod strings {
        use super::*;

        #[test]
        fn works_for_string() {
            let command: String = "true".to_string();
            cmd_unit!(command);
        }

        #[test]
        fn splits_strings_into_words() {
            let command: String = "echo foo".to_string();
            let output: String = cmd!(command);
            assert_eq!(output, "foo\n");
        }

        #[test]
        fn multiple_strings() {
            let command: String = "echo".to_string();
            let argument: String = "foo".to_string();
            let output: String = cmd!(command, argument);
            assert_eq!(output, "foo\n");
        }

        #[test]
        fn mix_ref_str_and_string() {
            let argument: String = "foo".to_string();
            let output: String = cmd!("echo", argument);
            assert_eq!(output, "foo\n");
        }

        #[test]
        fn does_not_split_strings_in_vectors() {
            in_temporary_directory(|| {
                let argument: Vec<String> = vec!["filename with spaces".to_string()];
                cmd_unit!("touch", argument);
                assert!(PathBuf::from("filename with spaces").exists());
            });
        }
    }

    mod stdout {
        use super::*;
        use std::{thread, time::Duration};

        #[test]
        fn relays_stdout_by_default() {
            let context = Context::test();
            cmd_with_context_unit!(context.clone(), "echo foo");
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn relays_stdout_for_non_zero_exit_codes() {
            let context = Context::test();
            let _: Result<(), Error> = cmd_with_context!(
                context.clone(),
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["output foo and exit with 42"]
            );
            assert_eq!(context.stdout(), "output to stdout\n");
        }

        #[test]
        fn streams_stdout() {
            in_temporary_directory(|| {
                let context = Context::test();
                let context_clone = context.clone();
                let thread = thread::spawn(|| {
                    cmd_with_context_unit!(
                        context_clone,
                        executable_path("stir_test_helper").to_str().unwrap(),
                        vec!["stream chunk then wait for file"]
                    );
                });
                while (context.stdout()) != "foo\n" {
                    thread::sleep(Duration::from_secs_f32(0.05));
                }
                cmd_unit!("touch file");
                thread.join().unwrap();
            });
        }

        #[test]
        fn does_not_relay_stdout_when_collecting_into_string() {
            let context = Context::test();
            let _: String = cmd_with_context!(context.clone(), "echo foo");
            assert_eq!(context.stdout(), "");
        }

        #[test]
        fn does_not_relay_stdout_when_collecting_into_result_of_string() {
            let context = Context::test();
            let _: Result<String, Error> = cmd_with_context!(context.clone(), "echo foo");
            assert_eq!(context.stdout(), "");
        }
    }

    mod stderr {
        use super::*;
        use pretty_assertions::assert_eq;
        use std::{thread, time::Duration};

        #[test]
        fn relays_stderr_by_default() {
            let context = Context::test();
            cmd_with_context_unit!(
                context.clone(),
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr"]
            );
            assert_eq!(context.stderr(), "output to stderr\n");
        }

        #[test]
        fn relays_stderr_for_non_zero_exit_codes() {
            let context = Context::test();
            let _: Result<(), Error> = cmd_with_context!(
                context.clone(),
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr and exit with 42"]
            );
            assert_eq!(context.stderr(), "output to stderr\n");
        }

        #[test]
        fn streams_stderr() {
            in_temporary_directory(|| {
                let context = Context::test();
                let context_clone = context.clone();
                let thread = thread::spawn(|| {
                    cmd_with_context_unit!(
                        context_clone,
                        executable_path("stir_test_helper").to_str().unwrap(),
                        vec!["stream chunk to stderr then wait for file"]
                    );
                });
                loop {
                    let expected = "foo\n";
                    let stderr = context.stderr();
                    if stderr == expected {
                        break;
                    }
                    assert!(
                        stderr.len() <= expected.len(),
                        "expected: {}, got: {}",
                        expected,
                        stderr
                    );
                    thread::sleep(Duration::from_secs_f32(0.05));
                }
                cmd_unit!("touch file");
                thread.join().unwrap();
            });
        }

        #[test]
        fn capture_stderr() {
            let Stderr(stderr) = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr"]
            );
            assert_eq!(stderr, "output to stderr\n");
        }

        #[test]
        fn assumes_stderr_is_utf_8() {
            let test_helper = executable_path("stir_test_helper");
            let test_helper = test_helper.to_str().unwrap();
            let result: Result<Stderr, Error> = cmd!(test_helper, vec!["invalid utf-8 stderr"]);
            assert_eq!(
                result.unwrap_err().to_string(),
                format!(
                    "{} 'invalid utf-8 stderr':\n  invalid utf-8 written to stderr",
                    test_helper
                )
            );
        }

        #[test]
        fn does_allow_invalid_utf_8_to_stderr_when_not_capturing() {
            cmd_unit!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["invalid utf-8 stderr"]
            );
        }

        #[test]
        fn does_not_relay_stderr_when_catpuring() {
            let context = Context::test();
            let Stderr(_) = cmd_with_context!(
                context.clone(),
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr"]
            );
            assert_eq!(context.stderr(), "");
        }
    }

    mod log_commands {
        use super::*;

        #[test]
        fn logs_simple_commands() {
            let context = Context::test();
            cmd_with_context_unit!(context.clone(), LogCommand, "true");
            assert_eq!(context.stderr(), "+ true\n");
        }

        #[test]
        fn logs_commands_with_arguments() {
            let context = Context::test();
            cmd_with_context_unit!(context.clone(), LogCommand, "echo foo");
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[test]
        fn quotes_arguments_with_spaces() {
            let context = Context::test();
            cmd_with_context_unit!(context.clone(), LogCommand, "echo", vec!["foo bar"]);
            assert_eq!(context.stderr(), "+ echo 'foo bar'\n");
        }
    }

    mod exit_status {
        use super::*;

        #[test]
        fn zero() {
            let Exit(exit_status) = cmd!("true");
            assert!(exit_status.success());
        }

        #[test]
        fn one() {
            let Exit(exit_status) = cmd!("false");
            assert!(!exit_status.success());
        }

        #[test]
        fn forty_two() {
            let Exit(exit_status) = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["exit code 42"]
            );
            assert!(!exit_status.success());
            assert_eq!(exit_status.code(), Some(42));
        }

        #[test]
        fn result_of_exit() {
            let result: Result<Exit, Error> = cmd!("false");
            assert!(!result.unwrap().0.success());
        }
    }

    mod tuple_outputs {
        use super::*;

        #[test]
        fn two_tuple_1() {
            let (output, Exit(status)) = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["output foo and exit with 42"]
            );
            let _: String = output;
            assert_eq!(output, "output to stdout\n");
            assert_eq!(status.code(), Some(42));
        }

        #[test]
        fn two_tuple_2() {
            let (Exit(status), output) = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["output foo and exit with 42"]
            );
            let _: String = output;
            assert_eq!(output, "output to stdout\n");
            assert_eq!(status.code(), Some(42));
        }

        #[test]
        fn result_of_tuple() {
            let result: Result<(String, Exit), Error> = cmd!("echo foo");
            let (output, Exit(status)) = result.unwrap();
            assert_eq!(output, "foo\n");
            assert!(status.success());
        }

        #[test]
        fn result_of_tuple_when_erroring() {
            let result: Result<(String, Exit), Error> = cmd!("false");
            let (output, Exit(status)) = result.unwrap();
            assert_eq!(output, "");
            assert_eq!(status.code(), Some(1));
        }

        #[test]
        fn tuple_containing_result() {
            let (result, output): (Result<Exit, Error>, String) = cmd!("echo foo");
            assert!(result.unwrap().0.success());
            assert_eq!(output, "foo\n");
        }

        #[test]
        fn tuple_containing_result_when_erroring() {
            let (result, output): (Result<Exit, Error>, String) = cmd!("false");
            assert!(!result.unwrap().0.success());
            assert_eq!(output, "");
        }

        #[test]
        fn three_tuples() {
            let (result, output, Exit(status)): (Result<(), Error>, String, Exit) =
                cmd!("echo foo");
            assert!(result.is_ok());
            assert_eq!(output, "foo\n");
            assert_eq!(status.code(), Some(0));
        }
    }
}
