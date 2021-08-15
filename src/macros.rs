/// Executes a child process without capturing any output.
///
/// ```
/// # let temp_dir = tempfile::TempDir::new().unwrap();
/// # std::env::set_current_dir(&temp_dir).unwrap();
/// use cradle::prelude::*;
///
/// run!(%"touch ./foo");
/// ```
///
/// If an error occurs, `run!` will panic.
/// See [`crate::error::Error`] for possible errors.
///
/// For capturing output from child processes, see [`crate::run_output!`].
#[macro_export]
macro_rules! run {
    ($($args:tt)*) => {{
        let () = $crate::run_output!($($args)*);
    }}
}

/// Execute child processes, and capture some output.
/// For example you can capture what the child process writes to stdout:
///
/// ```
/// use cradle::prelude::*;
///
/// let StdoutUntrimmed(output) = run_output!(%"echo foo");
/// assert_eq!(output, "foo\n");
/// ```
///
/// [`run_output!`] uses return-type polymorphism.
/// So by using a different return type,
/// you can control what outputs of child processes you want to capture.
/// Here's an example to capture an exit code:
///
/// ```
/// use cradle::prelude::*;
///
/// let Status(status) = run_output!("false");
/// assert_eq!(status.code(), Some(1));
/// ```
///
/// You can use any type that implements [`crate::output::Output`] as the return type.
/// See the module documentation for more comprehensive documentation.
#[macro_export]
macro_rules! run_output {
    ($($args:tt)*) => {{
        let context = $crate::context::Context::production();
        $crate::error::panic_on_error($crate::run_result_with_context!(context, $($args)*))
    }}
}

/// Like [`run_output!`], but fixes the return type to [`Result<T, Error>`],
/// where `T` is any type that implements [`Output`](crate::output::Output).
#[macro_export]
macro_rules! run_result {
    ($($args:tt)*) => {{
        let context = $crate::context::Context::production();
        $crate::run_result_with_context!(context, $($args)*)
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! run_result_with_context {
    ($context:expr, $($args:tt)*) => {{
        let mut config = $crate::config::Config::default();
        $crate::configure!(config: config, args: $($args)*);
        $crate::child_output::ChildOutput::run_child_process_output($context, config)
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! configure {
    (config: $config:ident, args: % $head:expr $(,)?) => {
        $crate::input::Input::configure($crate::input::Split($head), &mut $config);
    };
    (config: $config:ident, args: $head:expr $(,)?) => {
        $crate::input::Input::configure($head, &mut $config);
    };
    (config: $config:ident, args: % $head:expr, $($tail:tt)*) => {
        $crate::input::Input::configure($crate::input::Split($head), &mut $config);
        $crate::configure!(config: $config, args: $($tail)*);
    };
    (config: $config:ident, args: $head:expr, $($tail:tt)*) => {
        $crate::input::Input::configure($head, &mut $config);
        $crate::configure!(config: $config, args: $($tail)*);
    };
}
