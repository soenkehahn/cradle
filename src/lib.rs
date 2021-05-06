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
//! to specify arguments, as long as they implement the [`CmdArgument`]
//! trait:
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
//! use stir::cmd;
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
//! use stir::cmd;
//!
//! let () = cmd!("touch foo");
//! ```
//!
//! Since that's a very common case, `stir` provides the [`cmd_unit!`]
//! shortcut, that behaves exactly like [`cmd!`], but pins the return
//! type down to `()`:
//!
//! ```
//! use stir::cmd_unit;
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
//! use stir::cmd_unit;
//!
//! // panics with "ls: exited with exit code: 1"
//! cmd_unit!("ls does-not-exist");
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

mod cmd_argument;
mod cmd_output;
mod config;
mod context;
mod error;

pub use crate::{
    cmd_argument::{CmdArgument, LogCommand},
    cmd_output::CmdOutput,
    error::{Error, Result},
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
        let context = &mut $crate::Context::production();
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

#[doc(hidden)]
pub fn run_cmd<Stdout, Stderr, T>(context: &mut Context<Stdout, Stderr>, mut config: Config) -> T
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
    T: CmdOutput,
{
    T::prepare_config(&mut config);
    match T::from_run_result(run_cmd_safe(context, config)) {
        Ok(result) => result,
        Err(error) => panic!("{}", error),
    }
}

#[doc(hidden)]
pub struct RunResult {
    stdout: Vec<u8>,
}

fn run_cmd_safe<Stdout, Stderr>(
    context: &mut Context<Stdout, Stderr>,
    config: Config,
) -> Result<RunResult>
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
{
    let (command, arguments) = parse_input(config.arguments.clone())?;
    if config.log_commands {
        write!(context.stderr, "+ {}", config.full_command()).expect("fixme");
    }
    let mut child = Command::new(&command)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| Error::command_io_error(&command, error))?;
    let collected_stdout = context.spawn_standard_stream_relaying(
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
        .map_err(|error| Error::command_io_error(&command, error))?;
    let collected_stdout = collected_stdout
        .join()
        .map_err(|error| Error::command_io_error(&command, error))?;
    check_exit_status(config, exit_status)?;
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

fn check_exit_status(config: Config, exit_status: ExitStatus) -> Result<()> {
    if !exit_status.success() {
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
        F: FnOnce(),
    {
        let temp_dir = TempDir::new().unwrap();
        let original_working_directory = current_dir().unwrap();
        set_current_dir(&temp_dir).unwrap();
        f();
        set_current_dir(original_working_directory).unwrap();
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
            #[should_panic(expected = "false:\n  exited with exit code: 1")]
            fn non_zero_exit_codes() {
                cmd_unit!("false");
            }

            #[test]
            #[should_panic(expected = "false:\n  exited with exit code: 1")]
            fn combine_panics_with_other_outputs() {
                let _: String = cmd!("false");
            }

            #[test]
            #[should_panic(expected = "false foo bar:\n  exited with exit code: 1")]
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
                cmd_unit!("does-not-exist");
            }

            #[test]
            #[should_panic(expected = "cmd!: no arguments given")]
            fn no_executable() {
                cmd_unit!("");
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
                let result: Result<()> = cmd!("");
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
            let context = &mut Context::test();
            cmd_with_context_unit!(context, "echo foo");
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn relays_stdout_for_non_zero_exit_codes() {
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
                    cmd_with_context_unit!(
                        &mut context_clone,
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
            let _: String = cmd_with_context!(&mut context.clone(), "echo foo");
            assert_eq!(context.stdout(), "");
        }

        #[test]
        fn does_not_relay_stdout_when_collecting_into_result_of_string() {
            let context = Context::test();
            let _: Result<String> = cmd_with_context!(&mut context.clone(), "echo foo");
            assert_eq!(context.stdout(), "");
        }
    }

    mod stderr {
        use super::*;
        use std::{thread, time::Duration};

        #[test]
        fn relays_stderr_by_default() {
            let context = &mut Context::test();
            cmd_with_context_unit!(
                context,
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr"]
            );
            assert_eq!(context.stderr(), "foo\n");
        }

        #[test]
        fn relays_stderr_for_non_zero_exit_codes() {
            let context = &mut Context::test();
            let _: Result<()> = cmd_with_context!(
                context,
                executable_path("stir_test_helper").to_str().unwrap(),
                vec!["write to stderr and exit with 42"]
            );
            assert_eq!(context.stderr(), "foo\n");
        }

        #[test]
        fn streams_stderr() {
            in_temporary_directory(|| {
                let context = Context::test();
                let mut context_clone = context.clone();
                let thread = thread::spawn(move || {
                    cmd_with_context_unit!(
                        &mut context_clone,
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
    }

    mod log_commands {
        use super::*;

        #[test]
        fn logs_simple_commands() {
            let mut context = Context::test();
            cmd_with_context_unit!(&mut context, LogCommand, "true");
            assert_eq!(context.stderr(), "+ true");
        }

        #[test]
        fn logs_commands_with_arguments() {
            let mut context = Context::test();
            cmd_with_context_unit!(&mut context, LogCommand, "echo foo");
            assert_eq!(context.stderr(), "+ echo foo");
        }

        #[test]
        fn quotes_arguments_with_spaces_in_single_quotes() {
            let mut context = Context::test();
            cmd_with_context_unit!(&mut context, LogCommand, "echo", vec!["foo bar"]);
            assert_eq!(context.stderr(), "+ echo 'foo bar'");
        }
    }
}
