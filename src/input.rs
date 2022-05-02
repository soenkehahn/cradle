//! The [`Input`] trait that defines all possible inputs to a child process.

use crate::{
    child_output::ChildOutput,
    config::Config,
    context::Context,
    error::{panic_on_error, Error},
    output::Output,
};
use std::{
    ffi::{OsStr, OsString},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

/// All types that are possible arguments to [`run!`], [`run_output!`] or
/// [`run_result!`] must implement this trait.
/// This makes `cradle` very flexible.
/// For example you can pass in an executable as a String,
/// and a variable number of arguments as a [`Vec`]:
///
/// ```
/// use cradle::prelude::*;
///
/// let executable = "echo";
/// let arguments = vec!["foo", "bar"];
/// let StdoutUntrimmed(output) = run_output!(executable, arguments);
/// assert_eq!(output, "foo bar\n");
/// ```
///
/// For more documentation on all possible input types,
/// see the documentation for the individual impls of [`Input`].
/// Here's a non-exhaustive list of the most commonly used types to get you started:
///
/// - [`String`] and [`&str`],
/// - [`Split`] (and its shortcut `%`) to split commands by whitespace,
/// - [`PathBuf`] and [`&Path`],
/// - multiple sequence types, like [`vectors`], [`slices`] and (since version 1.51) [`arrays`],
/// - [`CurrentDir`],
/// - [`Env`] for setting environment variables,
/// - [`Stdin`], and
/// - [`LogCommand`].
///
/// [`String`]: trait.Input.html#impl-Input-for-String
/// [`&str`]: trait.Input.html#impl-Input-for-%26str
/// [`PathBuf`]: trait.Input.html#impl-Input-for-PathBuf
/// [`&Path`]: trait.Input.html#impl-Input-for-%26Path
/// [`vectors`]: trait.Input.html#impl-Input-for-Vec<T>
/// [`slices`]: trait.Input.html#impl-Input-for-%26[T]
/// [`arrays`]: trait.Input.html#impl-Input-for-[T%3B%20N]
///
/// ## Tuples
///
/// `cradle` also implements [`Input`] for tuples of types that themselves implement [`Input`].
/// Instead of passing multiple arguments to [`run!`], they can be passed in a single tuple:
///
/// ```
/// use cradle::prelude::*;
///
/// let args = ("echo", "foo");
/// let StdoutTrimmed(output) = run_output!(args);
/// assert_eq!(output, "foo");
/// ```
///
/// This can be used to group arguments:
///
/// ```
/// use cradle::prelude::*;
///
/// let to_hex_command = ("xxd", "-ps", "-u", LogCommand);
/// let StdoutTrimmed(output) = run_output!(to_hex_command, Stdin(&[14, 15, 16]));
/// assert_eq!(output, "0E0F10");
/// ```
///
/// Also, tuples make it possible to write wrappers around [`run!`] without requiring the use of macros:
///
/// ```
/// use cradle::prelude::*;
///
/// fn to_hex<I: Input>(input: I) -> String {
///   let StdoutTrimmed(hex) = run_output!(%"xxd -ps -u", input);
///   hex
/// }
///
/// // It works for slices:
/// let hex = to_hex(Stdin(&[14, 15, 16]));
/// assert_eq!(hex, "0E0F10");
///
/// // Vectors:
/// let hex = to_hex(Stdin(vec![14, 15, 16]));
/// assert_eq!(hex, "0E0F10");
///
/// // And multiple arguments using tuples:
/// let hex = to_hex((Stdin(&[14, 15, 16]), Stdin(&[17, 18, 19])));
/// assert_eq!(hex, "0E0F10111213");
/// ```
///
/// ## Custom [`Input`] impls
///
/// The provided `Input` implementations should be sufficient for most use cases,
/// but custom `Input` implementations can be written to extend `cradle`.
///
/// Here's an example of an `Environment` type, that wraps
/// [`BTreeMap`](std::collections::BTreeMap) and adds all contained
/// key-value pairs to the environment of the child process.
///
/// ```
/// use cradle::prelude::*;
/// use cradle::config::Config;
/// use std::collections::BTreeMap;
///
/// struct Environment(BTreeMap<String, String>);
///
/// impl Environment {
///     fn new() -> Self {
///         Environment(BTreeMap::new())
///     }
///
///     fn add(mut self, key: &str, value: &str) -> Self {
///         self.0.insert(key.to_owned(), value.to_owned());
///         self
///     }
/// }
///
/// impl Input for Environment {
///     fn configure(self, config: &mut Config) {
///         for (key, value) in self.0.into_iter() {
///             Env(key, value).configure(config)
///         }
///     }
/// }
///
/// let env_vars = Environment::new()
///     .add("FOO", "foo")
///     .add("BAR", "bar");
///
/// let StdoutUntrimmed(output) = run_output!("env", env_vars);
/// assert!(output.contains("FOO=foo\n"));
/// assert!(output.contains("BAR=bar\n"));
/// ```
///
/// It is not recommended to override [`run`](Input::run),
/// [`run_output`](Input::run_output) or [`run_result`](Input::run_result).
///
/// Also note that all fields of the type [`Config`] are private.
/// That means that when you're writing your own [`Input`] impls,
/// you _have_ to implement the [`Input::configure`] method
/// of your type in terms of the [`Input::configure`] methods
/// of the various [`Input`] types that `cradle` provides --
/// as demonstrated in the code snippet above.
/// [`Config`]'s fields are private to allow to add new features to `cradle`
/// without introducing breaking API changes.
pub trait Input: Sized {
    /// Configures the given [`Config`](crate::config::Config) for the [`Input`] `self`.
    /// Usually you won't have to write your own custom impls for [`Input`],
    /// nor call this function yourself.
    /// So you can safely ignore this method.
    ///
    /// See also [Custom `Input` impls](#custom-input-impls).
    fn configure(self, config: &mut Config);

    /// `input.run()` runs `input` as a child process.
    /// It's equivalent to `run!(input)`.
    ///
    /// ```
    /// # let temp_dir = tempfile::TempDir::new().unwrap();
    /// # std::env::set_current_dir(&temp_dir).unwrap();
    /// use cradle::prelude::*;
    ///
    /// ("touch", "foo").run();
    /// ```
    #[rustversion::attr(since(1.46), track_caller)]
    fn run(self) {
        self.run_output()
    }

    /// `input.run()` runs `input` as a child process.
    /// It's equivalent to `run_output!(input)`.
    ///
    /// ```
    /// use cradle::prelude::*;
    ///
    /// let StdoutTrimmed(output) = ("echo", "foo").run_output();
    /// assert_eq!(output, "foo");
    /// ```
    #[rustversion::attr(since(1.46), track_caller)]
    fn run_output<O>(self) -> O
    where
        O: Output,
    {
        panic_on_error(self.run_result())
    }

    /// `input.run_result()` runs `input` as a child process.
    /// It's equivalent to `run_result!(input)`.
    ///
    /// ```
    /// use cradle::prelude::*;
    ///
    /// # fn build() -> Result<(), Error> {
    /// // make sure build tools are installed
    /// run_result!(%"which make")?;
    /// run_result!(%"which gcc")?;
    /// run_result!(%"which ld")?;
    /// run_result!(%"make build")?;
    /// # Ok(())
    /// # }
    /// ```
    fn run_result<O>(self) -> Result<O, crate::error::Error>
    where
        O: Output,
    {
        let context = Context::production();
        run_result_with_context(context, self)
    }
}

pub(crate) fn run_result_with_context<Stdout, Stderr, I, O>(
    context: Context<Stdout, Stderr>,
    input: I,
) -> Result<O, Error>
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
    I: Input,
    O: Output,
{
    let mut config = Config::default();
    input.configure(&mut config);
    ChildOutput::run_child_process_output(context, config)
}

#[cfg(test)]
pub(crate) fn run_result_with_context_unit<Stdout, Stderr, I>(
    context: Context<Stdout, Stderr>,
    input: I,
) -> Result<(), Error>
where
    Stdout: Write + Clone + Send + 'static,
    Stderr: Write + Clone + Send + 'static,
    I: Input,
{
    run_result_with_context(context, input)
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
/// use cradle::prelude::*;
///
/// run!("ls", std::env::var_os("HOME").unwrap());
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
/// use cradle::prelude::*;
///
/// run!("echo", std::env::current_dir().unwrap().file_name().unwrap());
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
/// use cradle::prelude::*;
///
/// let StdoutTrimmed(output) = run_output!("echo", "foo");
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
/// use cradle::prelude::*;
///
/// let executable: String = "echo".to_string();
/// let argument: String = "foo".to_string();
/// let StdoutTrimmed(output) = run_output!(executable, argument);
/// assert_eq!(output, "foo");
/// ```
impl Input for String {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        OsString::from(self).configure(config);
    }
}

/// Splits the contained string by whitespace (using [`split_whitespace`])
/// and uses the resulting words as separate arguments.
///
/// ```
/// use cradle::prelude::*;
///
/// let StdoutTrimmed(output) = run_output!(Split("echo foo"));
/// assert_eq!(output, "foo");
/// ```
///
/// Since this is such a common case, `cradle` also provides a syntactic shortcut
/// for [`Split`], the `%` symbol:
///
/// ```
/// use cradle::prelude::*;
///
/// let StdoutTrimmed(output) = run_output!(%"echo foo");
/// assert_eq!(output, "foo");
/// ```
///
/// [`split_whitespace`]: str::split_whitespace
#[derive(Debug, PartialEq, Clone)]
pub struct Split(pub &'static str);

impl Input for crate::input::Split {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        for argument in self.0.split_whitespace() {
            argument.configure(config);
        }
    }
}

/// Allows to use [`split`] to split your argument into words:
///
/// ```
/// use cradle::prelude::*;
///
/// let StdoutTrimmed(output) = run_output!("echo foo".split(' '));
/// assert_eq!(output, "foo");
/// ```
///
/// Arguments to [`split`] must be of type [`char`].
///
/// [`split`]: str::split
impl Input for std::str::Split<'static, char> {
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
/// use cradle::prelude::*;
///
/// let StdoutTrimmed(output) = run_output!("echo foo".split_whitespace());
/// assert_eq!(output, "foo");
/// ```
///
/// [`split_whitespace`]: str::split_whitespace
impl Input for std::str::SplitWhitespace<'static> {
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
/// use cradle::prelude::*;
///
/// let StdoutTrimmed(output) = run_output!("echo foo".split_ascii_whitespace());
/// assert_eq!(output, "foo");
/// ```
///
/// [`split_ascii_whitespace`]: str::split_ascii_whitespace
impl Input for std::str::SplitAsciiWhitespace<'static> {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        for word in self {
            word.configure(config);
        }
    }
}

impl Input for () {
    #[doc(hidden)]
    fn configure(self, _: &mut Config) {}
}

macro_rules! tuple_impl {
    ($($index:tt, $generics:ident,)+) => {
        impl<$($generics),+> Input for ($($generics,)+)
        where
            $($generics: Input,)+
        {
            #[doc(hidden)]
            fn configure(self, config: &mut Config) {
                $(<$generics as Input>::configure(self.$index, config);)+
            }
        }
    };
}

tuple_impl!(0, A,);
tuple_impl!(0, A, 1, B,);
tuple_impl!(0, A, 1, B, 2, C,);
tuple_impl!(0, A, 1, B, 2, C, 3, D,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E, 5, F,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H, 8, I,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H, 8, I, 9, J,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H, 8, I, 9, J, 10, K,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H, 8, I, 9, J, 10, K, 11, L,);
tuple_impl!(0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H, 8, I, 9, J, 10, K, 11, L, 12, M,);
tuple_impl!(
    0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H, 8, I, 9, J, 10, K, 11, L, 12, M, 13, N,
);
tuple_impl!(
    0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H, 8, I, 9, J, 10, K, 11, L, 12, M, 13, N, 14, O,
);
tuple_impl!(
    0, A, 1, B, 2, C, 3, D, 4, E, 5, F, 6, G, 7, H, 8, I, 9, J, 10, K, 11, L, 12, M, 13, N, 14, O,
    15, P,
);

/// All elements of the given [`Vec`] are used as arguments to the child process.
/// Same as passing in the elements separately.
///
/// ```
/// use cradle::prelude::*;
///
/// let StdoutTrimmed(output) = run_output!(vec!["echo", "foo"]);
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
/// use cradle::prelude::*;
///
/// let StdoutTrimmed(output) = run_output!(["echo", "foo"]);
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
        #[rustversion::before(1.59)]
        fn array_to_iter<T, const N: usize>(array: [T; N]) -> impl Iterator<Item = T> {
            std::array::IntoIter::new(array)
        }
        #[rustversion::since(1.59)]
        fn array_to_iter<T, const N: usize>(array: [T; N]) -> impl Iterator<Item = T> {
            IntoIterator::into_iter(array)
        }

        for t in array_to_iter(self) {
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

/// Passing in [`LogCommand`] as an argument to `cradle` will cause it
/// to log the commands (including all arguments) to `stderr`.
/// (This is similar `bash`'s `-x` option.)
///
/// ```
/// use cradle::prelude::*;
///
/// run!(LogCommand, %"echo foo");
/// // writes '+ echo foo' to stderr
/// ```
#[derive(Debug, Clone, Copy)]
pub struct LogCommand;

impl Input for LogCommand {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        config.log_command = true;
    }
}

/// By default child processes inherit the current directory from their
/// parent. You can override this with [`CurrentDir`]:
///
/// ```
/// use cradle::prelude::*;
///
/// # #[cfg(target_os = "linux")]
/// # {
/// let StdoutTrimmed(output) = run_output!("pwd", CurrentDir("/tmp"));
/// assert_eq!(output, "/tmp");
/// # }
/// ```
///
/// Paths that are relative to the parent's current directory are allowed.
#[derive(Debug, Clone)]
pub struct CurrentDir<T: AsRef<Path>>(pub T);

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
/// use cradle::prelude::*;
/// use std::path::PathBuf;
///
/// let current_dir: PathBuf = std::env::current_dir().unwrap();
/// run!("ls", current_dir);
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
/// # let temp_dir = tempfile::TempDir::new().unwrap();
/// # std::env::set_current_dir(&temp_dir).unwrap();
/// use cradle::prelude::*;
/// use std::path::Path;
///
/// let file: &Path = Path::new("./foo");
/// run!("touch", file);
/// ```
///
/// [`&Path`]: std::path::Path
impl Input for &Path {
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        self.as_os_str().to_os_string().configure(config);
    }
}

/// Writes the given byte slice to the child's standard input.
///
/// ```
/// use cradle::prelude::*;
///
/// # #[cfg(target_os = "linux")]
/// # {
/// let StdoutUntrimmed(output) = run_output!("sort", Stdin("foo\nbar\n"));
/// assert_eq!(output, "bar\nfoo\n");
/// # }
/// ```
///
/// If `Stdin` is used multiple times, all given bytes slices will be written
/// to the child's standard input in order.
#[derive(Debug, Clone)]
pub struct Stdin<T: AsRef<[u8]>>(pub T);

impl<T> Input for Stdin<T>
where
    T: AsRef<[u8]>,
{
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        Arc::make_mut(&mut config.stdin).extend_from_slice(self.0.as_ref());
    }
}

/// Adds an environment variable to the environment of the child process.
///
/// ```
/// use cradle::prelude::*;
///
/// let StdoutUntrimmed(output) = run_output!("env", Env("FOO", "bar"));
/// assert!(output.contains("FOO=bar\n"));
/// ```
///
/// Child processes inherit the environment of the parent process.
/// [`Env`] only adds environment variables to that inherited environment.
/// If the environment variable is also set in the parent process,
/// it is overwritten by [`Env`].
#[derive(Debug, Clone)]
pub struct Env<Key, Value>(pub Key, pub Value)
where
    Key: AsRef<OsStr>,
    Value: AsRef<OsStr>;

impl<Key, Value> Input for Env<Key, Value>
where
    Key: AsRef<OsStr>,
    Value: AsRef<OsStr>,
{
    #[doc(hidden)]
    fn configure(self, config: &mut Config) {
        let Self(key, value) = self;
        config
            .added_environment_variables
            .push((key.as_ref().to_os_string(), value.as_ref().to_os_string()));
    }
}
