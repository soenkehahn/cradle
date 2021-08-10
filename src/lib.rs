#![deny(missing_debug_implementations)]

//! (`cradle` is in an early stage of development.
//! APIs may change drastically!
//! Use at your own risk!)
//!
//! `cradle` provides the [`cmd!`] macro, that makes
//! it easy to run child processes from rust programs.
//!
//! ```
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(stdout) = cmd!(%"echo foo");
//! assert_eq!(stdout, "foo");
//! ```
//!
//! # Arguments
//!
//! You can pass in multiple arguments (of different types) to [`cmd!`]
//! to specify arguments, as long as they implement the [`Input`](input::Input)
//! trait:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(stdout) = cmd!("echo", "foo", "bar");
//! assert_eq!(stdout, "foo bar");
//! ```
//!
//! For all possible inputs to [`cmd!`], see the documentation of [`Input`](input::Input).
//!
//! ## Whitespace Splitting
//!
//! `cradle` does *not* split given string arguments on whitespace by default.
//! So for example this code fails:
//!
//! ``` should_panic
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(_) = cmd!("echo foo");
//! ```
//!
//! In this code `cradle` tries to run a process from an executable called
//! `"echo foo"`, including the space in the file name of the executable.
//! That fails, because an executable with that name doesn't exist.
//! `cradle` provides a new-type wrapper [`Split`](input::Split) to help with that:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(output) = cmd!(Split("echo foo"));
//! assert_eq!(output, "foo");
//! ```
//!
//! Wrapping an argument of type `&str` in [`Split`](input::Split) will cause `cradle` to first
//! split it by whitespace and then use the resulting words as if they were passed
//! into [`cmd!`] as separate arguments.
//!
//! And -- since this is such a common case -- `cradle` provides a syntactic shortcut
//! for [`Split`](input::Split), the `%` symbol:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(output) = cmd!(%"echo foo");
//! assert_eq!(output, "foo");
//! ```
//!
//! # Output
//!
//! You can choose which return type you want [`cmd!`] to return,
//! as long as the chosen return type implements [`Output`].
//! For example you can use e.g. [`StdoutTrimmed`](output::StdoutTrimmed)
//! to collect what the child process writes to `stdout`,
//! trimmed of leading and trailing whitespace:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(output) = cmd!(%"echo foo");
//! assert_eq!(output, "foo");
//! ```
//!
//! (By default, the child's `stdout` is written to the parent's `stdout`.
//! Using `StdoutTrimmed` as the return type suppresses that.)
//!
//! If you don't want any result from [`cmd!`], you can use `()`
//! as the return value:
//!
//! ```
//! # let temp_dir = tempfile::TempDir::new().unwrap();
//! # std::env::set_current_dir(&temp_dir).unwrap();
//! use cradle::prelude::*;
//!
//! let () = cmd!(%"touch foo");
//! ```
//!
//! Since that's a very common case, `cradle` provides the [`cmd_unit!`] shortcut.
//! It's named after [the unit type `()`](https://doc.rust-lang.org/std/primitive.unit.html).
//! It behaves exactly like [`cmd!`] but always returns `()`.
//!
//! ```
//! # let temp_dir = tempfile::TempDir::new().unwrap();
//! # std::env::set_current_dir(&temp_dir).unwrap();
//! use cradle::prelude::*;
//!
//! cmd_unit!(%"touch foo");
//! ```
//!
//! See the implementations for [`Output`] for all the supported types.
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
//! use cradle::prelude::*;
//!
//! // panics with "false:\n  exited with exit code: 1"
//! cmd_unit!("false");
//! ```
//!
//! You can suppress panics caused by non-zero exit codes by using the
//! [`Status`](output::Status) type as a return type of [`cmd!`]:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let Status(exit_status) = cmd!("false");
//! assert_eq!(exit_status.code(), Some(1));
//! ```
//!
//! You can also turn **all** panics into [`std::result::Result::Err`]s
//! by using [`cmd_result!`]. This will return a value of type
//! [`Result<T, cradle::Error>`], where
//! `T` is any type that implements [`Output`].
//! Here's some examples:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let result: Result<(), cradle::Error> = cmd_result!("false");
//! let error_message = format!("{}", result.unwrap_err());
//! assert_eq!(
//!     error_message,
//!     "false:\n  exited with exit code: 1"
//! );
//!
//! let result = cmd_result!(%"echo foo");
//! let StdoutTrimmed(output) = result.unwrap();
//! assert_eq!(output, "foo".to_string());
//! ```
//!
//! [`cmd_result`] can also be combined with `?` to handle errors in an
//! idiomatic way, for example:
//!
//! ```
//! use cradle::prelude::*;
//!
//! fn build() -> Result<(), Error> {
//!     cmd_result!(%"which make")?;
//!     cmd_result!(%"which gcc")?;
//!     cmd_result!(%"which ld")?;
//!     cmd_result!(%"make build")?;
//!     Ok(())
//! }
//! ```
//!
//! # Alternative interface
//!
//! `cradle` also provides an alternative interface to execute commands
//! through methods on the [`Input`](input::Input) trait:
//! [`.run()`](input::Input::run), [`.run_unit()`](input::Input::run_unit)
//! and [`.run_result()`](input::Input::run_result).
//! These methods can be invoked on all values whose types implement
//! [`Input`](input::Input).
//! When using these methods, it's especially useful that
//! [`Input`](input::Input) is implemented by tuples.
//! They work analog to [`cmd!`], [`cmd_unit!`] and [`cmd_result!`].
//! Here are some examples:
//!
//! ```
//! # let temp_dir = tempfile::TempDir::new().unwrap();
//! # std::env::set_current_dir(&temp_dir).unwrap();
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(output) = ("echo", "foo").run();
//! assert_eq!(output, "foo");
//!
//! ("touch", "foo").run_unit();
//!
//! let result: Result<(), cradle::Error> = "false".run_result();
//! let error_message = format!("{}", result.unwrap_err());
//! assert_eq!(
//!     error_message,
//!     "false:\n  exited with exit code: 1"
//! );
//! ```
//!
//! Note: The `%` shortcut for [`Split`](input::Split) is not available in this notation.
//! You can either use tuples, or [`Split`](input::Split) explicitly:
//!
//! ```
//! use cradle::prelude::*;
//!
//! ("echo", "foo").run_unit();
//! Split("echo foo").run_unit();
//! ```
//!
//! # Prior Art
//!
//! `cradle` is heavily inspired by [shake](https://shakebuild.com/),
//! specifically by its
//! [`cmd`](https://hackage.haskell.org/package/shake-0.19.4/docs/Development-Shake.html#v:cmd)
//! function.

mod collected_output;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod context;
pub mod error;
pub mod input;
pub mod output;
pub mod prelude;

use crate::{collected_output::Waiter, config::Config, context::Context, output::Output};
pub use error::Error;
use std::{
    ffi::OsString,
    io::Write,
    process::{Command, ExitStatus, Stdio},
    sync::Arc,
};

/// Execute child processes. See the module documentation on how to use it.
#[macro_export]
macro_rules! cmd {
    ($($args:tt)*) => {{
        let context = $crate::context::Context::production();
        $crate::error::panic_on_error($crate::cmd_result_with_context!(context, $($args)*))
    }}
}

/// Like [`cmd!`], but fixes the return type to `()`.
/// It's named after [the unit type `()`](https://doc.rust-lang.org/std/primitive.unit.html).
///
/// ```
/// # let temp_dir = tempfile::TempDir::new().unwrap();
/// # std::env::set_current_dir(&temp_dir).unwrap();
/// use cradle::prelude::*;
///
/// cmd_unit!(%"touch ./foo");
/// ```
#[macro_export]
macro_rules! cmd_unit {
    ($($args:tt)*) => {{
        let () = $crate::cmd!($($args)*);
    }}
}

/// Like [`cmd!`], but fixes the return type to [`Result<T, Error>`],
/// where `T` is any type that implements [`Output`](output::Output).
#[macro_export]
macro_rules! cmd_result {
    ($($args:tt)*) => {{
        let context = $crate::context::Context::production();
        $crate::cmd_result_with_context!(context, $($args)*)
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! cmd_result_with_context {
    ($context:expr, $($args:tt)*) => {{
        let mut config = $crate::config::Config::default();
        $crate::configure!(config: config, args: $($args)*);
        $crate::run_cmd($context, config)
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! configure {
    (config: $config:ident, args: % $head:expr $(,)?) => {
        $crate::input::Input::configure($crate::input::Split($head), &mut $config);
    };
    (config: $config:ident, args: $head:expr $(,)?) => {
        $crate::input::Input::configure($head, &mut $config);
    };
    (config: $config:ident, args: % $head:expr, $($tail:tt)*) => {
        $crate::input::Input::configure($crate::input::Split($head), &mut $config);
        $crate::configure!(config: $config, args: $($tail)*);
    };
    (config: $config:ident, args: $head:expr, $($tail:tt)*) => {
        $crate::input::Input::configure($head, &mut $config);
        $crate::configure!(config: $config, args: $($tail)*);
    };
}

#[doc(hidden)]
pub fn run_cmd<Stdout, Stderr, T>(
    context: Context<Stdout, Stderr>,
    mut config: Config,
) -> Result<T, Error>
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
    T: Output,
{
    <T as Output>::configure(&mut config);
    let result = run_cmd_safe(context, &config);
    T::from_run_result(&config, result)
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct RunResult {
    stdout: Option<Vec<u8>>,
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
            .map_err(|error| Error::command_io_error(config, error))?;
    }
    let mut command = Command::new(&executable);
    command.args(arguments);
    for (key, value) in &config.added_environment_variables {
        command.env(key, value);
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(working_directory) = &config.working_directory {
        command.current_dir(working_directory);
    }
    let mut child = command.spawn().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFoundWhenExecuting {
                executable,
                source: Arc::new(error),
            }
        } else {
            Error::command_io_error(config, error)
        }
    })?;
    let waiter = Waiter::spawn_standard_stream_relaying(
        &context,
        config,
        child.stdin.take().expect("child process should have stdin"),
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
        .map_err(|error| Error::command_io_error(config, error))?;
    let collected_output = waiter
        .join()
        .map_err(|error| Error::command_io_error(config, error))?;
    check_exit_status(config, exit_status)?;
    Ok(RunResult {
        stdout: collected_output.stdout,
        stderr: collected_output.stderr,
        exit_status,
    })
}

fn parse_input(input: Vec<OsString>) -> Result<(OsString, impl Iterator<Item = OsString>), Error> {
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
    use crate::context::Context;
    use crate::prelude::*;
    use lazy_static::lazy_static;
    use std::{
        collections::BTreeSet,
        env::{current_dir, set_current_dir},
        ffi::{OsStr, OsString},
        fs,
        path::PathBuf,
        sync::{Arc, Mutex},
    };
    use tempfile::TempDir;

    fn in_temporary_directory<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        lazy_static! {
            static ref CURRENT_DIR_LOCK: Mutex<()> = Mutex::new(());
        }
        let _lock = CURRENT_DIR_LOCK.lock();
        let temp_dir = TempDir::new().unwrap();
        let original_working_directory = current_dir().unwrap();
        set_current_dir(&temp_dir).unwrap();
        let result = std::panic::catch_unwind(|| {
            f();
        });
        set_current_dir(original_working_directory).unwrap();
        result.unwrap();
    }

    fn test_executable(name: &str) -> PathBuf {
        lazy_static! {
            static ref BUILT: Arc<Mutex<BTreeSet<String>>> = Arc::new(Mutex::new(BTreeSet::new()));
        }
        let mut set = BUILT.lock().unwrap();
        if !set.contains(name) {
            set.insert(name.to_owned());
            cmd_unit!(
                LogCommand,
                CurrentDir(std::env::var("CARGO_MANIFEST_DIR").unwrap()),
                %"cargo build",
                ("--bin", name),
                %"--features test_executables",
            );
        }
        executable_path::executable_path(name)
    }

    fn test_helper() -> PathBuf {
        test_executable("test_executables_helper")
    }

    macro_rules! cmd_result_with_context_unit {
        ($context:expr, $($args:tt)*) => {{
            let result: std::result::Result<(), $crate::Error> =
              $crate::cmd_result_with_context!($context, $($args)*);
            result
        }}
    }

    #[test]
    fn allows_to_execute_a_command() {
        in_temporary_directory(|| {
            cmd_unit!(%"touch foo");
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
                let StdoutTrimmed(_) = cmd!("false");
            }

            #[test]
            #[should_panic(expected = "cmd!: false foo bar:\n  exited with exit code: 1")]
            fn includes_full_command_on_non_zero_exit_codes() {
                cmd_unit!(%"false foo bar");
            }

            #[test]
            #[should_panic(expected = "exited with exit code: 42")]
            fn other_exit_codes() {
                cmd_unit!(test_helper(), "exit code 42");
            }

            #[test]
            #[should_panic(expected = "cmd!: File not found error when executing 'does-not-exist'")]
            fn executable_cannot_be_found() {
                cmd_unit!("does-not-exist");
            }

            #[test]
            #[cfg(unix)]
            #[should_panic(expected = "/file foo bar:\n  Permission denied (os error 13)")]
            fn includes_full_command_on_io_errors() {
                let temp_dir = TempDir::new().unwrap();
                let without_executable_bit = temp_dir.path().join("file");
                fs::write(&without_executable_bit, "").unwrap();
                cmd_unit!(without_executable_bit, %"foo bar");
            }

            #[rustversion::since(1.46)]
            #[test]
            fn includes_source_location_of_cmd_call() {
                let (Status(_), Stderr(stderr)) = cmd!(test_executable("test_executables_panic"));
                let expected = "src/test_executables/panic.rs:4:5";
                assert!(
                    stderr.contains(expected),
                    "{:?}\n  does not contain\n{:?}",
                    stderr,
                    expected
                );
            }

            #[test]
            #[should_panic(expected = "cmd!: no arguments given")]
            fn no_executable() {
                let vector: Vec<String> = Vec::new();
                cmd_unit!(vector);
            }

            #[test]
            #[should_panic(expected = "invalid utf-8 written to stdout")]
            fn invalid_utf8_stdout() {
                let StdoutTrimmed(_) = cmd!(test_helper(), "invalid utf-8 stdout");
            }

            #[test]
            #[cfg(not(windows))]
            fn invalid_utf8_to_stdout_is_allowed_when_not_captured() {
                cmd_unit!(test_helper(), "invalid utf-8 stdout");
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
                let result: Result<(), Error> = cmd_result!("true");
                result.unwrap();
            }

            #[test]
            fn combine_ok_with_other_outputs() {
                let StdoutTrimmed(output) = cmd_result!(%"echo foo").unwrap();
                assert_eq!(output, "foo".to_string());
            }

            #[test]
            fn combine_err_with_other_outputs() {
                let result: Result<StdoutTrimmed, Error> = cmd_result!("false");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_non_zero_exit_codes() {
                let result: Result<(), Error> = cmd_result!(%"false foo bar");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false foo bar:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_io_errors() {
                in_temporary_directory(|| {
                    fs::write("without-executable-bit", "").unwrap();
                    let result: Result<(), Error> =
                        cmd_result!(%"./without-executable-bit foo bar");
                    assert_eq!(
                        result.unwrap_err().to_string(),
                        if cfg!(windows) {
                            "./without-executable-bit foo bar:\n  %1 is not a valid Win32 application. (os error 193)"
                        } else {
                            "./without-executable-bit foo bar:\n  Permission denied (os error 13)"
                        }
                    );
                });
            }

            #[test]
            fn other_exit_codes() {
                let result: Result<(), Error> = cmd_result!(test_helper(), "exit code 42");
                assert!(result
                    .unwrap_err()
                    .to_string()
                    .contains("exited with exit code: 42"));
            }

            #[test]
            fn missing_executable_file_error_message() {
                let result: Result<(), Error> = cmd_result!("does-not-exist");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "File not found error when executing 'does-not-exist'"
                );
            }

            #[test]
            fn missing_executable_file_error_can_be_matched_against() {
                let result: Result<(), Error> = cmd_result!("does-not-exist");
                match result {
                    Err(Error::FileNotFoundWhenExecuting { executable, .. }) => {
                        assert_eq!(executable, "does-not-exist");
                    }
                    _ => panic!("should match Error::FileNotFoundWhenExecuting"),
                }
            }

            #[test]
            fn missing_executable_file_error_can_be_caused_by_relative_paths() {
                let result: Result<(), Error> = cmd_result!("./does-not-exist");
                match result {
                    Err(Error::FileNotFoundWhenExecuting { executable, .. }) => {
                        assert_eq!(executable, "./does-not-exist");
                    }
                    _ => panic!("should match Error::FileNotFoundWhenExecuting"),
                }
            }

            #[test]
            fn missing_executable_file_with_spaces_includes_hint() {
                let result: Result<(), Error> = cmd_result!("does not exist");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    vec![
                        "File not found error when executing 'does not exist'",
                        "note: Executable name 'does not exist' includes spaces.",
                        "  Did you mean to run 'does', with [\"not\", \"exist\"] as arguments?",
                        "  Consider using Split: https://docs.rs/cradle/latest/cradle/input/struct.Split.html",
                    ]
                    .join("\n")
                );
            }

            #[test]
            fn no_executable() {
                let vector: Vec<String> = Vec::new();
                let result: Result<(), Error> = cmd_result!(vector);
                assert_eq!(result.unwrap_err().to_string(), "no arguments given");
            }

            #[test]
            fn invalid_utf8_stdout() {
                let test_helper = test_helper();
                let result: Result<StdoutTrimmed, Error> =
                    cmd_result!(&test_helper, "invalid utf-8 stdout");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    format!(
                        "{} 'invalid utf-8 stdout':\n  invalid utf-8 written to stdout",
                        test_helper.display()
                    )
                );
            }
        }
    }

    #[test]
    fn allows_to_retrieve_stdout() {
        let StdoutTrimmed(stdout) = cmd!(%"echo foo");
        assert_eq!(stdout, "foo");
    }

    #[test]
    fn command_and_argument_as_separate_ref_str() {
        let StdoutTrimmed(stdout) = cmd!("echo", "foo");
        assert_eq!(stdout, "foo");
    }

    #[test]
    fn multiple_arguments_as_ref_str() {
        let StdoutTrimmed(stdout) = cmd!("echo", "foo", "bar");
        assert_eq!(stdout, "foo bar");
    }

    #[test]
    fn arguments_can_be_given_as_references() {
        let reference: &LogCommand = &LogCommand;
        let executable: &String = &"echo".to_string();
        let argument: &String = &"foo".to_string();
        let StdoutTrimmed(stdout) = cmd!(reference, executable, argument);
        assert_eq!(stdout, "foo");
    }

    mod sequences {
        use super::*;

        #[test]
        fn allows_to_pass_in_arguments_as_a_vec_of_ref_str() {
            let args: Vec<&str> = vec!["foo"];
            let StdoutTrimmed(stdout) = cmd!("echo", args);
            assert_eq!(stdout, "foo");
        }

        #[test]
        fn vector_of_non_strings() {
            let context = Context::test();
            let config: Vec<LogCommand> = vec![LogCommand];
            let StdoutTrimmed(stdout) =
                cmd_result_with_context!(context.clone(), config, %"echo foo").unwrap();
            assert_eq!(stdout, "foo");
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[rustversion::since(1.51)]
        #[test]
        fn arrays_as_arguments() {
            let args: [&str; 2] = ["echo", "foo"];
            let StdoutTrimmed(stdout) = cmd!(args);
            assert_eq!(stdout, "foo");
        }

        #[rustversion::since(1.51)]
        #[test]
        fn arrays_of_non_strings() {
            let context = Context::test();
            let config: [LogCommand; 1] = [LogCommand];
            let StdoutTrimmed(stdout) =
                cmd_result_with_context!(context.clone(), config, %"echo foo").unwrap();
            assert_eq!(stdout, "foo");
            assert_eq!(context.stderr(), "+ echo foo\n");
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
            let StdoutTrimmed(stdout) = cmd!(args);
            assert_eq!(stdout, "foo");
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
            let StdoutTrimmed(stdout) = cmd!(args);
            assert_eq!(stdout, "foo");
        }

        #[test]
        fn slices_of_non_strings() {
            let context = Context::test();
            let config: &[LogCommand] = &[LogCommand];
            let StdoutTrimmed(stdout) =
                cmd_result_with_context!(context.clone(), config, %"echo foo").unwrap();
            assert_eq!(stdout, "foo");
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[test]
        fn elements_in_slices_are_not_split_by_whitespace() {
            in_temporary_directory(|| {
                let args: &[&str] = &["foo bar"];
                cmd_unit!("touch", args);
                assert!(PathBuf::from("foo bar").exists());
            });
        }

        #[test]
        fn vector_of_vectors() {
            let StdoutTrimmed(output) = cmd!(vec![vec!["echo"], vec!["foo", "bar"]]);
            assert_eq!(output, "foo bar");
        }
    }

    mod strings {
        use super::*;

        #[test]
        fn works_for_string() {
            let command: String = "true".to_string();
            cmd_unit!(command);
        }

        #[test]
        fn multiple_strings() {
            let command: String = "echo".to_string();
            let argument: String = "foo".to_string();
            let StdoutTrimmed(output) = cmd!(command, argument);
            assert_eq!(output, "foo");
        }

        #[test]
        fn mix_ref_str_and_string() {
            let argument: String = "foo".to_string();
            let StdoutTrimmed(output) = cmd!("echo", argument);
            assert_eq!(output, "foo");
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

    mod os_strings {
        use super::*;

        #[test]
        fn works_for_os_string() {
            cmd_unit!(OsString::from("true"));
        }

        #[test]
        fn works_for_os_str() {
            cmd_unit!(OsStr::new("true"));
        }
    }

    mod stdout {
        use super::*;
        use std::{thread, time::Duration};

        #[test]
        fn relays_stdout_by_default() {
            let context = Context::test();
            cmd_result_with_context_unit!(context.clone(), %"echo foo").unwrap();
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn relays_stdout_for_non_zero_exit_codes() {
            let context = Context::test();
            let _: Result<(), Error> = cmd_result_with_context!(
                context.clone(),
                test_helper(),
                "output foo and exit with 42"
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
                        test_helper(),
                        "stream chunk then wait for file"
                    )
                    .unwrap();
                });
                while (context.stdout()) != "foo\n" {
                    thread::sleep(Duration::from_secs_f32(0.05));
                }
                cmd_unit!(%"touch file");
                thread.join().unwrap();
            });
        }

        #[test]
        fn does_not_relay_stdout_when_collecting_into_string() {
            let context = Context::test();
            let StdoutTrimmed(_) = cmd_result_with_context!(context.clone(), %"echo foo").unwrap();
            assert_eq!(context.stdout(), "");
        }

        #[test]
        fn does_not_relay_stdout_when_collecting_into_result_of_string() {
            let context = Context::test();
            let _: Result<StdoutTrimmed, Error> =
                cmd_result_with_context!(context.clone(), %"echo foo");
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
            cmd_result_with_context_unit!(context.clone(), test_helper(), "write to stderr")
                .unwrap();
            assert_eq!(context.stderr(), "foo\n");
        }

        #[test]
        fn relays_stderr_for_non_zero_exit_codes() {
            let context = Context::test();
            let _: Result<(), Error> = cmd_result_with_context!(
                context.clone(),
                test_helper(),
                "write to stderr and exit with 42"
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
                        test_helper(),
                        "stream chunk to stderr then wait for file"
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
                cmd_unit!(%"touch file");
                thread.join().unwrap();
            });
        }

        #[test]
        fn capture_stderr() {
            let Stderr(stderr) = cmd!(test_helper(), "write to stderr");
            assert_eq!(stderr, "foo\n");
        }

        #[test]
        fn assumes_stderr_is_utf_8() {
            let result: Result<Stderr, Error> = cmd_result!(test_helper(), "invalid utf-8 stderr");
            assert_eq!(
                result.unwrap_err().to_string(),
                format!(
                    "{} 'invalid utf-8 stderr':\n  invalid utf-8 written to stderr",
                    test_helper().display(),
                )
            );
        }

        #[test]
        #[cfg(not(windows))]
        fn does_allow_invalid_utf_8_to_stderr_when_not_captured() {
            cmd_unit!(test_helper(), "invalid utf-8 stderr");
        }

        #[test]
        fn does_not_relay_stderr_when_catpuring() {
            let context = Context::test();
            let Stderr(_) =
                cmd_result_with_context!(context.clone(), test_helper(), "write to stderr")
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
            cmd_result_with_context_unit!(context.clone(), LogCommand, %"echo foo").unwrap();
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[test]
        fn quotes_arguments_with_spaces() {
            let context = Context::test();
            cmd_result_with_context_unit!(context.clone(), LogCommand, "echo", "foo bar").unwrap();
            assert_eq!(context.stderr(), "+ echo 'foo bar'\n");
        }

        #[test]
        fn quotes_empty_arguments() {
            let context = Context::test();
            cmd_result_with_context_unit!(context.clone(), LogCommand, "echo", "").unwrap();
            assert_eq!(context.stderr(), "+ echo ''\n");
        }

        #[test]
        #[cfg(unix)]
        fn arguments_with_invalid_utf8_will_be_logged_with_lossy_conversion() {
            use std::{ffi::OsStr, os::unix::prelude::OsStrExt, path::Path};
            let context = Context::test();
            let argument_with_invalid_utf8: &OsStr =
                OsStrExt::from_bytes(&[102, 111, 111, 0x80, 98, 97, 114]);
            let argument_with_invalid_utf8: &Path = argument_with_invalid_utf8.as_ref();
            cmd_result_with_context_unit!(
                context.clone(),
                LogCommand,
                "echo",
                argument_with_invalid_utf8
            )
            .unwrap();
            assert_eq!(context.stderr(), "+ echo foo�bar\n");
        }
    }

    mod exit_status {
        use super::*;

        #[test]
        fn zero() {
            let Status(exit_status) = cmd!("true");
            assert!(exit_status.success());
        }

        #[test]
        fn one() {
            let Status(exit_status) = cmd!("false");
            assert!(!exit_status.success());
        }

        #[test]
        fn forty_two() {
            let Status(exit_status) = cmd!(test_helper(), "exit code 42");
            assert!(!exit_status.success());
            assert_eq!(exit_status.code(), Some(42));
        }

        #[test]
        fn failing_commands_return_oks_when_exit_status_is_captured() {
            let Status(exit_status) = cmd_result!("false").unwrap();
            assert!(!exit_status.success());
        }
    }

    mod bool_output {
        #[test]
        fn success_exit_status_is_true() {
            assert!(cmd!("true"));
        }

        #[test]
        fn failure_exit_status_is_false() {
            assert!(!cmd!("false"));
        }

        #[test]
        #[should_panic]
        fn io_error_panics() {
            assert!(cmd!("/"));
        }
    }

    mod tuple_inputs {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn two_tuple() {
            let StdoutTrimmed(output) = cmd!(("echo", "foo"));
            assert_eq!(output, "foo");
        }

        #[test]
        fn three_tuples() {
            let StdoutTrimmed(output) = cmd!(("echo", "foo", "bar"));
            assert_eq!(output, "foo bar");
        }

        #[test]
        fn nested_tuples() {
            let StdoutTrimmed(output) = cmd!(("echo", ("foo", "bar")));
            assert_eq!(output, "foo bar");
        }

        #[test]
        fn unit_input() {
            let StdoutTrimmed(output) = cmd!(("echo", ()));
            assert_eq!(output, "");
        }
    }

    mod tuple_outputs {
        use super::*;

        #[test]
        fn two_tuple_1() {
            let (StdoutTrimmed(output), Status(exit_status)) =
                cmd!(test_helper(), "output foo and exit with 42");
            assert_eq!(output, "foo");
            assert_eq!(exit_status.code(), Some(42));
        }

        #[test]
        fn two_tuple_2() {
            let (Status(exit_status), StdoutTrimmed(output)) =
                cmd!(test_helper(), "output foo and exit with 42");
            assert_eq!(output, "foo");
            assert_eq!(exit_status.code(), Some(42));
        }

        #[test]
        fn result_of_tuple() {
            let (StdoutTrimmed(output), Status(exit_status)) = cmd_result!(%"echo foo").unwrap();
            assert_eq!(output, "foo");
            assert!(exit_status.success());
        }

        #[test]
        fn result_of_tuple_when_erroring() {
            let (StdoutTrimmed(output), Status(exit_status)) = cmd_result!("false").unwrap();
            assert_eq!(output, "");
            assert_eq!(exit_status.code(), Some(1));
        }

        #[test]
        fn three_tuples() {
            let (Stderr(stderr), StdoutTrimmed(stdout), Status(exit_status)) = cmd!(%"echo foo");
            assert_eq!(stderr, "");
            assert_eq!(stdout, "foo");
            assert_eq!(exit_status.code(), Some(0));
        }

        #[test]
        fn capturing_stdout_on_errors() {
            let (StdoutTrimmed(output), Status(exit_status)) =
                cmd!(test_helper(), "output foo and exit with 42");
            assert!(!exit_status.success());
            assert_eq!(output, "foo");
        }

        #[test]
        fn capturing_stderr_on_errors() {
            let (Stderr(output), Status(exit_status)) =
                cmd!(test_helper(), "write to stderr and exit with 42");
            assert!(!exit_status.success());
            assert_eq!(output, "foo\n");
        }
    }

    mod current_dir {
        use super::*;
        use std::path::Path;

        #[test]
        fn sets_the_working_directory() {
            in_temporary_directory(|| {
                fs::create_dir("dir").unwrap();
                fs::write("dir/file", "foo").unwrap();
                fs::write("file", "wrong file").unwrap();
                let StdoutUntrimmed(output) = cmd!(%"cat file", CurrentDir("dir"));
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

    mod capturing_stdout {
        use super::*;

        mod trimmed {
            use super::*;

            #[test]
            fn trims_trailing_whitespace() {
                let StdoutTrimmed(output) = cmd!(%"echo foo");
                assert_eq!(output, "foo");
            }

            #[test]
            fn trims_leading_whitespace() {
                let StdoutTrimmed(output) = cmd!(%"echo -n", " foo");
                assert_eq!(output, "foo");
            }

            #[test]
            fn does_not_remove_whitespace_within_output() {
                let StdoutTrimmed(output) = cmd!(%"echo -n", "foo bar");
                assert_eq!(output, "foo bar");
            }

            #[test]
            fn does_not_modify_output_without_whitespace() {
                let StdoutTrimmed(output) = cmd!(%"echo -n", "foo");
                assert_eq!(output, "foo");
            }

            #[test]
            fn does_not_relay_stdout() {
                let context = Context::test();
                let StdoutTrimmed(_) =
                    cmd_result_with_context!(context.clone(), %"echo foo").unwrap();
                assert_eq!(context.stdout(), "");
            }
        }

        mod untrimmed {
            use super::*;

            #[test]
            fn does_not_trim_trailing_newline() {
                let StdoutUntrimmed(output) = cmd!(%"echo foo");
                assert_eq!(output, "foo\n");
            }

            #[test]
            fn does_not_trim_leading_whitespace() {
                let StdoutUntrimmed(output) = cmd!(%"echo -n", " foo");
                assert_eq!(output, " foo");
            }

            #[test]
            fn does_not_relay_stdout() {
                let context = Context::test();
                let StdoutUntrimmed(_) =
                    cmd_result_with_context!(context.clone(), %"echo foo").unwrap();
                assert_eq!(context.stdout(), "");
            }
        }
    }

    mod split {
        use super::*;

        #[test]
        fn splits_words_by_whitespace() {
            let StdoutTrimmed(output) = cmd!(Split("echo foo"));
            assert_eq!(output, "foo");
        }

        #[test]
        fn splits_owned_strings() {
            let StdoutTrimmed(output) = cmd!(Split("echo foo".to_string()));
            assert_eq!(output, "foo");
        }

        #[test]
        fn skips_multiple_whitespace_characters() {
            let StdoutUntrimmed(output) = cmd!("echo", Split("foo  bar"));
            assert_eq!(output, "foo bar\n");
        }

        #[test]
        fn trims_leading_whitespace() {
            let StdoutTrimmed(output) = cmd!(Split(" echo foo"));
            assert_eq!(output, "foo");
        }

        #[test]
        fn trims_trailing_whitespace() {
            let StdoutUntrimmed(output) = cmd!("echo", Split("foo "));
            assert_eq!(output, "foo\n");
        }

        mod percent_sign {
            use super::*;

            #[test]
            fn splits_words() {
                let StdoutUntrimmed(output) = cmd!(%"echo foo");
                assert_eq!(output, "foo\n");
            }

            #[test]
            fn works_for_later_arguments() {
                let StdoutUntrimmed(output) = cmd!("echo", %"foo\tbar");
                assert_eq!(output, "foo bar\n");
            }

            #[test]
            fn for_first_of_multiple_arguments() {
                let StdoutUntrimmed(output) = cmd!(%"echo foo", "bar");
                assert_eq!(output, "foo bar\n");
            }

            #[test]
            fn non_literals() {
                let command = "echo foo";
                let StdoutUntrimmed(output) = cmd!(%command);
                assert_eq!(output, "foo\n");
            }

            #[test]
            fn in_cmd_unit() {
                cmd_unit!(%"echo foo");
            }

            #[test]
            fn in_cmd_result() {
                let StdoutTrimmed(_) = cmd_result!(%"echo foo").unwrap();
            }
        }
    }

    mod splitting_with_library_functions {
        use super::*;

        #[test]
        fn allow_to_use_split() {
            let StdoutTrimmed(output) = cmd!("echo foo".split(' '));
            assert_eq!(output, "foo");
        }

        #[test]
        fn split_whitespace() {
            let StdoutTrimmed(output) = cmd!("echo foo".split_whitespace());
            assert_eq!(output, "foo");
        }

        #[test]
        fn split_ascii_whitespace() {
            let StdoutTrimmed(output) = cmd!("echo foo".split_ascii_whitespace());
            assert_eq!(output, "foo");
        }
    }

    mod paths {
        use super::*;
        use pretty_assertions::assert_eq;
        use std::path::Path;

        fn write_test_script() -> PathBuf {
            if cfg!(unix) {
                let file = PathBuf::from("./test-script");
                let script = "#!/usr/bin/env bash\necho test-output\n";
                fs::write(&file, script).unwrap();
                cmd_unit!(%"chmod +x test-script");
                file
            } else {
                let file = PathBuf::from("./test-script.bat");
                let script = "@echo test-output\n";
                fs::write(&file, script).unwrap();
                file
            }
        }

        #[test]
        fn ref_path_as_argument() {
            in_temporary_directory(|| {
                let file: &Path = Path::new("file");
                fs::write(file, "test-contents").unwrap();
                let StdoutUntrimmed(output) = cmd!("cat", file);
                assert_eq!(output, "test-contents");
            })
        }

        #[test]
        fn ref_path_as_executable() {
            in_temporary_directory(|| {
                let file: &Path = &write_test_script();
                let StdoutTrimmed(output) = cmd!(file);
                assert_eq!(output, "test-output");
            })
        }

        #[test]
        fn path_buf_as_argument() {
            in_temporary_directory(|| {
                let file: PathBuf = PathBuf::from("file");
                fs::write(&file, "test-contents").unwrap();
                let StdoutUntrimmed(output) = cmd!("cat", file);
                assert_eq!(output, "test-contents");
            })
        }

        #[test]
        fn path_buf_as_executable() {
            in_temporary_directory(|| {
                let file: PathBuf = write_test_script();
                let StdoutTrimmed(output) = cmd!(file);
                assert_eq!(output, "test-output");
            })
        }
    }

    mod stdin {
        use super::*;

        #[test]
        fn allows_to_pass_in_strings_as_stdin() {
            let StdoutUntrimmed(output) = cmd!(test_helper(), "reverse", Stdin("foo"));
            assert_eq!(output, "oof");
        }

        #[test]
        fn allows_passing_in_u8_slices_as_stdin() {
            let StdoutUntrimmed(output) = cmd!(test_helper(), "reverse", Stdin(&[0, 1, 2]));
            assert_eq!(output, "\x02\x01\x00");
        }

        #[test]
        #[cfg(unix)]
        fn stdin_is_closed_by_default() {
            let StdoutTrimmed(output) = cmd!(test_helper(), "wait until stdin is closed");
            assert_eq!(output, "stdin is closed");
        }

        #[test]
        fn writing_too_many_bytes_into_a_non_reading_child_may_error() {
            let big_string = String::from_utf8(vec![b'a'; 2_usize.pow(16) + 1]).unwrap();
            let result: Result<(), crate::Error> = cmd_result!("true", Stdin(big_string));
            let message = result.unwrap_err().to_string();
            assert!(if cfg!(unix) {
                message == "true:\n  Broken pipe (os error 32)"
            } else {
                [
                    "true:\n  The pipe is being closed. (os error 232)",
                    "true:\n  The pipe has been ended. (os error 109)",
                ]
                .contains(&message.as_str())
            });
        }

        #[test]
        fn multiple_stdin_arguments_are_all_passed_into_the_child_process() {
            let StdoutUntrimmed(output) =
                cmd!(test_helper(), "reverse", Stdin("foo"), Stdin("bar"));
            assert_eq!(output, "raboof");
        }

        #[test]
        fn works_for_owned_strings() {
            let argument: String = "foo".to_string();
            let StdoutUntrimmed(output) = cmd!(test_helper(), "reverse", Stdin(argument));
            assert_eq!(output, "oof");
        }
    }

    mod invocation_syntax {
        use super::*;

        #[test]
        fn trailing_comma_is_accepted_after_normal_argument() {
            cmd_unit!("echo", "foo",);
            let StdoutUntrimmed(_) = cmd!("echo", "foo",);
            let _result: Result<(), Error> = cmd_result!("echo", "foo",);
        }

        #[test]
        fn trailing_comma_is_accepted_after_split_argument() {
            cmd_unit!("echo", %"foo",);
            let StdoutUntrimmed(_) = cmd!("echo", %"foo",);
            let _result: Result<(), Error> = cmd_result!("echo", %"foo",);
        }
    }

    mod environment_variables {
        use super::*;
        use pretty_assertions::assert_eq;
        use std::env;

        #[test]
        fn allows_to_add_variables() {
            let StdoutTrimmed(output) = cmd!(
                test_helper(),
                %"echo FOO",
                Env("FOO", "bar")
            );
            assert_eq!(output, "bar");
        }

        #[test]
        fn works_for_multiple_variables() {
            let StdoutUntrimmed(output) = cmd!(
                test_helper(),
                %"echo FOO BAR",
                Env("FOO", "a"),
                Env("BAR", "b")
            );
            assert_eq!(output, "a\nb\n");
        }

        fn find_unused_environment_variable() -> String {
            let mut i = 0;
            loop {
                let key = format!("CRADLE_TEST_VARIABLE_{}", i);
                if env::var_os(&key).is_none() {
                    break key;
                }
                i += 1;
            }
        }

        #[test]
        fn child_processes_inherit_the_environment() {
            let unused_key = find_unused_environment_variable();
            env::set_var(&unused_key, "foo");
            let StdoutTrimmed(output) = cmd!(test_helper(), "echo", unused_key);
            assert_eq!(output, "foo");
        }

        #[test]
        fn overwrites_existing_parent_variables() {
            let unused_key = find_unused_environment_variable();
            env::set_var(&unused_key, "foo");
            let StdoutTrimmed(output) =
                cmd!(test_helper(), "echo", &unused_key, Env(unused_key, "bar"));
            assert_eq!(output, "bar");
        }

        #[test]
        fn variables_are_overwritten_by_subsequent_variables_with_the_same_name() {
            let StdoutTrimmed(output) = cmd!(
                test_helper(),
                "echo",
                "FOO",
                Env("FOO", "a"),
                Env("FOO", "b"),
            );
            assert_eq!(output, "b");
        }

        #[test]
        fn variables_can_be_set_to_the_empty_string() {
            let StdoutUntrimmed(output) = cmd!(test_helper(), "echo", "FOO", Env("FOO", ""),);
            assert_eq!(output, "empty variable: FOO\n");
        }
    }

    mod run_interface {
        use super::*;
        use std::path::Path;

        #[test]
        fn allows_to_run_commands_with_dot_run() {
            let StdoutTrimmed(output) = Split("echo foo").run();
            assert_eq!(output, "foo");
        }

        #[test]
        fn allows_to_bundle_arguments_up_in_tuples() {
            let StdoutTrimmed(output) = ("echo", "foo").run();
            assert_eq!(output, "foo");
        }

        #[test]
        fn works_for_different_output_types() {
            let Status(status) = "false".run();
            assert!(!status.success());
        }

        #[test]
        fn run_unit() {
            in_temporary_directory(|| {
                ("touch", "foo").run_unit();
                assert!(Path::new("foo").exists());
            });
        }

        #[test]
        fn run_result() {
            let StdoutTrimmed(output) = ("echo", "foo").run_result().unwrap();
            assert_eq!(output, "foo");
            let result: Result<(), Error> = "does-not-exist".run_result();
            match result {
                Err(Error::FileNotFoundWhenExecuting { .. }) => {}
                _ => panic!("should match Error::FileNotFoundWhenExecuting"),
            }
        }
    }
}
