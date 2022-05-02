#![deny(missing_debug_implementations)]

//! `cradle` provides the [`run!`] macro, that makes
//! it easy to run child processes from rust programs.
//!
//! ```
//! # let temp_dir = tempfile::TempDir::new().unwrap();
//! # std::env::set_current_dir(&temp_dir).unwrap();
//! use cradle::prelude::*;
//! use std::path::Path;
//!
//! run!(%"touch foo");
//! assert!(Path::new("foo").is_file());
//! ```
//!
//! # Input
//!
//! You can pass in multiple arguments (of different types) to [`run!`]
//! to specify arguments, as long as they implement the [`Input`] trait:
//!
//! ```
//! # let temp_dir = tempfile::TempDir::new().unwrap();
//! # std::env::set_current_dir(&temp_dir).unwrap();
//! use cradle::prelude::*;
//! use std::path::Path;
//!
//! run!("mkdir", "-p", "foo/bar/baz");
//! assert!(Path::new("foo/bar/baz").is_dir());
//! ```
//!
//! For all possible inputs to [`run!`], see the documentation of [`Input`].
//!
//! # Output
//!
//! `cradle` also provides a [`run_output!`] macro.
//! It allows to capture outputs of the child process.
//! It uses return-type polymorphism, so you can control which outputs
//! are captured by choosing the return type of [`run_output!`].
//! The only constraint is that the chosen return type has to implement [`Output`].
//! For example you can use e.g. [`StdoutTrimmed`]
//! to collect what the child process writes to `stdout`,
//! trimmed of leading and trailing whitespace:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(output) = run_output!(%"echo foo");
//! assert_eq!(output, "foo");
//! ```
//!
//! (By default, the child's `stdout` is written to the parent's `stdout`.
//! Using `StdoutTrimmed` as the return type suppresses that.)
//!
//! If you don't want any result from [`run_output!`], you can use `()`
//! as the return value:
//!
//! ```
//! # let temp_dir = tempfile::TempDir::new().unwrap();
//! # std::env::set_current_dir(&temp_dir).unwrap();
//! use cradle::prelude::*;
//!
//! let () = run_output!(%"touch foo");
//! ```
//!
//! Since that's a very common case, `cradle` provides the [`run!`] shortcut,
//! that we've already seen above.
//! It behaves exactly like [`run_output!`] but always returns `()`:
//!
//! ```
//! # let temp_dir = tempfile::TempDir::new().unwrap();
//! # std::env::set_current_dir(&temp_dir).unwrap();
//! use cradle::prelude::*;
//!
//! run!(%"touch foo");
//! ```
//!
//! See the implementations for [`output::Output`] for all the supported types.
//!
//! # Whitespace Splitting of Inputs
//!
//! `cradle` does *not* split given string arguments on whitespace by default.
//! So for example this code fails:
//!
//! ``` should_panic
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(_) = run_output!("echo foo");
//! ```
//!
//! In this code `cradle` tries to run a process from an executable called
//! `"echo foo"`, including the space in the file name of the executable.
//! That fails, because an executable with that name doesn't exist.
//! `cradle` provides a new-type wrapper [`Split`] to help with that:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(output) = run_output!(Split("echo foo"));
//! assert_eq!(output, "foo");
//! ```
//!
//! Wrapping an argument of type `&str` in [`Split`] will cause `cradle` to first
//! split it by whitespace and then use the resulting words as if they were passed
//! into [`run_output!`] as separate arguments.
//!
//! And -- since this is such a common case -- `cradle` provides a syntactic shortcut
//! for [`Split`], the `%` symbol:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let StdoutTrimmed(output) = run_output!(%"echo foo");
//! assert_eq!(output, "foo");
//! ```
//!
//! # Error Handling
//!
//! **tl;dr:** [`run!`] and [`run_output!`] will panic on errors,
//! [`run_result!`] will not.
//!
//! ## Panicking
//!
//! By default [`run!`] and [`run_output!`] panic when something goes wrong,
//! for example when the executable cannot be found or
//! when a child process exits with a non-zero exit code.
//! This is by design to allow `cradle` to be used in contexts
//! where more complex error handling is not needed or desired,
//! for example in scripts.
//!
//! ``` should_panic
//! use cradle::prelude::*;
//!
//! // panics with "false:\n  exited with exit code: 1"
//! run!("false");
//! ```
//!
//! For a full list of reasons why [`run!`] and [`run_output!`] may panic,
//! see the documentation of `cradle`'s [`Error`] type.
//!
//! ## Preventing Panics
//!
//! You can also turn **all** panics into [`std::result::Result::Err`]s
//! by using [`run_result!`]. This will return a value of type
//! [`Result<T, cradle::Error>`], where
//! `T` is any type that implements [`output::Output`].
//! Here's some examples:
//!
//! ```
//! use cradle::prelude::*;
//!
//! let result: Result<(), cradle::Error> = run_result!("false");
//! let error_message = format!("{}", result.unwrap_err());
//! assert_eq!(
//!     error_message,
//!     "false:\n  exited with exit code: 1"
//! );
//!
//! let result = run_result!(%"echo foo");
//! let StdoutTrimmed(output) = result.unwrap();
//! assert_eq!(output, "foo".to_string());
//! ```
//!
//! [`run_result!`] can also be combined with `?` to handle errors in an
//! idiomatic way, for example:
//!
//! ```
//! use cradle::prelude::*;
//!
//! fn build() -> Result<(), Error> {
//!     run_result!(%"which make")?;
//!     run_result!(%"which gcc")?;
//!     run_result!(%"which ld")?;
//!     run_result!(%"make build")?;
//!     Ok(())
//! }
//! ```
//!
//! If you don't want to prevent **all** panics,
//! but just panics caused by non-zero exit codes,
//! you can use [`Status`].
//!
//! # Alternative Interface: Methods on [`input::Input`]
//!
//! `cradle` also provides an alternative interface to execute commands
//! through methods on the [`Input`] trait:
//! [`.run()`](Input::run), [`.run_output()`](Input::run_output)
//! and [`.run_result()`](Input::run_result).
//! These methods can be invoked on all values whose types implement
//! [`Input`].
//! When using these methods, it's especially useful that
//! [`Input`] is implemented by tuples.
//! They work analog to [`run!`], [`run_output!`] and [`run_result!`].
//! Here are some examples:
//!
//! ```
//! # let temp_dir = tempfile::TempDir::new().unwrap();
//! # std::env::set_current_dir(&temp_dir).unwrap();
//! use cradle::prelude::*;
//!
//! ("touch", "foo").run();
//!
//! let StdoutTrimmed(output) = ("echo", "foo").run_output();
//! assert_eq!(output, "foo");
//!
//! let result: Result<(), cradle::Error> = "false".run_result();
//! let error_message = format!("{}", result.unwrap_err());
//! assert_eq!(
//!     error_message,
//!     "false:\n  exited with exit code: 1"
//! );
//! ```
//!
//! Note: The `%` shortcut for [`Split`] is not available in this notation.
//! You can either use tuples, or [`Split`] explicitly:
//!
//! ```
//! use cradle::prelude::*;
//!
//! ("echo", "foo").run();
//! Split("echo foo").run();
//! ```
//!
//! # Prior Art
//!
//! `cradle` is heavily inspired by [shake](https://shakebuild.com/),
//! specifically by its
//! [`cmd`](https://hackage.haskell.org/package/shake-0.19.4/docs/Development-Shake.html#v:cmd)
//! function.

pub mod child_output;
mod collected_output;
pub mod config;
mod context;
pub mod error;
pub mod input;
mod macros;
pub mod output;
pub mod prelude;

include!("common_re_exports.rs.snippet");

#[cfg(test)]
mod tests {
    use crate::{
        context::Context,
        input::{run_result_with_context, run_result_with_context_unit},
        prelude::*,
    };
    use lazy_static::lazy_static;
    use std::{
        collections::BTreeSet,
        env::{current_dir, set_current_dir},
        ffi::{OsStr, OsString},
        fs,
        io::Write,
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
        let mut set = match BUILT.lock() {
            Ok(set) => set,
            Err(error) => {
                let _ = write!(
                    std::io::stderr(),
                    "test_executable: BUILT poisoned: {}",
                    error
                );
                let _ = std::io::stderr().flush();
                std::process::exit(1)
            }
        };
        if !set.contains(name) {
            set.insert(name.to_owned());
            run!(
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

    #[test]
    fn allows_to_execute_a_command() {
        in_temporary_directory(|| {
            run!(%"touch foo");
            assert!(PathBuf::from("foo").exists());
        })
    }

    mod errors {
        use super::*;

        mod panics_by_default {
            use super::*;

            #[test]
            #[should_panic(expected = "cradle error: false:\n  exited with exit code: 1")]
            fn non_zero_exit_codes() {
                run!("false");
            }

            #[test]
            #[should_panic(expected = "cradle error: false:\n  exited with exit code: 1")]
            fn combine_panics_with_other_outputs() {
                let StdoutTrimmed(_) = run_output!("false");
            }

            #[test]
            #[should_panic(expected = "cradle error: false foo bar:\n  exited with exit code: 1")]
            fn includes_full_command_on_non_zero_exit_codes() {
                run!(%"false foo bar");
            }

            #[test]
            #[should_panic(expected = "exited with exit code: 42")]
            fn other_exit_codes() {
                run!(test_helper(), "exit code 42");
            }

            #[test]
            #[should_panic(
                expected = "cradle error: File not found error when executing 'does-not-exist'"
            )]
            fn executable_cannot_be_found() {
                run!("does-not-exist");
            }

            #[test]
            #[cfg(unix)]
            #[should_panic(expected = "/file foo bar:\n  Permission denied (os error 13)")]
            fn includes_full_command_on_io_errors() {
                let temp_dir = TempDir::new().unwrap();
                let without_executable_bit = temp_dir.path().join("file");
                fs::write(&without_executable_bit, "").unwrap();
                run!(without_executable_bit, %"foo bar");
            }

            #[rustversion::since(1.46)]
            #[test]
            fn includes_source_location_of_run_run_call() {
                let (Status(_), Stderr(stderr)) =
                    run_output!(test_executable("test_executables_panic"));
                let expected = "src/test_executables/panic.rs:4:5";
                assert!(
                    stderr.contains(expected),
                    "{:?}\n  does not contain\n{:?}",
                    stderr,
                    expected
                );
            }

            #[test]
            #[should_panic(expected = "cradle error: no arguments given")]
            fn no_executable() {
                let vector: Vec<String> = Vec::new();
                run!(vector);
            }

            #[test]
            #[should_panic(expected = "invalid utf-8 written to stdout")]
            fn invalid_utf8_stdout() {
                let StdoutTrimmed(_) = run_output!(test_helper(), "invalid utf-8 stdout");
            }

            #[test]
            #[cfg(not(windows))]
            fn invalid_utf8_to_stdout_is_allowed_when_not_captured() {
                run!(test_helper(), "invalid utf-8 stdout");
            }
        }

        mod result_types {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn non_zero_exit_codes() {
                let result: Result<(), Error> = run_result!("false");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn no_errors() {
                let result: Result<(), Error> = run_result!("true");
                result.unwrap();
            }

            #[test]
            fn combine_ok_with_other_outputs() {
                let StdoutTrimmed(output) = run_result!(%"echo foo").unwrap();
                assert_eq!(output, "foo".to_string());
            }

            #[test]
            fn combine_err_with_other_outputs() {
                let result: Result<StdoutTrimmed, Error> = run_result!("false");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "false:\n  exited with exit code: 1"
                );
            }

            #[test]
            fn includes_full_command_on_non_zero_exit_codes() {
                let result: Result<(), Error> = run_result!(%"false foo bar");
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
                        run_result!(%"./without-executable-bit foo bar");
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
                let result: Result<(), Error> = run_result!(test_helper(), "exit code 42");
                assert!(result
                    .unwrap_err()
                    .to_string()
                    .contains("exited with exit code: 42"));
            }

            #[test]
            fn missing_executable_file_error_message() {
                let result: Result<(), Error> = run_result!("does-not-exist");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "File not found error when executing 'does-not-exist'"
                );
            }

            #[test]
            fn missing_executable_file_error_can_be_matched_against() {
                let result: Result<(), Error> = run_result!("does-not-exist");
                match result {
                    Err(Error::FileNotFound { executable, .. }) => {
                        assert_eq!(executable, "does-not-exist");
                    }
                    _ => panic!("should match Error::FileNotFound"),
                }
            }

            #[test]
            fn missing_executable_file_error_can_be_caused_by_relative_paths() {
                let result: Result<(), Error> = run_result!("./does-not-exist");
                match result {
                    Err(Error::FileNotFound { executable, .. }) => {
                        assert_eq!(executable, "./does-not-exist");
                    }
                    _ => panic!("should match Error::FileNotFound"),
                }
            }

            #[test]
            fn no_executable() {
                let vector: Vec<String> = Vec::new();
                let result: Result<(), Error> = run_result!(vector);
                assert_eq!(result.unwrap_err().to_string(), "no arguments given");
            }

            #[test]
            fn invalid_utf8_stdout() {
                let test_helper = test_helper();
                let result: Result<StdoutTrimmed, Error> =
                    run_result!(&test_helper, "invalid utf-8 stdout");
                assert_eq!(
                    result.unwrap_err().to_string(),
                    format!(
                        "{} 'invalid utf-8 stdout':\n  invalid utf-8 written to stdout",
                        test_helper.display()
                    )
                );
            }
        }

        mod whitespace_in_executable_note {
            use super::*;
            use pretty_assertions::assert_eq;
            use unindent::Unindent;

            #[test]
            fn missing_executable_file_with_whitespace_includes_note() {
                let result: Result<(), Error> = run_result!("does not exist");
                let expected = "
                    File not found error when executing 'does not exist'
                    note: Given executable name 'does not exist' contains whitespace.
                      Did you mean to run 'does', with 'not' and 'exist' as arguments?
                      Consider using Split: https://docs.rs/cradle/latest/cradle/input/struct.Split.html
                "
                .unindent()
                .trim()
                .to_string();
                assert_eq!(result.unwrap_err().to_string(), expected);
            }

            #[test]
            fn single_argument() {
                let result: Result<(), Error> = run_result!("foo bar");
                let expected = "
                    File not found error when executing 'foo bar'
                    note: Given executable name 'foo bar' contains whitespace.
                      Did you mean to run 'foo', with 'bar' as the argument?
                      Consider using Split: https://docs.rs/cradle/latest/cradle/input/struct.Split.html
                "
                .unindent()
                .trim()
                .to_string();
                assert_eq!(result.unwrap_err().to_string(), expected);
            }
        }
    }

    #[test]
    fn allows_to_retrieve_stdout() {
        let StdoutTrimmed(stdout) = run_output!(%"echo foo");
        assert_eq!(stdout, "foo");
    }

    #[test]
    fn command_and_argument_as_separate_ref_str() {
        let StdoutTrimmed(stdout) = run_output!("echo", "foo");
        assert_eq!(stdout, "foo");
    }

    #[test]
    fn multiple_arguments_as_ref_str() {
        let StdoutTrimmed(stdout) = run_output!("echo", "foo", "bar");
        assert_eq!(stdout, "foo bar");
    }

    #[test]
    fn arguments_can_be_given_as_references() {
        let reference: &LogCommand = &LogCommand;
        let executable: &String = &"echo".to_string();
        let argument: &String = &"foo".to_string();
        let StdoutTrimmed(stdout) = run_output!(reference, executable, argument);
        assert_eq!(stdout, "foo");
    }

    mod sequences {
        use super::*;

        #[test]
        fn allows_to_pass_in_arguments_as_a_vec_of_ref_str() {
            let args: Vec<&str> = vec!["foo"];
            let StdoutTrimmed(stdout) = run_output!("echo", args);
            assert_eq!(stdout, "foo");
        }

        #[test]
        fn vector_of_non_strings() {
            let context = Context::test();
            let log_commands: Vec<LogCommand> = vec![LogCommand];
            let StdoutTrimmed(stdout) =
                run_result_with_context(context.clone(), (log_commands, Split("echo foo")))
                    .unwrap();
            assert_eq!(stdout, "foo");
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[rustversion::since(1.51)]
        #[test]
        fn arrays_as_arguments() {
            let args: [&str; 2] = ["echo", "foo"];
            let StdoutTrimmed(stdout) = run_output!(args);
            assert_eq!(stdout, "foo");
        }

        #[rustversion::since(1.51)]
        #[test]
        fn arrays_of_non_strings() {
            let context = Context::test();
            let log_commands: [LogCommand; 1] = [LogCommand];
            let StdoutTrimmed(stdout) =
                run_result_with_context(context.clone(), (log_commands, Split("echo foo")))
                    .unwrap();
            assert_eq!(stdout, "foo");
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[rustversion::since(1.51)]
        #[test]
        fn elements_in_arrays_are_not_split_by_whitespace() {
            in_temporary_directory(|| {
                let args: [&str; 1] = ["foo bar"];
                run!("touch", args);
                assert!(PathBuf::from("foo bar").exists());
            });
        }

        #[rustversion::since(1.51)]
        #[test]
        fn array_refs_as_arguments() {
            let args: &[&str; 2] = &["echo", "foo"];
            let StdoutTrimmed(stdout) = run_output!(args);
            assert_eq!(stdout, "foo");
        }

        #[rustversion::since(1.51)]
        #[test]
        fn elements_in_array_refs_are_not_split_by_whitespace() {
            in_temporary_directory(|| {
                let args: &[&str; 1] = &["foo bar"];
                run!("touch", args);
                assert!(PathBuf::from("foo bar").exists());
            });
        }

        #[test]
        fn slices_as_arguments() {
            let args: &[&str] = &["echo", "foo"];
            let StdoutTrimmed(stdout) = run_output!(args);
            assert_eq!(stdout, "foo");
        }

        #[test]
        fn slices_of_non_strings() {
            let context = Context::test();
            let log_commands: &[LogCommand] = &[LogCommand];
            let StdoutTrimmed(stdout) =
                run_result_with_context(context.clone(), (log_commands, Split("echo foo")))
                    .unwrap();
            assert_eq!(stdout, "foo");
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[test]
        fn elements_in_slices_are_not_split_by_whitespace() {
            in_temporary_directory(|| {
                let args: &[&str] = &["foo bar"];
                run!("touch", args);
                assert!(PathBuf::from("foo bar").exists());
            });
        }

        #[test]
        fn vector_of_vectors() {
            let StdoutTrimmed(output) = run_output!(vec![vec!["echo"], vec!["foo", "bar"]]);
            assert_eq!(output, "foo bar");
        }
    }

    mod strings {
        use super::*;

        #[test]
        fn works_for_string() {
            let command: String = "true".to_string();
            run!(command);
        }

        #[test]
        fn multiple_strings() {
            let command: String = "echo".to_string();
            let argument: String = "foo".to_string();
            let StdoutTrimmed(output) = run_output!(command, argument);
            assert_eq!(output, "foo");
        }

        #[test]
        fn mix_ref_str_and_string() {
            let argument: String = "foo".to_string();
            let StdoutTrimmed(output) = run_output!("echo", argument);
            assert_eq!(output, "foo");
        }

        #[test]
        fn does_not_split_strings_in_vectors() {
            in_temporary_directory(|| {
                let argument: Vec<String> = vec!["filename with spaces".to_string()];
                run!("touch", argument);
                assert!(PathBuf::from("filename with spaces").exists());
            });
        }
    }

    mod os_strings {
        use super::*;

        #[test]
        fn works_for_os_string() {
            run!(OsString::from("true"));
        }

        #[test]
        fn works_for_os_str() {
            run!(OsStr::new("true"));
        }
    }

    mod stdout {
        use super::*;
        use std::{thread, time::Duration};

        #[test]
        fn relays_stdout_by_default() {
            let context = Context::test();
            run_result_with_context_unit(context.clone(), Split("echo foo")).unwrap();
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn relays_stdout_for_non_zero_exit_codes() {
            let context = Context::test();
            let _ = run_result_with_context_unit(
                context.clone(),
                (test_helper(), "output foo and exit with 42"),
            );
            assert_eq!(context.stdout(), "foo\n");
        }

        #[test]
        fn streams_stdout() {
            in_temporary_directory(|| {
                let context = Context::test();
                let context_clone = context.clone();
                let thread = thread::spawn(|| {
                    run_result_with_context_unit(
                        context_clone,
                        (test_helper(), "stream chunk then wait for file"),
                    )
                    .unwrap();
                });
                while (context.stdout()) != "foo\n" {
                    thread::sleep(Duration::from_secs_f32(0.05));
                }
                run!(%"touch file");
                thread.join().unwrap();
            });
        }

        #[test]
        fn does_not_relay_stdout_when_collecting_into_string() {
            let context = Context::test();
            let StdoutTrimmed(_) =
                run_result_with_context(context.clone(), Split("echo foo")).unwrap();
            assert_eq!(context.stdout(), "");
        }

        #[test]
        fn does_not_relay_stdout_when_collecting_into_result_of_string() {
            let context = Context::test();
            let _: Result<StdoutTrimmed, Error> =
                run_result_with_context(context.clone(), Split("echo foo"));
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
            run_result_with_context_unit(context.clone(), (test_helper(), "write to stderr"))
                .unwrap();
            assert_eq!(context.stderr(), "foo\n");
        }

        #[test]
        fn relays_stderr_for_non_zero_exit_codes() {
            let context = Context::test();
            let _: Result<(), Error> = run_result_with_context(
                context.clone(),
                (test_helper(), "write to stderr and exit with 42"),
            );
            assert_eq!(context.stderr(), "foo\n");
        }

        #[test]
        fn streams_stderr() {
            in_temporary_directory(|| {
                let context = Context::test();
                let context_clone = context.clone();
                let thread = thread::spawn(|| {
                    run_result_with_context_unit(
                        context_clone,
                        (test_helper(), "stream chunk to stderr then wait for file"),
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
                run!(%"touch file");
                thread.join().unwrap();
            });
        }

        #[test]
        fn capture_stderr() {
            let Stderr(stderr) = run_output!(test_helper(), "write to stderr");
            assert_eq!(stderr, "foo\n");
        }

        #[test]
        fn assumes_stderr_is_utf_8() {
            let result: Result<Stderr, Error> = run_result!(test_helper(), "invalid utf-8 stderr");
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
            run!(test_helper(), "invalid utf-8 stderr");
        }

        #[test]
        fn does_not_relay_stderr_when_catpuring() {
            let context = Context::test();
            let Stderr(_) =
                run_result_with_context(context.clone(), (test_helper(), "write to stderr"))
                    .unwrap();
            assert_eq!(context.stderr(), "");
        }
    }

    mod log_commands {
        use super::*;

        #[test]
        fn logs_simple_commands() {
            let context = Context::test();
            run_result_with_context_unit(context.clone(), (LogCommand, "true")).unwrap();
            assert_eq!(context.stderr(), "+ true\n");
        }

        #[test]
        fn logs_commands_with_arguments() {
            let context = Context::test();
            run_result_with_context_unit(context.clone(), (LogCommand, Split("echo foo"))).unwrap();
            assert_eq!(context.stderr(), "+ echo foo\n");
        }

        #[test]
        fn quotes_arguments_with_spaces() {
            let context = Context::test();
            run_result_with_context_unit(context.clone(), (LogCommand, "echo", "foo bar")).unwrap();
            assert_eq!(context.stderr(), "+ echo 'foo bar'\n");
        }

        #[test]
        fn quotes_empty_arguments() {
            let context = Context::test();
            run_result_with_context_unit(context.clone(), (LogCommand, "echo", "")).unwrap();
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
            run_result_with_context_unit(
                context.clone(),
                (LogCommand, "echo", argument_with_invalid_utf8),
            )
            .unwrap();
            assert_eq!(context.stderr(), "+ echo foo�bar\n");
        }
    }

    mod exit_status {
        use super::*;

        #[test]
        fn zero() {
            let Status(exit_status) = run_output!("true");
            assert!(exit_status.success());
        }

        #[test]
        fn one() {
            let Status(exit_status) = run_output!("false");
            assert!(!exit_status.success());
        }

        #[test]
        fn forty_two() {
            let Status(exit_status) = run_output!(test_helper(), "exit code 42");
            assert!(!exit_status.success());
            assert_eq!(exit_status.code(), Some(42));
        }

        #[test]
        fn failing_commands_return_oks_when_exit_status_is_captured() {
            let Status(exit_status) = run_result!("false").unwrap();
            assert!(!exit_status.success());
        }
    }

    mod bool_output {
        use super::*;

        #[test]
        fn success_exit_status_is_true() {
            assert!(run_output!("true"));
        }

        #[test]
        fn failure_exit_status_is_false() {
            assert!(!run_output!("false"));
        }

        #[test]
        #[should_panic]
        fn io_error_panics() {
            assert!(run_output!("/"));
        }
    }

    mod tuple_inputs {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn two_tuple() {
            let StdoutTrimmed(output) = run_output!(("echo", "foo"));
            assert_eq!(output, "foo");
        }

        #[test]
        fn three_tuples() {
            let StdoutTrimmed(output) = run_output!(("echo", "foo", "bar"));
            assert_eq!(output, "foo bar");
        }

        #[test]
        fn nested_tuples() {
            let StdoutTrimmed(output) = run_output!(("echo", ("foo", "bar")));
            assert_eq!(output, "foo bar");
        }

        #[test]
        fn unit_input() {
            let StdoutTrimmed(output) = run_output!(("echo", ()));
            assert_eq!(output, "");
        }
    }

    mod tuple_outputs {
        use super::*;

        #[test]
        fn two_tuple_1() {
            let (StdoutTrimmed(output), Status(exit_status)) =
                run_output!(test_helper(), "output foo and exit with 42");
            assert_eq!(output, "foo");
            assert_eq!(exit_status.code(), Some(42));
        }

        #[test]
        fn two_tuple_2() {
            let (Status(exit_status), StdoutTrimmed(output)) =
                run_output!(test_helper(), "output foo and exit with 42");
            assert_eq!(output, "foo");
            assert_eq!(exit_status.code(), Some(42));
        }

        #[test]
        fn result_of_tuple() {
            let (StdoutTrimmed(output), Status(exit_status)) = run_result!(%"echo foo").unwrap();
            assert_eq!(output, "foo");
            assert!(exit_status.success());
        }

        #[test]
        fn result_of_tuple_when_erroring() {
            let (StdoutTrimmed(output), Status(exit_status)) = run_result!("false").unwrap();
            assert_eq!(output, "");
            assert_eq!(exit_status.code(), Some(1));
        }

        #[test]
        fn three_tuples() {
            let (Stderr(stderr), StdoutTrimmed(stdout), Status(exit_status)) =
                run_output!(%"echo foo");
            assert_eq!(stderr, "");
            assert_eq!(stdout, "foo");
            assert_eq!(exit_status.code(), Some(0));
        }

        #[test]
        fn capturing_stdout_on_errors() {
            let (StdoutTrimmed(output), Status(exit_status)) =
                run_output!(test_helper(), "output foo and exit with 42");
            assert!(!exit_status.success());
            assert_eq!(output, "foo");
        }

        #[test]
        fn capturing_stderr_on_errors() {
            let (Stderr(output), Status(exit_status)) =
                run_output!(test_helper(), "write to stderr and exit with 42");
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
                let StdoutUntrimmed(output) = run_output!(%"cat file", CurrentDir("dir"));
                assert_eq!(output, "foo");
            });
        }

        #[test]
        fn works_for_other_types() {
            in_temporary_directory(|| {
                fs::create_dir("dir").unwrap();
                let dir: String = "dir".to_string();
                run!("true", CurrentDir(dir));
                let dir: PathBuf = PathBuf::from("dir");
                run!("true", CurrentDir(dir));
                let dir: &Path = Path::new("dir");
                run!("true", CurrentDir(dir));
            });
        }
    }

    mod capturing_stdout {
        use super::*;

        mod trimmed {
            use super::*;

            #[test]
            fn trims_trailing_whitespace() {
                let StdoutTrimmed(output) = run_output!(%"echo foo");
                assert_eq!(output, "foo");
            }

            #[test]
            fn trims_leading_whitespace() {
                let StdoutTrimmed(output) = run_output!(%"echo -n", " foo");
                assert_eq!(output, "foo");
            }

            #[test]
            fn does_not_remove_whitespace_within_output() {
                let StdoutTrimmed(output) = run_output!(%"echo -n", "foo bar");
                assert_eq!(output, "foo bar");
            }

            #[test]
            fn does_not_modify_output_without_whitespace() {
                let StdoutTrimmed(output) = run_output!(%"echo -n", "foo");
                assert_eq!(output, "foo");
            }

            #[test]
            fn does_not_relay_stdout() {
                let context = Context::test();
                let StdoutTrimmed(_) =
                    run_result_with_context(context.clone(), Split("echo foo")).unwrap();
                assert_eq!(context.stdout(), "");
            }
        }

        mod untrimmed {
            use super::*;

            #[test]
            fn does_not_trim_trailing_newline() {
                let StdoutUntrimmed(output) = run_output!(%"echo foo");
                assert_eq!(output, "foo\n");
            }

            #[test]
            fn does_not_trim_leading_whitespace() {
                let StdoutUntrimmed(output) = run_output!(%"echo -n", " foo");
                assert_eq!(output, " foo");
            }

            #[test]
            fn does_not_relay_stdout() {
                let context = Context::test();
                let StdoutUntrimmed(_) =
                    run_result_with_context(context.clone(), Split("echo foo")).unwrap();
                assert_eq!(context.stdout(), "");
            }
        }
    }

    mod split {
        use super::*;

        #[test]
        fn splits_words_by_whitespace() {
            let StdoutTrimmed(output) = run_output!(Split("echo foo"));
            assert_eq!(output, "foo");
        }

        #[test]
        fn skips_multiple_whitespace_characters() {
            let StdoutUntrimmed(output) = run_output!("echo", Split("foo  bar"));
            assert_eq!(output, "foo bar\n");
        }

        #[test]
        fn trims_leading_whitespace() {
            let StdoutTrimmed(output) = run_output!(Split(" echo foo"));
            assert_eq!(output, "foo");
        }

        #[test]
        fn trims_trailing_whitespace() {
            let StdoutUntrimmed(output) = run_output!("echo", Split("foo "));
            assert_eq!(output, "foo\n");
        }

        mod percent_sign {
            use super::*;

            #[test]
            fn splits_words() {
                let StdoutUntrimmed(output) = run_output!(%"echo foo");
                assert_eq!(output, "foo\n");
            }

            #[test]
            fn works_for_later_arguments() {
                let StdoutUntrimmed(output) = run_output!("echo", %"foo\tbar");
                assert_eq!(output, "foo bar\n");
            }

            #[test]
            fn for_first_of_multiple_arguments() {
                let StdoutUntrimmed(output) = run_output!(%"echo foo", "bar");
                assert_eq!(output, "foo bar\n");
            }

            #[test]
            fn non_literals() {
                let command = "echo foo";
                let StdoutUntrimmed(output) = run_output!(%command);
                assert_eq!(output, "foo\n");
            }

            #[test]
            fn in_run() {
                run!(%"echo foo");
            }

            #[test]
            fn in_run_result() {
                let StdoutTrimmed(_) = run_result!(%"echo foo").unwrap();
            }
        }
    }

    mod splitting_with_library_functions {
        use super::*;

        #[test]
        fn allow_to_use_split() {
            let StdoutTrimmed(output) = run_output!("echo foo".split(' '));
            assert_eq!(output, "foo");
        }

        #[test]
        fn split_whitespace() {
            let StdoutTrimmed(output) = run_output!("echo foo".split_whitespace());
            assert_eq!(output, "foo");
        }

        #[test]
        fn split_ascii_whitespace() {
            let StdoutTrimmed(output) = run_output!("echo foo".split_ascii_whitespace());
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
                run!(%"chmod +x test-script");
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
                let StdoutUntrimmed(output) = run_output!("cat", file);
                assert_eq!(output, "test-contents");
            })
        }

        #[test]
        fn ref_path_as_executable() {
            in_temporary_directory(|| {
                let file: &Path = &write_test_script();
                let StdoutTrimmed(output) = run_output!(file);
                assert_eq!(output, "test-output");
            })
        }

        #[test]
        fn path_buf_as_argument() {
            in_temporary_directory(|| {
                let file: PathBuf = PathBuf::from("file");
                fs::write(&file, "test-contents").unwrap();
                let StdoutUntrimmed(output) = run_output!("cat", file);
                assert_eq!(output, "test-contents");
            })
        }

        #[test]
        fn path_buf_as_executable() {
            in_temporary_directory(|| {
                let file: PathBuf = write_test_script();
                let StdoutTrimmed(output) = run_output!(file);
                assert_eq!(output, "test-output");
            })
        }
    }

    mod stdin {
        use super::*;

        #[test]
        fn allows_to_pass_in_strings_as_stdin() {
            let StdoutUntrimmed(output) = run_output!(test_helper(), "reverse", Stdin("foo"));
            assert_eq!(output, "oof");
        }

        #[test]
        fn allows_passing_in_u8_slices_as_stdin() {
            let StdoutUntrimmed(output) = run_output!(test_helper(), "reverse", Stdin(&[0, 1, 2]));
            assert_eq!(output, "\x02\x01\x00");
        }

        #[test]
        #[cfg(unix)]
        fn stdin_is_closed_by_default() {
            let StdoutTrimmed(output) = run_output!(test_helper(), "wait until stdin is closed");
            assert_eq!(output, "stdin is closed");
        }

        #[test]
        #[cfg(unix)]
        fn writing_too_many_bytes_into_a_non_reading_child_may_error() {
            let big_string = String::from_utf8(vec![b'a'; 2_usize.pow(16) + 1]).unwrap();
            let result: Result<(), crate::Error> = run_result!("true", Stdin(big_string));
            dbg!(&result);
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
                run_output!(test_helper(), "reverse", Stdin("foo"), Stdin("bar"));
            assert_eq!(output, "raboof");
        }

        #[test]
        fn works_for_owned_strings() {
            let argument: String = "foo".to_string();
            let StdoutUntrimmed(output) = run_output!(test_helper(), "reverse", Stdin(argument));
            assert_eq!(output, "oof");
        }
    }

    mod invocation_syntax {
        use super::*;

        #[test]
        fn trailing_comma_is_accepted_after_normal_argument() {
            run!("echo", "foo",);
            let StdoutUntrimmed(_) = run_output!("echo", "foo",);
            let _result: Result<(), Error> = run_result!("echo", "foo",);
        }

        #[test]
        fn trailing_comma_is_accepted_after_split_argument() {
            run!("echo", %"foo",);
            let StdoutUntrimmed(_) = run_output!("echo", %"foo",);
            let _result: Result<(), Error> = run_result!("echo", %"foo",);
        }
    }

    mod environment_variables {
        use super::*;
        use pretty_assertions::assert_eq;
        use std::env;

        #[test]
        fn allows_to_add_variables() {
            let StdoutTrimmed(output) = run_output!(
                test_helper(),
                %"echo FOO",
                Env("FOO", "bar")
            );
            assert_eq!(output, "bar");
        }

        #[test]
        fn works_for_multiple_variables() {
            let StdoutUntrimmed(output) = run_output!(
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
            let StdoutTrimmed(output) = run_output!(test_helper(), "echo", unused_key);
            assert_eq!(output, "foo");
        }

        #[test]
        fn overwrites_existing_parent_variables() {
            let unused_key = find_unused_environment_variable();
            env::set_var(&unused_key, "foo");
            let StdoutTrimmed(output) =
                run_output!(test_helper(), "echo", &unused_key, Env(&unused_key, "bar"));
            assert_eq!(output, "bar");
        }

        #[test]
        fn variables_are_overwritten_by_subsequent_variables_with_the_same_name() {
            let StdoutTrimmed(output) = run_output!(
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
            let StdoutUntrimmed(output) =
                run_output!(test_helper(), "echo", "FOO", Env("FOO", ""),);
            assert_eq!(output, "empty variable: FOO\n");
        }
    }

    mod run_interface {
        use super::*;
        use std::path::Path;

        #[test]
        fn allows_to_run_commands_with_dot_run() {
            let StdoutTrimmed(output) = Split("echo foo").run_output();
            assert_eq!(output, "foo");
        }

        #[test]
        fn allows_to_bundle_arguments_up_in_tuples() {
            let StdoutTrimmed(output) = ("echo", "foo").run_output();
            assert_eq!(output, "foo");
        }

        #[test]
        fn works_for_different_output_types() {
            let Status(status) = "false".run_output();
            assert!(!status.success());
        }

        #[test]
        fn run() {
            in_temporary_directory(|| {
                ("touch", "foo").run();
                assert!(Path::new("foo").exists());
            });
        }

        #[test]
        fn run_result() {
            let StdoutTrimmed(output) = ("echo", "foo").run_result().unwrap();
            assert_eq!(output, "foo");
            let result: Result<(), Error> = "does-not-exist".run_result();
            match result {
                Err(Error::FileNotFound { .. }) => {}
                _ => panic!("should match Error::FileNotFound"),
            }
        }
    }
}
