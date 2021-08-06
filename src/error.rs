//! The [`Error`] type used in the return type of [`cmd_result!`].

use crate::config::Config;
use std::{ffi::OsString, fmt::Display, io, process::ExitStatus, string::FromUtf8Error, sync::Arc};

#[derive(Debug, Clone)]
pub enum Error {
    NoArgumentsGiven,
    FileNotFoundWhenExecuting {
        executable: OsString,
        source: Arc<io::Error>,
    },
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
    CradleBug {
        full_command: String,
        config: Config,
    },
}

impl Error {
    pub(crate) fn command_io_error(config: &Config, error: io::Error) -> Error {
        Error::CommandIoError {
            message: format!("{}:\n  {}", config.full_command(), error),
            source: Arc::new(error),
        }
    }

    pub(crate) fn cradle_bug(config: &Config) -> Error {
        Error::CradleBug {
            full_command: config.full_command(),
            config: config.clone(),
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
        use Error::*;
        match self {
            NoArgumentsGiven => write!(f, "no arguments given"),
            FileNotFoundWhenExecuting { executable, .. } => write!(
                f,
                "File not found error when executing '{}'",
                executable.to_string_lossy()
            ),
            CommandIoError { message, .. } => write!(f, "{}", message),
            NonZeroExitCode {
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
            InvalidUtf8ToStdout { full_command, .. } => {
                write!(f, "{}:\n  invalid utf-8 written to stdout", full_command)
            }
            InvalidUtf8ToStderr { full_command, .. } => {
                write!(f, "{}:\n  invalid utf-8 written to stderr", full_command)
            }
            CradleBug { .. } => {
                let snippets = vec![
                  "Congratulations, you've found a bug in cradle! :/",
                  "Please, consider reporting a bug on https://github.com/soenkehahn/cradle/issues,",
                  "including the following information:",
                ];
                writeln!(f, "{}\n{:#?}", snippets.join(" "), self)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            FileNotFoundWhenExecuting { source, .. } | CommandIoError { source, .. } => {
                Some(&**source)
            }
            InvalidUtf8ToStdout { source, .. } | InvalidUtf8ToStderr { source, .. } => {
                Some(&**source)
            }
            NoArgumentsGiven | NonZeroExitCode { .. } | CradleBug { .. } => None,
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
