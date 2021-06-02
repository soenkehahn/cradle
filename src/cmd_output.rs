use crate::{Config, Error, RunResult};
use std::process::ExitStatus;

/// All possible return types of [`cmd!`] have to implement this trait.
/// For documentation about what these return types do, see the
/// individual implementations below.
///
/// Except for tuples: All [`CmdOutput`] implementations for tuples serve
/// the same purpose: combining multiple types that implement [`CmdOutput`]
/// to retrieve more information from a child process. The following code
/// for example retrieves what's written to `stdout` **and** the
/// [`ExitStatus`]:
///
/// ```
/// use cradle::*;
///
/// let (StdoutUntrimmed(stdout), Exit(status)) = cmd!("echo foo");
/// assert_eq!(stdout, "foo\n");
/// assert!(status.success());
/// ```
pub trait CmdOutput: Sized {
    #[doc(hidden)]
    fn prepare_config(config: &mut Config);

    #[doc(hidden)]
    fn from_run_result(config: &Config, result: Result<RunResult, Error>) -> Result<Self, Error>;
}

/// Use this when you don't need any result from the child process.
impl CmdOutput for () {
    #[doc(hidden)]
    fn prepare_config(_config: &mut Config) {}

    #[doc(hidden)]
    fn from_run_result(_config: &Config, result: Result<RunResult, Error>) -> Result<Self, Error> {
        result?;
        Ok(())
    }
}

/// See the [`CmdOutput`] implementation for [`StdoutTrimmed`] below.
#[derive(Debug, PartialEq, Clone)]
pub struct StdoutTrimmed(pub String);

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
/// use cradle::*;
///
/// # #[cfg(unix)]
/// # {
/// let StdoutTrimmed(output) = cmd!("which ls");
/// assert!(Path::new(&output).exists());
/// # }
/// ```
impl CmdOutput for StdoutTrimmed {
    fn prepare_config(config: &mut Config) {
        StdoutUntrimmed::prepare_config(config);
    }

    fn from_run_result(config: &Config, result: Result<RunResult, Error>) -> Result<Self, Error> {
        let StdoutUntrimmed(stdout) = StdoutUntrimmed::from_run_result(config, result)?;
        Ok(StdoutTrimmed(stdout.trim().to_owned()))
    }
}

/// See the [`CmdOutput`] implementation for [`StdoutUntrimmed`] below.
#[derive(Debug, PartialEq, Clone)]
pub struct StdoutUntrimmed(pub String);

/// Same as [`StdoutTrimmed`], but does not trim whitespace from the output:
///
/// ```
/// use cradle::*;
///
/// let StdoutUntrimmed(output) = cmd!("echo foo");
/// assert_eq!(output, "foo\n");
/// ```
impl CmdOutput for StdoutUntrimmed {
    #[doc(hidden)]
    fn prepare_config(config: &mut Config) {
        config.relay_stdout = false;
    }

    #[doc(hidden)]
    fn from_run_result(config: &Config, result: Result<RunResult, Error>) -> Result<Self, Error> {
        let result = result?;
        Ok(StdoutUntrimmed(String::from_utf8(result.stdout).map_err(
            |source| Error::InvalidUtf8ToStdout {
                full_command: config.full_command(),
                source,
            },
        )?))
    }
}

macro_rules! tuple_impl {
    ($($generics:ident,)+) => {
        impl<$($generics),+> CmdOutput for ($($generics,)+)
        where
            $($generics: CmdOutput,)+
        {
            #[doc(hidden)]
            fn prepare_config(config: &mut Config) {
                $($generics::prepare_config(config);)+
            }

            #[doc(hidden)]
            fn from_run_result(config: &Config, result: Result<RunResult, Error>) -> Result<Self, Error> {
                Ok((
                    $($generics::from_run_result(config, result.clone())?,)+
                ))
            }
        }
    };
}

tuple_impl!(A,);
tuple_impl!(A, B,);
tuple_impl!(A, B, C,);

/// See the [`CmdOutput`] implementation for [`Exit`] below.
pub struct Exit(pub ExitStatus);

/// Using [`Exit`] as the return type for [`cmd!`] allows to
/// retrieve the [`ExitStatus`] of the child process:
///
/// ```
/// use cradle::*;
///
/// let Exit(status) = cmd!("echo foo");
/// assert!(status.success());
/// ```
///
/// Also, when using [`Exit`], non-zero exit codes won't
/// result in neither a panic nor a [`std::result::Result::Err`]:
///
/// ```
/// use cradle::*;
///
/// let Exit(status) = cmd!("false");
/// assert_eq!(status.code(), Some(1));
/// let result: Result<Exit, cradle::Error> = cmd_result!("false");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().0.code(), Some(1));
/// ```
///
/// Also see the
/// [section about error handling](index.html#error-handling) in
/// the module documentation.
impl CmdOutput for Exit {
    #[doc(hidden)]
    fn prepare_config(config: &mut Config) {
        config.error_on_non_zero_exit_code = false;
    }

    #[doc(hidden)]
    fn from_run_result(_config: &Config, result: Result<RunResult, Error>) -> Result<Self, Error> {
        Ok(Exit(result?.exit_status))
    }
}

/// See the [`CmdOutput`] implementation for [`Stderr`] below.
#[derive(Debug)]
pub struct Stderr(pub String);

/// [`Stderr`] allows to capture the `stderr` of a child process:
///
/// ```
/// use cradle::*;
///
/// // (`Exit` is used here to suppress panics caused by `ls`
/// // terminating with a non-zero exit code.)
/// let (Stderr(stderr), Exit(_)) = cmd!("ls does-not-exist");
/// assert!(stderr.contains("No such file or directory"));
/// ```
///
/// This assumes that the output written to `stderr` is encoded
/// as utf-8, and will error otherwise.
///
/// By default, what is written to `stderr` by the child process
/// is relayed to the parent's `stderr`. However, when [`Stderr`]
/// is used, this is switched off.
impl CmdOutput for Stderr {
    #[doc(hidden)]
    fn prepare_config(config: &mut Config) {
        config.relay_stderr = false;
    }

    #[doc(hidden)]
    fn from_run_result(config: &Config, result: Result<RunResult, Error>) -> Result<Self, Error> {
        Ok(Stderr(String::from_utf8(result?.stderr).map_err(
            |source| Error::InvalidUtf8ToStderr {
                full_command: config.full_command(),
                source,
            },
        )?))
    }
}
