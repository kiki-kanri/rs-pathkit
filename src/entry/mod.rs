//! Directory entry wrappers for pathkit-fluent directory traversal.

#[cfg(feature = "async-fs-ops")]
pub(crate) mod r#async;
pub(crate) mod sync;
