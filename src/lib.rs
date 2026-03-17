//! # pathkit
//!
//! A Rust library that provides a `Path` structure similar to Python's pathlib,
//! with both synchronous and asynchronous file manipulation methods.
//!
//! ## Features
//!
//! - **Path Operations**: Extended path manipulation methods beyond `std::path::Path`
//! - **Synchronous I/O**: Blocking file system operations via [`SyncFsOps`] trait
//! - **Asynchronous I/O**: Non-blocking file system operations via [`AsyncFsOps`] trait
//! - **Serde Support**: Serialize and deserialize Path with `#[derive(Serialize, Deserialize)]`
//! - **Path Joining**: Use `/` operator for intuitive path composition
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```bash
//! cargo add pathkit
//! ```
//!
//! For async support:
//!
//! ```bash
//! cargo add pathkit --features async-fs-ops
//! ```
//!
//! ## Usage
//!
//! ### Basic Path Operations
//!
//! ```rust
//! use pathkit::Path;
//!
//! // Create a new path
//! let path = Path::new("/home/user/project");
//!
//! // Join paths
//! let config = path.join("config.json");
//!
//! // Using / operator (note: this consumes the path)
//! let nested = Path::new("/home/user") / "project" / "subdir";
//!
//! // Get path components
//! let parent = path.parent();
//! let file_name = path.file_name();
//! let extension = path.extension();
//! ```
//!
//! ### Synchronous File Operations
//!
//! ```rust,ignore
//! use pathkit::{Path, SyncFsOps};
//!
//! let path = Path::new("/tmp/test.txt");
//!
//! // Read/write files
//! path.write_sync(b"Hello, world!")?;
//! let content = path.read_sync()?;
//!
//! // Check existence and type
//! if path.exists_sync()? {
//!     println!("File size: {}", path.get_file_size_sync()?);
//! }
//!
//! // Create directories
//! Path::new("/tmp/new_project").create_dir_all_sync()?;
//!
//! // Read JSON
//! #[derive(Deserialize)]
//! struct Config { name: String }
//! let config: Config = path.read_json_sync()?;
//! ```
//!
//! ### Asynchronous File Operations
//!
//! ```rust,ignore
//! use pathkit::{Path, AsyncFsOps};
//!
//! let path = Path::new("/tmp/test.txt");
//!
//! // Async read/write
//! path.write(b"Hello, world!").await?;
//! let content = path.read().await?;
//!
//! // Async directory operations
//! Path::new("/tmp/new_project").create_dir_all().await?;
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `async-fs-ops` | Enable async file system operations (requires tokio) |
//! | `full` | Enable all features |
//!
//! ## Platform Support
//!
//! - **Unix/Linux/macOS**: Full support including `chmod`, `chown`, and special file type checks
//! - **Windows**: Core functionality supported; some Unix-specific features are conditionally compiled out

#[cfg(feature = "async-fs-ops")]
mod async_fs_ops;
mod core;
mod div;
mod sync_fs_ops;
mod traits;

#[cfg(feature = "async-fs-ops")]
pub use crate::async_fs_ops::AsyncFsOps;
pub use crate::{
    core::Path,
    sync_fs_ops::SyncFsOps,
};
