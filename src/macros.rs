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
        $crate::input::Input::run($crate::tuple_up!($($args)*))
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
      $crate::input::Input::run_output($crate::tuple_up!($($args)*))
    }}
}

/// Like [`run_output!`], but fixes the return type to [`Result<T, Error>`],
/// where `T` is any type that implements [`Output`](crate::output::Output).
#[macro_export]
macro_rules! run_result {
    ($($args:tt)*) => {{
        $crate::input::Input::run_result($crate::tuple_up!($($args)*))
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! tuple_up {
    (% $last:expr $(,)?) => {
        $crate::input::Split($last)
    };
    ($last:expr $(,)?) => {
        $last
    };
    (% $head:expr, $($tail:tt)*) => {
        ($crate::input::Split($head), $crate::tuple_up!($($tail)*))
    };
    ($head:expr, $($tail:tt)*) => {
        ($head, $crate::tuple_up!($($tail)*))
    };
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    mod tuple_up {
        use super::*;

        #[test]
        #[allow(clippy::eq_op)]
        fn one_value() {
            assert_eq!(tuple_up!(1), 1);
        }

        #[test]
        fn two_values() {
            assert_eq!(tuple_up!(1, 2), (1, 2));
        }

        #[test]
        fn three_values() {
            assert_eq!(tuple_up!(1, 2, 3), (1, (2, 3)));
        }

        #[test]
        fn nested_tuples() {
            assert_eq!(tuple_up!(1, (2, 3), 4), (1, ((2, 3), 4)));
        }

        #[test]
        fn percent_shortcut() {
            assert_eq!(tuple_up!(%"foo"), Split("foo"));
        }

        #[test]
        fn percent_shortcut_with_subsequent_values() {
            assert_eq!(tuple_up!(%"foo", "bar"), (Split("foo"), "bar"));
        }

        #[test]
        fn percent_shortcut_with_preceeding_values() {
            assert_eq!(tuple_up!("foo", %"bar"), ("foo", Split("bar")));
        }

        #[test]
        fn percent_shortcut_with_multiple_values() {
            assert_eq!(
                tuple_up!(%"foo", "bar", %"baz"),
                (Split("foo"), ("bar", Split("baz")))
            );
        }
    }
}
