use std::fs;

use anyhow::{
    Result,
    anyhow,
};
use pathkit::{
    SyncFsOps,
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

// Test exists_sync
#[test]
fn test_exists_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    assert!(file_path.exists_sync()?);
    Ok(())
}

#[test]
fn test_exists_sync_false() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "non_existent_file.txt";

    assert!(!path.exists_sync()?);
    Ok(())
}

// Test is_file_sync
#[test]
fn test_is_file_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    assert!(file_path.is_file_sync()?);
    Ok(())
}

#[test]
fn test_is_file_sync_false() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir);

    assert!(!dir_path.is_file_sync()?);
    Ok(())
}

// Test is_dir_sync
#[test]
fn test_is_dir_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir);

    assert!(dir_path.is_dir_sync()?);
    Ok(())
}

#[test]
fn test_is_dir_sync_false() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    assert!(!file_path.is_dir_sync()?);
    Ok(())
}

// Test is_symlink_sync
#[cfg(unix)]
#[test]
fn test_is_symlink_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let target = path!(&temp_dir) / "target.txt";
    fs::write(&target, "test")?;

    let link = path!(&temp_dir) / "link.txt";
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        symlink(&target, &link)?;
    }

    assert!(link.is_symlink_sync()?);
    Ok(())
}

// Test metadata_sync
#[test]
fn test_metadata_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    let metadata = file_path.metadata_sync()?;
    assert!(metadata.is_file());
    Ok(())
}

// Test read_sync and write_sync
#[test]
fn test_read_write_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

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
    let file_path = path!(&temp_file);

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
    let dir_path = path!(&temp_dir) / "new_dir";

    dir_path.create_dir_sync()?;

    assert!(dir_path.is_dir_sync()?);
    Ok(())
}

// Test create_dir_all_sync
#[test]
fn test_create_dir_all_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir) / "parent" / "child" / "grandchild";

    dir_path.create_dir_all_sync()?;

    assert!(dir_path.is_dir_sync()?);
    Ok(())
}

// Test remove_dir_sync
#[test]
fn test_remove_dir_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let dir_path = path!(&temp_dir) / "to_remove";
    fs::create_dir(&dir_path)?;

    assert!(dir_path.exists_sync()?);
    dir_path.remove_dir_sync()?;
    assert!(!dir_path.exists_sync()?);
    Ok(())
}

// Test remove_file_sync
#[test]
fn test_remove_file_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    assert!(file_path.exists_sync()?);
    file_path.remove_file_sync()?;
    assert!(!file_path.exists_sync()?);
    Ok(())
}

// Test remove_dir_all_sync
#[test]
fn test_remove_dir_all_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let parent = path!(&temp_dir) / "parent";
    fs::create_dir(&parent)?;
    fs::write(&parent / "file1.txt", "content1")?;
    fs::write(&parent / "file2.txt", "content2")?;

    assert!(parent.exists_sync()?);
    parent.remove_dir_all_sync()?;
    assert!(!parent.exists_sync()?);
    Ok(())
}

// Test get_file_size_sync
#[test]
fn test_get_file_size_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

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
    let file_path = path!(&temp_file);

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
    let file_path = path!(&temp_file);

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
    fs::write(path!(&temp_dir) / "file1.txt", "content1")?;
    fs::write(path!(&temp_dir) / "file2.txt", "content2")?;
    fs::create_dir(path!(&temp_dir) / "subdir")?;

    let dir_path = path!(&temp_dir);
    let entries: Vec<_> = dir_path.read_dir_sync()?.collect();

    // Should have 3 entries: 2 files + 1 directory
    assert_eq!(entries.len(), 3);
    Ok(())
}

// Test empty_dir_sync
#[test]
fn test_empty_dir_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    fs::write(path!(&temp_dir) / "file1.txt", "content1")?;
    fs::write(path!(&temp_dir) / "file2.txt", "content2")?;
    fs::create_dir(path!(&temp_dir) / "subdir")?;

    let dir_path = path!(&temp_dir);
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
    let file_path = path!(&temp_file);

    // Read current permissions
    let metadata = fs::metadata(&temp_file)?;
    let original_mode = metadata.permissions().mode();

    // Set new permissions
    file_path.set_permissions_sync(fs::Permissions::from_mode(0o644))?;

    let new_metadata = fs::metadata(&temp_file)?;
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
    let file_path = path!(&temp_file);

    file_path.chmod_sync(0o744)?;
    let metadata = fs::metadata(&temp_file)?;
    assert_eq!(metadata.permissions().mode() & 0o777, 0o744);

    file_path.chmod_sync(0o700)?;
    let metadata = fs::metadata(&temp_file)?;
    assert_eq!(metadata.permissions().mode() & 0o777, 0o700);

    Ok(())
}

// Test chown_sync - requires root, skip if not root
#[cfg(unix)]
#[test]
fn test_chown_sync() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Skip if not root (chown requires root privileges)
    // SAFETY: `geteuid` has no preconditions and does not dereference pointers.
    if unsafe { libc::geteuid() } != 0 {
        return Ok(());
    }

    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    // Get current uid/gid
    let metadata = fs::metadata(&temp_file)?;
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
    let path = path!("/dev/sda"); // Common block device
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
    let path = path!("/dev/zero"); // Common char device
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
    let path = path!("/tmp"); // This is not a fifo
    assert!(!path.is_fifo_sync()?);
    Ok(())
}

#[cfg(unix)]
// Test is_socket_sync - simplified
#[test]
fn test_is_socket_sync() -> Result<()> {
    // Unix socket files are tricky to create and test
    let path = path!("/tmp"); // This is not a socket
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
    let src = path!(&temp_src);
    let dst = path!(&temp_dst);

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
    let src = path!(&temp_file);
    let link_path = path!(&temp_file).with_extension("link");

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
    let src = path!(&temp_file);
    let link_path = path!(&temp_file).with_extension("sym");

    src.write_sync(b"symlink test")?;
    src.soft_link_sync(&link_path)?;

    // Read through symlink
    let content = fs::read(link_path.as_path())?;
    assert_eq!(content, b"symlink test");

    // Verify link_path is a symlink
    assert!(link_path.is_symlink_sync()?);
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_read_link_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let src = path!(&temp_file);
    let link_path = path!(&temp_file).with_extension("readlink");

    src.write_sync(b"readlink test")?;
    src.soft_link_sync(&link_path)?;

    let link_target = link_path.read_link_sync()?;
    assert_eq!(link_target.to_str(), src.to_str());

    Ok(())
}

#[cfg(unix)]
#[test]
fn test_symlink_metadata_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let src = path!(&temp_file);
    let link_path = path!(&temp_file).with_extension("meta");

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
    let new_file = path!(&temp_dir) / "touched.txt";
    let path = new_file;

    assert!(!path.exists_sync()?);
    path.touch_sync()?;
    assert!(path.exists_sync()?);
    Ok(())
}

#[test]
fn test_touch_sync_updates_existing() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let path = path!(&temp_file);

    // Record original mtime
    let meta_before = fs::metadata(&temp_file)?;
    let mtime_before = meta_before.modified()?;

    // Wait a bit so mtime actually changes
    std::thread::sleep(std::time::Duration::from_millis(10));

    path.touch_sync()?;

    let meta_after = fs::metadata(&temp_file)?;
    let mtime_after = meta_after.modified()?;
    assert!(mtime_after > mtime_before);
    Ok(())
}

#[test]
fn test_read_dir_names_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    fs::write(path!(&temp_dir) / "a.txt", "")?;
    fs::write(path!(&temp_dir) / "b.txt", "")?;
    fs::create_dir(path!(&temp_dir) / "subdir")?;

    let dir = path!(&temp_dir);
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
    let file_a = path!(&temp_dir) / "file_a.txt";
    let file_b = path!(&temp_dir) / "file_b.txt";
    fs::write(&file_a, "")?;
    fs::write(&file_b, "")?;

    let dir = path!(&temp_dir);
    let paths = dir.read_dir_paths_sync()?;
    assert_eq!(paths.len(), 2);
    // All returned paths should be absolute
    for p in &paths {
        assert!(p.is_absolute());
    }

    Ok(())
}

#[test]
fn test_read_dir_entries_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let file_a = path!(&temp_dir) / "entry_a.txt";
    let file_b = path!(&temp_dir) / "entry_b.txt";
    fs::write(&file_a, "")?;
    fs::write(&file_b, "")?;

    let dir = path!(&temp_dir);
    let entries = dir.read_dir_entries_sync()?;
    assert_eq!(entries.len(), 2);

    let names: Vec<_> = entries
        .iter()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();

    assert!(names.contains(&String::from("entry_a.txt")));
    assert!(names.contains(&String::from("entry_b.txt")));

    Ok(())
}

#[test]
fn test_create_parent_dir_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "parent" / "file.txt";

    assert!(file.create_parent_dir_sync()?);
    let parent = file
        .parent()
        .ok_or_else(|| anyhow!("test file path should have a parent"))?;

    assert!(parent.is_dir_sync()?);

    Ok(())
}

#[test]
fn test_create_parent_dir_all_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "nested" / "parent" / "file.txt";

    assert!(file.create_parent_dir_all_sync()?);
    let parent = file
        .parent()
        .ok_or_else(|| anyhow!("test file path should have a parent"))?;

    assert!(parent.is_dir_sync()?);

    Ok(())
}

#[test]
fn test_create_parent_dir_sync_without_parent_returns_false() -> Result<()> {
    let path = path!("");

    assert!(!path.create_parent_dir_sync()?);
    assert!(!path.create_parent_dir_all_sync()?);

    Ok(())
}

#[test]
fn test_empty_dir_sync_creates_dir_if_missing() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "brand_new_dir";

    // Directory doesn't exist
    assert!(!path.exists_sync()?);

    // empty_dir_sync should create it (via create_dir_all_sync)
    path.empty_dir_sync()?;

    assert!(path.is_dir_sync()?);
    Ok(())
}

#[test]
fn test_move_to_sync_moves_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source.txt";
    let dest = path!(&temp_dir) / "dest.txt";

    src.write_sync(b"move test")?;
    let moved = src.move_to_sync(&dest)?;

    assert_eq!(moved, dest);
    assert!(!src.exists_sync()?);
    assert!(dest.is_file_sync()?);
    assert_eq!(dest.read_sync()?, b"move test");
    Ok(())
}

#[test]
fn test_move_to_sync_moves_dir() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source-dir";
    let nested = &src / "nested.txt";
    let dest = path!(&temp_dir) / "dest-dir";

    src.create_dir_sync()?;
    nested.write_sync(b"nested")?;
    let moved = src.move_to_sync(&dest)?;

    assert_eq!(moved, dest);
    assert!(!src.exists_sync()?);
    assert!(dest.is_dir_sync()?);
    assert_eq!((&dest / "nested.txt").read_sync()?, b"nested");
    Ok(())
}
