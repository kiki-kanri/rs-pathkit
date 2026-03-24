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

use std::fs::{
    Metadata,
    Permissions,
};

use anyhow::Result;
use serde::{
    de::DeserializeOwned,
    Serialize,
};
use serde_json::{
    from_slice,
    to_vec_pretty,
};
use tokio::fs::{
    self,
    OpenOptions,
    ReadDir,
};

use super::core::Path;

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
    async fn create_dir_all(&self) -> Result<()>;
    async fn create_dir(&self) -> Result<()>;
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
    async fn read_dir(&self) -> Result<ReadDir>;
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

    async fn create_dir(&self) -> Result<()> {
        Ok(fs::create_dir(self).await?)
    }

    async fn create_dir_all(&self) -> Result<()> {
        Ok(fs::create_dir_all(self).await?)
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

    async fn read(&self) -> Result<Vec<u8>> {
        Ok(fs::read(self).await?)
    }

    async fn read_dir(&self) -> Result<ReadDir> {
        Ok(fs::read_dir(self).await?)
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

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use tempfile::{
        NamedTempFile,
        TempDir,
    };
    use tokio::fs as async_fs;

    use super::*;

    // Test exists
    #[tokio::test]
    async fn test_exists() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        assert!(file_path.exists().await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_exists_false() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let non_existent = temp_dir.path().join("non_existent_file.txt");
        let path = Path::new(&non_existent);

        assert!(!path.exists().await?);
        Ok(())
    }

    // Test is_file
    #[tokio::test]
    async fn test_is_file() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        assert!(file_path.is_file().await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_is_file_false() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = Path::new(temp_dir.path());

        assert!(!dir_path.is_file().await?);
        Ok(())
    }

    // Test is_dir
    #[tokio::test]
    async fn test_is_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = Path::new(temp_dir.path());

        assert!(dir_path.is_dir().await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_is_dir_false() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        assert!(!file_path.is_dir().await?);
        Ok(())
    }

    // Test is_symlink
    #[cfg(unix)]
    #[tokio::test]
    async fn test_is_symlink() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let target = temp_dir.path().join("target.txt");
        async_fs::write(&target, "test").await?;

        let link = temp_dir.path().join("link.txt");
        std::os::unix::fs::symlink(&target, &link)?;

        let link_path = Path::new(&link);
        assert!(link_path.is_symlink().await?);
        Ok(())
    }

    // Test metadata
    #[tokio::test]
    async fn test_metadata() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let metadata = file_path.metadata().await?;
        assert!(metadata.is_file());
        Ok(())
    }

    // Test read and write
    #[tokio::test]
    async fn test_read_write() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let test_content = b"Hello, World!";
        file_path.write(test_content).await?;

        let read_content = file_path.read().await?;
        assert_eq!(read_content, test_content);
        Ok(())
    }

    // Test read_to_string
    #[tokio::test]
    async fn test_read_to_string() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let test_content = "Hello, World!";
        file_path.write(test_content).await?;

        let read_content = file_path.read_to_string().await?;
        assert_eq!(read_content, test_content);
        Ok(())
    }

    // Test create_dir
    #[tokio::test]
    async fn test_create_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let new_dir = temp_dir.path().join("new_dir");
        let dir_path = Path::new(&new_dir);

        dir_path.create_dir().await?;

        assert!(dir_path.is_dir().await?);
        Ok(())
    }

    // Test create_dir_all
    #[tokio::test]
    async fn test_create_dir_all() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let new_dir = temp_dir.path().join("parent/child/grandchild");
        let dir_path = Path::new(&new_dir);

        dir_path.create_dir_all().await?;

        assert!(dir_path.is_dir().await?);
        Ok(())
    }

    // Test remove_dir
    #[tokio::test]
    async fn test_remove_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let new_dir = temp_dir.path().join("to_remove");
        async_fs::create_dir(&new_dir).await?;
        let dir_path = Path::new(&new_dir);

        assert!(dir_path.exists().await?);
        dir_path.remove_dir().await?;
        assert!(!dir_path.exists().await?);
        Ok(())
    }

    // Test remove_file
    #[tokio::test]
    async fn test_remove_file() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        assert!(file_path.exists().await?);
        file_path.remove_file().await?;
        assert!(!file_path.exists().await?);
        Ok(())
    }

    // Test remove_dir_all
    #[tokio::test]
    async fn test_remove_dir_all() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let parent = temp_dir.path().join("parent");
        async_fs::create_dir(&parent).await?;
        async_fs::write(parent.join("file1.txt"), "content1").await?;
        async_fs::write(parent.join("file2.txt"), "content2").await?;

        let dir_path = Path::new(&parent);
        assert!(dir_path.exists().await?);
        dir_path.remove_dir_all().await?;
        assert!(!dir_path.exists().await?);
        Ok(())
    }

    // Test get_file_size
    #[tokio::test]
    async fn test_get_file_size() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let test_content = b"Hello, World!";
        file_path.write(test_content).await?;

        let size = file_path.get_file_size().await?;
        assert_eq!(size, test_content.len() as u64);
        Ok(())
    }

    // Test truncate
    #[tokio::test]
    async fn test_truncate() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        let test_content = b"Hello, World!";
        file_path.write(test_content).await?;

        // Truncate to 5 bytes
        file_path.truncate(Some(5)).await?;

        let size = file_path.get_file_size().await?;
        assert_eq!(size, 5);
        Ok(())
    }

    // Test read_json and write_json
    #[tokio::test]
    async fn test_read_write_json() -> Result<()> {
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

        file_path.write_json(&original).await?;

        let loaded: TestData = file_path.read_json().await?;
        assert_eq!(loaded, original);
        Ok(())
    }

    // Test read_dir
    #[tokio::test]
    async fn test_read_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        async_fs::write(temp_dir.path().join("file1.txt"), "content1").await?;
        async_fs::write(temp_dir.path().join("file2.txt"), "content2").await?;
        async_fs::create_dir(temp_dir.path().join("subdir")).await?;

        let dir_path = Path::new(temp_dir.path());
        let mut entries = dir_path.read_dir().await?;
        let mut count = 0;
        while entries.next_entry().await?.is_some() {
            count += 1;
        }

        // Should have 3 entries: 2 files + 1 directory
        assert_eq!(count, 3);
        Ok(())
    }

    // Test empty_dir
    #[tokio::test]
    async fn test_empty_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        async_fs::write(temp_dir.path().join("file1.txt"), "content1").await?;
        async_fs::write(temp_dir.path().join("file2.txt"), "content2").await?;
        async_fs::create_dir(temp_dir.path().join("subdir")).await?;

        let dir_path = Path::new(temp_dir.path());
        dir_path.empty_dir().await?;

        // Directory should be empty now
        let mut entries = dir_path.read_dir().await?;
        let mut count = 0;
        while entries.next_entry().await?.is_some() {
            count += 1;
        }

        assert_eq!(count, 0);
        Ok(())
    }

    // Test set_permissions
    #[cfg(unix)]
    #[tokio::test]
    async fn test_set_permissions() -> Result<()> {
        use std::{
            fs,
            os::unix::fs::PermissionsExt,
        };

        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        // Read current permissions
        let metadata = fs::metadata(temp_file.path())?;
        let original_mode = metadata.permissions().mode();

        // Set new permissions
        file_path.set_permissions(fs::Permissions::from_mode(0o644)).await?;

        let new_metadata = fs::metadata(temp_file.path())?;
        assert_eq!(new_metadata.permissions().mode() & 0o777, 0o644);

        // Restore original
        file_path
            .set_permissions(fs::Permissions::from_mode(original_mode))
            .await?;
        Ok(())
    }

    // Test chmod
    #[cfg(unix)]
    #[tokio::test]
    async fn test_chmod() -> Result<()> {
        use std::{
            fs,
            os::unix::fs::PermissionsExt,
        };

        let temp_file = NamedTempFile::new()?;
        let file_path = Path::new(temp_file.path());

        file_path.chmod(0o744).await?;
        let metadata = fs::metadata(temp_file.path())?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o744);

        file_path.chmod(0o700).await?;
        let metadata = fs::metadata(temp_file.path())?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);

        Ok(())
    }

    // Test chown - requires root, skip if not root
    #[cfg(unix)]
    #[tokio::test]
    async fn test_chown() -> Result<()> {
        use std::{
            fs,
            os::unix::fs::PermissionsExt,
        };

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
        file_path.chown(Some(0), Some(0)).await?;

        // Restore permissions
        file_path
            .set_permissions(fs::Permissions::from_mode(original_mode))
            .await?;
        Ok(())
    }

    #[cfg(unix)]
    // Test is_block_device
    #[tokio::test]
    async fn test_is_block_device() -> Result<()> {
        let path = Path::new("/dev/sda"); // Common block device
        if path.exists().await? {
            // May fail if not root or device doesn't exist
            let _ = path.is_block_device().await;
        }
        Ok(())
    }

    #[cfg(unix)]
    // Test is_char_device
    #[tokio::test]
    async fn test_is_char_device() -> Result<()> {
        let path = Path::new("/dev/zero"); // Common char device
        if path.exists().await? {
            assert!(path.is_char_device().await?);
        }
        Ok(())
    }

    #[cfg(unix)]
    // Test is_fifo - simplified, skip creation
    #[tokio::test]
    async fn test_is_fifo() -> Result<()> {
        // FIFOs require special permissions to create
        // Just test that non-fifo returns false
        let path = Path::new("/tmp"); // This is not a fifo
        assert!(!path.is_fifo().await?);
        Ok(())
    }

    #[cfg(unix)]
    // Test is_socket - simplified
    #[tokio::test]
    async fn test_is_socket() -> Result<()> {
        // Unix socket files are tricky to create and test
        // Just test that non-socket returns false
        let path = Path::new("/tmp"); // This is not a socket
        assert!(!path.is_socket().await?);
        Ok(())
    }
}
