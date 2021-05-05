/// All types that are possible arguments to [`cmd!`] have to implement this trait.
pub trait CmdArgument {
    #[doc(hidden)]
    fn add_as_argument(self, accumulator: &mut Vec<String>);
}

impl CmdArgument for &str {
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        for argument in self.split_whitespace() {
            accumulator.push(argument.to_string());
        }
    }
}

impl CmdArgument for String {
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        for argument in self.split_whitespace() {
            accumulator.push(argument.to_string());
        }
    }
}

impl CmdArgument for Vec<&str> {
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        for argument in self {
            accumulator.push(argument.to_string());
        }
    }
}

impl CmdArgument for Vec<String> {
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        accumulator.extend(self);
    }
}
