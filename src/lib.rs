//! Owned filesystem paths with synchronous and optional asynchronous file operations.
//!
//! [`Path`] retains platform-native path data and dereferences to [`std::path::Path`].
//! Lexical transformations return new paths; [`SyncFsOps`] performs blocking filesystem
//! operations without updating the stored path. The [`path!`] macro supports direct
//! path expressions and formatted string literals. Directory entries expose owned
//! paths through [`PathEntry`].
//!
//! # Getting started
//!
//! ```rust
//! use pathkit::path;
//!
//! let root = path!("project");
//! let config = &root / "config" / "app.json";
//! assert_eq!(config, root.join("config").join("app.json"));
//! assert_eq!(root, path!("project"));
//! ```
//!
//! # Features
//!
//! No optional features are enabled by default. Serde support and synchronous operations
//! are always available.
//!
//! - `async-fs-ops` enables `AsyncFsOps` and `AsyncPathEntry`, adding Tokio and `async-trait`.
//!   Async filesystem operations require a Tokio runtime with blocking-task support.
//!   Dropping a future does not roll back completed changes or necessarily stop submitted I/O.
//! - `sea-orm` adds [SeaORM](https://www.sea-ql.org/SeaORM/) model-field and value conversions.
//!   Paths are stored as strings;
//!   writing a non-Unicode path replaces invalid Unicode with the replacement character.
//! - `all` enables both optional features.
//! - `full` enables `all`.
//!
//! Enable a feature with `cargo add pathkit --features async-fs-ops`, replacing the feature
//! name as needed. Serde delegates to [`std::path::PathBuf`]; serialization fails for
//! native paths that are not valid UTF-8, rather than replacing invalid Unicode.
//!
//! # Platform support
//!
//! Path parsing, joining, and ordering follow the standard library's platform-specific
//! semantics. Unix permission modes, ownership operations, and special-file checks are
//! available only on Unix. Synchronous hard links are available on supported platforms;
//! the synchronous symbolic-link helpers are Unix-only. Filesystem permissions, link
//! behavior, and rename restrictions depend on the operating system and filesystem.
//!
//! # Examples
//!
//! The following example creates and removes only a temporary directory. Write and
//! remove operations elsewhere can overwrite or delete existing data.
//!
//! ```rust
//! use pathkit::{
//!     SyncFsOps,
//!     path,
//! };
//!
//! let directory = tempfile::tempdir()?;
//! let file = path!(directory.path()) / "message.txt";
//! file.write_sync(b"Hello!")?;
//! assert_eq!(file.read_to_string_sync()?, "Hello!");
//! assert_eq!(file.get_file_size_sync()?, 6);
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! # See also
//!
//! - [`Path`]
//! - [`SyncFsOps`]
//! - [`PathEntry`]

#[cfg(feature = "async-fs-ops")]
mod async_fs_ops;
mod core;
mod div;
mod entry;
mod macros;
#[cfg(feature = "sea-orm")]
mod sea_orm;
mod sync_fs_ops;
mod traits;

#[cfg(feature = "async-fs-ops")]
pub use crate::async_fs_ops::AsyncFsOps;
#[cfg(feature = "async-fs-ops")]
pub use crate::entry::r#async::AsyncPathEntry;
pub use crate::{
    core::Path,
    entry::sync::PathEntry,
    sync_fs_ops::SyncFsOps,
};
