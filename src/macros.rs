/// Execute child processes. See the module documentation on how to use it.
#[macro_export]
macro_rules! cmd {
    ($($args:tt)*) => {{
        let context = $crate::context::Context::production();
        $crate::error::panic_on_error($crate::cmd_result_with_context!(context, $($args)*))
    }}
}

/// Like [`cmd!`], but fixes the return type to `()`.
/// It's named after [the unit type `()`](https://doc.rust-lang.org/std/primitive.unit.html).
///
/// ```
/// # let temp_dir = tempfile::TempDir::new().unwrap();
/// # std::env::set_current_dir(&temp_dir).unwrap();
/// use cradle::prelude::*;
///
/// cmd_unit!(%"touch ./foo");
/// ```
#[macro_export]
macro_rules! cmd_unit {
    ($($args:tt)*) => {{
        let () = $crate::cmd!($($args)*);
    }}
}

/// Like [`cmd!`], but fixes the return type to [`Result<T, Error>`],
/// where `T` is any type that implements [`Output`](crate::output::Output).
#[macro_export]
macro_rules! cmd_result {
    ($($args:tt)*) => {{
        let context = $crate::context::Context::production();
        $crate::cmd_result_with_context!(context, $($args)*)
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! cmd_result_with_context {
    ($context:expr, $($args:tt)*) => {{
        let mut config = $crate::config::Config::default();
        $crate::configure!(config: config, args: $($args)*);
        $crate::run_result::RunResult::run_cmd($context, config)
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
