#[doc(hidden)]
#[derive(Clone)]
pub struct Config {
    pub(crate) arguments: Vec<String>,
    pub(crate) relay_stdout: bool,
}

impl Config {
    pub(crate) fn full_command(&self) -> String {
        self.arguments.join(" ")
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            arguments: Vec::new(),
            relay_stdout: true,
        }
    }
}
