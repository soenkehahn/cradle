use crate::config::Config;
use std::path::Path;

/// All types that are possible arguments to [`cmd!`] have to implement this trait.
pub trait CmdArgument {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config);
}

impl<T> CmdArgument for &T
where
    T: CmdArgument + Clone,
{
    fn prepare_config(self, config: &mut Config) {
        <T as CmdArgument>::prepare_config(self.clone(), config);
    }
}

/// Arguments of type [`&str`] are passed into the child process
/// as arguments.
impl CmdArgument for &str {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.arguments.push(self.to_string());
    }
}

/// Arguments of type [`String`] are passed into the child process
/// as arguments.
impl CmdArgument for String {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.arguments.push(self);
    }
}

/// See the [`CmdArgument`] implementation for [`Split`] below.
pub struct Split<'a>(pub &'a str);

/// Splits the contained string by whitespace and passes the words into
/// the child process as arguments.
impl<'a> CmdArgument for Split<'a> {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for argument in self.0.split_whitespace() {
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
/// use cradle::*;
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
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!(["echo", "foo"]);
/// assert_eq!(output, "foo");
/// ```
#[rustversion::since(1.51)]
impl<const N: usize> CmdArgument for [&str; N] {
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
#[derive(Clone, Debug)]
pub struct LogCommand;

/// Passing in [`LogCommand`] as an argument to [`cmd!`] will cause it
/// to log the commands (including all arguments) to `stderr`.
/// (This is similar `bash`'s `-x` option.)
///
/// ```
/// use cradle::*;
///
/// cmd_unit!(LogCommand, Split("echo foo"));
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
/// use cradle::*;
///
/// # #[cfg(target_os = "linux")]
/// # {
/// let StdoutTrimmed(output) = cmd!("pwd", CurrentDir("/tmp"));
/// assert_eq!(output, "/tmp");
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
