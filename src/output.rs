//! The [`Output`] trait that defines all possible outputs of a child process.

use crate::{child_output::ChildOutput, config::Config, error::Error};
use std::process::ExitStatus;

/// All possible return types of [`run!`], [`run_output!`] or
/// [`run_result!`] must implement this trait.
/// This return-type polymorphism makes cradle very flexible.
/// For example, if you want to capture what a command writes
/// to `stdout` you can do that using [`StdoutUntrimmed`]:
///
/// ```
/// use cradle::prelude::*;
///
/// let StdoutUntrimmed(output) = run_output!(%"echo foo");
/// assert_eq!(output, "foo\n");
/// ```
///
/// But if instead you want to capture the command's [`ExitStatus`],
/// you can use [`Status`]:
///
/// ```
/// use cradle::prelude::*;
///
/// let Status(exit_status) = run_output!("false");
/// assert_eq!(exit_status.code(), Some(1));
/// ```
///
/// For documentation on what all the possible return types do,
/// see the documentation for the individual impls of [`Output`].
/// Here's a non-exhaustive list of the more commonly used return types to get you started:
///
/// - [`()`]: In case you don't want to capture anything. See also [`run`].
/// - To capture output streams:
///   - [`StdoutTrimmed`]: To capture `stdout`, trimmed of whitespace.
///   - [`StdoutUntrimmed`]: To capture `stdout` untrimmed.
///   - [`Stderr`]: To capture `stderr`.
/// - [`Status`]: To capture the command's [`ExitStatus`].
///
/// Also, [`Output`] is implemented for tuples.
/// You can use this to combine multiple return types that implement [`Output`].
/// The following code for example retrieves the command's [`ExitStatus`]
/// _and_ what it writes to `stdout`:
///
/// ```
/// use cradle::prelude::*;
///
/// let (Status(exit_status), StdoutUntrimmed(stdout)) = run_output!(%"echo foo");
/// assert!(exit_status.success());
/// assert_eq!(stdout, "foo\n");
/// ```
///
/// [`()`]: trait.Output.html#impl-Output-for-()
///
/// ## Custom [`Output`] impls
///
/// It is possible, but not recommended, to write `Output` implementations for your
/// own types. The API is inconvenient, under-documented, and easy to misuse, i.e
/// easy to provoke [`Internal`](Error::Internal) errors.
///
/// See
/// [Issue 184: Provide a better API for writing custom Output impls](https://github.com/soenkehahn/cradle/issues/184)
/// for more details and discussion.
pub trait Output: Sized {
    fn configure(config: &mut Config);

    fn from_child_output(config: &Config, result: &ChildOutput) -> Result<Self, Error>;
}

/// Use this when you don't need any result from the child process.
///
/// ```
/// # let temp_dir = tempfile::TempDir::new().unwrap();
/// # std::env::set_current_dir(&temp_dir).unwrap();
/// use cradle::prelude::*;
///
/// let () = run_output!(%"touch ./foo");
/// ```
///
/// Since [`run_output!`] (and [`run_result`]) use return type polymorphism,
/// you have to make sure the compiler can figure out which return type you want to use.
/// In this example that happens through the `let () =`.
/// So you can't just omit that.
///
/// See also [`run!`] for a more convenient way to use `()` as the return type.
impl Output for () {
    #[doc(hidden)]
    fn configure(_config: &mut Config) {}

    #[doc(hidden)]
    fn from_child_output(_config: &Config, _child_output: &ChildOutput) -> Result<Self, Error> {
        Ok(())
    }
}

macro_rules! tuple_impl {
    ($($generics:ident,)+) => {
        impl<$($generics),+> Output for ($($generics,)+)
        where
            $($generics: Output,)+
        {
            #[doc(hidden)]
            fn configure(config: &mut Config) {
                $(<$generics as Output>::configure(config);)+
            }

            #[doc(hidden)]
            fn from_child_output(config: &Config, child_output: &ChildOutput) -> Result<Self, Error> {
                Ok((
                    $(<$generics as Output>::from_child_output(config, child_output)?,)+
                ))
            }
        }
    };
}

tuple_impl!(A,);
tuple_impl!(A, B,);
tuple_impl!(A, B, C,);
tuple_impl!(A, B, C, D,);
tuple_impl!(A, B, C, D, E,);
tuple_impl!(A, B, C, D, E, F,);

/// Returns what the child process writes to `stdout`, interpreted as utf-8,
/// collected into a string, trimmed of leading and trailing whitespace.
/// This also suppresses output of the child's `stdout`
/// to the parent's `stdout`. (Which would be the default when not using [`StdoutTrimmed`]
/// as the return value.)
///
/// It's recommended to pattern-match to get to the inner [`String`].
/// This will make sure that the return type can be inferred.
/// Here's an example:
///
/// ```
/// use std::path::Path;
/// use cradle::prelude::*;
///
/// # #[cfg(unix)]
/// # {
/// let StdoutTrimmed(output) = run_output!(%"which ls");
/// assert!(Path::new(&output).exists());
/// # }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct StdoutTrimmed(pub String);

impl Output for StdoutTrimmed {
    #[doc(hidden)]
    fn configure(config: &mut Config) {
        StdoutUntrimmed::configure(config);
    }

    #[doc(hidden)]
    fn from_child_output(config: &Config, child_output: &ChildOutput) -> Result<Self, Error> {
        let StdoutUntrimmed(stdout) = StdoutUntrimmed::from_child_output(config, child_output)?;
        Ok(StdoutTrimmed(stdout.trim().to_owned()))
    }
}

/// Same as [`StdoutTrimmed`], but does not trim whitespace from the output:
///
/// ```
/// use cradle::prelude::*;
///
/// let StdoutUntrimmed(output) = run_output!(%"echo foo");
/// assert_eq!(output, "foo\n");
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct StdoutUntrimmed(pub String);

impl Output for StdoutUntrimmed {
    #[doc(hidden)]
    fn configure(config: &mut Config) {
        config.capture_stdout = true;
    }

    #[doc(hidden)]
    fn from_child_output(config: &Config, child_output: &ChildOutput) -> Result<Self, Error> {
        let stdout = child_output
            .stdout
            .clone()
            .ok_or_else(|| Error::internal("stdout not captured", config))?;
        Ok(StdoutUntrimmed(String::from_utf8(stdout).map_err(
            |source| Error::InvalidUtf8ToStdout {
                full_command: config.full_command(),
                source,
            },
        )?))
    }
}

/// [`Stderr`] allows to capture the `stderr` of a child process:
///
/// ```
/// use cradle::prelude::*;
///
/// // (`Status` is used here to suppress panics caused by `ls`
/// // terminating with a non-zero exit code.)
/// let (Stderr(stderr), Status(_)) = run_output!(%"ls does-not-exist");
/// assert!(stderr.contains("No such file or directory"));
/// ```
///
/// This assumes that the output written to `stderr` is encoded
/// as utf-8, and will error otherwise.
///
/// By default, what is written to `stderr` by the child process
/// is relayed to the parent's `stderr`. However, when [`Stderr`]
/// is used, this is switched off.
#[derive(Debug, Clone)]
pub struct Stderr(pub String);

impl Output for Stderr {
    #[doc(hidden)]
    fn configure(config: &mut Config) {
        config.capture_stderr = true;
    }

    #[doc(hidden)]
    fn from_child_output(config: &Config, child_output: &ChildOutput) -> Result<Self, Error> {
        let stderr = child_output
            .stderr
            .clone()
            .ok_or_else(|| Error::internal("stderr not captured", config))?;
        Ok(Stderr(String::from_utf8(stderr).map_err(|source| {
            Error::InvalidUtf8ToStderr {
                full_command: config.full_command(),
                source,
            }
        })?))
    }
}

/// Use [`Status`] as the return type for [`run_output!`] to retrieve the
/// [`ExitStatus`] of the child process:
///
/// ```
/// use cradle::prelude::*;
///
/// let Status(exit_status) = run_output!(%"echo foo");
/// assert!(exit_status.success());
/// ```
///
/// Also, when using [`Status`], non-zero exit codes won't
/// result in neither a panic (when used with [`run!`] or
/// [`run_output!`]) nor an [`std::result::Result::Err`]
/// (when used with [`run_result!`]):
///
/// ```
/// use cradle::prelude::*;
///
/// let Status(exit_status) = run_output!("false");
/// assert_eq!(exit_status.code(), Some(1));
/// let result: Result<Status, cradle::Error> = run_result!("false");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().0.code(), Some(1));
/// ```
///
/// Also see the
/// [section about error handling](index.html#error-handling) in
/// the module documentation.
#[derive(Debug, Clone)]
pub struct Status(pub ExitStatus);

impl Output for Status {
    #[doc(hidden)]
    fn configure(config: &mut Config) {
        config.error_on_non_zero_exit_code = false;
    }

    #[doc(hidden)]
    fn from_child_output(_config: &Config, child_output: &ChildOutput) -> Result<Self, Error> {
        Ok(Status(child_output.exit_status))
    }
}

/// Using [`bool`] as the return type for [`run_output!`] will return `true` if
/// the command returned successfully, and `false` otherwise:
///
/// ```
/// use cradle::prelude::*;
///
/// if !run_output!(%"which cargo") {
///     panic!("Cargo is not installed!");
/// }
/// ```
///
/// Also, when using [`bool`], non-zero exit codes will not result in a panic
/// or [`std::result::Result::Err`]:
///
/// ```
/// use cradle::prelude::*;
///
/// let success: bool = run_output!("false");
/// assert!(!success);
/// let result: Result<bool, cradle::Error> = run_result!("false");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), false);
/// ```
///
/// Also see the
/// [section about error handling](index.html#error-handling) in
/// the module documentation.
impl Output for bool {
    #[doc(hidden)]
    fn configure(config: &mut Config) {
        config.error_on_non_zero_exit_code = false;
    }

    #[doc(hidden)]
    fn from_child_output(_config: &Config, child_output: &ChildOutput) -> Result<Self, Error> {
        Ok(child_output.exit_status.success())
    }
}
