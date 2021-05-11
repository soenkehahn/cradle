use crate::Config;
use std::{fmt::Display, io, process::ExitStatus};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Debug, Clone)]
pub enum Error {
    NoArgumentsGiven,
    CommandIoError {
        message: String,
    },
    NonZeroExitCode {
        full_command: String,
        exit_status: ExitStatus,
    },
    InvalidUtf8ToStdout {
        full_command: String,
    },
    InvalidUtf8ToStderr {
        full_command: String,
    },
}

impl Error {
    pub(crate) fn command_io_error(config: &Config, error: io::Error) -> Error {
        Error::CommandIoError {
            message: format!("{}:\n  {}", config.full_command(), error),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoArgumentsGiven => write!(f, "no arguments given"),
            Error::CommandIoError { message } => write!(f, "{}", message),
            Error::NonZeroExitCode {
                full_command,
                exit_status,
            } => write!(f, "{}:\n  exited with {}", full_command, exit_status),
            Error::InvalidUtf8ToStdout { full_command } => {
                write!(f, "{}:\n  invalid utf-8 written to stdout", full_command)
            }
            Error::InvalidUtf8ToStderr { full_command } => {
                write!(f, "{}:\n  invalid utf-8 written to stderr", full_command)
            }
        }
    }
}
