extern crate core;

pub mod tag;
pub mod io;

#[cfg(feature = "snbt")]
pub mod snbt;

pub(crate) mod util;

mod unsafe_ops;
