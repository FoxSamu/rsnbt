extern crate core;

pub mod tag;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "snbt")]
pub mod snbt;

#[cfg(feature = "lz4b")]
pub mod lz4b;

#[cfg(feature = "region")]
pub mod region;

#[cfg(feature = "bitpack")]
pub mod bitpack;

#[macro_use]
pub(crate) mod util;

