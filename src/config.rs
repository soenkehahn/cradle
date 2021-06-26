use std::{ffi::OsString, path::PathBuf, sync::Arc};

#[doc(hidden)]
pub struct Config {
    pub(crate) arguments: Vec<OsString>,
    pub(crate) log_command: bool,
    pub(crate) working_directory: Option<PathBuf>,
    pub(crate) stdin: Arc<Vec<String>>,
    pub(crate) relay_stdout: bool,
    pub(crate) relay_stderr: bool,
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
            let needs_quotes = argument.contains(' ');
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
            stdin: Arc::new(Vec::new()),
            relay_stdout: true,
            relay_stderr: true,
            error_on_non_zero_exit_code: true,
        }
    }
}
