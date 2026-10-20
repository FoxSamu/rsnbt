pub(crate) mod ring_buf;

#[cfg(any(
    feature = "io",
    feature = "region",
    feature = "lz4b",
))]
pub(crate) mod rw;

#[macro_use]
pub(crate) mod unsafe_ops;
