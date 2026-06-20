//! Synchronous file system operations module
//!
//! This module provides the `SyncFsOps` trait for synchronous file system operations.
//! Implement this trait on any type to provide blocking file operations.
//!
//! # Example
//!
//! ```rust,ignore
//! use pathkit::{Path, SyncFsOps};
//!
//! let path = Path::new("/tmp/test.txt");
//! path.write_sync(b"Hello!")?;
//! let content = path.read_sync()?;
//! ```

use std::{
    fs::{
        self,
        File,
        Metadata,
        OpenOptions,
        Permissions,
        ReadDir,
    },
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

/// Trait for synchronous file system operations.
///
/// This trait provides blocking file system operations similar to Python's pathlib.
/// It is implemented for `Path` but can be implemented for other types as well.
///
/// # Example
///
/// ```rust,ignore
/// use pathkit::{Path, SyncFsOps};
///
/// let path = Path::new("/tmp/test.txt");
///
/// // Check if file exists
/// if path.exists_sync()? {
///     // Read file contents
///     let content = path.read_sync()?;
/// }
///
/// // Write to file
/// path.write_sync(b"Hello, world!")?;
///
/// // Get file size
/// let size = path.get_file_size_sync()?;
/// ```
pub trait SyncFsOps {
    #[cfg(unix)]
    fn chmod_sync(&self, mode: u32) -> Result<()>;
    #[cfg(unix)]
    fn chown_sync(&self, uid: Option<u32>, gid: Option<u32>) -> Result<()>;
    fn copy_file_sync(&self, dest: impl AsRef<Path>) -> Result<u64>;
    fn create_dir_all_sync(&self) -> Result<()>;
    fn create_dir_sync(&self) -> Result<()>;
    fn create_parent_dir_all_sync(&self) -> Result<bool>;
    fn create_parent_dir_sync(&self) -> Result<bool>;
    fn empty_dir_sync(&self) -> Result<()>;
    fn exists_sync(&self) -> Result<bool>;
    fn get_file_size_sync(&self) -> Result<u64>;
    fn hard_link_sync(&self, link: impl AsRef<Path>) -> Result<()>;
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
    fn soft_link_sync(&self, link: impl AsRef<Path>) -> Result<()>;
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

    fn copy_file_sync(&self, dest: impl AsRef<Path>) -> Result<u64> {
        Ok(fs::copy(self, dest.as_ref())?)
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

    fn hard_link_sync(&self, link: impl AsRef<Path>) -> Result<()> {
        Ok(fs::hard_link(self, link.as_ref())?)
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
    fn soft_link_sync(&self, link: impl AsRef<Path>) -> Result<()> {
        use std::os::unix::fs::symlink;

        Ok(symlink(self, link.as_ref())?)
    }

    fn symlink_metadata_sync(&self) -> Result<Metadata> {
        Ok(fs::symlink_metadata(self)?)
    }

    fn touch_sync(&self) -> Result<()> {
        if self.exists_sync()? {
            let t = SystemTime::now();
            set_file_mtime(self, FileTime::from_system_time(t))?;
        } else {
            File::create(self)?;
        }

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
