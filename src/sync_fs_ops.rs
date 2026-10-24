//! Blocking filesystem operations for owned paths.
//!
//! [`SyncFsOps`] defines the shared operation contract and is implemented for [`Path`].

use std::{
    fs::{
        self,
        File,
        Metadata,
        OpenOptions,
        Permissions,
        ReadDir,
    },
    io::Result as IoResult,
    path::Path as StdPath,
    time::SystemTime,
};

use filetime::{
    FileTime,
    set_file_mtime,
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use serde_json::{
    Error as JsonError,
    Result as JsonResult,
    from_slice,
    to_vec_pretty,
};

use crate::{
    core::Path,
    entry::sync::PathEntry,
};

/// Blocking filesystem operations for paths.
///
/// [`Path`] implements this trait. For that implementation, methods borrow the stored
/// path without changing it; filesystem changes occur during the call. Paths are
/// interpreted by the operating system, including relative paths resolved against the
/// current working directory. Multi-step operations are not transactional and can leave
/// partial changes on failure. The behavior described below applies to that implementation.
///
/// The [`Path`] implementation provides these shared behaviors:
///
/// - Metadata and type checks follow symlinks, except [`Self::is_symlink_sync`] and
///   [`Self::symlink_metadata_sync`]. Type checks return an error for missing paths,
///   rather than `false`. [`Self::exists_sync`] returns `false` for a missing target,
///   including a dangling symlink, and propagates other inspection errors.
/// - [`Self::get_file_size_sync`] returns metadata length in bytes, which is not
///   necessarily allocated disk space or a meaningful content size for non-files.
/// - [`Self::open_sync`] opens an existing file read-only. [`Self::open_with_options_sync`]
///   uses the supplied options without changing them. Returned files follow
///   [`std::fs::File`] ownership and close behavior.
/// - [`Self::write_sync`] and [`Self::write_json_sync`] create or truncate a file without
///   creating parent directories. JSON uses pretty serialization. [`Self::read_json_sync`]
///   deserializes the complete file contents.
/// - [`Self::truncate_sync`] opens an existing file for writing and sets its length in
///   bytes; `None` means `0`. Extending a file fills the added range with zero bytes.
/// - [`Self::touch_sync`] updates only the modification time of an existing target to
///   the current time, or creates a missing file. It does not truncate existing content
///   or create parent directories.
/// - [`Self::create_parent_dir_sync`] creates only the immediate parent and returns `true`
///   on successful creation; an existing parent is an error. [`Self::create_parent_dir_all_sync`]
///   creates missing ancestors and returns `true` on success even if the parent already
///   exists. Both return `false` when the lexical path has no parent.
/// - [`Self::empty_dir_sync`] creates a missing directory with its parents or removes
///   the entries of an existing directory, keeping the directory itself. Removal is
///   incremental and uses the standard library's file and recursive-directory removal rules.
/// - Directory collection methods return entries in filesystem iteration order, without
///   sorting. [`Self::read_dir_names_sync`] converts names to strings with invalid Unicode
///   replaced; [`Self::read_dir_paths_sync`] and [`Self::read_dir_entries_sync`] retain
///   native path data. [`Self::read_dir_sync`] returns a lazy iterator whose entries can fail.
/// - [`Self::copy_file_sync`] copies to the exact destination, overwriting an existing file
///   and returning the number of bytes copied. [`Self::rename_to_sync`] renames to the exact
///   destination and returns a new [`Path`]; it does not update the source value or fall back
///   to copying across filesystems. Replacement rules depend on the platform.
/// - Link methods use `self` as the source or target and `link` as the new link path.
///   [`Self::hard_link_sync`] requires filesystem support. Unix-only symbolic-link methods
///   store the target as supplied and return the stored target without resolving it.
///
/// Unix-only methods expose permission modes, owner and group IDs, and special-file
/// checks. For ownership changes, `None` preserves the corresponding owner or group ID.
/// Filesystem methods otherwise follow the corresponding [`std::fs`] operations.
///
/// # Errors
///
/// Propagates filesystem errors such as missing paths, permission errors, invalid file
/// or directory types, and unsupported operations. Directory collection methods also
/// propagate iteration errors instead of returning partial collections. UTF-8 string
/// reads fail for invalid UTF-8; JSON methods return [`serde_json::Error`], with
/// filesystem errors wrapped by [`serde_json::Error::io`]. Changes made before an error
/// are not rolled back.
///
/// # Examples
///
/// ```rust
/// use pathkit::{
///     SyncFsOps,
///     path,
/// };
///
/// let directory = tempfile::tempdir()?;
/// let file = path!(directory.path()) / "message.txt";
/// file.write_sync(b"Hello!")?;
/// assert_eq!(file.read_sync()?, b"Hello!");
/// assert_eq!(file.get_file_size_sync()?, 6);
/// # Ok::<(), std::io::Error>(())
/// ```
pub trait SyncFsOps {
    /// Returns a new absolute path with symlinks resolved by the filesystem.
    ///
    /// Uses [`std::fs::canonicalize`], leaving the original value unchanged. Unlike
    /// [`Path::absolutize`], this operation requires the path to exist. On Windows, the
    /// result uses the standard library's extended-length path representation.
    ///
    /// # Errors
    ///
    /// Propagates filesystem errors, including missing path components, permission errors,
    /// and non-directory components before the end of the path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::{
    ///     SyncFsOps,
    ///     path,
    /// };
    ///
    /// let directory = tempfile::tempdir()?;
    /// let canonical = path!(directory.path()).canonicalize_sync()?;
    /// assert!(canonical.is_absolute());
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn canonicalize_sync(&self) -> IoResult<Path>;

    #[cfg(unix)]
    fn chmod_sync(&self, mode: u32) -> IoResult<()>;

    #[cfg(unix)]
    fn chown_sync(&self, uid: Option<u32>, gid: Option<u32>) -> IoResult<()>;

    fn copy_file_sync(&self, dest: impl AsRef<StdPath>) -> IoResult<u64>;

    fn create_dir_all_sync(&self) -> IoResult<()>;

    fn create_dir_sync(&self) -> IoResult<()>;

    fn create_parent_dir_all_sync(&self) -> IoResult<bool>;

    fn create_parent_dir_sync(&self) -> IoResult<bool>;

    fn empty_dir_sync(&self) -> IoResult<()>;

    fn exists_sync(&self) -> IoResult<bool>;

    fn get_file_size_sync(&self) -> IoResult<u64>;

    fn hard_link_sync(&self, link: impl AsRef<StdPath>) -> IoResult<()>;

    #[cfg(unix)]
    fn is_block_device_sync(&self) -> IoResult<bool>;

    #[cfg(unix)]
    fn is_char_device_sync(&self) -> IoResult<bool>;

    fn is_dir_sync(&self) -> IoResult<bool>;

    #[cfg(unix)]
    fn is_fifo_sync(&self) -> IoResult<bool>;

    fn is_file_sync(&self) -> IoResult<bool>;

    #[cfg(unix)]
    fn is_socket_sync(&self) -> IoResult<bool>;

    fn is_symlink_sync(&self) -> IoResult<bool>;

    fn metadata_sync(&self) -> IoResult<Metadata>;

    fn open_sync(&self) -> IoResult<File>;

    fn open_with_options_sync(&self, options: &OpenOptions) -> IoResult<File>;

    fn read_dir_entries_sync(&self) -> IoResult<Vec<PathEntry>>;

    fn read_dir_names_sync(&self) -> IoResult<Vec<String>>;

    fn read_dir_paths_sync(&self) -> IoResult<Vec<Path>>;

    fn read_dir_sync(&self) -> IoResult<ReadDir>;

    fn read_json_sync<T: DeserializeOwned>(&self) -> JsonResult<T>;

    #[cfg(unix)]
    fn read_link_sync(&self) -> IoResult<Path>;

    fn read_sync(&self) -> IoResult<Vec<u8>>;

    fn read_to_string_sync(&self) -> IoResult<String>;

    fn remove_dir_all_sync(&self) -> IoResult<()>;

    fn remove_dir_sync(&self) -> IoResult<()>;

    fn remove_file_sync(&self) -> IoResult<()>;

    fn rename_to_sync(&self, dest: impl AsRef<StdPath>) -> IoResult<Path>;

    fn set_permissions_sync(&self, permissions: Permissions) -> IoResult<()>;

    #[cfg(unix)]
    fn soft_link_sync(&self, link: impl AsRef<StdPath>) -> IoResult<()>;

    fn symlink_metadata_sync(&self) -> IoResult<Metadata>;

    fn touch_sync(&self) -> IoResult<()>;

    fn truncate_sync(&self, len: Option<u64>) -> IoResult<()>;

    fn write_json_sync<T: Serialize>(&self, data: T) -> JsonResult<()>;

    fn write_sync(&self, contents: impl AsRef<[u8]>) -> IoResult<()>;
}

impl SyncFsOps for Path {
    fn canonicalize_sync(&self) -> IoResult<Path> {
        fs::canonicalize(self).map(Self::new)
    }

    #[cfg(unix)]
    fn chmod_sync(&self, mode: u32) -> IoResult<()> {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(self, Permissions::from_mode(mode))
    }

    #[cfg(unix)]
    fn chown_sync(&self, uid: Option<u32>, gid: Option<u32>) -> IoResult<()> {
        std::os::unix::fs::chown(self, uid, gid)
    }

    fn copy_file_sync(&self, dest: impl AsRef<StdPath>) -> IoResult<u64> {
        fs::copy(self, dest)
    }

    fn create_dir_all_sync(&self) -> IoResult<()> {
        fs::create_dir_all(self)
    }

    fn create_dir_sync(&self) -> IoResult<()> {
        fs::create_dir(self)
    }

    fn create_parent_dir_all_sync(&self) -> IoResult<bool> {
        if let Some(parent) = self.parent() {
            parent.create_dir_all_sync()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn create_parent_dir_sync(&self) -> IoResult<bool> {
        if let Some(parent) = self.parent() {
            parent.create_dir_sync()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn empty_dir_sync(&self) -> IoResult<()> {
        if !self.exists_sync()? {
            return self.create_dir_all_sync();
        }

        for entry in fs::read_dir(self)? {
            let entry_path = entry?.path();
            if entry_path.is_dir() {
                fs::remove_dir_all(entry_path)?;
            } else {
                fs::remove_file(entry_path)?;
            }
        }

        Ok(())
    }

    fn exists_sync(&self) -> IoResult<bool> {
        self.try_exists()
    }

    fn get_file_size_sync(&self) -> IoResult<u64> {
        Ok(self.metadata_sync()?.len())
    }

    fn hard_link_sync(&self, link: impl AsRef<StdPath>) -> IoResult<()> {
        fs::hard_link(self, link)
    }

    #[cfg(unix)]
    fn is_block_device_sync(&self) -> IoResult<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_block_device())
    }

    #[cfg(unix)]
    fn is_char_device_sync(&self) -> IoResult<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_char_device())
    }

    fn is_dir_sync(&self) -> IoResult<bool> {
        Ok(self.metadata_sync()?.is_dir())
    }

    #[cfg(unix)]
    fn is_fifo_sync(&self) -> IoResult<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_fifo())
    }

    fn is_file_sync(&self) -> IoResult<bool> {
        Ok(self.metadata_sync()?.is_file())
    }

    #[cfg(unix)]
    fn is_socket_sync(&self) -> IoResult<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_socket())
    }

    fn is_symlink_sync(&self) -> IoResult<bool> {
        Ok(self.symlink_metadata_sync()?.file_type().is_symlink())
    }

    fn metadata_sync(&self) -> IoResult<Metadata> {
        fs::metadata(self)
    }

    fn open_sync(&self) -> IoResult<File> {
        File::open(self)
    }

    fn open_with_options_sync(&self, options: &OpenOptions) -> IoResult<File> {
        options.open(self)
    }

    fn read_dir_entries_sync(&self) -> IoResult<Vec<PathEntry>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(self)? {
            entries.push(PathEntry::new(entry?));
        }

        Ok(entries)
    }

    fn read_dir_names_sync(&self) -> IoResult<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(self)? {
            names.push(entry?.file_name().to_string_lossy().into());
        }

        Ok(names)
    }

    fn read_dir_paths_sync(&self) -> IoResult<Vec<Path>> {
        let mut paths = Vec::new();
        for entry in fs::read_dir(self)? {
            paths.push(Self::new(entry?.path()));
        }

        Ok(paths)
    }

    fn read_dir_sync(&self) -> IoResult<ReadDir> {
        fs::read_dir(self)
    }

    fn read_json_sync<T: DeserializeOwned>(&self) -> JsonResult<T> {
        from_slice(&self.read_sync().map_err(JsonError::io)?)
    }

    #[cfg(unix)]
    fn read_link_sync(&self) -> IoResult<Path> {
        Ok(Self::new(fs::read_link(self)?))
    }

    fn read_sync(&self) -> IoResult<Vec<u8>> {
        fs::read(self)
    }

    fn read_to_string_sync(&self) -> IoResult<String> {
        fs::read_to_string(self)
    }

    fn remove_dir_all_sync(&self) -> IoResult<()> {
        fs::remove_dir_all(self)
    }

    fn remove_dir_sync(&self) -> IoResult<()> {
        fs::remove_dir(self)
    }

    fn remove_file_sync(&self) -> IoResult<()> {
        fs::remove_file(self)
    }

    fn rename_to_sync(&self, dest: impl AsRef<StdPath>) -> IoResult<Path> {
        fs::rename(self, &dest)?;
        Ok(crate::path!(dest))
    }

    fn set_permissions_sync(&self, permissions: Permissions) -> IoResult<()> {
        fs::set_permissions(self, permissions)
    }

    #[cfg(unix)]
    fn soft_link_sync(&self, link: impl AsRef<StdPath>) -> IoResult<()> {
        use std::os::unix::fs::symlink;

        symlink(self, link)
    }

    fn symlink_metadata_sync(&self) -> IoResult<Metadata> {
        fs::symlink_metadata(self)
    }

    fn touch_sync(&self) -> IoResult<()> {
        if !self.exists_sync()? {
            OpenOptions::new().write(true).create(true).truncate(false).open(self)?;
        }

        let t = SystemTime::now();
        set_file_mtime(self, FileTime::from_system_time(t))?;
        Ok(())
    }

    fn truncate_sync(&self, len: Option<u64>) -> IoResult<()> {
        OpenOptions::new().write(true).open(self)?.set_len(len.unwrap_or(0))
    }

    fn write_json_sync<T: Serialize>(&self, data: T) -> JsonResult<()> {
        self.write_sync(to_vec_pretty(&data)?).map_err(JsonError::io)
    }

    fn write_sync(&self, contents: impl AsRef<[u8]>) -> IoResult<()> {
        fs::write(self, contents)
    }
}
