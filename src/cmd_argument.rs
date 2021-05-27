use crate::config::Config;
use std::path::Path;

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
/// use stir::*;
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

/// Similar to the implementation for [`Vec<&str>`].
/// All elements of the given [`Vec`] are being passed into the child
/// process as arguments, **without** splitting them by whitespace.
impl CmdArgument for Vec<String> {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.arguments.extend(self);
    }
}

/// Similar to the implementation for [`Vec<&str>`].
/// All elements of the array will be passed into the child
/// process as arguments, **without** splitting them by whitespace.
///
/// ```
/// use stir::*;
///
/// let Stdout(output) = cmd!(["echo", "foo"]);
/// assert_eq!(output, "foo\n");
/// ```
#[rustversion::since(1.51)]
impl<const N: usize> CmdArgument for [&str; N] {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        self[..].prepare_config(config);
    }
}

/// Similar to the implementation for [`Vec<&str>`].
/// All elements of the array will be passed into the child
/// process as arguments, **without** splitting them by whitespace.
#[rustversion::since(1.51)]
impl<const N: usize> CmdArgument for &[&str; N] {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        self[..].prepare_config(config);
    }
}

/// Similar to the implementation for [`Vec<&str>`].
/// All elements of the slice will be passed into the child
/// process as arguments, **without** splitting them by whitespace.
impl CmdArgument for &[&str] {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for argument in self.iter() {
            config.arguments.push((*argument).to_string());
        }
    }
}

/// See the [`CmdArgument`] implementation for [`LogCommand`] below.
pub struct LogCommand;

/// Passing in [`LogCommand`] as an argument to [`cmd!`] will cause it
/// to log the commands (including all arguments) to `stderr`.
/// (This is similar `bash`'s `-x` option.)
///
/// ```
/// use stir::*;
///
/// cmd_unit!(LogCommand, "echo foo");
/// // writes '+ echo foo' to stderr
/// ```
impl CmdArgument for LogCommand {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.log_command = true;
    }
}

/// See the [`CmdArgument`] implementation for [`CurrentDir`] below.
pub struct CurrentDir<T: AsRef<Path>>(pub T);

/// By default child processes inherit the current directory from their
/// parent. You can override this with [`CurrentDir`]:
///
/// ```
/// use stir::*;
///
/// # #[cfg(target_os = "linux")]
/// # {
/// let Stdout(output) = cmd!("pwd", CurrentDir("/tmp"));
/// assert_eq!(output, "/tmp\n");
/// # }
/// ```
///
/// Paths that are relative to the parent's current directory are allowed.
impl<T> CmdArgument for CurrentDir<T>
where
    T: AsRef<Path>,
{
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.working_directory = Some(self.0.as_ref().to_owned());
    }
}
