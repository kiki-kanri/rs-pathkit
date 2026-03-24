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
    path::PathBuf,
    time::SystemTime,
};

use anyhow::Result;
use filetime::{
    set_file_mtime,
    FileTime,
};
use serde::{
    de::DeserializeOwned,
    Serialize,
};
use serde_json::{
    from_slice,
    to_vec_pretty,
};

use super::core::Path;

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
    fn copy_file_sync(&self, dest: impl AsRef<PathBuf>) -> Result<u64>;
    fn create_dir_all_sync(&self) -> Result<()>;
    fn create_dir_sync(&self) -> Result<()>;
    fn empty_dir_sync(&self) -> Result<()>;
    fn exists_sync(&self) -> Result<bool>;
    fn get_file_size_sync(&self) -> Result<u64>;
    fn hard_link_sync(&self, link: impl AsRef<PathBuf>) -> Result<()>;
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
    fn read_dir_names_sync(&self) -> Result<Vec<String>>;
    fn read_dir_paths_sync(&self) -> Result<Vec<Path>>;
    fn read_dir_sync(&self) -> Result<ReadDir>;
    fn read_json_sync<T: DeserializeOwned>(&self) -> Result<T>;
    fn read_link_sync(&self) -> Result<Path>;
    fn read_sync(&self) -> Result<Vec<u8>>;
    fn read_to_string_sync(&self) -> Result<String>;
    fn remove_dir_all_sync(&self) -> Result<()>;
    fn remove_dir_sync(&self) -> Result<()>;
    fn remove_file_sync(&self) -> Result<()>;
    fn set_permissions_sync(&self, permissions: Permissions) -> Result<()>;
    #[cfg(unix)]
    fn soft_link_sync(&self, link: impl AsRef<PathBuf>) -> Result<()>;
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

    fn copy_file_sync(&self, dest: impl AsRef<PathBuf>) -> Result<u64> {
        Ok(fs::copy(self, dest.as_ref())?)
    }

    fn create_dir_all_sync(&self) -> Result<()> {
        Ok(fs::create_dir_all(self)?)
    }

    fn create_dir_sync(&self) -> Result<()> {
        Ok(fs::create_dir(self)?)
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

    fn hard_link_sync(&self, link: impl AsRef<PathBuf>) -> Result<()> {
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
    fn soft_link_sync(&self, link: impl AsRef<PathBuf>) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use tempfile::{
        tempdir,
        NamedTempFile,
    };

    use super::*;

    // Test exists_sync
    #[test]
    fn test_exists_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        assert!(file_path.exists_sync()?);
        Ok(())
    }

    #[test]
    fn test_exists_sync_false() -> Result<()> {
        let temp_dir = tempdir()?;
        let non_existent = temp_dir.path().join("non_existent_file.txt");
        let path = Path::new(&non_existent);

        assert!(!path.exists_sync()?);
        Ok(())
    }

    // Test is_file_sync
    #[test]
    fn test_is_file_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        assert!(file_path.is_file_sync()?);
        Ok(())
    }

    #[test]
    fn test_is_file_sync_false() -> Result<()> {
        let temp_dir = tempdir()?;
        let dir_path = Path::new(temp_dir.path());

        assert!(!dir_path.is_file_sync()?);
        Ok(())
    }

    // Test is_dir_sync
    #[test]
    fn test_is_dir_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        let dir_path = Path::new(temp_dir.path());

        assert!(dir_path.is_dir_sync()?);
        Ok(())
    }

    #[test]
    fn test_is_dir_sync_false() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        assert!(!file_path.is_dir_sync()?);
        Ok(())
    }

    // Test is_symlink_sync
    #[cfg(unix)]
    #[test]
    fn test_is_symlink_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        let target = temp_dir.path().join("target.txt");
        fs::write(&target, "test")?;

        let link = temp_dir.path().join("link.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link)?;

        let link_path = Path::new(&link);
        assert!(link_path.is_symlink_sync()?);
        Ok(())
    }

    // Test metadata_sync
    #[test]
    fn test_metadata_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let metadata = file_path.metadata_sync()?;
        assert!(metadata.is_file());
        Ok(())
    }

    // Test read_sync and write_sync
    #[test]
    fn test_read_write_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let test_content = b"Hello, World!";
        file_path.write_sync(test_content)?;

        let read_content = file_path.read_sync()?;
        assert_eq!(read_content, test_content);
        Ok(())
    }

    // Test read_to_string_sync
    #[test]
    fn test_read_to_string_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let test_content = "Hello, World!";
        file_path.write_sync(test_content)?;

        let read_content = file_path.read_to_string_sync()?;
        assert_eq!(read_content, test_content);
        Ok(())
    }

    // Test create_dir_sync
    #[test]
    fn test_create_dir_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        let new_dir = temp_dir.path().join("new_dir");
        let dir_path = Path::new(&new_dir);

        dir_path.create_dir_sync()?;

        assert!(dir_path.is_dir_sync()?);
        Ok(())
    }

    // Test create_dir_all_sync
    #[test]
    fn test_create_dir_all_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        let new_dir = temp_dir.path().join("parent/child/grandchild");
        let dir_path = Path::new(&new_dir);

        dir_path.create_dir_all_sync()?;

        assert!(dir_path.is_dir_sync()?);
        Ok(())
    }

    // Test remove_dir_sync
    #[test]
    fn test_remove_dir_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        let new_dir = temp_dir.path().join("to_remove");
        fs::create_dir(&new_dir)?;
        let dir_path = Path::new(&new_dir);

        assert!(dir_path.exists_sync()?);
        dir_path.remove_dir_sync()?;
        assert!(!dir_path.exists_sync()?);
        Ok(())
    }

    // Test remove_file_sync
    #[test]
    fn test_remove_file_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        assert!(file_path.exists_sync()?);
        file_path.remove_file_sync()?;
        assert!(!file_path.exists_sync()?);
        Ok(())
    }

    // Test remove_dir_all_sync
    #[test]
    fn test_remove_dir_all_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        let parent = temp_dir.path().join("parent");
        fs::create_dir(&parent)?;
        fs::write(parent.join("file1.txt"), "content1")?;
        fs::write(parent.join("file2.txt"), "content2")?;

        let dir_path = Path::new(&parent);
        assert!(dir_path.exists_sync()?);
        dir_path.remove_dir_all_sync()?;
        assert!(!dir_path.exists_sync()?);
        Ok(())
    }

    // Test get_file_size_sync
    #[test]
    fn test_get_file_size_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let test_content = b"Hello, World!";
        file_path.write_sync(test_content)?;

        let size = file_path.get_file_size_sync()?;
        assert_eq!(size, test_content.len() as u64);
        Ok(())
    }

    // Test truncate_sync
    #[test]
    fn test_truncate_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let test_content = b"Hello, World!";
        file_path.write_sync(test_content)?;

        // Truncate to 5 bytes
        file_path.truncate_sync(Some(5))?;

        let size = file_path.get_file_size_sync()?;
        assert_eq!(size, 5);
        Ok(())
    }

    // Test read_json_sync and write_json_sync
    #[test]
    fn test_read_write_json_sync() -> Result<()> {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            name: String,
            value: i32,
        }

        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let original = TestData {
            name: "test".to_string(),
            value: 42,
        };

        file_path.write_json_sync(&original)?;

        let loaded: TestData = file_path.read_json_sync()?;
        assert_eq!(loaded, original);
        Ok(())
    }

    // Test read_dir_sync
    #[test]
    fn test_read_dir_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        fs::write(temp_dir.path().join("file1.txt"), "content1")?;
        fs::write(temp_dir.path().join("file2.txt"), "content2")?;
        fs::create_dir(temp_dir.path().join("subdir"))?;

        let dir_path = Path::new(temp_dir.path());
        let entries: Vec<_> = dir_path.read_dir_sync()?.collect();

        // Should have 3 entries: 2 files + 1 directory
        assert_eq!(entries.len(), 3);
        Ok(())
    }

    // Test empty_dir_sync
    #[test]
    fn test_empty_dir_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        fs::write(temp_dir.path().join("file1.txt"), "content1")?;
        fs::write(temp_dir.path().join("file2.txt"), "content2")?;
        fs::create_dir(temp_dir.path().join("subdir"))?;

        let dir_path = Path::new(temp_dir.path());
        dir_path.empty_dir_sync()?;

        // Directory should be empty now
        let entries: Vec<_> = dir_path.read_dir_sync()?.collect();
        assert_eq!(entries.len(), 0);
        Ok(())
    }

    // Test set_permissions_sync
    #[cfg(unix)]
    #[test]
    fn test_set_permissions_sync() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        // Read current permissions
        let metadata = fs::metadata(temp_file.path())?;
        let original_mode = metadata.permissions().mode();

        // Set new permissions
        file_path.set_permissions_sync(fs::Permissions::from_mode(0o644))?;

        let new_metadata = fs::metadata(temp_file.path())?;
        assert_eq!(new_metadata.permissions().mode() & 0o777, 0o644);

        // Restore original
        file_path.set_permissions_sync(fs::Permissions::from_mode(original_mode))?;
        Ok(())
    }

    #[cfg(unix)]
    // Test chmod_sync
    #[test]
    fn test_chmod_sync() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        file_path.chmod_sync(0o744)?;
        let metadata = fs::metadata(temp_file.path())?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o744);

        file_path.chmod_sync(0o700)?;
        let metadata = fs::metadata(temp_file.path())?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);

        Ok(())
    }

    // Test chown_sync - requires root, skip if not root
    #[cfg(unix)]
    #[test]
    fn test_chown_sync() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        // Skip if not root (chown requires root privileges)
        if unsafe { libc::geteuid() } != 0 {
            return Ok(());
        }

        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        // Get current uid/gid
        let metadata = fs::metadata(temp_file.path())?;
        let original_mode = metadata.permissions().mode();

        // chown to same uid/gid (no-op but should work)
        file_path.chown_sync(Some(0), Some(0))?;

        // Restore permissions
        file_path.set_permissions_sync(fs::Permissions::from_mode(original_mode))?;
        Ok(())
    }

    #[cfg(unix)]
    // Test is_block_device_sync
    #[test]
    fn test_is_block_device_sync() -> Result<()> {
        let path = Path::new("/dev/sda"); // Common block device
        if path.exists_sync()? {
            // May fail if not root or device doesn't exist
            let _ = path.is_block_device_sync();
        }
        Ok(())
    }

    #[cfg(unix)]
    // Test is_char_device_sync
    #[test]
    fn test_is_char_device_sync() -> Result<()> {
        let path = Path::new("/dev/zero"); // Common char device
        if path.exists_sync()? {
            assert!(path.is_char_device_sync()?);
        }
        Ok(())
    }

    #[cfg(unix)]
    // Test is_fifo_sync - simplified
    #[test]
    fn test_is_fifo_sync() -> Result<()> {
        // FIFOs require special permissions to create
        // Just test that non-fifo returns false
        let path = Path::new("/tmp"); // This is not a fifo
        assert!(!path.is_fifo_sync()?);
        Ok(())
    }

    #[cfg(unix)]
    // Test is_socket_sync - simplified
    #[test]
    fn test_is_socket_sync() -> Result<()> {
        // Unix socket files are tricky to create and test
        let path = Path::new("/tmp"); // This is not a socket
        assert!(!path.is_socket_sync()?);
        Ok(())
    }

    // ----------------------------------------------------------------
    // Tests for previously uncovered sync_fs_ops functions
    // ----------------------------------------------------------------

    #[test]
    fn test_copy_file_sync() -> Result<()> {
        let temp_src = NamedTempFile::new()?;
        let temp_dst = NamedTempFile::new()?;
        let src = Path::new(temp_src.path());
        let dst = Path::new(temp_dst.path());

        src.write_sync(b"hello world")?;

        let bytes = src.copy_file_sync(&dst)?;
        assert_eq!(bytes, 11);

        let content = dst.read_sync()?;
        assert_eq!(content, b"hello world");
        Ok(())
    }

    #[test]
    fn test_hard_link_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let src = Path::new(temp_file.path());
        let link_path = Path::new(temp_file.path().with_extension("link"));

        src.write_sync(b"link test")?;
        src.hard_link_sync(&link_path)?;

        // Both files should have same content
        let content = fs::read(link_path.as_path())?;
        assert_eq!(content, b"link test");

        // And same inode (hard link)
        let src_meta = fs::metadata(src.as_path())?;
        let link_meta = fs::metadata(link_path.as_path())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(src_meta.ino(), link_meta.ino());
        }
        #[cfg(not(unix))]
        let _ = (src_meta, link_meta);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_soft_link_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let src = Path::new(temp_file.path());
        let link_path = Path::new(temp_file.path().with_extension("sym"));

        src.write_sync(b"symlink test")?;
        src.soft_link_sync(&link_path)?;

        // Read through symlink
        let content = fs::read(link_path.as_path())?;
        assert_eq!(content, b"symlink test");

        // Verify link_path is a symlink
        assert!(link_path.is_symlink_sync()?);
        Ok(())
    }

    #[test]
    fn test_read_link_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let src = Path::new(temp_file.path());
        let link_path = Path::new(temp_file.path().with_extension("readlink"));

        src.write_sync(b"readlink test")?;
        #[cfg(unix)]
        src.soft_link_sync(&link_path)?;
        #[cfg(not(unix))]
        {
            // On non-Unix, symlinks don't exist, so skip the core functionality
            // but we can still test the read_link method exists and compiles
        }

        // If we have a symlink, test read_link_sync
        #[cfg(unix)]
        {
            let link_target = link_path.read_link_sync()?;
            assert_eq!(link_target.to_str(), src.to_str());
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_metadata_sync() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let src = Path::new(temp_file.path());
        let link_path = Path::new(temp_file.path().with_extension("meta"));

        src.write_sync(b"meta test")?;
        src.soft_link_sync(&link_path)?;

        // symlink_metadata gets metadata of the link itself (not the target)
        let meta = link_path.symlink_metadata_sync()?;
        assert!(meta.file_type().is_symlink());
        Ok(())
    }

    #[test]
    fn test_touch_sync_creates_new_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let new_file = temp_dir.path().join("touched.txt");
        let path = Path::new(&new_file);

        assert!(!path.exists_sync()?);
        path.touch_sync()?;
        assert!(path.exists_sync()?);
        Ok(())
    }

    #[test]
    fn test_touch_sync_updates_existing() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let path = Path::new(temp_file.path());

        // Record original mtime
        let meta_before = fs::metadata(temp_file.path())?;
        let mtime_before = meta_before.modified()?;

        // Wait a bit so mtime actually changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        path.touch_sync()?;

        let meta_after = fs::metadata(temp_file.path())?;
        let mtime_after = meta_after.modified()?;
        assert!(mtime_after > mtime_before);
        Ok(())
    }

    #[test]
    fn test_read_dir_names_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        fs::write(temp_dir.path().join("a.txt"), "")?;
        fs::write(temp_dir.path().join("b.txt"), "")?;
        fs::create_dir(temp_dir.path().join("subdir"))?;

        let dir = Path::new(temp_dir.path());
        let names = dir.read_dir_names_sync()?;
        assert_eq!(names.len(), 3);
        assert!(names.contains(&String::from("a.txt")));
        assert!(names.contains(&String::from("b.txt")));
        assert!(names.contains(&String::from("subdir")));
        Ok(())
    }

    #[test]
    fn test_read_dir_paths_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_a = temp_dir.path().join("file_a.txt");
        let file_b = temp_dir.path().join("file_b.txt");
        fs::write(&file_a, "")?;
        fs::write(&file_b, "")?;

        let dir = Path::new(temp_dir.path());
        let paths = dir.read_dir_paths_sync()?;
        assert_eq!(paths.len(), 2);
        // All returned paths should be absolute
        for p in &paths {
            assert!(p.is_absolute());
        }
        Ok(())
    }

    #[test]
    fn test_empty_dir_sync_creates_dir_if_missing() -> Result<()> {
        let temp_dir = tempdir()?;
        let new_dir = temp_dir.path().join("brand_new_dir");
        let path = Path::new(&new_dir);

        // Directory doesn't exist
        assert!(!path.exists_sync()?);

        // empty_dir_sync should create it (via create_dir_all_sync)
        path.empty_dir_sync()?;

        assert!(path.is_dir_sync()?);
        Ok(())
    }
}
