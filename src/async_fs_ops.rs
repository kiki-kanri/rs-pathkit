use std::{
    ffi::OsString,
    fs::{
        FileType,
        Metadata,
        Permissions,
    },
    io,
    path::Path as StdPath,
    task::{
        Context,
        Poll,
    },
};

use derive_more::From;
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use tokio::{
    fs::{
        self,
        DirEntry,
        File,
        OpenOptions,
    },
    task::spawn_blocking,
};

use crate::{
    Path,
    sync_fs_ops::{
        SyncFsOps,
        none_if_not_found,
    },
};

/// Async counterparts of [`SyncFsOps`]; each method behaves like the `_sync` method of the same
/// name.
///
/// Composite operations (`empty_dir`, `touch`, `write_atomic*`, `persist_to`), `chown`, and
/// `prepare` callbacks run as a single task on Tokio's blocking pool; panics there surface as
/// [`io::Error`]. Write methods copy `contents` before handing it to the blocking pool.
///
/// With this trait in scope its methods take precedence over the synchronous
/// [`std::path::Path`] methods of the same name reached through `Deref`, such as `try_exists`,
/// `is_dir`, `metadata`, and `read_dir`; call `as_path()` to reach the std method.
///
/// # Panics
///
/// Polling outside a Tokio runtime panics once the operation submits blocking work.
///
/// # Cancellation safety
///
/// Dropping a future does not roll back completed changes, and work already handed to the
/// blocking pool can still run. A canceled write may therefore take effect even if the caller
/// retries it.
///
/// ```rust
/// use pathkit::{
///     async_fs_ops::AsyncFsOps,
///     path,
/// };
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> std::io::Result<()> {
///     let directory = tempfile::tempdir()?;
///     let file = path!(directory.path()) / "message.txt";
///     file.write(b"Hello!").await?;
///     assert_eq!(file.read_to_string().await?, "Hello!");
///     Ok(())
/// }
/// ```
#[async_trait::async_trait]
pub trait AsyncFsOps {
    async fn canonicalize(&self) -> io::Result<Path>;

    #[cfg(unix)]
    async fn chmod(&self, mode: u32) -> io::Result<()>;

    #[cfg(unix)]
    async fn chown(&self, uid: Option<u32>, gid: Option<u32>) -> io::Result<()>;

    async fn copy_file(&self, dest: impl AsRef<StdPath> + Send) -> io::Result<u64>;

    async fn create_dir_all(&self) -> io::Result<()>;

    async fn create_dir(&self) -> io::Result<()>;

    async fn create_parent_dir_all(&self) -> io::Result<()>;

    async fn empty_dir(&self) -> io::Result<()>;

    async fn try_exists(&self) -> io::Result<bool>;

    async fn hard_link_to(&self, target: impl AsRef<StdPath> + Send) -> io::Result<()>;

    #[cfg(unix)]
    async fn is_block_device(&self) -> io::Result<bool>;

    #[cfg(unix)]
    async fn is_char_device(&self) -> io::Result<bool>;

    async fn is_dir(&self) -> io::Result<bool>;

    #[cfg(unix)]
    async fn is_fifo(&self) -> io::Result<bool>;

    async fn is_file(&self) -> io::Result<bool>;

    #[cfg(unix)]
    async fn is_socket(&self) -> io::Result<bool>;

    async fn is_symlink(&self) -> io::Result<bool>;

    async fn metadata_if_exists(&self) -> io::Result<Option<Metadata>>;

    async fn metadata(&self) -> io::Result<Metadata>;

    async fn open(&self) -> io::Result<File>;

    async fn open_with_options(&self, options: &OpenOptions) -> io::Result<File>;

    async fn persist_to(&self, dest: impl AsRef<StdPath> + Send) -> io::Result<Path>;

    async fn read_dir_if_exists(&self) -> io::Result<Option<AsyncReadDir>>;

    async fn read_dir(&self) -> io::Result<AsyncReadDir>;

    async fn read_dir_entries(&self) -> io::Result<Vec<AsyncPathEntry>>;

    async fn read_dir_names_lossy(&self) -> io::Result<Vec<String>>;

    async fn read_dir_paths(&self) -> io::Result<Vec<Path>>;

    async fn read_if_exists(&self) -> io::Result<Option<Vec<u8>>>;

    /// Deserialization runs on the polling thread.
    async fn read_json<T: DeserializeOwned>(&self) -> serde_json::Result<T>;

    async fn read_link(&self) -> io::Result<Path>;

    async fn read(&self) -> io::Result<Vec<u8>>;

    async fn read_to_string_if_exists(&self) -> io::Result<Option<String>>;

    async fn read_to_string(&self) -> io::Result<String>;

    async fn remove_dir_all(&self) -> io::Result<()>;

    async fn remove_dir(&self) -> io::Result<()>;

    async fn remove_file_if_exists(&self) -> io::Result<bool>;

    async fn remove_file(&self) -> io::Result<()>;

    async fn rename_to(&self, dest: impl AsRef<StdPath> + Send) -> io::Result<Path>;

    async fn set_permissions(&self, permissions: Permissions) -> io::Result<()>;

    #[cfg(unix)]
    async fn symlink_to(&self, target: impl AsRef<StdPath> + Send) -> io::Result<()>;

    async fn symlink_metadata_if_exists(&self) -> io::Result<Option<Metadata>>;

    async fn symlink_metadata(&self) -> io::Result<Metadata>;

    async fn touch(&self) -> io::Result<()>;

    async fn write_atomic(&self, contents: impl AsRef<[u8]> + Send) -> io::Result<()>;

    async fn write_atomic_with(
        &self,
        contents: impl AsRef<[u8]> + Send,
        prepare: impl for<'a> FnOnce(&'a Path) -> io::Result<()> + Send + 'static,
    ) -> io::Result<()>;

    /// Serialization runs on the polling thread.
    async fn write_json_pretty_atomic<T: Serialize + Sync + ?Sized>(&self, data: &T) -> serde_json::Result<()>;

    /// Serialization runs on the polling thread.
    async fn write_json_pretty<T: Serialize + Sync + ?Sized>(&self, data: &T) -> serde_json::Result<()>;

    async fn write(&self, contents: impl AsRef<[u8]> + Send) -> io::Result<()>;
}

#[async_trait::async_trait]
impl AsyncFsOps for Path {
    async fn canonicalize(&self) -> io::Result<Path> {
        fs::canonicalize(self).await.map(Self::from)
    }

    #[cfg(unix)]
    async fn chmod(&self, mode: u32) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(self, Permissions::from_mode(mode)).await
    }

    #[cfg(unix)]
    async fn chown(&self, uid: Option<u32>, gid: Option<u32>) -> io::Result<()> {
        let path = self.clone();
        spawn_blocking(move || path.chown_sync(uid, gid)).await?
    }

    async fn copy_file(&self, dest: impl AsRef<StdPath> + Send) -> io::Result<u64> {
        fs::copy(self, dest).await
    }

    async fn create_dir_all(&self) -> io::Result<()> {
        fs::create_dir_all(self).await
    }

    async fn create_dir(&self) -> io::Result<()> {
        fs::create_dir(self).await
    }

    async fn create_parent_dir_all(&self) -> io::Result<()> {
        match self.parent() {
            Some(parent) => parent.create_dir_all().await,
            None => Ok(()),
        }
    }

    async fn empty_dir(&self) -> io::Result<()> {
        let path = self.clone();
        spawn_blocking(move || path.empty_dir_sync()).await?
    }

    async fn try_exists(&self) -> io::Result<bool> {
        fs::try_exists(self).await
    }

    async fn hard_link_to(&self, target: impl AsRef<StdPath> + Send) -> io::Result<()> {
        fs::hard_link(target, self).await
    }

    #[cfg(unix)]
    async fn is_block_device(&self) -> io::Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata().await?.file_type().is_block_device())
    }

    #[cfg(unix)]
    async fn is_char_device(&self) -> io::Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata().await?.file_type().is_char_device())
    }

    async fn is_dir(&self) -> io::Result<bool> {
        Ok(self.metadata().await?.is_dir())
    }

    #[cfg(unix)]
    async fn is_fifo(&self) -> io::Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata().await?.file_type().is_fifo())
    }

    async fn is_file(&self) -> io::Result<bool> {
        Ok(self.metadata().await?.is_file())
    }

    #[cfg(unix)]
    async fn is_socket(&self) -> io::Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata().await?.file_type().is_socket())
    }

    async fn is_symlink(&self) -> io::Result<bool> {
        Ok(self.symlink_metadata().await?.file_type().is_symlink())
    }

    async fn metadata_if_exists(&self) -> io::Result<Option<Metadata>> {
        none_if_not_found(self.metadata().await)
    }

    async fn metadata(&self) -> io::Result<Metadata> {
        fs::metadata(self).await
    }

    async fn open(&self) -> io::Result<File> {
        File::open(self).await
    }

    async fn open_with_options(&self, options: &OpenOptions) -> io::Result<File> {
        options.open(self).await
    }

    async fn persist_to(&self, dest: impl AsRef<StdPath> + Send) -> io::Result<Path> {
        let source = self.clone();
        let dest = Self::new(dest);
        spawn_blocking(move || source.persist_to_sync(dest)).await?
    }

    async fn read_dir_if_exists(&self) -> io::Result<Option<AsyncReadDir>> {
        none_if_not_found(self.read_dir().await)
    }

    async fn read_dir(&self) -> io::Result<AsyncReadDir> {
        fs::read_dir(self).await.map(AsyncReadDir)
    }

    async fn read_dir_entries(&self) -> io::Result<Vec<AsyncPathEntry>> {
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(self).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            entries.push(AsyncPathEntry(entry));
        }

        Ok(entries)
    }

    async fn read_dir_names_lossy(&self) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        let mut read_dir = fs::read_dir(self).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }

        Ok(names)
    }

    async fn read_dir_paths(&self) -> io::Result<Vec<Path>> {
        let mut paths = Vec::new();
        let mut read_dir = fs::read_dir(self).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            paths.push(Self::from(entry.path()));
        }

        Ok(paths)
    }

    async fn read_if_exists(&self) -> io::Result<Option<Vec<u8>>> {
        none_if_not_found(self.read().await)
    }

    async fn read_json<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_slice(&self.read().await.map_err(serde_json::Error::io)?)
    }

    async fn read_link(&self) -> io::Result<Path> {
        fs::read_link(self).await.map(Self::from)
    }

    async fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(self).await
    }

    async fn read_to_string_if_exists(&self) -> io::Result<Option<String>> {
        none_if_not_found(self.read_to_string().await)
    }

    async fn read_to_string(&self) -> io::Result<String> {
        fs::read_to_string(self).await
    }

    async fn remove_dir_all(&self) -> io::Result<()> {
        fs::remove_dir_all(self).await
    }

    async fn remove_dir(&self) -> io::Result<()> {
        fs::remove_dir(self).await
    }

    async fn remove_file_if_exists(&self) -> io::Result<bool> {
        Ok(none_if_not_found(self.remove_file().await)?.is_some())
    }

    async fn remove_file(&self) -> io::Result<()> {
        fs::remove_file(self).await
    }

    async fn rename_to(&self, dest: impl AsRef<StdPath> + Send) -> io::Result<Path> {
        fs::rename(self, dest.as_ref()).await?;
        Ok(Self::new(dest))
    }

    async fn set_permissions(&self, permissions: Permissions) -> io::Result<()> {
        fs::set_permissions(self, permissions).await
    }

    #[cfg(unix)]
    async fn symlink_to(&self, target: impl AsRef<StdPath> + Send) -> io::Result<()> {
        fs::symlink(target, self).await
    }

    async fn symlink_metadata_if_exists(&self) -> io::Result<Option<Metadata>> {
        none_if_not_found(self.symlink_metadata().await)
    }

    async fn symlink_metadata(&self) -> io::Result<Metadata> {
        fs::symlink_metadata(self).await
    }

    async fn touch(&self) -> io::Result<()> {
        let path = self.clone();
        spawn_blocking(move || path.touch_sync()).await?
    }

    async fn write_atomic(&self, contents: impl AsRef<[u8]> + Send) -> io::Result<()> {
        self.write_atomic_with(contents, |_| Ok(())).await
    }

    async fn write_atomic_with(
        &self,
        contents: impl AsRef<[u8]> + Send,
        prepare: impl for<'a> FnOnce(&'a Path) -> io::Result<()> + Send + 'static,
    ) -> io::Result<()> {
        let path = self.clone();
        let contents = contents.as_ref().to_vec();
        spawn_blocking(move || path.write_atomic_with_sync(contents, prepare)).await?
    }

    async fn write_json_pretty_atomic<T: Serialize + Sync + ?Sized>(&self, data: &T) -> serde_json::Result<()> {
        self.write_atomic(serde_json::to_vec_pretty(data)?)
            .await
            .map_err(serde_json::Error::io)
    }

    async fn write_json_pretty<T: Serialize + Sync + ?Sized>(&self, data: &T) -> serde_json::Result<()> {
        self.write(serde_json::to_vec_pretty(data)?)
            .await
            .map_err(serde_json::Error::io)
    }

    async fn write(&self, contents: impl AsRef<[u8]> + Send) -> io::Result<()> {
        fs::write(self, contents).await
    }
}

/// Metadata and file-type queries do not follow symlinks and may run on Tokio's blocking pool. On
/// Unix, a retained entry can keep its directory's file descriptor open after the stream is
/// dropped.
#[derive(Debug, From)]
pub struct AsyncPathEntry(DirEntry);

impl AsyncPathEntry {
    #[inline]
    pub fn path(&self) -> Path {
        Path::from(self.0.path())
    }

    #[inline]
    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata().await
    }

    pub async fn file_type(&self) -> io::Result<FileType> {
        self.0.file_type().await
    }

    #[inline]
    pub fn as_dir_entry(&self) -> &DirEntry {
        &self.0
    }

    #[inline]
    pub fn into_dir_entry(self) -> DirEntry {
        self.0
    }
}

#[derive(Debug)]
pub struct AsyncReadDir(fs::ReadDir);

impl AsyncReadDir {
    /// Cancel safe, like [`tokio::fs::ReadDir::next_entry`].
    pub async fn next_entry(&mut self) -> io::Result<Option<AsyncPathEntry>> {
        Ok(self.0.next_entry().await?.map(AsyncPathEntry))
    }

    /// Polling counterpart of [`Self::next_entry`], for building a `Stream`.
    pub fn poll_next_entry(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<Option<AsyncPathEntry>>> {
        self.0.poll_next_entry(cx).map_ok(|entry| entry.map(AsyncPathEntry))
    }
}
