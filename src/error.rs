//! The [`Error`] type used in the return type of [`cmd_result!`].

use crate::config::Config;
use std::{fmt::Display, io, process::ExitStatus, string::FromUtf8Error, sync::Arc};

#[derive(Debug, Clone)]
pub enum Error {
    NoArgumentsGiven,
    CommandIoError {
        message: String,
        source: Arc<io::Error>,
    },
    NonZeroExitCode {
        full_command: String,
        exit_status: ExitStatus,
    },
    InvalidUtf8ToStdout {
        full_command: String,
        source: Arc<FromUtf8Error>,
    },
    InvalidUtf8ToStderr {
        full_command: String,
        source: Arc<FromUtf8Error>,
    },
}

impl Error {
    pub(crate) fn command_io_error(config: &Config, error: io::Error) -> Error {
        Error::CommandIoError {
            message: format!("{}:\n  {}", config.full_command(), error),
            source: Arc::new(error),
        }
    }
}

#[doc(hidden)]
#[rustversion::attr(since(1.46), track_caller)]
pub fn panic_on_error<T>(result: Result<T, Error>) -> T {
    match result {
        Ok(t) => t,
        Err(error) => panic!("cmd!: {}", error),
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoArgumentsGiven => write!(f, "no arguments given"),
            Error::CommandIoError { message, .. } => write!(f, "{}", message),
            Error::NonZeroExitCode {
                full_command,
                exit_status,
            } => {
                if let Some(exit_code) = exit_status.code() {
                    write!(
                        f,
                        "{}:\n  exited with exit code: {}",
                        full_command, exit_code
                    )
                } else {
                    write!(f, "{}:\n  exited with {}", full_command, exit_status)
                }
            }
            Error::InvalidUtf8ToStdout { full_command, .. } => {
                write!(f, "{}:\n  invalid utf-8 written to stdout", full_command)
            }
            Error::InvalidUtf8ToStderr { full_command, .. } => {
                write!(f, "{}:\n  invalid utf-8 written to stderr", full_command)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::CommandIoError { source, .. } => Some(&**source),
            Error::InvalidUtf8ToStdout { source, .. }
            | Error::InvalidUtf8ToStderr { source, .. } => Some(&**source),
            Error::NoArgumentsGiven | Error::NonZeroExitCode { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use executable_path::executable_path;
    use std::error::Error;

    #[test]
    fn invalid_utf8_to_stdout_has_source() {
        let result: Result<StdoutUntrimmed, crate::Error> = cmd_result!(
            executable_path("cradle_test_helper").to_str().unwrap(),
            "invalid utf-8 stdout"
        );
        assert!(result.unwrap_err().source().is_some());
    }

    #[test]
    fn invalid_utf8_to_stderr_has_source() {
        let result: Result<Stderr, crate::Error> = cmd_result!(
            executable_path("cradle_test_helper").to_str().unwrap(),
            "invalid utf-8 stderr"
        );
        assert!(result.unwrap_err().source().is_some());
    }
}
