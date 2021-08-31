//! The [`Error`] type used in the return type of [`run_result!`].

use crate::config::Config;
use std::{ffi::OsString, fmt::Display, io, process::ExitStatus, string::FromUtf8Error, sync::Arc};

/// Error type returned when an error occurs while using [`run_result!`]
/// or [`crate::input::Input::run_result`].
///
/// [`run!`], [`crate::input::Input::run`], [`run_output!`],
/// and [`crate::input::Input::run_output`] will turn these errors
/// into panics.
#[derive(Debug, Clone)]
pub enum Error {
    /// The [`Input`](crate::Input)s to a command must produce
    /// at least one argument: the executable to run.
    ///
    /// ```
    /// use cradle::prelude::*;
    ///
    /// let result: Result<(), cradle::Error> = run_result!(());
    /// match result {
    ///   Err(Error::NoExecutableGiven) => {}
    ///   _ => panic!(),
    /// }
    /// ```
    NoExecutableGiven,
    /// A `file not found` error occurred while trying to spawn
    /// the child process:
    ///
    /// ```
    /// use cradle::prelude::*;
    ///
    /// let result: Result<(), Error> = run_result!("does-not-exist");
    /// match result {
    ///   Err(Error::FileNotFound { .. }) => {}
    ///   _ => panic!(),
    /// }
    /// ```
    ///
    /// Note that this error doesn't necessarily mean that the executable file
    /// could not be found.
    /// A few other circumstances in which this can occur are:
    ///
    /// - a binary is dynamically linked against a library,
    ///   but that library cannot be found, or
    /// - the executable starts with a
    ///   [shebang](https://en.wikipedia.org/wiki/Shebang_(Unix)),
    ///   but the interpreter specified in the shebang cannot be found.
    FileNotFound {
        executable: OsString,
        source: Arc<io::Error>,
    },
    /// An IO error during execution. A few circumstances in which this can occur are:
    ///
    /// - spawning the child process fails (for another reason than
    ///   [`FileNotFound`](Error::FileNotFound)),
    /// - writing to `stdin` of the child process fails,
    /// - reading from `stdout` or `stderr` of the child process fails,
    /// - writing to the parent's `stdout` or `stderr` fails,
    /// - the given executable doesn't have the executable flag set.
    CommandIoError {
        message: String,
        source: Arc<io::Error>,
    },
    /// The child process exited with a non-zero exit code.
    ///
    /// ```
    /// use cradle::prelude::*;
    ///
    /// let result: Result<(), cradle::Error> = run_result!("false");
    /// match result {
    ///   Err(Error::NonZeroExitCode { .. }) => {}
    ///   _ => panic!(),
    /// }
    /// ```
    ///
    /// This error will be suppressed when [`Status`](crate::Status) is used.
    NonZeroExitCode {
        full_command: String,
        exit_status: ExitStatus,
    },
    /// The child process's `stdout` is being captured,
    /// (e.g. with [`StdoutUntrimmed`](crate::StdoutUntrimmed)),
    /// but the process wrote bytes to its `stdout` that are not
    /// valid utf-8.
    InvalidUtf8ToStdout {
        full_command: String,
        source: Arc<FromUtf8Error>,
    },
    /// The child process's `stderr` is being captured,
    /// (with [`Stderr`](crate::Stderr)),
    /// but the process wrote bytes to its `stderr` that are not
    /// valid utf-8.
    InvalidUtf8ToStderr {
        full_command: String,
        source: Arc<FromUtf8Error>,
    },
    /// This error is raised when an internal invariant of `cradle` is broken,
    /// and likely indicates a bug.
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
        Err(error) => panic!("cradle error: {}", error),
    }
}

fn english_list(list: &[&str]) -> String {
    let mut result = String::new();
    for (i, word) in list.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == list.len() - 1;
        if !is_first {
            result.push_str(if is_last { " and " } else { ", " });
        }
        result.push('\'');
        result.push_str(word);
        result.push('\'');
    }
    result
}

fn executable_with_whitespace_note(executable: &str) -> Option<String> {
    let words = executable.split_whitespace().collect::<Vec<&str>>();
    if words.len() >= 2 {
        let intended_executable = words[0];
        let intended_arguments = &words[1..];
        let mut result = format!(
            "note: Given executable name '{}' contains whitespace.\n",
            executable
        );
        result.push_str(&format!(
            "  Did you mean to run '{}', with {} as {}?\n",
            intended_executable,
            english_list(intended_arguments),
            if intended_arguments.len() == 1 {
                "the argument"
            } else {
                "arguments"
            },
        ));
        result.push_str(concat!(
            "  Consider using Split: ",
            "https://docs.rs/cradle/latest/cradle/input/struct.Split.html"
        ));
        Some(result)
    } else {
        None
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            NoExecutableGiven => write!(f, "no arguments given"),
            FileNotFound { executable, .. } => {
                let executable = executable.to_string_lossy();
                write!(f, "File not found error when executing '{}'", executable)?;
                if let Some(whitespace_note) = executable_with_whitespace_note(executable.as_ref())
                {
                    write!(f, "\n{}", whitespace_note)?;
                }
                Ok(())
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
            FileNotFound { source, .. } | CommandIoError { source, .. } => Some(&**source),
            InvalidUtf8ToStdout { source, .. } | InvalidUtf8ToStderr { source, .. } => {
                Some(&**source)
            }
            NoExecutableGiven | NonZeroExitCode { .. } | Internal { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use executable_path::executable_path;

    #[test]
    fn invalid_utf8_to_stdout_has_source() {
        let result: Result<StdoutUntrimmed, crate::Error> = run_result!(
            executable_path("cradle_test_helper").to_str().unwrap(),
            "invalid utf-8 stdout"
        );
        assert!(std::error::Error::source(&result.unwrap_err()).is_some());
    }

    #[test]
    fn invalid_utf8_to_stderr_has_source() {
        let result: Result<Stderr, crate::Error> = run_result!(
            executable_path("cradle_test_helper").to_str().unwrap(),
            "invalid utf-8 stderr"
        );
        assert!(std::error::Error::source(&result.unwrap_err()).is_some());
    }

    mod english_list {
        use super::*;
        use pretty_assertions::assert_eq;

        macro_rules! test {
            ($name:ident, $input:expr, $expected:expr) => {
                #[test]
                fn $name() {
                    assert_eq!(english_list($input), $expected);
                }
            };
        }

        test!(one, &["foo"], "'foo'");
        test!(two, &["foo", "bar"], "'foo' and 'bar'");
        test!(three, &["foo", "bar", "baz"], "'foo', 'bar' and 'baz'");
        test!(
            four,
            &["foo", "bar", "baz", "boo"],
            "'foo', 'bar', 'baz' and 'boo'"
        );
    }
}
