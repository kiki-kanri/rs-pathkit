#![cfg(feature = "async-fs-ops")]

use anyhow::{
    Result,
    anyhow,
};
use pathkit::{
    AsyncFsOps,
    AsyncPathEntry,
    path,
};
use serde::{
    Deserialize,
    Serialize,
};
use tempfile::{
    NamedTempFile,
    tempdir,
};
use tokio::{
    fs as async_fs,
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
};

// Test exists
#[tokio::test]
async fn test_exists() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    assert!(file_path.exists().await?);
    Ok(())
}

#[tokio::test]
async fn test_exists_false() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "non_existent_file.txt";

    assert!(!path.exists().await?);
    Ok(())
}

// Test is_file
#[tokio::test]
async fn test_is_file() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    assert!(file_path.is_file().await?);
    Ok(())
}

#[tokio::test]
async fn test_is_file_false() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir);

    assert!(!dir_path.is_file().await?);
    Ok(())
}

// Test is_dir
#[tokio::test]
async fn test_is_dir() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir);

    assert!(dir_path.is_dir().await?);
    Ok(())
}

#[tokio::test]
async fn test_is_dir_false() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    assert!(!file_path.is_dir().await?);
    Ok(())
}

// Test is_symlink
#[cfg(unix)]
#[tokio::test]
async fn test_is_symlink() -> Result<()> {
    use std::os::unix::fs::symlink;

    let temp_dir = tempdir()?;
    let target = path!(&temp_dir) / "target.txt";
    async_fs::write(&target, "test").await?;

    let link = path!(&temp_dir) / "link.txt";
    symlink(&target, &link)?;

    assert!(link.is_symlink().await?);
    Ok(())
}

// Test metadata
#[tokio::test]
async fn test_metadata() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    let metadata = file_path.metadata().await?;
    assert!(metadata.is_file());
    Ok(())
}

// Test read and write
#[tokio::test]
async fn test_read_write() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

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
    let file_path = path!(&temp_file);

    let test_content = "Hello, World!";
    file_path.write(test_content).await?;

    let read_content = file_path.read_to_string().await?;
    assert_eq!(read_content, test_content);
    Ok(())
}

// Test create_dir
#[tokio::test]
async fn test_create_dir() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir) / "new_dir";

    dir_path.create_dir().await?;

    assert!(dir_path.is_dir().await?);
    Ok(())
}

// Test create_dir_all
#[tokio::test]
async fn test_create_dir_all() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir) / "parent" / "child" / "grandchild";

    dir_path.create_dir_all().await?;

    assert!(dir_path.is_dir().await?);
    Ok(())
}

// Test remove_dir
#[tokio::test]
async fn test_remove_dir() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir) / "to_remove";
    async_fs::create_dir(&dir_path).await?;

    assert!(dir_path.exists().await?);
    dir_path.remove_dir().await?;
    assert!(!dir_path.exists().await?);
    Ok(())
}

// Test remove_file
#[tokio::test]
async fn test_remove_file() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    assert!(file_path.exists().await?);
    file_path.remove_file().await?;
    assert!(!file_path.exists().await?);
    Ok(())
}

// Test remove_dir_all
#[tokio::test]
async fn test_remove_dir_all() -> Result<()> {
    let temp_dir = tempdir()?;
    let parent = path!(&temp_dir) / "parent";
    async_fs::create_dir(&parent).await?;
    async_fs::write(&parent / "file1.txt", "content1").await?;
    async_fs::write(&parent / "file2.txt", "content2").await?;

    assert!(parent.exists().await?);
    parent.remove_dir_all().await?;
    assert!(!parent.exists().await?);
    Ok(())
}

// Test get_file_size
#[tokio::test]
async fn test_get_file_size() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

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
    let file_path = path!(&temp_file);

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
    let file_path = path!(&temp_file);

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
    let temp_dir = tempdir()?;
    async_fs::write(path!(&temp_dir) / "file1.txt", "content1").await?;
    async_fs::write(path!(&temp_dir) / "file2.txt", "content2").await?;
    async_fs::create_dir(path!(&temp_dir) / "subdir").await?;

    let dir_path = path!(&temp_dir);
    let mut entries = dir_path.read_dir().await?;
    let mut count = 0;
    while entries.next_entry().await?.is_some() {
        count += 1;
    }

    // Should have 3 entries: 2 files + 1 directory
    assert_eq!(count, 3);
    Ok(())
}

#[tokio::test]
async fn test_read_dir_entries() -> Result<()> {
    let temp_dir = tempdir()?;
    async_fs::write(path!(&temp_dir) / "file1.txt", "content1").await?;
    async_fs::write(path!(&temp_dir) / "file2.txt", "content2").await?;

    let dir_path = path!(&temp_dir);
    let entries = dir_path.read_dir_entries().await?;
    assert_eq!(entries.len(), 2);

    let paths: Vec<_> = entries.iter().map(AsyncPathEntry::path).collect();
    let path_strings: Vec<_> = paths
        .iter()
        .filter_map(|p| p.file_name().map(|name| name.to_string_lossy().to_string()))
        .collect();

    assert!(path_strings.contains(&"file1.txt".to_string()));
    assert!(path_strings.contains(&"file2.txt".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_read_dir_names() -> Result<()> {
    let temp_dir = tempdir()?;
    async_fs::write(path!(&temp_dir) / "a.txt", "").await?;
    async_fs::write(path!(&temp_dir) / "b.txt", "").await?;
    async_fs::create_dir(path!(&temp_dir) / "subdir").await?;

    let dir = path!(&temp_dir);
    let names = dir.read_dir_names().await?;
    assert_eq!(names.len(), 3);
    assert!(names.contains(&String::from("a.txt")));
    assert!(names.contains(&String::from("b.txt")));
    assert!(names.contains(&String::from("subdir")));
    Ok(())
}

#[tokio::test]
async fn test_read_dir_paths() -> Result<()> {
    let temp_dir = tempdir()?;
    let file_a = path!(&temp_dir) / "file_a.txt";
    let file_b = path!(&temp_dir) / "file_b.txt";
    async_fs::write(&file_a, "").await?;
    async_fs::write(&file_b, "").await?;

    let dir = path!(&temp_dir);
    let paths = dir.read_dir_paths().await?;
    assert_eq!(paths.len(), 2);
    for path in &paths {
        assert!(path.is_absolute());
    }
    Ok(())
}

#[tokio::test]
async fn test_create_parent_dir() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "parent" / "file.txt";

    assert!(file.create_parent_dir().await?);
    let parent = file
        .parent()
        .ok_or_else(|| anyhow!("test file path should have a parent"))?;

    assert!(parent.is_dir().await?);

    Ok(())
}

#[tokio::test]
async fn test_create_parent_dir_all() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "nested" / "parent" / "file.txt";

    assert!(file.create_parent_dir_all().await?);
    let parent = file
        .parent()
        .ok_or_else(|| anyhow!("test file path should have a parent"))?;

    assert!(parent.is_dir().await?);

    Ok(())
}

#[tokio::test]
async fn test_create_parent_dir_without_parent_returns_false() -> Result<()> {
    let path = path!("");

    assert!(!path.create_parent_dir().await?);
    assert!(!path.create_parent_dir_all().await?);

    Ok(())
}

// Test empty_dir
#[tokio::test]
async fn test_empty_dir() -> Result<()> {
    let temp_dir = tempdir()?;
    async_fs::write(path!(&temp_dir) / "file1.txt", "content1").await?;
    async_fs::write(path!(&temp_dir) / "file2.txt", "content2").await?;
    async_fs::create_dir(path!(&temp_dir) / "subdir").await?;

    let dir_path = path!(&temp_dir);
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
        fs::{
            Permissions,
            metadata,
        },
        os::unix::fs::PermissionsExt,
    };

    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    // Read current permissions
    let file_metadata = metadata(&temp_file)?;
    let original_mode = file_metadata.permissions().mode();

    // Set new permissions
    file_path.set_permissions(Permissions::from_mode(0o644)).await?;

    let new_metadata = metadata(&temp_file)?;
    assert_eq!(new_metadata.permissions().mode() & 0o777, 0o644);

    // Restore original
    file_path.set_permissions(Permissions::from_mode(original_mode)).await?;
    Ok(())
}

// Test chmod
#[cfg(unix)]
#[tokio::test]
async fn test_chmod() -> Result<()> {
    use std::{
        fs::metadata,
        os::unix::fs::PermissionsExt,
    };

    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    file_path.chmod(0o744).await?;
    let file_metadata = metadata(&temp_file)?;
    assert_eq!(file_metadata.permissions().mode() & 0o777, 0o744);

    file_path.chmod(0o700).await?;
    let file_metadata = metadata(&temp_file)?;
    assert_eq!(file_metadata.permissions().mode() & 0o777, 0o700);

    Ok(())
}

// Test chown - requires root, skip if not root
#[cfg(unix)]
#[tokio::test]
async fn test_chown() -> Result<()> {
    use std::{
        fs::{
            Permissions,
            metadata,
        },
        os::unix::fs::PermissionsExt,
    };

    // Skip if not root (chown requires root privileges)
    // SAFETY: `geteuid` has no preconditions and does not dereference pointers.
    if unsafe { libc::geteuid() } != 0 {
        return Ok(());
    }

    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    // Get current uid/gid
    let file_metadata = metadata(&temp_file)?;
    let original_mode = file_metadata.permissions().mode();

    // chown to same uid/gid (no-op but should work)
    file_path.chown(Some(0), Some(0)).await?;

    // Restore permissions
    file_path.set_permissions(Permissions::from_mode(original_mode)).await?;
    Ok(())
}

#[cfg(unix)]
// Test is_block_device
#[tokio::test]
async fn test_is_block_device() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    assert!(!path!(&temp_file).is_block_device().await?);

    let path = path!("/dev/sda"); // Common block device
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
    let temp_file = NamedTempFile::new()?;
    assert!(!path!(&temp_file).is_char_device().await?);

    let path = path!("/dev/zero"); // Common char device
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
    let path = path!("/tmp"); // This is not a fifo
    assert!(!path.is_fifo().await?);
    Ok(())
}

#[cfg(unix)]
// Test is_socket - simplified
#[tokio::test]
async fn test_is_socket() -> Result<()> {
    // Unix socket files are tricky to create and test
    // Just test that non-socket returns false
    let path = path!("/tmp"); // This is not a socket
    assert!(!path.is_socket().await?);
    Ok(())
}

// Test copy_file
#[tokio::test]
async fn test_copy_file() -> Result<()> {
    let temp_src = NamedTempFile::new()?;
    let temp_dst = NamedTempFile::new()?;
    let src = path!(&temp_src);
    let dst = path!(&temp_dst);

    src.write(b"hello world").await?;

    let bytes = src.copy_file(&dst).await?;
    assert_eq!(bytes, 11);

    let content = dst.read().await?;
    assert_eq!(content, b"hello world");
    Ok(())
}

#[tokio::test]
async fn test_move_to_moves_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source.txt";
    let dest = path!(&temp_dir) / "dest.txt";

    src.write(b"move test").await?;
    let moved = src.move_to(&dest).await?;

    assert_eq!(moved, dest);
    assert!(!src.exists().await?);
    assert!(dest.is_file().await?);
    assert_eq!(dest.read().await?, b"move test");
    Ok(())
}

#[tokio::test]
async fn test_move_to_moves_dir() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source-dir";
    let nested = &src / "nested.txt";
    let dest = path!(&temp_dir) / "dest-dir";

    src.create_dir().await?;
    nested.write(b"nested").await?;
    let moved = src.move_to(&dest).await?;

    assert_eq!(moved, dest);
    assert!(!src.exists().await?);
    assert!(dest.is_dir().await?);
    assert_eq!((&dest / "nested.txt").read().await?, b"nested");
    Ok(())
}

#[tokio::test]
async fn test_open_returns_readable_file() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let path = path!(&temp_file);
    path.write(b"open test").await?;

    let mut file = path.open().await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;

    assert_eq!(content, "open test");
    Ok(())
}

#[tokio::test]
async fn test_open_with_options_uses_options() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "created.txt";
    let mut options = async_fs::OpenOptions::new();
    options.write(true).create_new(true);

    let mut file = path.open_with_options(&options).await?;
    file.write_all(b"created").await?;
    drop(file);

    assert_eq!(path.read().await?, b"created");
    Ok(())
}
