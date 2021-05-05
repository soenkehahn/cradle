use crate::{error::Result, Context, Error, RunResult};

/// All possible return types of [`cmd!`] have to implement this trait.
pub trait CmdOutput: Sized {
    #[doc(hidden)]
    fn prepare_context<Stdout, Stderr>(context: &mut Context<Stdout, Stderr>);

    #[doc(hidden)]
    fn from_run_result(output: Result<RunResult>) -> Result<Self>;
}

/// Use this when you don't need any result from the child process.
impl CmdOutput for () {
    #[doc(hidden)]
    fn prepare_context<Stdout, Stderr>(_context: &mut Context<Stdout, Stderr>) {}

    #[doc(hidden)]
    fn from_run_result(output: Result<RunResult>) -> Result<Self> {
        output?;
        Ok(())
    }
}

/// Returns what the child process writes to `stdout`, interpreted as utf-8,
/// collected into a string. This also suppresses output of the child's `stdout`
/// to the parent's `stdout`. (Which would be the default when not using [`String`]
/// as the return value.)
impl CmdOutput for String {
    #[doc(hidden)]
    fn prepare_context<Stdout, Stderr>(context: &mut Context<Stdout, Stderr>) {
        context.stdout = None;
    }

    #[doc(hidden)]
    fn from_run_result(output: Result<RunResult>) -> Result<Self> {
        let output = output?;
        String::from_utf8(output.stdout).map_err(|_| Error::InvalidUtf8ToStdout)
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
    fn prepare_context<Stdout, Stderr>(context: &mut Context<Stdout, Stderr>) {
        T::prepare_context(context);
    }

    #[doc(hidden)]
    fn from_run_result(output: Result<RunResult>) -> Result<Self> {
        Ok(match output {
            Ok(_) => T::from_run_result(output),
            Err(error) => Err(error),
        })
    }
}
