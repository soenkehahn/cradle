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
    Internal {
        message: String,
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

    pub(crate) fn internal(message: &str, config: &Config) -> Error {
        Error::Internal {
            message: message.to_string(),
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
            FileNotFoundWhenExecuting { executable, .. } => {
                let executable = executable.to_string_lossy();
                let mut message = vec![format!(
                    "File not found error when executing '{}'",
                    executable
                )];
                match executable
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .as_slice()
                {
                    [intended_executable, intended_arguments @ ..]
                        if !intended_arguments.is_empty() =>
                    {
                        let intended_arguments = {
                            let mut result = "[".to_string();
                            let mut first = true;
                            for argument in intended_arguments {
                                result.push('\'');
                                result.push_str(argument);
                                result.push('\'');
                                if first {
                                    first = false;
                                    result.push_str(", ");
                                }
                            }
                            result.push(']');
                            result
                        };
                        message.extend(vec![
                            format!(
                                "note: Given executable name '{}' contains whitespace.",
                                executable
                            ),
                            format!(
                                "  Did you mean to run '{}', with {} as arguments?",
                                intended_executable, intended_arguments
                            ),
                            "  Consider using Split: https://docs.rs/cradle/latest/cradle/input/struct.Split.html".to_string(),
                        ]);
                    }
                    _ => {}
                }
                write!(f, "{}", message.join("\n"))
            }
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
            Internal { .. } => {
                let snippets = vec![
                    "Congratulations, you've found a bug in cradle! :/",
                    "Please, open an issue on https://github.com/soenkehahn/cradle/issues",
                    "with the following information:",
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
            NoArgumentsGiven | NonZeroExitCode { .. } | Internal { .. } => None,
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
