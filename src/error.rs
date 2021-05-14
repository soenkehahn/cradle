use std::{fmt::Display, io, process::ExitStatus};

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
    InvalidUtf8ToStdout,
    InvalidUtf8ToStderr,
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
            Error::InvalidUtf8ToStderr => write!(f, "cmd!: invalid utf-8 written to stderr"),
        }
    }
}
