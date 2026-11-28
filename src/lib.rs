//! Owned filesystem paths with synchronous and optional asynchronous file operations.
//!
//! [`Path`] wraps [`std::path::PathBuf`] and dereferences to [`std::path::Path`];
//! [`sync_fs_ops::SyncFsOps`] and, with `async-fs-ops`, `async_fs_ops::AsyncFsOps` add filesystem
//! operations on top of it.
//!
//! ```rust
//! use pathkit::{
//!     path,
//!     sync_fs_ops::SyncFsOps,
//! };
//!
//! let directory = tempfile::tempdir()?;
//! let file = path!(directory.path()) / "message.txt";
//! file.write_sync(b"Hello!")?;
//! assert_eq!(file.read_to_string_sync()?, "Hello!");
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Features
//!
//! No optional feature is enabled by default; serde support and synchronous operations are always
//! available.
//!
//! - `async-fs-ops`: the `async_fs_ops` module, backed by Tokio; calls need a Tokio runtime.
//! - `full`: every optional feature.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async-fs-ops")]
pub mod async_fs_ops;
mod atomic_write;
mod macros;
mod path;
pub mod sync_fs_ops;

pub use self::path::Path;
