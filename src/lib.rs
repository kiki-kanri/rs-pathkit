//! Owned filesystem paths with synchronous and optional asynchronous file operations.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async-fs-ops")]
pub mod async_fs_ops;
mod atomic_write;
mod macros;
mod path;
pub mod sync_fs_ops;

pub use self::path::Path;
