pub use crate::{
    error::{panic_on_error, Error},
    input::{CurrentDir, Input, LogCommand, Split, Stdin},
    output::{Exit, Output, Stderr, StdoutTrimmed, StdoutUntrimmed},
};

/// Execute child processes. See the module documentation on how to use it.
#[macro_export]
macro_rules! _cmd {
    ($($args:tt)*) => {{
        let context = $crate::Context::production();
        $crate::prelude::panic_on_error($crate::cmd_result_with_context!(context, $($args)*))
    }}
}
pub use _cmd as cmd;

/// Like [`cmd!`], but fixes the return type to `()`.
#[macro_export]
macro_rules! _cmd_unit {
    ($($args:tt)*) => {{
        let () = $crate::_cmd!($($args)*);
    }}
}
pub use _cmd_unit as cmd_unit;

/// Like [`cmd!`], but fixes the return type to [`Result<T, Error>`],
/// where `T` is any type that implements [`Output`].
#[macro_export]
macro_rules! _cmd_result {
    ($($args:tt)*) => {{
        let context = $crate::Context::production();
        $crate::cmd_result_with_context!(context, $($args)*)
    }}
}
pub use _cmd_result as cmd_result;
