//! `cradle`'s `prelude` module.
//! It re-exports the most commonly used items from cradle.
//! We recommend importing cradle like this:
//! `use cradle::prelude::*;`
//!
//! For documentation about how to use cradle,
//! see the documentation in the [crate root](crate).

include!("common_re_exports.rs.snippet");
pub use crate::{run, run_output, run_result};
