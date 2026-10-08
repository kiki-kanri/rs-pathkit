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
    path::Path as StdPath,
    time::SystemTime,
};

use anyhow::Result;
use filetime::{
    FileTime,
    set_file_mtime,
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use serde_json::{
    from_slice,
    to_vec_pretty,
};

use super::{
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
///   and returning the number of bytes copied. [`Self::move_to_sync`] renames to the exact
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
/// reads fail for invalid UTF-8; JSON reads and writes also propagate deserialization
/// or serialization errors. Changes made before an error are not rolled back.
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
/// # Ok::<(), anyhow::Error>(())
/// ```
pub trait SyncFsOps {
    #[cfg(unix)]
    fn chmod_sync(&self, mode: u32) -> Result<()>;
    #[cfg(unix)]
    fn chown_sync(&self, uid: Option<u32>, gid: Option<u32>) -> Result<()>;
    fn copy_file_sync(&self, dest: impl AsRef<StdPath>) -> Result<u64>;
    fn create_dir_all_sync(&self) -> Result<()>;
    fn create_dir_sync(&self) -> Result<()>;
    fn create_parent_dir_all_sync(&self) -> Result<bool>;
    fn create_parent_dir_sync(&self) -> Result<bool>;
    fn empty_dir_sync(&self) -> Result<()>;
    fn exists_sync(&self) -> Result<bool>;
    fn get_file_size_sync(&self) -> Result<u64>;
    fn hard_link_sync(&self, link: impl AsRef<StdPath>) -> Result<()>;
    #[cfg(unix)]
    fn is_block_device_sync(&self) -> Result<bool>;
    #[cfg(unix)]
    fn is_char_device_sync(&self) -> Result<bool>;
    fn is_dir_sync(&self) -> Result<bool>;
    #[cfg(unix)]
    fn is_fifo_sync(&self) -> Result<bool>;
    fn is_file_sync(&self) -> Result<bool>;
    #[cfg(unix)]
    fn is_socket_sync(&self) -> Result<bool>;
    fn is_symlink_sync(&self) -> Result<bool>;
    fn metadata_sync(&self) -> Result<Metadata>;
    fn move_to_sync(&self, dest: impl AsRef<StdPath>) -> Result<Path>;
    fn open_sync(&self) -> Result<File>;
    fn open_with_options_sync(&self, options: &OpenOptions) -> Result<File>;
    fn read_dir_entries_sync(&self) -> Result<Vec<PathEntry>>;
    fn read_dir_names_sync(&self) -> Result<Vec<String>>;
    fn read_dir_paths_sync(&self) -> Result<Vec<Path>>;
    fn read_dir_sync(&self) -> Result<ReadDir>;
    fn read_json_sync<T: DeserializeOwned>(&self) -> Result<T>;
    #[cfg(unix)]
    fn read_link_sync(&self) -> Result<Path>;
    fn read_sync(&self) -> Result<Vec<u8>>;
    fn read_to_string_sync(&self) -> Result<String>;
    fn remove_dir_all_sync(&self) -> Result<()>;
    fn remove_dir_sync(&self) -> Result<()>;
    fn remove_file_sync(&self) -> Result<()>;
    fn set_permissions_sync(&self, permissions: Permissions) -> Result<()>;
    #[cfg(unix)]
    fn soft_link_sync(&self, link: impl AsRef<StdPath>) -> Result<()>;
    fn symlink_metadata_sync(&self) -> Result<Metadata>;
    fn touch_sync(&self) -> Result<()>;
    fn truncate_sync(&self, len: Option<u64>) -> Result<()>;
    fn write_json_sync<T: Serialize>(&self, data: T) -> Result<()>;
    fn write_sync(&self, contents: impl AsRef<[u8]>) -> Result<()>;
}

impl SyncFsOps for Path {
    #[cfg(unix)]
    fn chmod_sync(&self, mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        Ok(fs::set_permissions(self, Permissions::from_mode(mode))?)
    }

    #[cfg(unix)]
    fn chown_sync(&self, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
        Ok(std::os::unix::fs::chown(self, uid, gid)?)
    }

    fn copy_file_sync(&self, dest: impl AsRef<StdPath>) -> Result<u64> {
        Ok(fs::copy(self, dest)?)
    }

    fn create_dir_all_sync(&self) -> Result<()> {
        Ok(fs::create_dir_all(self)?)
    }

    fn create_dir_sync(&self) -> Result<()> {
        Ok(fs::create_dir(self)?)
    }

    fn create_parent_dir_all_sync(&self) -> Result<bool> {
        if let Some(parent) = self.parent() {
            parent.create_dir_all_sync()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn create_parent_dir_sync(&self) -> Result<bool> {
        if let Some(parent) = self.parent() {
            parent.create_dir_sync()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn empty_dir_sync(&self) -> Result<()> {
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

    fn exists_sync(&self) -> Result<bool> {
        Ok(self.try_exists()?)
    }

    fn get_file_size_sync(&self) -> Result<u64> {
        Ok(self.metadata_sync()?.len())
    }

    fn hard_link_sync(&self, link: impl AsRef<StdPath>) -> Result<()> {
        Ok(fs::hard_link(self, link)?)
    }

    #[cfg(unix)]
    fn is_block_device_sync(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_block_device())
    }

    #[cfg(unix)]
    fn is_char_device_sync(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_char_device())
    }

    fn is_dir_sync(&self) -> Result<bool> {
        Ok(self.metadata_sync()?.is_dir())
    }

    #[cfg(unix)]
    fn is_fifo_sync(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_fifo())
    }

    fn is_file_sync(&self) -> Result<bool> {
        Ok(self.metadata_sync()?.is_file())
    }

    #[cfg(unix)]
    fn is_socket_sync(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;

        Ok(self.metadata_sync()?.file_type().is_socket())
    }

    fn is_symlink_sync(&self) -> Result<bool> {
        Ok(fs::symlink_metadata(self)?.file_type().is_symlink())
    }

    fn metadata_sync(&self) -> Result<Metadata> {
        Ok(fs::metadata(self)?)
    }

    fn move_to_sync(&self, dest: impl AsRef<StdPath>) -> Result<Path> {
        let dest = Path::new(dest);
        fs::rename(self, &dest)?;
        Ok(dest)
    }

    fn open_sync(&self) -> Result<File> {
        Ok(File::open(self)?)
    }

    fn open_with_options_sync(&self, options: &OpenOptions) -> Result<File> {
        Ok(options.open(self)?)
    }

    fn read_dir_entries_sync(&self) -> Result<Vec<PathEntry>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(self)? {
            entries.push(PathEntry::new(entry?));
        }

        Ok(entries)
    }

    fn read_dir_names_sync(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(self)? {
            names.push(entry?.file_name().to_string_lossy().into());
        }

        Ok(names)
    }

    fn read_dir_paths_sync(&self) -> Result<Vec<Path>> {
        let mut paths = Vec::new();
        for entry in fs::read_dir(self)? {
            paths.push(Self::new(entry?.path()));
        }

        Ok(paths)
    }

    fn read_dir_sync(&self) -> Result<ReadDir> {
        Ok(fs::read_dir(self)?)
    }

    fn read_json_sync<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(from_slice::<T>(&self.read_sync()?)?)
    }

    #[cfg(unix)]
    fn read_link_sync(&self) -> Result<Path> {
        Ok(Self::new(fs::read_link(self)?))
    }

    fn read_sync(&self) -> Result<Vec<u8>> {
        Ok(fs::read(self)?)
    }

    fn read_to_string_sync(&self) -> Result<String> {
        Ok(fs::read_to_string(self)?)
    }

    fn remove_dir_all_sync(&self) -> Result<()> {
        Ok(fs::remove_dir_all(self)?)
    }

    fn remove_dir_sync(&self) -> Result<()> {
        Ok(fs::remove_dir(self)?)
    }

    fn remove_file_sync(&self) -> Result<()> {
        Ok(fs::remove_file(self)?)
    }

    fn set_permissions_sync(&self, permissions: Permissions) -> Result<()> {
        Ok(fs::set_permissions(self, permissions)?)
    }

    #[cfg(unix)]
    fn soft_link_sync(&self, link: impl AsRef<StdPath>) -> Result<()> {
        use std::os::unix::fs::symlink;

        Ok(symlink(self, link)?)
    }

    fn symlink_metadata_sync(&self) -> Result<Metadata> {
        Ok(fs::symlink_metadata(self)?)
    }

    fn touch_sync(&self) -> Result<()> {
        if !self.exists_sync()? {
            OpenOptions::new().write(true).create(true).truncate(false).open(self)?;
        }

        let t = SystemTime::now();
        set_file_mtime(self, FileTime::from_system_time(t))?;
        Ok(())
    }

    fn truncate_sync(&self, len: Option<u64>) -> Result<()> {
        Ok(OpenOptions::new().write(true).open(self)?.set_len(len.unwrap_or(0))?)
    }

    fn write_json_sync<T: Serialize>(&self, data: T) -> Result<()> {
        self.write_sync(to_vec_pretty(&data)?)
    }

    fn write_sync(&self, contents: impl AsRef<[u8]>) -> Result<()> {
        Ok(fs::write(self, contents)?)
    }
}
