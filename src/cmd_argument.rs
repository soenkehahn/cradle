use crate::config::Config;
use std::path::{Path, PathBuf};

/// All types that are possible arguments to [`cmd!`] have to implement this trait.
pub trait CmdArgument {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config);
}

/// Blanket implementation for `&_`.
impl<T> CmdArgument for &T
where
    T: CmdArgument + Clone,
{
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        self.clone().prepare_config(config);
    }
}

/// Arguments of type [`&str`] are passed to the child process
/// as arguments.
impl CmdArgument for &str {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.arguments.push(self.into());
    }
}

/// Arguments of type [`String`] are passed to the child process
/// as arguments.
impl CmdArgument for String {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.arguments.push(self.into());
    }
}

/// See the [`CmdArgument`] implementation for [`Split`] below.
pub struct Split<'a>(pub &'a str);

/// Splits the contained string by whitespace (using [`split_whitespace`])
/// and uses the resulting words as separate arguments.
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!(Split("echo foo"));
/// assert_eq!(output, "foo");
/// ```
///
/// Since this is such a common case, `cradle` also provides a syntactic shortcut
/// for [`Split`], the `%` symbol:
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!(%"echo foo");
/// assert_eq!(output, "foo");
/// ```
///
/// [`split_whitespace`]: str::split_whitespace
impl<'a> CmdArgument for Split<'a> {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for argument in self.0.split_whitespace() {
            argument.prepare_config(config);
        }
    }
}

/// Allows to use [`split`] to split your argument into words:
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!("echo foo".split(' '));
/// assert_eq!(output, "foo");
/// ```
///
/// Arguments to [`split`] must be of type [`char`].
///
/// [`split`]: str::split
impl<'a> CmdArgument for std::str::Split<'a, char> {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for word in self {
            word.prepare_config(config);
        }
    }
}

/// Allows to use [`split_whitespace`] to split your argument into words:
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!("echo foo".split_whitespace());
/// assert_eq!(output, "foo");
/// ```
///
/// [`split_whitespace`]: str::split_whitespace
impl<'a> CmdArgument for std::str::SplitWhitespace<'a> {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for word in self {
            word.prepare_config(config);
        }
    }
}

/// Allows to use [`split_ascii_whitespace`] to split your argument into words:
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!("echo foo".split_ascii_whitespace());
/// assert_eq!(output, "foo");
/// ```
///
/// [`split_ascii_whitespace`]: str::split_ascii_whitespace
impl<'a> CmdArgument for std::str::SplitAsciiWhitespace<'a> {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for word in self {
            word.prepare_config(config);
        }
    }
}

/// All elements of the given [`Vec`] are used as arguments to [`cmd!`].
/// Same as passing in the elements separately.
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!(vec!["echo", "foo"]);
/// assert_eq!(output, "foo");
/// ```
impl<T> CmdArgument for Vec<T>
where
    T: CmdArgument,
{
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for t in self.into_iter() {
            t.prepare_config(config);
        }
    }
}

/// Similar to the implementation for [`Vec<T>`].
/// All elements of the array will be used as arguments.
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!(["echo", "foo"]);
/// assert_eq!(output, "foo");
/// ```
///
/// Only works on rust version `1.51` and up.
#[rustversion::since(1.51)]
impl<T, const N: usize> CmdArgument for [T; N]
where
    T: CmdArgument,
{
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        for t in std::array::IntoIter::new(self) {
            t.prepare_config(config);
        }
    }
}

/// Similar to the implementation for [`Vec<T>`].
/// All elements of the slice will be used as arguments.
impl<T> CmdArgument for &[T]
where
    T: CmdArgument + Clone,
{
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        self.to_vec().prepare_config(config);
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
/// cmd_unit!(LogCommand, %"echo foo");
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

/// Arguments of type [`PathBuf`] are passed to the child process
/// as arguments.
///
/// ```
/// use cradle::*;
/// use std::path::PathBuf;
///
/// let current_dir: PathBuf = std::env::current_dir().unwrap();
/// cmd_unit!("ls", current_dir);
/// ```
impl CmdArgument for PathBuf {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        config.arguments.push(self.into());
    }
}

/// Arguments of type [`&Path`] are passed to the child process
/// as arguments.
///
/// Similar to the impl for [`PathBuf`].
///
/// [`&Path`]: std::path::Path
impl CmdArgument for &Path {
    #[doc(hidden)]
    fn prepare_config(self, config: &mut Config) {
        self.to_path_buf().prepare_config(config);
    }
}
