//! Asynchronous file system operations module
//!
//! This module provides the `AsyncFsOps` trait for asynchronous file system operations.
//! Requires the `async-fs-ops` feature to be enabled.
//!
//! # Example
//!
//! ```rust,ignore
//! use pathkit::{Path, AsyncFsOps};
//!
//! let path = Path::new("/tmp/test.txt");
//! path.write(b"Hello!").await?;
//! let content = path.read().await?;
//! ```

use std::{
    fs::{
        Metadata,
        Permissions,
    },
    path::Path as StdPath,
};

use anyhow::Result;
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use serde_json::{
    from_slice,
    to_vec_pretty,
};
use tokio::fs::{
    self,
    File,
    OpenOptions,
    ReadDir,
};

use super::{
    core::Path,
    entry::r#async::AsyncPathEntry,
};

/// Trait for asynchronous file system operations.
///
/// This trait provides non-blocking file system operations similar to Python's pathlib.
/// It is implemented for `Path` but can be implemented for other types as well.
///
/// Requires the `async-fs-ops` feature to be enabled.
///
/// # Example
///
/// ```rust,ignore
/// use pathkit::{Path, AsyncFsOps};
///
/// let path = Path::new("/tmp/test.txt");
///
/// // Check if file exists
/// if path.exists().await? {
///     // Read file contents
///     let content = path.read().await?;
/// }
///
/// // Write to file
/// path.write(b"Hello, world!").await?;
///
/// // Get file size
/// let size = path.get_file_size().await?;
/// ```
#[async_trait::async_trait]
pub trait AsyncFsOps {
    #[cfg(unix)]
    async fn chmod(&self, mode: u32) -> Result<()>;
    #[cfg(unix)]
    async fn chown(&self, uid: Option<u32>, gid: Option<u32>) -> Result<()>;
    async fn copy_file(&self, dest: impl AsRef<StdPath> + Send) -> Result<u64>;
    async fn create_dir_all(&self) -> Result<()>;
    async fn create_dir(&self) -> Result<()>;
    async fn create_parent_dir_all(&self) -> Result<bool>;
    async fn create_parent_dir(&self) -> Result<bool>;
    async fn empty_dir(&self) -> Result<()>;
    async fn exists(&self) -> Result<bool>;
    async fn get_file_size(&self) -> Result<u64>;
    #[cfg(unix)]
    async fn is_block_device(&self) -> Result<bool>;
    #[cfg(unix)]
    async fn is_char_device(&self) -> Result<bool>;
    async fn is_dir(&self) -> Result<bool>;
    #[cfg(unix)]
    async fn is_fifo(&self) -> Result<bool>;
    async fn is_file(&self) -> Result<bool>;
    #[cfg(unix)]
    async fn is_socket(&self) -> Result<bool>;
    async fn is_symlink(&self) -> Result<bool>;
    async fn metadata(&self) -> Result<Metadata>;
    async fn move_to(&self, dest: impl AsRef<StdPath> + Send) -> Result<Path>;
    async fn open(&self) -> Result<File>;
    async fn open_with_options(&self, options: &OpenOptions) -> Result<File>;
    async fn read_dir(&self) -> Result<ReadDir>;
    async fn read_dir_entries(&self) -> Result<Vec<AsyncPathEntry>>;
    async fn read_dir_names(&self) -> Result<Vec<String>>;
    async fn read_dir_paths(&self) -> Result<Vec<Path>>;
    async fn read_json<T: DeserializeOwned>(&self) -> Result<T>;
    async fn read(&self) -> Result<Vec<u8>>;
    async fn read_to_string(&self) -> Result<String>;
    async fn remove_dir_all(&self) -> Result<()>;
    async fn remove_dir(&self) -> Result<()>;
    async fn remove_file(&self) -> Result<()>;
    async fn set_permissions(&self, permissions: Permissions) -> Result<()>;
    async fn truncate(&self, len: Option<u64>) -> Result<()>;
    async fn write_json<T: Serialize + Send>(&self, data: T) -> Result<()>;
    async fn write(&self, contents: impl AsRef<[u8]> + Send) -> Result<()>;
}

#[async_trait::async_trait]
impl AsyncFsOps for Path {
    #[cfg(unix)]
    async fn chmod(&self, mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        Ok(fs::set_permissions(self, Permissions::from_mode(mode)).await?)
    }

    #[cfg(unix)]
    async fn chown(&self, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
        use tokio::task::spawn_blocking;

        let path = self.clone();
        Ok(spawn_blocking(move || std::os::unix::fs::chown(path, uid, gid)).await??)
    }

    async fn copy_file(&self, dest: impl AsRef<StdPath> + Send) -> Result<u64> {
        Ok(fs::copy(self, dest).await?)
    }

    async fn create_dir(&self) -> Result<()> {
        Ok(fs::create_dir(self).await?)
    }

    async fn create_dir_all(&self) -> Result<()> {
        Ok(fs::create_dir_all(self).await?)
    }

    async fn create_parent_dir_all(&self) -> Result<bool> {
        if let Some(parent) = self.parent() {
            parent.create_dir_all().await?;
            return Ok(true);
        }

        Ok(false)
    }

    async fn create_parent_dir(&self) -> Result<bool> {
        if let Some(parent) = self.parent() {
            parent.create_dir().await?;
            return Ok(true);
        }

        Ok(false)
    }

    async fn empty_dir(&self) -> Result<()> {
        if !self.exists().await? {
            self.create_dir_all().await?;
        }

        let mut entries = fs::read_dir(self).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                fs::remove_dir_all(entry_path).await?;
            } else {
                fs::remove_file(entry_path).await?;
            }
        }

        Ok(())
    }

    async fn exists(&self) -> Result<bool> {
        Ok(fs::try_exists(self).await?)
    }

    async fn get_file_size(&self) -> Result<u64> {
        Ok(self.metadata().await?.len())
    }

    #[cfg(unix)]
    async fn is_block_device(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata().await?.file_type().is_block_device())
    }

    #[cfg(unix)]
    async fn is_char_device(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata().await?.file_type().is_char_device())
    }

    async fn is_dir(&self) -> Result<bool> {
        Ok(self.metadata().await?.is_dir())
    }

    #[cfg(unix)]
    async fn is_fifo(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata().await?.file_type().is_fifo())
    }

    async fn is_file(&self) -> Result<bool> {
        Ok(self.metadata().await?.is_file())
    }

    #[cfg(unix)]
    async fn is_socket(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata().await?.file_type().is_socket())
    }

    async fn is_symlink(&self) -> Result<bool> {
        Ok(fs::symlink_metadata(self).await?.file_type().is_symlink())
    }

    async fn metadata(&self) -> Result<Metadata> {
        Ok(fs::metadata(self).await?)
    }

    async fn move_to(&self, dest: impl AsRef<StdPath> + Send) -> Result<Path> {
        let dest = Path::new(dest);
        fs::rename(self, &dest).await?;
        Ok(dest)
    }

    async fn open(&self) -> Result<File> {
        Ok(File::open(self).await?)
    }

    async fn open_with_options(&self, options: &OpenOptions) -> Result<File> {
        Ok(options.open(self).await?)
    }

    async fn read(&self) -> Result<Vec<u8>> {
        Ok(fs::read(self).await?)
    }

    async fn read_dir(&self) -> Result<ReadDir> {
        Ok(fs::read_dir(self).await?)
    }

    async fn read_dir_entries(&self) -> Result<Vec<AsyncPathEntry>> {
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(self).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            entries.push(AsyncPathEntry::new(entry));
        }

        Ok(entries)
    }

    async fn read_dir_names(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        let mut read_dir = fs::read_dir(self).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            names.push(entry.file_name().to_string_lossy().into());
        }

        Ok(names)
    }

    async fn read_dir_paths(&self) -> Result<Vec<Path>> {
        let mut paths = Vec::new();
        let mut read_dir = fs::read_dir(self).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            paths.push(Self::new(entry.path()));
        }

        Ok(paths)
    }

    async fn read_json<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(from_slice::<T>(&self.read().await?)?)
    }

    async fn read_to_string(&self) -> Result<String> {
        Ok(fs::read_to_string(self).await?)
    }

    async fn remove_dir(&self) -> Result<()> {
        Ok(fs::remove_dir(self).await?)
    }

    async fn remove_file(&self) -> Result<()> {
        Ok(fs::remove_file(self).await?)
    }

    async fn remove_dir_all(&self) -> Result<()> {
        Ok(fs::remove_dir_all(self).await?)
    }

    async fn set_permissions(&self, permissions: Permissions) -> Result<()> {
        Ok(fs::set_permissions(self, permissions).await?)
    }

    async fn truncate(&self, len: Option<u64>) -> Result<()> {
        Ok(OpenOptions::new()
            .write(true)
            .open(self)
            .await?
            .set_len(len.unwrap_or(0))
            .await?)
    }

    async fn write_json<T: Serialize + Send>(&self, data: T) -> Result<()> {
        self.write(to_vec_pretty(&data)?).await
    }

    async fn write(&self, contents: impl AsRef<[u8]> + Send) -> Result<()> {
        Ok(fs::write(self, contents).await?)
    }
}
