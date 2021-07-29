//! Cradle's `prelude` module.
//! It re-exports the most commonly used items from cradle.
//! We recommend importing cradle like this:
//! `use cradle::prelude::*;`
//!
//! For documentation about how to use cradle,
//! see the documentation in the [crate root](crate).

pub use crate::{
    cmd, cmd_result, cmd_unit,
    error::Error,
    input::{CurrentDir, Env, Input, LogCommand, Split, Stdin},
    output::{Output, Status, Stderr, StdoutTrimmed, StdoutUntrimmed},
};
