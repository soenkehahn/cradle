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
/// assert_eq!(stdout, "foo");
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
/// as the return value.) Also, this trims trailing `\n` or `\r\n` characters, if
/// they exist:
///
/// ```
/// use stir::cmd;
///
/// let output: String = cmd!("echo foo");
/// assert_eq!(output, "foo"); // trims '\n' character at the end
/// let output: String = cmd!("echo", ["\nfoo "]);
/// assert_eq!(output, "\nfoo "); // does not trim other whitespace
/// ```
impl CmdOutput for String {
    #[doc(hidden)]
    fn prepare_config(config: &mut Config) {
        config.relay_stdout = false;
    }

    #[doc(hidden)]
    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        let result = result?;
        Ok(trim_trailing_newline(
            String::from_utf8(result.stdout).map_err(|_| Error::InvalidUtf8ToStdout)?,
        ))
    }
}

fn trim_trailing_newline(input: String) -> String {
    if input.as_bytes().last() == Some(&b'\n') {
        if input.as_bytes()[input.len() - 2] == b'\r' {
            input[..input.len() - 2].to_string()
        } else {
            input[..input.len() - 1].to_string()
        }
    } else {
        input
    }
}

/// Please, see the [`CmdOutput`] implementation for [`UntrimmedStdout`] below.
pub struct UntrimmedStdout(pub String);

/// Returns what the child process writes to `stdout`, interpreted as utf-8,
/// collected into a string. This also suppresses output of the child's `stdout`
/// to the parent's `stdout`, which otherwise would be done by default.
/// Contrary to [`String`] this does **not** trim trailing `\n` or `\r\n` characters:
///
/// ```
/// use stir::{cmd, UntrimmedStdout};
///
/// let UntrimmedStdout(output) = cmd!("echo foo");
/// assert_eq!(output, "foo\n");
/// ```
impl CmdOutput for UntrimmedStdout {
    #[doc(hidden)]
    fn prepare_config(_config: &mut Config) {}

    #[doc(hidden)]
    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        Ok(UntrimmedStdout(
            String::from_utf8(result?.stdout).map_err(|_| Error::InvalidUtf8ToStdout)?,
        ))
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
#[derive(Debug)]
pub struct Stderr(pub String);

/// [`Stderr`] allows to capture the `stderr` of a child process:
///
/// ```
/// use stir::{cmd, Exit, Stderr};
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
    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        Ok(Stderr(
            String::from_utf8(result?.stderr).map_err(|_| Error::InvalidUtf8ToStderr)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod trim_trailing_newline {
        use super::*;

        macro_rules! test {
            ($name:ident, $input:expr, $expected:expr) => {
                #[test]
                fn $name() {
                    assert_eq!(trim_trailing_newline($input.to_string()), $expected);
                }
            };
        }

        test!(trims_trailing_newline, "foo\n", "foo");
        test!(
            does_not_modify_strings_without_trailing_newline,
            "foo",
            "foo"
        );
        test!(trims_trailing_carriage_return_and_newline, "foo\r\n", "foo");
        test!(does_not_trim_other_whitespace_at_the_end, "foo ", "foo ");
        test!(does_not_trim_whitespace_at_the_start, " foo", " foo");
        test!(
            does_not_trim_whitespace_at_the_start_with_trailing_newline,
            " foo\n",
            " foo"
        );
        test!(does_not_trim_trailing_carriage_return, "foo\r", "foo\r");
        test!(unicode_values, "😂", "😂");
    }
}
