use crate::{Config, Error, RunResult};
use std::{io, process::ExitStatus};

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
/// use stir::*;
///
/// let (stdout, Exit(status)) = cmd!("echo foo");
/// let _: String = stdout;
/// assert_eq!(stdout, "foo\n");
/// assert!(status.success());
/// ```
pub trait CmdOutput: Sized {
    #[doc(hidden)]
    fn prepare_config(config: &mut Config);

    #[doc(hidden)]
    fn from_run_result(config: &Config, result: RunResult) -> Result<Self, Error>;
}

/// Use this when you don't need any result from the child process.
impl CmdOutput for () {
    #[doc(hidden)]
    fn prepare_config(_config: &mut Config) {}

    #[doc(hidden)]
    fn from_run_result(_config: &Config, _result: RunResult) -> Result<Self, Error> {
        Ok(())
    }
}

/// Returns what the child process writes to `stdout`, interpreted as utf-8,
/// collected into a string. This also suppresses output of the child's `stdout`
/// to the parent's `stdout`. (Which would be the default when not using [`String`]
/// as the return value.)
impl CmdOutput for String {
    #[doc(hidden)]
    fn prepare_config(config: &mut Config) {
        config.relay_stdout = false;
    }

    #[doc(hidden)]
    fn from_run_result(config: &Config, result: RunResult) -> Result<Self, Error> {
        match result {
            RunResult::EarlyError(_) => todo!(),
            RunResult::LaterError {
                collected_output, ..
            } => {
                String::from_utf8(collected_output.stdout).map_err(|_| Error::InvalidUtf8ToStdout {
                    full_command: config.full_command(),
                })
            }
            RunResult::Success { stdout, .. } => {
                String::from_utf8(stdout).map_err(|_| Error::InvalidUtf8ToStdout {
                    full_command: config.full_command(),
                })
            }
        }
    }
}

/// To turn all possible panics of [`cmd!`] into [`std::result::Result::Err`]s
/// you can use a return type of `Result<T, stir::Error>`. `T` can be any type that
/// implements [`CmdOutput`].
impl<T> CmdOutput for Result<T, Error>
where
    T: CmdOutput,
{
    #[doc(hidden)]
    fn prepare_config(config: &mut Config) {
        config.should_panic = false;
        T::prepare_config(config);
    }

    #[doc(hidden)]
    fn from_run_result(config: &Config, result: RunResult) -> Result<Self, Error> {
        Ok(match result {
            RunResult::EarlyError(error) | RunResult::LaterError { error, .. } => Err(error),
            run_result @ RunResult::Success { .. } => T::from_run_result(config, run_result),
        })
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
            fn from_run_result(config: &Config, result: RunResult) -> Result<Self, Error> {
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

/// Please, see the [`CmdOutput`] implementation for [`Exit`] below.
#[derive(Debug)]
pub struct Exit(pub ExitStatus);

/// Using [`Exit`] as the return type for [`cmd!`] allows to
/// retrieve the [`ExitStatus`] of the child process:
///
/// ```
/// use stir::*;
///
/// let Exit(status) = cmd!("echo foo");
/// assert!(status.success());
/// ```
///
/// Also, when using [`Exit`], non-zero exit codes won't
/// result in neither a panic nor a [`std::result::Result::Err`]:
///
/// ```
/// use stir::*;
///
/// let Exit(status) = cmd!("false");
/// assert_eq!(status.code(), Some(1));
/// let result: Result<Exit, stir::Error> = cmd!("false");
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
    fn from_run_result(_config: &Config, result: RunResult) -> Result<Self, Error> {
        match result {
            RunResult::EarlyError(Error::CommandIoError { error_kind, .. })
                if error_kind == io::ErrorKind::NotFound =>
            {
                Ok(Exit(exit_status(127)))
            }
            RunResult::EarlyError(error) => Ok(Exit(exit_status(1))),
            RunResult::LaterError {
                collected_output,
                error,
            } => todo!(),
            RunResult::Success { exit_status, .. } => Ok(Exit(exit_status)),
        }
    }
}

#[cfg(windows)]
fn exit_status(exit_code: i32) -> ExitStatus {
    std::os::windows::process::ExitStatusExt::from_raw(exit_code)
}
#[cfg(unix)]
fn exit_status(exit_code: i32) -> ExitStatus {
    std::os::unix::prelude::ExitStatusExt::from_raw(exit_code << 8)
}

/// Please, see the [`CmdOutput`] implementation for [`Stderr`] below.
#[derive(Debug)]
pub struct Stderr(pub String);

/// [`Stderr`] allows to capture the `stderr` of a child process:
///
/// ```
/// use stir::*;
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
    fn from_run_result(config: &Config, result: RunResult) -> Result<Self, Error> {
        match result {
            RunResult::EarlyError(error) => {
                dbg!(error);
                Ok(Stderr("fixme".to_string()))
            }
            // fixme: combine match arms
            // fixme: remove Oks
            RunResult::LaterError {
                collected_output, ..
            } => Ok(Stderr(String::from_utf8(collected_output.stderr).map_err(
                |_| Error::InvalidUtf8ToStderr {
                    full_command: config.full_command(),
                },
            )?)),
            RunResult::Success { stderr, .. } => {
                Ok(Stderr(String::from_utf8(stderr).map_err(|_| {
                    Error::InvalidUtf8ToStderr {
                        full_command: config.full_command(),
                    }
                })?))
            }
        }
    }
}
