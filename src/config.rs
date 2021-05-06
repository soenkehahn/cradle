#[doc(hidden)]
#[derive(Clone)]
pub struct Config {
    pub(crate) arguments: Vec<String>,
    pub(crate) relay_stdout: bool,
    pub(crate) log_commands: bool,
}

impl Config {
    pub(crate) fn full_command(&self) -> String {
        let mut result = String::new();
        for argument in self.arguments.iter() {
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
            relay_stdout: true,
            log_commands: false,
        }
    }
}
