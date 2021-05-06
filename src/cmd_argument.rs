use crate::config::Config;

/// All types that are possible arguments to [`cmd!`] have to implement this trait.
pub trait CmdArgument {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config);
}

/// Arguments of type [`&str`] are being split up into words by whitespace
/// and then passed into the child process as arguments.
impl CmdArgument for &str {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for argument in self.split_whitespace() {
            config.arguments.push(argument.to_string());
        }
    }
}

/// Same as for [`&str`], arguments of type [`String`] are being split
/// up into words by whitespace and then passed into the child process
/// as arguments.
impl CmdArgument for String {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for argument in self.split_whitespace() {
            config.arguments.push(argument.to_string());
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
/// use std::path::PathBuf;
/// use stir::cmd_unit;
///
/// cmd_unit!("touch", vec!["filename with spaces"]);
/// assert!(PathBuf::from("filename with spaces").exists());
/// ```
impl CmdArgument for Vec<&str> {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for argument in self {
            config.arguments.push(argument.to_string());
        }
    }
}

/// Similar to the implementation above for [`Vec<&str>`].
/// All elements of the given [`Vec`] are being passed into the child
/// process as arguments, **without** splitting them by whitespace.
impl CmdArgument for Vec<String> {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.arguments.extend(self);
    }
}

/// Please, see [**here**](trait.CmdArgument.html#impl-CmdArgument).
/// (The documentation for the [`CmdArgument`] implementation for
/// [`LogCommand`].)
pub struct LogCommand;

/// Passing in [`LogCommand`] as an argument to [`cmd!`] will cause it
/// to log the commands including arguments to `stderr`. This is similar
/// to what `bash` does after executing `set -x`.
///
/// ```
/// use stir::{cmd_unit, LogCommand};
///
/// cmd_unit!(LogCommand, "echo foo");
/// // writes '+ echo foo' to stderr
/// ```
impl CmdArgument for LogCommand {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.log_commands = true;
    }
}
