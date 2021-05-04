use std::{fmt::Display, io, process::ExitStatus};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Debug)]
pub enum Error {
    NoArgumentsGiven,
    CommandIoError {
        message: String,
    },
    NonZeroExitCode {
        full_command: String,
        exit_status: ExitStatus,
    },
    InvalidUtf8ToStdout,
}

impl Error {
    pub(crate) fn command_io_error(command: &str, error: io::Error) -> Error {
        Error::CommandIoError {
            message: format!("cmd!: {}: {}", command, error),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoArgumentsGiven => write!(f, "cmd!: no arguments given"),
            Error::CommandIoError { message } => write!(f, "{}", message),
            Error::NonZeroExitCode {
                full_command,
                exit_status,
            } => write!(f, "{}:\n  exited with {}", full_command, exit_status),
            Error::InvalidUtf8ToStdout => write!(f, "cmd!: invalid utf-8 written to stdout"),
        }
    }
}
