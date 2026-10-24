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
    io::{
        Error as IoError,
        ErrorKind as IoErrorKind,
        Result as IoResult,
        Write,
    },
    path::Path as StdPath,
    process,
    sync::atomic::{
        AtomicU64,
        Ordering,
    },
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
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
/// - [`Self::write_atomic_sync`] writes to a temporary file in the destination's directory,
///   syncs it, and replaces the destination, so readers see either the old or the new
///   contents. [`Self::write_atomic_with_sync`] first runs `prepare` on the empty temporary
///   file, for example to set permissions. The temporary file is created with mode `0600` on
///   Unix or inherits the parent directory's ACL on Windows; the destination keeps its final
///   permissions. On failure, the temporary file is removed when possible.
/// - [`Self::persist_to_sync`] syncs this file and replaces the exact destination the same
///   way, without falling back to copying across filesystems. On Windows, replacement can fail
///   while the destination is open or when a path exceeds `MAX_PATH`.
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
///   Child symlinks are removed without traversing their targets.
/// - Directory collection methods return entries in filesystem iteration order, without
///   sorting. [`Self::read_dir_names_sync`] converts names to strings with invalid Unicode
///   replaced; [`Self::read_dir_paths_sync`] and [`Self::read_dir_entries_sync`] retain
///   native path data. [`Self::read_dir_sync`] returns a lazy iterator whose entries can fail.
/// - [`Self::copy_file_sync`] copies to the exact destination, overwriting an existing file
///   and returning the number of bytes copied. [`Self::rename_to_sync`] renames to the exact
///   destination and returns a new [`Path`]; it does not update the source value or fall back
///   to copying across filesystems. Replacement rules depend on the platform.
/// - Link methods use `self` as the source or target and `link` as the new link path.
///   [`Self::hard_link_sync`] requires filesystem support. [`Self::read_link_sync`]
///   returns the stored target without resolving it. The Unix-only `soft_link_sync`
///   method stores the target as supplied.
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

    fn persist_to_sync(&self, dest: impl AsRef<StdPath>) -> IoResult<Path>;

    fn read_dir_entries_sync(&self) -> IoResult<Vec<PathEntry>>;

    fn read_dir_names_sync(&self) -> IoResult<Vec<String>>;

    fn read_dir_paths_sync(&self) -> IoResult<Vec<Path>>;

    fn read_dir_sync(&self) -> IoResult<ReadDir>;

    fn read_json_sync<T: DeserializeOwned>(&self) -> JsonResult<T>;

    fn read_link_sync(&self) -> IoResult<Path>;

    fn read_sync(&self) -> IoResult<Vec<u8>>;

    fn read_to_string_sync(&self) -> IoResult<String>;

    fn remove_dir_all_sync(&self) -> IoResult<()>;

    fn remove_dir_sync(&self) -> IoResult<()>;

    fn remove_file_sync(&self) -> IoResult<()>;

    fn rename_to_sync(&self, dest: impl AsRef<StdPath>) -> IoResult<Path>;

    fn set_permissions_sync(&self, permissions: Permissions) -> IoResult<()>;

    // TODO: Add Windows `symlink_file` and `symlink_dir` variants; Windows requires the link type
    // to be chosen explicitly instead of inferred from the target.
    #[cfg(unix)]
    fn soft_link_sync(&self, link: impl AsRef<StdPath>) -> IoResult<()>;

    fn symlink_metadata_sync(&self) -> IoResult<Metadata>;

    fn touch_sync(&self) -> IoResult<()>;

    fn truncate_sync(&self, len: Option<u64>) -> IoResult<()>;

    fn write_atomic_sync(&self, contents: impl AsRef<[u8]>) -> IoResult<()>;

    fn write_atomic_with_sync(
        &self,
        contents: impl AsRef<[u8]>,
        prepare: impl FnOnce(&Path) -> IoResult<()>,
    ) -> IoResult<()>;

    fn write_json_sync<T: Serialize>(&self, data: T) -> JsonResult<()>;

    fn write_sync(&self, contents: impl AsRef<[u8]>) -> IoResult<()>;
}

impl SyncFsOps for Path {
    fn canonicalize_sync(&self) -> IoResult<Path> {
        fs::canonicalize(self).map(Self::from)
    }

    #[cfg(unix)]
    fn chmod_sync(&self, mode: u32) -> IoResult<()> {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(self, Permissions::from_mode(mode))
    }

    #[cfg(unix)]
    fn chown_sync(&self, uid: Option<u32>, gid: Option<u32>) -> IoResult<()> {
        use std::os::unix::fs::chown;

        chown(self, uid, gid)
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
        #[cfg(windows)]
        use std::os::windows::fs::FileTypeExt;

        if !self.exists_sync()? {
            return self.create_dir_all_sync();
        }

        for entry in fs::read_dir(self)? {
            let entry = entry?;
            let entry_path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                fs::remove_dir_all(entry_path)?;
            } else {
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

    fn persist_to_sync(&self, dest: impl AsRef<StdPath>) -> IoResult<Path> {
        sync_file(self)?;
        replace_durably(self, dest.as_ref())?;
        Ok(Self::new(dest))
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
            paths.push(Self::from(entry?.path()));
        }

        Ok(paths)
    }

    fn read_dir_sync(&self) -> IoResult<ReadDir> {
        fs::read_dir(self)
    }

    fn read_json_sync<T: DeserializeOwned>(&self) -> JsonResult<T> {
        from_slice(&self.read_sync().map_err(JsonError::io)?)
    }

    fn read_link_sync(&self) -> IoResult<Path> {
        fs::read_link(self).map(Self::from)
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

        set_file_mtime(self, FileTime::now())
    }

    fn truncate_sync(&self, len: Option<u64>) -> IoResult<()> {
        OpenOptions::new().write(true).open(self)?.set_len(len.unwrap_or(0))
    }

    fn write_atomic_sync(&self, contents: impl AsRef<[u8]>) -> IoResult<()> {
        self.write_atomic_with_sync(contents, |_| Ok(()))
    }

    fn write_atomic_with_sync(
        &self,
        contents: impl AsRef<[u8]>,
        prepare: impl FnOnce(&Path) -> IoResult<()>,
    ) -> IoResult<()> {
        let (temp_path, file) = create_temp_sibling(self)?;
        let result = write_and_replace(file, &temp_path, self, contents.as_ref(), prepare);
        if result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }

        result
    }

    fn write_json_sync<T: Serialize>(&self, data: T) -> JsonResult<()> {
        self.write_sync(to_vec_pretty(&data)?).map_err(JsonError::io)
    }

    fn write_sync(&self, contents: impl AsRef<[u8]>) -> IoResult<()> {
        fs::write(self, contents)
    }
}

// Constants/Statics
const TEMP_FILE_ATTEMPTS: u32 = 64;
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

// Functions
fn create_temp_sibling(path: &Path) -> IoResult<(Path, File)> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    if path.file_name().is_none() {
        return Err(IoError::new(
            IoErrorKind::InvalidInput,
            "atomic write path has no file name",
        ));
    }

    let parent = parent_or_current_dir(path);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.subsec_nanos());

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);

    for _ in 0..TEMP_FILE_ATTEMPTS {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        // Keep the name short and independent of the destination to stay within name length limits.
        let temp_path = Path::from(parent.join(format!(".pathkit-{:x}-{nanos:x}-{counter:x}.tmp", process::id())));
        match options.open(&temp_path) {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == IoErrorKind::AlreadyExists => {},
            Err(error) => return Err(error),
        }
    }

    Err(IoError::new(
        IoErrorKind::AlreadyExists,
        "failed to create a unique temporary file for atomic write",
    ))
}

fn parent_or_current_dir(path: &StdPath) -> &StdPath {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| StdPath::new("."))
}

#[cfg(not(windows))]
fn replace_durably(source: &StdPath, dest: &StdPath) -> IoResult<()> {
    fs::rename(source, dest)?;

    #[cfg(unix)]
    File::open(parent_or_current_dir(dest))?.sync_all()?;
    Ok(())
}

#[cfg(windows)]
fn replace_durably(source: &StdPath, dest: &StdPath) -> IoResult<()> {
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING,
        MOVEFILE_WRITE_THROUGH,
        MoveFileExW,
    };

    fn wide_path(path: &StdPath) -> IoResult<Vec<u16>> {
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        if wide.contains(&0) {
            return Err(IoError::new(IoErrorKind::InvalidInput, "path contains a NUL character"));
        }

        wide.push(0);
        Ok(wide)
    }

    let source = wide_path(source)?;
    let dest = wide_path(dest)?;

    // SAFETY: both paths are NUL-terminated UTF-16 strings without interior NULs that outlive the call.
    let replaced = unsafe {
        MoveFileExW(
            source.as_ptr(),
            dest.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };

    if replaced == 0 {
        return Err(IoError::last_os_error());
    }

    Ok(())
}

// TODO: On macOS, `sync_all` uses `F_FULLFSYNC`, which some filesystems (SMB, VirtioFS, some FUSE)
// reject; fall back to `fsync` for those errors here and in `write_and_replace` and `replace_durably`.
fn sync_file(path: &StdPath) -> IoResult<()> {
    // Windows requires write access to flush file buffers.
    #[cfg(windows)]
    let file = OpenOptions::new().write(true).open(path)?;
    #[cfg(not(windows))]
    let file = File::open(path)?;

    file.sync_all()
}

fn write_and_replace(
    mut file: File,
    temp_path: &Path,
    dest: &Path,
    contents: &[u8],
    prepare: impl FnOnce(&Path) -> IoResult<()>,
) -> IoResult<()> {
    prepare(temp_path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    // Windows cannot replace the destination while this handle is open.
    drop(file);
    replace_durably(temp_path, dest)
}
