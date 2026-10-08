//! Async filesystem operations for owned paths.
//!
//! Available with the `async-fs-ops` feature. [`AsyncFsOps`] defines the operation and
//! cancellation contracts and is implemented for [`Path`].

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

/// Async filesystem operations for paths.
///
/// Available with the `async-fs-ops` feature. [`Path`] implements this trait using Tokio.
/// For that implementation, methods borrow the stored path without changing it; their
/// bodies run when their futures are polled, not merely when the methods are called.
/// A Tokio runtime with blocking-task support is required for filesystem I/O. Paths are
/// interpreted by the operating system, including relative paths resolved against the
/// current working directory. The behavior described below applies to that implementation.
///
/// The [`Path`] implementation provides these shared behaviors:
///
/// - Metadata and type checks follow symlinks, except [`Self::is_symlink`]. Type checks
///   return an error for missing paths, rather than `false`. [`Self::exists`] returns
///   `false` for a missing target, including a dangling symlink, and propagates other
///   inspection errors.
/// - [`Self::get_file_size`] returns metadata length in bytes, which is not necessarily
///   allocated disk space or a meaningful content size for non-files.
/// - [`Self::open`] opens an existing file read-only. [`Self::open_with_options`] uses
///   the supplied options without changing them. Returned files follow [`tokio::fs::File`]
///   behavior: writes can remain pending after an async write completes, and flushing
///   waits for pending I/O but does not itself guarantee persistence to disk.
/// - [`Self::write`] and [`Self::write_json`] create or truncate a file without creating
///   parent directories. JSON uses pretty serialization. [`Self::read_json`] deserializes
///   the complete file contents. JSON serialization and deserialization run synchronously
///   within the polled future.
/// - [`Self::truncate`] opens an existing file for writing and sets its length in bytes;
///   `None` means `0`. Extending a file fills the added range with zero bytes.
/// - [`Self::create_parent_dir`] creates only the immediate parent and returns `true` on
///   successful creation; an existing parent is an error. [`Self::create_parent_dir_all`]
///   creates missing ancestors and returns `true` on success even if the parent already
///   exists. Both return `false` when the lexical path has no parent.
/// - [`Self::empty_dir`] creates a missing directory with its parents or removes the
///   entries of an existing directory, keeping the directory itself. Removal is incremental
///   and uses Tokio's file and recursive-directory removal rules. Child symlinks are
///   removed without traversing their targets.
/// - Directory collection methods return entries in filesystem iteration order, without
///   sorting. [`Self::read_dir_names`] converts names to strings with invalid Unicode
///   replaced; [`Self::read_dir_paths`] and [`Self::read_dir_entries`] retain native path
///   data. [`Self::read_dir`] returns an iterator whose later async reads can fail.
/// - [`Self::copy_file`] copies to the exact destination, overwriting an existing file
///   and returning the number of bytes copied. [`Self::move_to`] renames to the exact
///   destination and returns a new [`Path`]; it does not update the source value or fall back
///   to copying across filesystems. Replacement rules depend on the platform.
///
/// Unix-only methods expose permission modes, owner and group IDs, and special-file
/// checks. For ownership changes, `None` preserves the corresponding owner or group ID.
/// Filesystem methods otherwise follow the corresponding [`tokio::fs`] operations.
///
/// # Errors
///
/// Propagates filesystem errors such as missing paths, permission errors, invalid file
/// or directory types, and unsupported operations. Directory collection methods also
/// propagate iteration errors instead of returning partial collections. UTF-8 string
/// reads fail for invalid UTF-8; JSON reads and writes also propagate deserialization
/// or serialization errors. Unix ownership changes additionally propagate blocking-task
/// join errors. Changes made before an error are not rolled back.
///
/// # Panics
///
/// Operations that submit blocking work can panic if polled outside a Tokio runtime.
///
/// # Cancellation safety
///
/// Dropping a future stops polling this operation but does not roll back completed
/// filesystem changes. I/O already submitted to Tokio's blocking pool can continue.
/// Multi-step operations, including directory creation, directory clearing, and file
/// truncation, can leave partial changes. Collected directory entries are discarded
/// when their collection future is dropped. Retrying a canceled operation is not an
/// exactly-once guarantee.
///
/// # Examples
///
/// ```rust
/// use pathkit::{
///     AsyncFsOps,
///     path,
/// };
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> anyhow::Result<()> {
///     let directory = tempfile::tempdir()?;
///     let file = path!(directory.path()) / "message.txt";
///     file.write(b"Hello!").await?;
///     assert_eq!(file.read_to_string().await?, "Hello!");
///     assert_eq!(file.get_file_size().await?, 6);
///     Ok(())
/// }
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
        #[cfg(windows)]
        use std::os::windows::fs::FileTypeExt;

        if !self.exists().await? {
            self.create_dir_all().await?;
        }

        let mut entries = fs::read_dir(self).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                fs::remove_dir_all(entry_path).await?;
            } else {
                #[cfg(windows)]
                if file_type.is_symlink_dir() {
                    fs::remove_dir(entry_path).await?;
                    continue;
                }

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
