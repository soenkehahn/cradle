/// All types that are possible arguments to [`cmd!`] have to implement this trait.
pub trait CmdArgument {
    #[doc(hidden)]
    fn add_as_argument(self, accumulator: &mut Vec<String>);
}

/// Arguments of type [`&str`] are being split up into words by whitespace
/// and then passed into the child process as arguments.
impl CmdArgument for &str {
    #[doc(hidden)]
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        for argument in self.split_whitespace() {
            accumulator.push(argument.to_string());
        }
    }
}

/// Same as for [`&str`], arguments of type [`String`] are being split
/// up into words by whitespace and then passed into the child process
/// as arguments.
impl CmdArgument for String {
    #[doc(hidden)]
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        for argument in self.split_whitespace() {
            accumulator.push(argument.to_string());
        }
    }
}

/// All elements of the given [`Vec`] are being passed into the child
/// process as arguments, **without** splitting them by whitespace.
///
/// This can come in handy to avoid whitespace splitting, even if you only want
/// to encode a single argument:
///
/// ```
/// use stir::cmd_unit;
///
/// cmd_unit!("touch", vec!["filename with spaces"]);
/// ```
impl CmdArgument for Vec<&str> {
    #[doc(hidden)]
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        for argument in self {
            accumulator.push(argument.to_string());
        }
    }
}

/// Similar to the implementation above for [`Vec<&str>`].
/// All elements of the given [`Vec`] are being passed into the child
/// process as arguments, **without** splitting them by whitespace.
impl CmdArgument for Vec<String> {
    #[doc(hidden)]
    fn add_as_argument(self, accumulator: &mut Vec<String>) {
        accumulator.extend(self);
    }
}
