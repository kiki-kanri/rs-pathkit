//! # pathkit
//!
//! A Rust library that provides a `Path` structure similar to Python's pathlib,
//! with synchronous and optional asynchronous file manipulation methods.
//!
//! ## Features
//!
//! - 🧭 **Path Operations**: pathlib-style [`Path`] wrapper around `std::path::PathBuf`
//! - 🧩 **Path Macro**: [`path!`] supports direct path expressions and `format!`-style construction
//! - ➗ **Path Joining**: use `/` for concise path composition
//! - 📁 **Synchronous I/O**: blocking file system operations via [`SyncFsOps`]
//! - ⚡ **Asynchronous I/O**: non-blocking file system operations via `AsyncFsOps` (requires `async-fs-ops`)
//! - 🔄 **Serde Support**: serialize and deserialize [`Path`]
//! - 🗄️ **SeaORM Integration**: use [`Path`] as a model field (requires `sea-orm`)
//!
//! ## Installation
//!
//! ```bash
//! cargo add pathkit
//! cargo add pathkit --features async-fs-ops
//! cargo add pathkit --features sea-orm
//! cargo add pathkit --features full
//! ```
//!
//! ## Basic Path Operations
//!
//! ```rust
//! use pathkit::path;
//!
//! let root = path!("/home/{}/project", "user");
//! let config = &root / "config" / "app.json";
//!
//! // `join` is still available for std-like APIs.
//! let readme = root.join("README.md");
//!
//! let parent = config.parent();
//! let file_name = config.file_name();
//! let extension = config.extension();
//! ```
//!
//! ## Synchronous File Operations
//!
//! ```rust,ignore
//! use pathkit::{
//!     SyncFsOps,
//!     path
//! };
//!
//! let path = path!("/tmp/test.txt");
//!
//! path.write_sync(b"Hello, world!")?;
//! let content = path.read_sync()?;
//! let file = path.open_sync()?;
//!
//! if path.exists_sync()? {
//!     println!("File size: {}", path.get_file_size_sync()?);
//! }
//!
//! let moved = path.move_to_sync("/tmp/moved.txt")?;
//! let config: Config = moved.read_json_sync()?;
//! ```
//!
//! Use `open_with_options_sync()` when you need custom `std::fs::OpenOptions`.
//!
//! ## Asynchronous File Operations
//!
//! Requires the `async-fs-ops` feature.
//!
//! ```rust,ignore
//! use pathkit::{
//!     AsyncFsOps,
//!     path
//! };
//!
//! let path = path!("/tmp/test.txt");
//!
//! path.write(b"Hello, world!").await?;
//! let content = path.read().await?;
//! let file = path.open().await?;
//!
//! path.create_parent_dir_all().await?;
//! let moved = path.move_to("/tmp/moved.txt").await?;
//! ```
//!
//! Use `open_with_options()` when you need custom `tokio::fs::OpenOptions`.
//!
//! ## SeaORM Integration
//!
//! Requires the `sea-orm` feature. [`Path`] is stored as `String` and can be used
//! directly in SeaORM models. Implemented SeaORM traits: `Into<Value>`,
//! `ValueType`, `Nullable`, and `TryGetable`.
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `async-fs-ops` | Enable async file system operations via tokio |
//! | `sea-orm` | Enable SeaORM value/model integration |
//! | `all` | Enable all optional features |
//! | `full` | Alias of `all` |
//!
//! ## API Overview
//!
//! Main entry points:
//!
//! - [`Path`] — owned path wrapper around `std::path::PathBuf`
//! - [`path!`] — convenient path construction macro
//! - `/` operator — concise path composition
//! - [`SyncFsOps`] — blocking filesystem operations
//! - `AsyncFsOps` — async filesystem operations with tokio
//! - [`PathEntry`] / `AsyncPathEntry` — typed directory entries
//!
//! See each item’s rustdoc for the complete method list.

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
