use crate::{error::Result, Config, Error, RunResult};
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
/// use stir::{cmd, Exit};
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
    fn from_run_result(result: Result<RunResult>) -> Result<Self>;
}

/// Use this when you don't need any result from the child process.
impl CmdOutput for () {
    #[doc(hidden)]
    fn prepare_config(_config: &mut Config) {}

    #[doc(hidden)]
    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        result?;
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
    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        let result = result?;
        String::from_utf8(result.stdout).map_err(|_| Error::InvalidUtf8ToStdout)
    }
}

/// To turn all possible panics of [`cmd!`] into [`std::result::Result::Err`]s
/// you can use a return type of `Result<T, Error>`. `T` can be any type that
/// implements [`CmdOutput`] and [`Error`] is stir's custom error type.
impl<T> CmdOutput for Result<T>
where
    T: CmdOutput,
{
    #[doc(hidden)]
    fn prepare_config(config: &mut Config) {
        T::prepare_config(config);
    }

    #[doc(hidden)]
    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        Ok(match result {
            Ok(_) => T::from_run_result(result),
            Err(error) => Err(error),
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
            fn from_run_result(result: Result<RunResult>) -> Result<Self> {
                Ok((
                    $($generics::from_run_result(result.clone())?,)+
                ))
            }
        }
    };
}

tuple_impl!(A,);
tuple_impl!(A, B,);
tuple_impl!(A, B, C,);

/// Please, see the [`CmdOutput`] implementation for [`Exit`] below.
pub struct Exit(pub ExitStatus);

/// Using [`Exit`] as the return type for [`cmd!`] allows to
/// retrieve the [`ExitStatus`] of the child process:
///
/// ```
/// use stir::{cmd, Exit};
///
/// let Exit(status) = cmd!("echo foo");
/// assert!(status.success());
/// ```
///
/// Also, when using [`Exit`], non-zero exit codes won't
/// result in neither a panic nor a [`std::result::Result::Err`]:
///
/// ```
/// use stir::{cmd, Result, Exit};
///
/// let Exit(status) = cmd!("false");
/// assert_eq!(status.code(), Some(1));
/// let result: Result<Exit> = cmd!("false");
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
    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        Ok(Exit(result?.exit_status))
    }
}

/// Please, see the [`CmdOutput`] implementation for [`Stderr`] below.
pub struct Stderr(String);

/// [`Stderr`] allows to capture the `stderr` of a child process:
///
/// ```
/// use stir::{cmd, Exit, Stderr};
///
/// // (`Exit` is used here to suppress panics caused by `ls`
/// // terminating with a non-zero exit code.)
/// let (Stderr(stderr), Exit(_)) = cmd!("ls does-not-exist");
/// assert_eq!(stderr, "ls: cannot access 'does-not-exist': No such file or directory");
/// ```
/// todo: example
/// todo: assume utf-8
/// todo: don't relay to parent's stderr
impl CmdOutput for i32 {
    fn prepare_config(config: &mut Config) {
        todo!()
    }

    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        todo!()
    }
}
