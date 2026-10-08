//! Directory entries with owned [`Path`](crate::Path) accessors.

#[cfg(feature = "async-fs-ops")]
pub(crate) mod r#async;
pub(crate) mod sync;
