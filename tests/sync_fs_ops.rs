use std::{
    fs,
    io::{
        Error as IoError,
        ErrorKind as IoErrorKind,
        Read,
        Write,
    },
    sync::Barrier,
    thread::scope,
};

use anyhow::{
    Result,
    anyhow,
};
use pathkit::{
    Path,
    SyncFsOps,
    path,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use tempfile::{
    NamedTempFile,
    tempdir,
};

// Test canonicalize_sync
#[test]
fn test_canonicalize_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let original = path!(&temp_dir) / ".";

    let canonical: Path = original.canonicalize_sync()?;
    assert!(canonical.is_absolute());
    assert_eq!(canonical.as_path(), fs::canonicalize(&original)?);
    Ok(())
}

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

// Test write_atomic_sync
#[test]
fn test_write_atomic_sync_replaces_existing_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "config.json";
    path.write_sync(b"old")?;

    path.write_atomic_sync(b"new")?;

    assert_eq!(path.read_sync()?, b"new");
    assert_eq!(path!(&temp_dir).read_dir_names_sync()?, ["config.json"]);
    Ok(())
}

#[test]
fn test_write_atomic_with_sync_prepares_empty_temporary_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "secret.key";

    path.write_atomic_with_sync(b"secret", |temp_path| {
        assert_ne!(temp_path, &path);
        assert_eq!(temp_path.parent(), path.parent());
        assert_eq!(temp_path.get_file_size_sync()?, 0);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_eq!(temp_path.metadata_sync()?.permissions().mode() & 0o077, 0);
            temp_path.chmod_sync(0o640)?;
        }

        Ok(())
    })?;

    assert_eq!(path.read_sync()?, b"secret");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        assert_eq!(path.metadata_sync()?.permissions().mode() & 0o777, 0o640);
    }

    Ok(())
}

#[test]
fn test_write_atomic_with_sync_removes_temporary_file_on_failure() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "config.json";
    path.write_sync(b"old")?;

    let error = path
        .write_atomic_with_sync(b"new", |_| Err(IoError::other("prepare failed")))
        .err()
        .ok_or_else(|| anyhow!("expected prepare error"))?;

    assert_eq!(error.kind(), IoErrorKind::Other);
    assert_eq!(path.read_sync()?, b"old");
    assert_eq!(path!(&temp_dir).read_dir_names_sync()?, ["config.json"]);
    Ok(())
}

#[test]
fn test_write_atomic_with_sync_removes_temporary_file_on_panic() -> Result<()> {
    use std::panic::{
        AssertUnwindSafe,
        catch_unwind,
    };

    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "config.json";

    let result = catch_unwind(AssertUnwindSafe(|| {
        path.write_atomic_with_sync(b"new", |_| panic!("prepare panicked"))
    }));

    assert!(result.is_err());
    assert_eq!(path!(&temp_dir).read_dir_names_sync()?.len(), 0);
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_write_atomic_sync_accepts_maximum_length_file_name() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "a".repeat(255);

    path.write_atomic_sync(b"long name")?;

    assert_eq!(path.read_sync()?, b"long name");
    Ok(())
}

#[test]
fn test_write_atomic_sync_rejects_path_without_file_name() -> Result<()> {
    let error = path!("/")
        .write_atomic_sync(b"data")
        .err()
        .ok_or_else(|| anyhow!("expected missing file name error"))?;

    assert_eq!(error.kind(), IoErrorKind::InvalidInput);
    Ok(())
}

// Test persist_to_sync
#[test]
fn test_persist_to_sync_replaces_destination() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "prepared.tmp";
    let dest = path!(&temp_dir) / "dest.txt";
    src.write_sync(b"new")?;
    dest.write_sync(b"old")?;

    let persisted = src.persist_to_sync(&dest)?;

    assert_eq!(persisted, dest);
    assert!(!src.exists_sync()?);
    assert_eq!(dest.read_sync()?, b"new");
    Ok(())
}

// Test read_json_sync and write_json_sync
#[test]
fn test_read_write_json_sync() -> Result<()> {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);

    let original = TestData {
        name: "test".to_owned(),
        value: 42,
    };

    file_path.write_json_sync(&original)?;

    let loaded: TestData = file_path.read_json_sync()?;
    assert_eq!(loaded, original);
    Ok(())
}

#[test]
fn test_read_json_sync_missing_file_returns_io_error() -> Result<()> {
    let temp_dir = tempdir()?;
    let missing = path!(&temp_dir) / "missing.json";

    let error = missing
        .read_json_sync::<Value>()
        .err()
        .ok_or_else(|| anyhow!("expected missing file error"))?;

    assert!(error.is_io());
    assert_eq!(error.io_error_kind(), Some(IoErrorKind::NotFound));
    Ok(())
}

#[test]
fn test_read_json_sync_invalid_json_returns_syntax_error() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = path!(&temp_file);
    file_path.write_sync(b"not json")?;

    let error = file_path
        .read_json_sync::<Value>()
        .err()
        .ok_or_else(|| anyhow!("expected invalid JSON error"))?;

    assert!(error.is_syntax());
    assert_eq!(error.io_error_kind(), None);
    Ok(())
}

#[test]
fn test_write_json_sync_missing_parent_returns_io_error() -> Result<()> {
    let temp_dir = tempdir()?;
    let file_path = path!(&temp_dir) / "missing" / "data.json";

    let error = file_path
        .write_json_sync([1, 2, 3])
        .err()
        .ok_or_else(|| anyhow!("expected missing parent error"))?;

    assert!(error.is_io());
    assert_eq!(error.io_error_kind(), Some(IoErrorKind::NotFound));
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

#[cfg(unix)]
#[test]
fn test_empty_dir_sync_removes_symlinks_without_removing_targets() -> Result<()> {
    use std::os::unix::fs::symlink;

    let directory = tempdir()?;
    let external = tempdir()?;
    let target_file = path!(&external) / "keep.txt";
    let target_dir = path!(&external) / "keep-dir";
    let nested_file = &target_dir / "keep.txt";
    fs::write(&target_file, b"preserved")?;
    fs::create_dir(&target_dir)?;
    fs::write(&nested_file, b"nested content")?;

    symlink(&target_file, path!(&directory) / "file-link")?;
    symlink(&target_dir, path!(&directory) / "dir-link")?;
    symlink(path!(&external) / "missing", path!(&directory) / "dangling-link")?;

    let path = path!(&directory);
    path.empty_dir_sync()?;

    assert_eq!(path.read_dir_paths_sync()?.len(), 0);
    assert_eq!(fs::read(&target_file)?, b"preserved");
    assert_eq!(fs::read(&nested_file)?, b"nested content");
    Ok(())
}

#[cfg(windows)]
#[test]
fn test_empty_dir_sync_removes_windows_directory_symlinks() -> Result<()> {
    use std::{
        env,
        os::windows::fs::{
            symlink_dir,
            symlink_file,
        },
    };

    const ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

    let directory = tempdir()?;
    let external = tempdir()?;
    let target_file = path!(&external) / "keep.txt";
    let target_dir = path!(&external) / "keep-dir";
    let nested_file = &target_dir / "keep.txt";
    fs::write(&target_file, b"preserved")?;
    fs::create_dir(&target_dir)?;
    fs::write(&nested_file, b"nested content")?;

    // Symlink creation needs a privilege on Windows: skip locally without it, but require it in CI.
    match symlink_dir(
        path!(&external) / "missing-dir",
        path!(&directory) / "dangling-dir-link",
    ) {
        Err(error) if error.raw_os_error() == Some(ERROR_PRIVILEGE_NOT_HELD) && env::var_os("CI").is_none() => {
            return Ok(());
        },
        result => result?,
    }

    symlink_dir(&target_dir, path!(&directory) / "dir-link")?;
    symlink_file(&target_file, path!(&directory) / "file-link")?;

    let path = path!(&directory);
    path.empty_dir_sync()?;

    assert_eq!(path.read_dir_paths_sync()?.len(), 0);
    assert_eq!(fs::read(&target_file)?, b"preserved");
    assert_eq!(fs::read(&nested_file)?, b"nested content");
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

// Test is_block_device_sync
#[cfg(unix)]
#[test]
fn test_is_block_device_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    assert!(!path!(&temp_file).is_block_device_sync()?);
    Ok(())
}

// Test is_char_device_sync
#[cfg(unix)]
#[test]
fn test_is_char_device_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    assert!(!path!(&temp_file).is_char_device_sync()?);
    assert!(path!("/dev/null").is_char_device_sync()?);
    Ok(())
}

// Test is_fifo_sync
#[cfg(unix)]
#[test]
fn test_is_fifo_sync() -> Result<()> {
    use std::{
        ffi::CString,
        io::Error as IoError,
        os::unix::ffi::OsStrExt,
    };

    use libc::mkfifo;

    let temp_dir = tempdir()?;
    let fifo = path!(&temp_dir) / "fifo";
    let fifo_path = CString::new(fifo.as_os_str().as_bytes())?;
    // SAFETY: `fifo_path` is a valid NUL-terminated string that outlives the call.
    if unsafe { mkfifo(fifo_path.as_ptr(), 0o600) } != 0 {
        return Err(IoError::last_os_error().into());
    }

    assert!(fifo.is_fifo_sync()?);
    assert!(!path!(&temp_dir).is_fifo_sync()?);
    Ok(())
}

// Test is_socket_sync
#[cfg(unix)]
#[test]
fn test_is_socket_sync() -> Result<()> {
    use std::os::unix::net::UnixListener;

    let temp_dir = tempdir()?;
    let socket = path!(&temp_dir) / "socket";
    let _listener = UnixListener::bind(&socket)?;

    assert!(socket.is_socket_sync()?);
    assert!(!path!(&temp_dir).is_socket_sync()?);
    Ok(())
}

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
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source.txt";
    let link_path = path!(&temp_dir) / "hard-link.txt";

    src.write_sync(b"link test")?;
    src.hard_link_sync(&link_path)?;

    assert_eq!(link_path.read_sync()?, b"link test");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        assert_eq!(src.metadata_sync()?.ino(), link_path.metadata_sync()?.ino());
    }

    Ok(())
}

#[cfg(unix)]
#[test]
fn test_soft_link_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source.txt";
    let link_path = path!(&temp_dir) / "soft-link.txt";

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
    let temp_dir = tempdir()?;
    let target = path!(&temp_dir) / "missing.txt";
    let link_path = path!(&temp_dir) / "link.txt";

    target.soft_link_sync(&link_path)?;

    assert_eq!(link_path.read_link_sync()?, target);
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_symlink_metadata_sync() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source.txt";
    let link_path = path!(&temp_dir) / "link.txt";

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
    use std::{
        fs::File,
        time::SystemTime,
    };

    let temp_file = NamedTempFile::new()?;
    let path = path!(&temp_file);
    path.write_sync(b"preserved")?;
    File::options()
        .write(true)
        .open(&path)?
        .set_modified(SystemTime::UNIX_EPOCH)?;

    path.touch_sync()?;

    assert!(path.metadata_sync()?.modified()? > SystemTime::UNIX_EPOCH);
    assert_eq!(path.read_sync()?, b"preserved");
    Ok(())
}

#[test]
fn test_touch_sync_preserves_concurrently_written_content() -> Result<()> {
    let directory = tempdir()?;
    for index in 0..32 {
        let file = path!(&directory) / format!("touch-{index}.txt");
        let barrier = Barrier::new(5);

        scope(|scope| -> Result<()> {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        file.touch_sync()
                    })
                })
                .collect();

            barrier.wait();
            let write_result = fs::write(&file, b"preserved content");
            for handle in handles {
                handle.join().map_err(|_| anyhow!("touch worker panicked"))??;
            }

            write_result?;
            Ok(())
        })?;

        assert_eq!(fs::read(&file)?, b"preserved content");
    }

    Ok(())
}

#[cfg(unix)]
#[test]
fn test_touch_sync_creates_dangling_symlink_target() -> Result<()> {
    use std::os::unix::fs::symlink;

    let directory = tempdir()?;
    let target = path!(&directory) / "target.txt";
    let link = path!(&directory) / "link.txt";
    symlink(&target, &link)?;

    link.touch_sync()?;

    assert!(fs::symlink_metadata(&link)?.file_type().is_symlink());
    assert!(target.is_file_sync()?);
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
fn test_rename_to_sync_renames_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source.txt";
    let dest = path!(&temp_dir) / "dest.txt";

    src.write_sync(b"move test")?;
    let moved = src.rename_to_sync(&dest)?;

    assert_eq!(moved, dest);
    assert!(!src.exists_sync()?);
    assert!(dest.is_file_sync()?);
    assert_eq!(dest.read_sync()?, b"move test");
    Ok(())
}

#[test]
fn test_rename_to_sync_renames_dir() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "source-dir";
    let nested = &src / "nested.txt";
    let dest = path!(&temp_dir) / "dest-dir";

    src.create_dir_sync()?;
    nested.write_sync(b"nested")?;
    let moved = src.rename_to_sync(&dest)?;

    assert_eq!(moved, dest);
    assert!(!src.exists_sync()?);
    assert!(dest.is_dir_sync()?);
    assert_eq!((&dest / "nested.txt").read_sync()?, b"nested");
    Ok(())
}

#[test]
fn test_open_sync_returns_readable_file() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let path = path!(&temp_file);
    path.write_sync(b"open test")?;

    let mut file = path.open_sync()?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    assert_eq!(content, "open test");
    Ok(())
}

#[test]
fn test_open_with_options_sync_uses_options() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "created.txt";
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);

    let mut file = path.open_with_options_sync(&options)?;
    file.write_all(b"created")?;
    drop(file);

    assert_eq!(path.read_sync()?, b"created");
    Ok(())
}
