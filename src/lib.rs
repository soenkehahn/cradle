//! `stir` provides the [`cmd!`] macro, that makes
//! it easy to run commands from rust programs.
//!
//! ```
//! use stir::*;
//!
//! let Stdout(stdout) = cmd!("echo -n foo");
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
//! let Stdout(stdout) = cmd!("echo", "foo", "bar");
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
//! let Stdout(stdout) = cmd!("echo", ["foo", "bar"]);
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
//! let Stdout(_) = cmd!("touch", ["filename with spaces"]);
//! assert!(PathBuf::from("filename with spaces").exists());
//! # }
//! # #[rustversion::before(1.51)]
//! # fn test() {}
//! # test();
//! ```
//!
//! Before rust version `1.51`, instead of arrays, use [`Vec<&str>`]:
//!
//! ```
//! use std::path::PathBuf;
//! use stir::*;
//!
//! let Stdout(_) = cmd!("touch", vec!["filename with spaces"]);
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
//! let Stdout(output) = cmd!("echo foo");
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
//! You can suppress panics caused by non-zero exit codes by using the
//! [`Exit`] type as a return type of [`cmd!`]:
//!
//! ```
//! use stir::*;
//!
//! let Exit(exit_status) = cmd!("false");
//! assert_eq!(exit_status.code(), Some(1));
//! ```
//!
//! You can also turn **all** panics into [`std::result::Result::Err`]s
//! by using [`cmd_result!`]. This will return a value of type
//! [`Result<T, stir::Error>`], where
//! `T` is any type that implements [`CmdOutput`].
//! Here's some examples:
//!
//! ```
//! use stir::*;
//!
//! let result: Result<(), stir::Error> = cmd_result!("false");
//! let error_message = format!("{}", result.unwrap_err());
//! assert_eq!(
//!     error_message,
//!     "false:\n  exited with exit code: 1"
//! );
//!
//! let result: Result<Stdout, stir::Error> = cmd_result!("echo foo");
//! // todo: use method
//! assert_eq!(result.unwrap().0, "foo\n".to_string());
//! ```
//!
//! [`cmd_result`] can also be combined with `?` to handle errors in an
//! idiomatic way, for example:
//!
//! ```
//! use stir::*;
//!
//! fn build() -> Result<(), Error> {
//!     cmd_result!("which make")?;
//!     cmd_result!("which gcc")?;
//!     cmd_result!("which ld")?;
//!     cmd_result!("make build")?;
//!     Ok(())
//! }
//! ```

mod cmd_argument;
mod cmd_output;
mod collected_output;
mod config;
mod context;
mod error;

use crate::collected_output::Waiter;
pub use crate::{
    cmd_argument::{CmdArgument, CurrentDir, LogCommand},
    cmd_output::{CmdOutput, Exit, Stderr, Stdout},
    error::{panic_on_error, Error},
};
#[doc(hidden)]
pub use crate::{config::Config, context::Context};
use std::{
    io::Write,
    process::{Command, ExitStatus, Stdio},
};

/// Execute child processes. See the module documentation on how to use it.
#[macro_export]
macro_rules! cmd {
    ($($args:expr),+) => {{
        let context = $crate::Context::production();
        $crate::panic_on_error($crate::cmd_result_with_context!(context, $($args),+))
    }}
}

/// Like [`cmd!`], but fixes the return type to `()`.
#[macro_export]
macro_rules! cmd_unit {
    ($($args:expr),+) => {{
        let () = $crate::cmd!($($args),+);
    }}
}

/// Like [`cmd!`], but fixes the return type to [`Result<T, Error>`],
/// where `T` is any type that implements [`CmdOutput`].
#[macro_export]
macro_rules! cmd_result {
    ($($args:expr),+) => {{
        let context = $crate::Context::production();
        $crate::cmd_result_with_context!(context, $($args),+)
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! cmd_result_with_context {
    ($context:expr, $($args:expr),+) => {{
        let mut config = $crate::Config::default();
        $($crate::CmdArgument::prepare_config($args, &mut config);)+
        $crate::run_cmd($context, config)
    }}
}

#[doc(hidden)]
pub fn run_cmd<Stdout, Stderr, T>(
    context: Context<Stdout, Stderr>,
    mut config: Config,
) -> Result<T, Error>
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
    T: CmdOutput,
{
    <T as CmdOutput>::prepare_config(&mut config);
    T::from_run_result(&config, run_cmd_safe(context, &config))
}

#[doc(hidden)]
#[derive(Clone)]
pub struct RunResult {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_status: ExitStatus,
}

fn run_cmd_safe<Stdout, Stderr>(
    mut context: Context<Stdout, Stderr>,
    config: &Config,
) -> Result<RunResult, Error>
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
{
    let (executable, arguments) = parse_input(config.arguments.clone())?;
    if config.log_command {
        writeln!(context.stderr, "+ {}", config.full_command())
            .map_err(|error| Error::command_io_error(&config, error))?;
    }
    let mut command = Command::new(&executable);
    command
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(working_directory) = &config.working_directory {
        command.current_dir(working_directory);
    }
    let mut child = command
        .spawn()
        .map_err(|error| Error::command_io_error(&config, error))?;
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
    let exit_status = child
        .wait()
        .map_err(|error| Error::command_io_error(&config, error))?;
    let collected_output = waiter
        .join()
        .map_err(|error| Error::command_io_error(&config, error))?;
    check_exit_status(&config, exit_status)?;
    Ok(RunResult {
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

    macro_rules! cmd_result_with_context_unit {
        ($context:expr, $($args:expr),+) => {{
            let result: std::result::Result<(), $crate::Error> =
              $crate::cmd_result_with_context!($context, $($args),+);
            result
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
                let Stdout(_) = cmd!("false");
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
                let Stdout(_) = cmd!(
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
                let result: Result<(), Error> = cmd_result!("false");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn no_errors() {
                let () = cmd_result!("true").unwrap();
            }

            #[test]
            fn combine_ok_with_other_outputs() {
                // todo: use method
                let result: Result<Stdout, Error> = cmd_result!("echo -n foo");
                assert_eq!(result.unwrap().0, "foo".to_string());
            }

            #[test]
            fn combine_err_with_other_outputs() {
                let result: Result<Stdout, Error> = cmd_result!("false");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_non_zero_exit_codes() {
                let result: Result<(), Error> = cmd_result!("false foo bar");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false foo bar:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_missing_executables() {
                let result: Result<(), Error> = cmd_result!("does-not-exist foo bar");
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
                let result: Result<(), Error> = cmd_result!(
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
                let result: Result<(), Error> = cmd_result!("does-not-exist");
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
                let result: Result<(), Error> = cmd_result!("");
                assert_eq!(result.unwrap_err().to_string(), "no arguments given");
            }

            #[test]
            fn invalid_utf8_stdout() {
                let test_helper = executable_path("stir_test_helper");
                let test_helper = test_helper.to_str().unwrap();
                let result: Result<Stdout, Error> =
                    cmd_result!(test_helper, vec!["invalid utf-8 stdout"]);
                assert_eq!(
                    result.unwrap_err().to_string(),
                    format!(
                        "{} 'invalid utf-8 stdout':\n  invalid utf-8 written to stdout",
                        test_helper
                    )
                );
            }
        }
    }

    #[test]
    fn allows_to_retrieve_stdout() {
        let Stdout(stdout) = cmd!("echo foo");
        assert_eq!(stdout, "foo\n");
    }

    #[test]
    fn command_and_argument_as_separate_ref_str() {
        let Stdout(stdout) = cmd!("echo", "foo");
        assert_eq!(stdout, "foo\n");
    }

    #[test]
    fn multiple_arguments_as_ref_str() {
        let Stdout(stdout) = cmd!("echo", "foo", "bar");
        assert_eq!(stdout, "foo bar\n");
    }

    #[test]
    fn allows_to_pass_in_arguments_as_a_vec_of_ref_str() {
        let args: Vec<&str> = vec!["foo"];
        let Stdout(stdout) = cmd!("echo", args);
        assert_eq!(stdout, "foo\n");
    }

    #[rustversion::since(1.51)]
    #[test]
    fn arrays_as_arguments() {
        let args: [&str; 2] = ["echo", "foo"];
        let Stdout(stdout) = cmd!(args);
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
        let Stdout(stdout) = cmd!(args);
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
        let Stdout(stdout) = cmd!(args);
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
            let Stdout(output) = cmd!(command);
            assert_eq!(output, "foo\n");
        }

        #[test]
        fn multiple_strings() {
            let command: String = "echo".to_string();
            let argument: String = "foo".to_string();
            let Stdout(output) = cmd!(command, argument);
            assert_eq!(output, "foo\n");
        }

        #[test]
        fn mix_ref_str_and_string() {
            let argument: String = "foo".to_string();
            let Stdout(output) = cmd!("echo", argument);
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
            cmd_result_with_context_unit!(context.clone(), "echo foo").unwrap();
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn relays_stdout_for_non_zero_exit_codes() {
            let context = Context::test();
            let _: Result<(), Error> = cmd_result_with_context!(
                context.clone(),
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["output foo and exit with 42"]
            );
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn streams_stdout() {
            in_temporary_directory(|| {
                let context = Context::test();
                let context_clone = context.clone();
                let thread = thread::spawn(|| {
                    cmd_result_with_context_unit!(
                        context_clone,
                        executable_path("stir_test_helper").to_str().unwrap(),
                        vec!["stream chunk then wait for file"]
                    )
                    .unwrap();
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
            let Stdout(_) = cmd_result_with_context!(context.clone(), "echo foo").unwrap();
            assert_eq!(context.stdout(), "");
        }

        #[test]
        fn does_not_relay_stdout_when_collecting_into_result_of_string() {
            let context = Context::test();
            let _: Result<Stdout, Error> = cmd_result_with_context!(context.clone(), "echo foo");
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
            cmd_result_with_context_unit!(
                context.clone(),
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr"]
            )
            .unwrap();
            assert_eq!(context.stderr(), "foo\n");
        }

        #[test]
        fn relays_stderr_for_non_zero_exit_codes() {
            let context = Context::test();
            let _: Result<(), Error> = cmd_result_with_context!(
                context.clone(),
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr and exit with 42"]
            );
            assert_eq!(context.stderr(), "foo\n");
        }

        #[test]
        fn streams_stderr() {
            in_temporary_directory(|| {
                let context = Context::test();
                let context_clone = context.clone();
                let thread = thread::spawn(|| {
                    cmd_result_with_context_unit!(
                        context_clone,
                        executable_path("stir_test_helper").to_str().unwrap(),
                        vec!["stream chunk to stderr then wait for file"]
                    )
                    .unwrap();
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
            assert_eq!(stderr, "foo\n");
        }

        #[test]
        fn assumes_stderr_is_utf_8() {
            let test_helper = executable_path("stir_test_helper");
            let test_helper = test_helper.to_str().unwrap();
            let result: Result<Stderr, Error> =
                cmd_result!(test_helper, vec!["invalid utf-8 stderr"]);
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
            let Stderr(_) = cmd_result_with_context!(
                context.clone(),
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr"]
            )
            .unwrap();
            assert_eq!(context.stderr(), "");
        }
    }

    mod log_commands {
        use super::*;

        #[test]
        fn logs_simple_commands() {
            let context = Context::test();
            cmd_result_with_context_unit!(context.clone(), LogCommand, "true").unwrap();
            assert_eq!(context.stderr(), "+ true\n");
        }

        #[test]
        fn logs_commands_with_arguments() {
            let context = Context::test();
            cmd_result_with_context_unit!(context.clone(), LogCommand, "echo foo").unwrap();
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[test]
        fn quotes_arguments_with_spaces() {
            let context = Context::test();
            cmd_result_with_context_unit!(context.clone(), LogCommand, "echo", vec!["foo bar"])
                .unwrap();
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
        fn failing_commands_return_oks_when_exit_status_is_captured() {
            let Exit(status) = cmd_result!("false").unwrap();
            assert!(!status.success());
        }
    }

    mod tuple_outputs {
        use super::*;

        #[test]
        fn two_tuple_1() {
            let (Stdout(output), Exit(status)) = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["output foo and exit with 42"]
            );
            assert_eq!(output, "foo\n");
            assert_eq!(status.code(), Some(42));
        }

        #[test]
        fn two_tuple_2() {
            let (Exit(status), Stdout(output)) = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["output foo and exit with 42"]
            );
            assert_eq!(output, "foo\n");
            assert_eq!(status.code(), Some(42));
        }

        #[test]
        fn result_of_tuple() {
            let (Stdout(output), Exit(status)) = cmd_result!("echo foo").unwrap();
            assert_eq!(output, "foo\n");
            assert!(status.success());
        }

        #[test]
        fn result_of_tuple_when_erroring() {
            let (Stdout(output), Exit(status)) = cmd_result!("false").unwrap();
            assert_eq!(output, "");
            assert_eq!(status.code(), Some(1));
        }

        #[test]
        fn three_tuples() {
            let (Stderr(stderr), Stdout(stdout), Exit(status)) = cmd!("echo foo");
            assert_eq!(stderr, "");
            assert_eq!(stdout, "foo\n");
            assert_eq!(status.code(), Some(0));
        }

        #[test]
        fn capturing_stdout_on_errors() {
            let (Stdout(output), Exit(status)) = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["output foo and exit with 42"]
            );
            assert!(!status.success());
            assert_eq!(output, "foo\n");
        }

        #[test]
        fn capturing_stderr_on_errors() {
            let (Stderr(output), Exit(status)) = cmd!(
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr and exit with 42"]
            );
            assert!(!status.success());
            assert_eq!(output, "foo\n");
        }
    }

    mod current_dir {
        use super::*;
        use std::{fs, path::Path};

        #[test]
        fn sets_the_working_directory() {
            in_temporary_directory(|| {
                fs::create_dir("dir").unwrap();
                fs::write("dir/file", "foo").unwrap();
                fs::write("file", "wrong file").unwrap();
                let Stdout(output) = cmd!("cat file", CurrentDir("dir"));
                assert_eq!(output, "foo");
            });
        }

        #[test]
        fn works_for_other_types() {
            in_temporary_directory(|| {
                fs::create_dir("dir").unwrap();
                let dir: String = "dir".to_string();
                cmd_unit!("true", CurrentDir(dir));
                let dir: PathBuf = PathBuf::from("dir");
                cmd_unit!("true", CurrentDir(dir));
                let dir: &Path = Path::new("dir");
                cmd_unit!("true", CurrentDir(dir));
            });
        }
    }
}
