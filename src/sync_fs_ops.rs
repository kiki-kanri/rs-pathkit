use std::{
    ffi::OsString,
    fs::{
        self,
        DirEntry,
        File,
        FileTimes,
        FileType,
        Metadata,
        OpenOptions,
        Permissions,
    },
    io::{
        self,
        Write,
    },
    path::Path as StdPath,
    time::SystemTime,
};

use derive_more::From;
use serde::{
    Serialize,
    de::DeserializeOwned,
};

use crate::{
    Path,
    atomic_write,
};

/// Blocking filesystem operations for paths.
///
/// Methods without their own docs wrap the std function they are named after (`copy_file` wraps
/// `fs::copy`, `open*` wrap `File::open` / `OpenOptions::open`) and return [`Path`] where std
/// returns `PathBuf`. Rules shared by all methods:
///
/// - Metadata and type checks follow symlinks, except `is_symlink_sync` and
///   `symlink_metadata*`. Type checks return `Err` for a missing path, not `false`.
/// - Multi-step operations are not transactional; a failure can leave partial changes.
/// - `read_dir_{entries,names_lossy,paths}_sync` return entries in filesystem iteration order and
///   fail as a whole on the first iteration error.
/// - `*_if_exists` methods map only [`io::ErrorKind::NotFound`] to `None` / `false`; other errors,
///   such as a non-directory parent component, are returned.
/// - JSON methods wrap filesystem failures with [`serde_json::Error::io`]; use
///   [`serde_json::Error::io_error_kind`] to inspect them.
///
/// ```rust
/// use pathkit::{
///     path,
///     sync_fs_ops::SyncFsOps,
/// };
///
/// let directory = tempfile::tempdir()?;
/// let file = path!(directory.path()) / "message.txt";
/// file.write_sync(b"Hello!")?;
/// assert_eq!(file.read_sync()?, b"Hello!");
/// assert_eq!(file.read_if_exists_sync()?, Some(b"Hello!".to_vec()));
/// # Ok::<(), std::io::Error>(())
/// ```
pub trait SyncFsOps {
    /// Starts a replacement of `self` that is written through the returned [`AtomicWriter`] and
    /// becomes visible only on [`AtomicWriter::commit`]. Use it when the contents are streamed or
    /// written by another library through [`AtomicWriter::path`]; otherwise prefer
    /// [`Self::write_atomic_sync`].
    ///
    /// # Errors
    ///
    /// Fails with [`io::ErrorKind::InvalidInput`] when the path has no file name.
    fn atomic_writer_sync(&self) -> io::Result<AtomicWriter>;

    fn canonicalize_sync(&self) -> io::Result<Path>;

    #[cfg(unix)]
    fn chmod_sync(&self, mode: u32) -> io::Result<()>;

    #[cfg(unix)]
    fn chown_sync(&self, uid: Option<u32>, gid: Option<u32>) -> io::Result<()>;

    fn copy_file_sync(&self, dest: impl AsRef<StdPath>) -> io::Result<u64>;

    fn create_dir_all_sync(&self) -> io::Result<()>;

    fn create_dir_sync(&self) -> io::Result<()>;

    /// Creates the lexical parent and its missing ancestors; does nothing when there is no parent.
    fn create_parent_dir_all_sync(&self) -> io::Result<()>;

    /// Removes everything inside the directory but keeps the directory itself, creating it (with
    /// parents) when missing. Child symlinks are removed without touching their targets; a symlink
    /// at `self` is followed, so its target directory is emptied.
    fn empty_dir_sync(&self) -> io::Result<()>;

    fn try_exists_sync(&self) -> io::Result<bool>;

    /// Makes `self` a new hard link to `target`.
    fn hard_link_to_sync(&self, target: impl AsRef<StdPath>) -> io::Result<()>;

    #[cfg(unix)]
    fn is_block_device_sync(&self) -> io::Result<bool>;

    #[cfg(unix)]
    fn is_char_device_sync(&self) -> io::Result<bool>;

    fn is_dir_sync(&self) -> io::Result<bool>;

    #[cfg(unix)]
    fn is_fifo_sync(&self) -> io::Result<bool>;

    fn is_file_sync(&self) -> io::Result<bool>;

    #[cfg(unix)]
    fn is_socket_sync(&self) -> io::Result<bool>;

    fn is_symlink_sync(&self) -> io::Result<bool>;

    fn metadata_if_exists_sync(&self) -> io::Result<Option<Metadata>>;

    fn metadata_sync(&self) -> io::Result<Metadata>;

    fn open_sync(&self) -> io::Result<File>;

    fn open_with_options_sync(&self, options: &OpenOptions) -> io::Result<File>;

    /// Syncs this regular file, renames it over `dest`, and on Unix syncs `dest`'s directory. Fails
    /// with [`io::ErrorKind::InvalidInput`] for a symlink or other non-regular source, and instead
    /// of copying when `dest` is on another filesystem. The file keeps its own permissions. On
    /// Windows the source must be writable, and transient replace failures are retried as in
    /// [`Self::write_atomic_with_sync`].
    fn persist_to_sync(&self, dest: impl AsRef<StdPath>) -> io::Result<Path>;

    fn read_dir_entries_sync(&self) -> io::Result<Vec<PathEntry>>;

    fn read_dir_names_lossy_sync(&self) -> io::Result<Vec<String>>;

    fn read_dir_paths_sync(&self) -> io::Result<Vec<Path>>;

    fn read_dir_if_exists_sync(&self) -> io::Result<Option<ReadDir>>;

    fn read_dir_sync(&self) -> io::Result<ReadDir>;

    fn read_if_exists_sync(&self) -> io::Result<Option<Vec<u8>>>;

    fn read_json_sync<T: DeserializeOwned>(&self) -> serde_json::Result<T>;

    fn read_link_sync(&self) -> io::Result<Path>;

    fn read_sync(&self) -> io::Result<Vec<u8>>;

    fn read_to_string_if_exists_sync(&self) -> io::Result<Option<String>>;

    fn read_to_string_sync(&self) -> io::Result<String>;

    fn remove_dir_all_sync(&self) -> io::Result<()>;

    fn remove_dir_sync(&self) -> io::Result<()>;

    /// Returns whether a file was removed.
    fn remove_file_if_exists_sync(&self) -> io::Result<bool>;

    fn remove_file_sync(&self) -> io::Result<()>;

    /// Not durable across a crash; use [`Self::persist_to_sync`] when it must be.
    fn rename_to_sync(&self, dest: impl AsRef<StdPath>) -> io::Result<Path>;

    fn set_permissions_sync(&self, permissions: Permissions) -> io::Result<()>;

    /// Makes `self` a symlink to `target`, stored exactly as given, so a relative target resolves
    /// against the link's directory.
    #[cfg(unix)]
    fn symlink_to_sync(&self, target: impl AsRef<StdPath>) -> io::Result<()>;

    fn symlink_metadata_if_exists_sync(&self) -> io::Result<Option<Metadata>>;

    fn symlink_metadata_sync(&self) -> io::Result<Metadata>;

    /// Sets the modification time to now, creating an empty file when missing. Never truncates. A
    /// symlink is followed; a dangling one gets its target created.
    fn touch_sync(&self) -> io::Result<()>;

    /// Writes to a temporary sibling file, syncs it, and renames it over `self`, so readers see
    /// either the old or the new contents. See [`Self::write_atomic_with_sync`].
    fn write_atomic_sync(&self, contents: impl AsRef<[u8]>) -> io::Result<()>;

    /// Like [`Self::write_atomic_sync`], but runs `prepare` on the still-empty temporary file first
    /// to change its metadata in place (for example permissions or ACLs); it must not replace or
    /// write the file.
    ///
    /// On Unix the result keeps the permission bits of an existing regular file at `self` (otherwise
    /// `0600`), but not its owner; on Windows it inherits the directory ACL. `prepare` runs afterwards and can override
    /// either. A symlink at `self` is replaced by a regular file rather than followed. On failure,
    /// removing the temporary file is attempted but not guaranteed. On Windows, a replace that fails
    /// because another process briefly holds a file open is retried for about one second.
    ///
    /// # Errors
    ///
    /// Fails with [`io::ErrorKind::InvalidInput`] when the path has no file name (such as `/` or a
    /// path ending in `..`). Needs write permission on the parent directory, so it can fail where
    /// [`Self::write_sync`] to an existing writable file succeeds.
    fn write_atomic_with_sync(
        &self,
        contents: impl AsRef<[u8]>,
        prepare: impl FnOnce(&Path) -> io::Result<()>,
    ) -> io::Result<()>;

    /// Like [`Self::write_json_sync`], but replaces the file through [`Self::write_atomic_sync`].
    fn write_json_atomic_sync<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<()>;

    /// Writes pretty-printed JSON.
    fn write_json_sync<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<()>;

    fn write_sync(&self, contents: impl AsRef<[u8]>) -> io::Result<()>;
}

impl SyncFsOps for Path {
    fn atomic_writer_sync(&self) -> io::Result<AtomicWriter> {
        let (file, temp) = atomic_write::stage(self)?;
        Ok(AtomicWriter {
            file,
            dest: self.clone(),
            temp,
        })
    }

    fn canonicalize_sync(&self) -> io::Result<Path> {
        fs::canonicalize(self).map(Self::from)
    }

    #[cfg(unix)]
    fn chmod_sync(&self, mode: u32) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(self, Permissions::from_mode(mode))
    }

    #[cfg(unix)]
    fn chown_sync(&self, uid: Option<u32>, gid: Option<u32>) -> io::Result<()> {
        std::os::unix::fs::chown(self, uid, gid)
    }

    fn copy_file_sync(&self, dest: impl AsRef<StdPath>) -> io::Result<u64> {
        fs::copy(self, dest)
    }

    fn create_dir_all_sync(&self) -> io::Result<()> {
        fs::create_dir_all(self)
    }

    fn create_dir_sync(&self) -> io::Result<()> {
        fs::create_dir(self)
    }

    fn create_parent_dir_all_sync(&self) -> io::Result<()> {
        self.parent().map_or(Ok(()), |parent| parent.create_dir_all_sync())
    }

    fn empty_dir_sync(&self) -> io::Result<()> {
        #[cfg(windows)]
        use std::os::windows::fs::FileTypeExt;

        if !self.try_exists_sync()? {
            return self.create_dir_all_sync();
        }

        for entry in fs::read_dir(self)? {
            let entry = entry?;
            let entry_path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                fs::remove_dir_all(entry_path)?;
            } else {
                // A Windows directory symlink must be removed as a directory.
                #[cfg(windows)]
                if file_type.is_symlink_dir() {
                    fs::remove_dir(entry_path)?;
                    continue;
                }

                fs::remove_file(entry_path)?;
            }
        }

        Ok(())
    }

    fn try_exists_sync(&self) -> io::Result<bool> {
        fs::exists(self)
    }

    fn hard_link_to_sync(&self, target: impl AsRef<StdPath>) -> io::Result<()> {
        fs::hard_link(target, self)
    }

    #[cfg(unix)]
    fn is_block_device_sync(&self) -> io::Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_block_device())
    }

    #[cfg(unix)]
    fn is_char_device_sync(&self) -> io::Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_char_device())
    }

    fn is_dir_sync(&self) -> io::Result<bool> {
        Ok(self.metadata_sync()?.is_dir())
    }

    #[cfg(unix)]
    fn is_fifo_sync(&self) -> io::Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_fifo())
    }

    fn is_file_sync(&self) -> io::Result<bool> {
        Ok(self.metadata_sync()?.is_file())
    }

    #[cfg(unix)]
    fn is_socket_sync(&self) -> io::Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_socket())
    }

    fn is_symlink_sync(&self) -> io::Result<bool> {
        Ok(self.symlink_metadata_sync()?.file_type().is_symlink())
    }

    fn metadata_if_exists_sync(&self) -> io::Result<Option<Metadata>> {
        none_if_not_found(self.metadata_sync())
    }

    fn metadata_sync(&self) -> io::Result<Metadata> {
        fs::metadata(self)
    }

    fn open_sync(&self) -> io::Result<File> {
        File::open(self)
    }

    fn open_with_options_sync(&self, options: &OpenOptions) -> io::Result<File> {
        options.open(self)
    }

    fn persist_to_sync(&self, dest: impl AsRef<StdPath>) -> io::Result<Path> {
        atomic_write::persist(self, dest.as_ref())?;
        Ok(Self::new(dest))
    }

    fn read_dir_entries_sync(&self) -> io::Result<Vec<PathEntry>> {
        fs::read_dir(self)?.map(|entry| entry.map(PathEntry)).collect()
    }

    fn read_dir_names_lossy_sync(&self) -> io::Result<Vec<String>> {
        fs::read_dir(self)?
            .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
            .collect()
    }

    fn read_dir_paths_sync(&self) -> io::Result<Vec<Path>> {
        fs::read_dir(self)?
            .map(|entry| entry.map(|entry| Self::from(entry.path())))
            .collect()
    }

    fn read_dir_if_exists_sync(&self) -> io::Result<Option<ReadDir>> {
        none_if_not_found(self.read_dir_sync())
    }

    fn read_dir_sync(&self) -> io::Result<ReadDir> {
        fs::read_dir(self).map(ReadDir)
    }

    fn read_if_exists_sync(&self) -> io::Result<Option<Vec<u8>>> {
        none_if_not_found(self.read_sync())
    }

    fn read_json_sync<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_slice(&self.read_sync().map_err(serde_json::Error::io)?)
    }

    fn read_link_sync(&self) -> io::Result<Path> {
        fs::read_link(self).map(Self::from)
    }

    fn read_sync(&self) -> io::Result<Vec<u8>> {
        fs::read(self)
    }

    fn read_to_string_if_exists_sync(&self) -> io::Result<Option<String>> {
        none_if_not_found(self.read_to_string_sync())
    }

    fn read_to_string_sync(&self) -> io::Result<String> {
        fs::read_to_string(self)
    }

    fn remove_dir_all_sync(&self) -> io::Result<()> {
        fs::remove_dir_all(self)
    }

    fn remove_dir_sync(&self) -> io::Result<()> {
        fs::remove_dir(self)
    }

    fn remove_file_if_exists_sync(&self) -> io::Result<bool> {
        Ok(none_if_not_found(self.remove_file_sync())?.is_some())
    }

    fn remove_file_sync(&self) -> io::Result<()> {
        fs::remove_file(self)
    }

    fn rename_to_sync(&self, dest: impl AsRef<StdPath>) -> io::Result<Path> {
        fs::rename(self, &dest)?;
        Ok(Self::new(dest))
    }

    fn set_permissions_sync(&self, permissions: Permissions) -> io::Result<()> {
        fs::set_permissions(self, permissions)
    }

    #[cfg(unix)]
    fn symlink_to_sync(&self, target: impl AsRef<StdPath>) -> io::Result<()> {
        std::os::unix::fs::symlink(target, self)
    }

    fn symlink_metadata_if_exists_sync(&self) -> io::Result<Option<Metadata>> {
        none_if_not_found(self.symlink_metadata_sync())
    }

    fn symlink_metadata_sync(&self) -> io::Result<Metadata> {
        fs::symlink_metadata(self)
    }

    fn touch_sync(&self) -> io::Result<()> {
        // Opening only when missing keeps `touch` working on directories and read-only files.
        if !self.try_exists_sync()? {
            OpenOptions::new().write(true).create(true).truncate(false).open(self)?;
        }

        fs::set_times(self, FileTimes::new().set_modified(SystemTime::now()))
    }

    fn write_atomic_sync(&self, contents: impl AsRef<[u8]>) -> io::Result<()> {
        self.write_atomic_with_sync(contents, |_| Ok(()))
    }

    fn write_atomic_with_sync(
        &self,
        contents: impl AsRef<[u8]>,
        prepare: impl FnOnce(&Path) -> io::Result<()>,
    ) -> io::Result<()> {
        let mut writer = self.atomic_writer_sync()?;
        prepare(writer.path())?;
        writer.write_all(contents.as_ref())?;
        writer.commit()
    }

    fn write_json_atomic_sync<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<()> {
        self.write_atomic_sync(serde_json::to_vec_pretty(data)?)
            .map_err(serde_json::Error::io)
    }

    fn write_json_sync<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<()> {
        self.write_sync(serde_json::to_vec_pretty(data)?)
            .map_err(serde_json::Error::io)
    }

    fn write_sync(&self, contents: impl AsRef<[u8]>) -> io::Result<()> {
        fs::write(self, contents)
    }
}

/// Metadata and file-type queries do not follow symlinks. On Unix, a retained entry can keep its
/// directory's file descriptor open after the iterator is dropped.
#[derive(Debug, From)]
pub struct PathEntry(DirEntry);

impl PathEntry {
    #[inline]
    pub fn path(&self) -> Path {
        Path::from(self.0.path())
    }

    #[inline]
    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }

    pub fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata()
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        self.0.file_type()
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

/// A pending replacement of a file, created by [`SyncFsOps::atomic_writer_sync`].
///
/// Writes go to a temporary file next to the destination, created like the one used by
/// [`SyncFsOps::write_atomic_with_sync`] (same permission rules). Dropping the writer without
/// committing tries to remove the temporary file; readers only ever see the old contents or the
/// complete new ones.
#[derive(Debug)]
pub struct AtomicWriter {
    file: File,
    dest: Path,
    temp: atomic_write::TempGuard,
}

impl AtomicWriter {
    /// The temporary file's path, for libraries that write by path. They must write in place;
    /// replacing the file at this path is not detected.
    #[inline]
    pub fn path(&self) -> &Path {
        &self.temp.path
    }

    #[inline]
    pub fn file(&self) -> &File {
        &self.file
    }

    #[inline]
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    /// Syncs the temporary file and renames it over the destination, as
    /// [`SyncFsOps::write_atomic_with_sync`] does; on Unix the parent directory is synced too.
    ///
    /// # Errors
    ///
    /// The destination is unchanged when syncing the file or renaming fails. A failure to sync the
    /// parent directory is reported after the destination was already replaced.
    pub fn commit(self) -> io::Result<()> {
        atomic_write::publish_replacing(self.file, self.temp, &self.dest)
    }

    /// Like [`Self::commit`], but never replaces an existing destination. Publishes through a hard
    /// link, so it needs a filesystem with hard links (not FAT or exFAT).
    ///
    /// # Errors
    ///
    /// Fails with [`io::ErrorKind::AlreadyExists`] when the destination exists.
    pub fn commit_noclobber(self) -> io::Result<()> {
        atomic_write::publish_noclobber(self.file, self.temp, &self.dest)
    }
}

impl Write for AtomicWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

#[derive(Debug)]
pub struct ReadDir(fs::ReadDir);

impl Iterator for ReadDir {
    type Item = io::Result<PathEntry>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|entry| entry.map(PathEntry))
    }
}

pub(crate) fn none_if_not_found<T>(result: io::Result<T>) -> io::Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
