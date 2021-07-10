use crate::config::Config;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::Arc,
};

/// All types that are possible arguments to [`cmd!`], [`cmd_unit!`] or
/// [`cmd_result!`] have to implement this trait.
/// This makes `cradle` very flexible.
/// For example passing in an executable can be done as a String,
/// and passing in a variable number of arguments can be done in a [`Vec`]:
///
/// ```
/// use cradle::*;
///
/// let executable = "echo";
/// let arguments = vec!["foo", "bar"];
/// let StdoutUntrimmed(output) = cmd!(executable, arguments);
/// assert_eq!(output, "foo bar\n");
/// ```
///
/// For more documentation on all possible input types,
/// see the documentation for the individual impls of [`Input`].
/// Here's a non-exhaustive list of the most commonly used types to get you started:
///
/// - [`String`] and [`&str`],
/// - [`Split`] (and its shortcut `%`),
/// - [`PathBuf`] and [`&Path`],
/// - multiple sequence types, like [`vectors`], [`slices`] and (since version 1.51) [`arrays`],
/// - [`CurrentDir`],
/// - [`StdIn`], and
/// - [`LogCommand`].
///
/// [`String`]: trait.Input.html#impl-Input-for-String
/// [`&str`]: trait.Input.html#impl-Input-for-%26str
/// [`Split`]: trait.Input.html#impl-Input-3
/// [`PathBuf`]: trait.Input.html#impl-Input-for-PathBuf
/// [`&Path`]: trait.Input.html#impl-Input-for-%26Path
/// [`vectors`]: trait.Input.html#impl-Input-for-Vec<T>
/// [`slices`]: trait.Input.html#impl-Input-for-%26[T]
/// [`arrays`]: trait.Input.html#impl-Input-for-[T%3B%20N]
/// [`CurrentDir`]: trait.Input.html#impl-Input-1
/// [`StdIn`]: trait.Input.html#impl-Input-2
/// [`LogCommand`]: trait.Input.html#impl-Input
pub trait Input {
    #[doc(hidden)]
    fn configure(self, config: &mut Config);
}

/// Blanket implementation for `&_`.
impl<T> Input for &T
where
    T: Input + Clone,
{
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        self.clone().configure(config);
    }
}

/// Arguments of type [`OsString`] are passed to the child process
/// as arguments.
///
/// ```
/// use cradle::*;
///
/// cmd_unit!("ls", std::env::var_os("HOME").unwrap());
/// ```
impl Input for OsString {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        config.arguments.push(self);
    }
}

/// Arguments of type [`&OsStr`] are passed to the child process
/// as arguments.
///
/// ```
/// use cradle::*;
///
/// cmd_unit!("echo", std::env::current_dir().unwrap().file_name().unwrap());
/// ```
///
/// [`&OsStr`]: std::ffi::OsStr
impl Input for &OsStr {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        self.to_os_string().configure(config);
    }
}

/// Arguments of type [`&str`] are passed to the child process as arguments.
/// This is especially useful because it allows you to use string literals:
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!("echo", "foo");
/// assert_eq!(output, "foo");
/// ```
impl Input for &str {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        OsStr::new(self).configure(config);
    }
}

/// Arguments of type [`String`] are passed to the child process
/// as arguments. Executables can also be passed as [`String`]s:
///
/// ```
/// use cradle::*;
///
/// let executable: String = "echo".to_string();
/// let argument: String = "foo".to_string();
/// let StdoutTrimmed(output) = cmd!(executable, argument);
/// assert_eq!(output, "foo");
/// ```
impl Input for String {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        OsString::from(self).configure(config);
    }
}

/// See the [`Input`] implementation for [`Split`] below.
pub struct Split<T: AsRef<str>>(pub T);

/// Splits the contained string by whitespace (using [`split_whitespace`])
/// and uses the resulting words as separate arguments.
///
/// ```
/// use cradle::*;
///
/// let StdoutTrimmed(output) = cmd!(Split("echo foo"));
/// assert_eq!(output, "foo");
///
/// let StdoutTrimmed(output) = cmd!(Split(format!("echo {}", 100)));
/// assert_eq!(output, "100");
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
impl<T: AsRef<str>> Input for Split<T> {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        for argument in self.0.as_ref().split_whitespace() {
            argument.configure(config);
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
impl<'a> Input for std::str::Split<'a, char> {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        for word in self {
            word.configure(config);
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
impl<'a> Input for std::str::SplitWhitespace<'a> {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        for word in self {
            word.configure(config);
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
impl<'a> Input for std::str::SplitAsciiWhitespace<'a> {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        for word in self {
            word.configure(config);
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
impl<T> Input for Vec<T>
where
    T: Input,
{
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        for t in self.into_iter() {
            t.configure(config);
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
impl<T, const N: usize> Input for [T; N]
where
    T: Input,
{
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        for t in std::array::IntoIter::new(self) {
            t.configure(config);
        }
    }
}

/// Similar to the implementation for [`Vec<T>`].
/// All elements of the slice will be used as arguments.
impl<T> Input for &[T]
where
    T: Input + Clone,
{
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        self.to_vec().configure(config);
    }
}

/// See the [`Input`] implementation for [`LogCommand`] below.
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
impl Input for LogCommand {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        config.log_command = true;
    }
}

/// See the [`Input`] implementation for [`CurrentDir`] below.
pub struct CurrentDir<T: AsRef<Path>>(pub T);

/// By default child processes inherit the current directory from their
/// parent. You can override this with [`CurrentDir`]:
///
/// ```
/// use cradle::*;
///
/// # #[cfg(linux)]
/// # {
/// let StdoutTrimmed(output) = cmd!("pwd", CurrentDir("/tmp"));
/// assert_eq!(output, "/tmp");
/// # }
/// ```
///
/// Paths that are relative to the parent's current directory are allowed.
impl<T> Input for CurrentDir<T>
where
    T: AsRef<Path>,
{
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
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
impl Input for PathBuf {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        self.into_os_string().configure(config);
    }
}

/// Arguments of type [`&Path`] are passed to the child process
/// as arguments.
///
/// ```
/// use cradle::*;
/// use std::path::Path;
///
/// let file: &Path = Path::new("./foo");
/// cmd_unit!("touch", file);
/// ```
///
/// [`&Path`]: std::path::Path
impl Input for &Path {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        self.as_os_str().to_os_string().configure(config);
    }
}

/// See the [`Input`] implementation for [`Stdin`] below.
pub struct Stdin<T: AsRef<[u8]>>(pub T);

/// Writes the given byte slice to the child's standard input.
///
/// ```
/// use cradle::*;
///
/// # #[cfg(linux)]
/// # {
/// let StdoutUntrimmed(output) = cmd!("sort", Stdin("foo\nbar\n"));
/// assert_eq!(output, "bar\nfoo\n");
/// # }
/// ```
///
/// If `Stdin` is used multiple times, all given bytes slices will be written
/// to the child's standard input in order.
impl<T> Input for Stdin<T>
where
    T: AsRef<[u8]>,
{
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        Arc::make_mut(&mut config.stdin).extend_from_slice(self.0.as_ref());
    }
}
