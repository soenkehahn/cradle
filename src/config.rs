//! An internal module used for configuring child processes.

use std::{ffi::OsString, path::PathBuf, sync::Arc};

/// Internal type that configures how to run a child process.
/// Usually you don't have to use this type directly.
///
/// See also [Custom `Input` impls](crate::Input#custom-input-impls).
#[rustversion::attr(since(1.48), allow(clippy::rc_buffer))]
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) arguments: Vec<OsString>,
    pub(crate) log_command: bool,
    pub(crate) working_directory: Option<PathBuf>,
    pub(crate) added_environment_variables: Vec<(OsString, OsString)>,
    pub(crate) stdin: Arc<Vec<u8>>,
    pub(crate) capture_stdout: bool,
    pub(crate) capture_stderr: bool,
    pub(crate) error_on_non_zero_exit_code: bool,
}

impl Config {
    pub(crate) fn full_command(&self) -> String {
        let mut result = String::new();
        for argument in self.arguments.iter() {
            let argument = argument.to_string_lossy();
            if !result.is_empty() {
                result.push(' ');
            }
            let needs_quotes = argument.is_empty() || argument.contains(' ');
            if needs_quotes {
                result.push('\'');
            }
            result.push_str(&argument);
            if needs_quotes {
                result.push('\'');
            }
        }
        result
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            arguments: Vec::new(),
            log_command: false,
            working_directory: None,
            added_environment_variables: Vec::new(),
            stdin: Arc::new(Vec::new()),
            capture_stdout: false,
            capture_stderr: false,
            error_on_non_zero_exit_code: true,
        }
    }
}
