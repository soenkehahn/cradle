use crate::{error::Result, Config, Error, RunResult};
use std::process::ExitStatus;

/// All possible return types of [`cmd!`] have to implement this trait.
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

/// todo
pub struct Exit(pub ExitStatus);

/// todo
impl CmdOutput for Exit {
    fn prepare_config(config: &mut Config) {
        config.error_on_non_zero_exit_code = false;
    }

    fn from_run_result(result: Result<RunResult>) -> Result<Self> {
        Ok(Exit(result?.exit_status))
    }
}
