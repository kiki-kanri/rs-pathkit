use std::{
    fs,
    io::{
        self,
        Write,
    },
    sync::Barrier,
    thread,
};

use anyhow::{
    Result,
    anyhow,
};
use pathkit::{
    path,
    sync_fs_ops::{
        PathEntry,
        SyncFsOps,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use tempfile::{
    NamedTempFile,
    tempdir,
};

#[test]
fn copy_hard_link_to_and_rename_preserve_file_contents() -> Result<()> {
    let temp_dir = tempdir()?;
    let source = path!(&temp_dir) / "a" / "b" / "source.txt";

    source.create_parent_dir_all_sync()?;
    source.write_sync(b"payload")?;
    let copy = path!(&temp_dir) / "copy.txt";
    assert_eq!(source.copy_file_sync(&copy)?, 7);
    let linked = path!(&temp_dir) / "hard-link.txt";
    linked.hard_link_to_sync(&copy)?;
    let moved = linked.rename_to_sync(path!(&temp_dir) / "moved.txt")?;

    assert!(!linked.try_exists_sync()?);
    assert!(moved.is_file_sync()?);
    assert_eq!(moved.read_to_string_sync()?, "payload");
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlink_to_stores_target_as_given_and_is_reported_as_symlink() -> Result<()> {
    let temp_dir = tempdir()?;
    let target = path!(&temp_dir) / "missing.txt";
    let link = path!(&temp_dir) / "link.txt";

    link.symlink_to_sync(&target)?;

    assert!(link.is_symlink_sync()?);
    assert_eq!(link.read_link_sync()?, target);
    Ok(())
}

#[test]
fn read_if_exists_sync_returns_none_only_for_missing_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "file.txt";

    assert_eq!(file.read_if_exists_sync()?, None);
    file.write_sync(b"content")?;
    assert_eq!(file.read_if_exists_sync()?, Some(b"content".to_vec()));
    assert!(path!(&temp_dir).read_if_exists_sync().is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn if_exists_sync_methods_propagate_errors_other_than_not_found() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "file.txt";
    file.write_sync(b"content")?;
    let child = &file / "child";

    let results = [
        ("read_if_exists_sync", child.read_if_exists_sync().map(drop)),
        (
            "read_to_string_if_exists_sync",
            child.read_to_string_if_exists_sync().map(drop),
        ),
        ("metadata_if_exists_sync", child.metadata_if_exists_sync().map(drop)),
        (
            "symlink_metadata_if_exists_sync",
            child.symlink_metadata_if_exists_sync().map(drop),
        ),
        ("read_dir_if_exists_sync", child.read_dir_if_exists_sync().map(drop)),
        (
            "remove_file_if_exists_sync",
            child.remove_file_if_exists_sync().map(drop),
        ),
    ];

    for (method, result) in results {
        assert_eq!(
            result.map_err(|error| error.kind()),
            Err(io::ErrorKind::NotADirectory),
            "{method}"
        );
    }

    Ok(())
}

#[test]
fn read_to_string_if_exists_sync_returns_none_only_for_missing_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "file.txt";

    assert_eq!(file.read_to_string_if_exists_sync()?, None);
    file.write_sync("content")?;
    assert_eq!(file.read_to_string_if_exists_sync()?, Some("content".to_owned()));

    file.write_sync([0xff, 0xfe])?;
    let error = file
        .read_to_string_if_exists_sync()
        .err()
        .ok_or_else(|| anyhow!("expected invalid UTF-8 error"))?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    Ok(())
}

#[test]
fn remove_file_if_exists_sync_reports_whether_a_file_was_removed() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "file.txt";

    assert!(!file.remove_file_if_exists_sync()?);
    file.write_sync(b"content")?;
    assert!(file.remove_file_if_exists_sync()?);
    assert!(!file.try_exists_sync()?);
    assert!(!file.remove_file_if_exists_sync()?);
    assert!(path!(&temp_dir).remove_file_if_exists_sync().is_err());
    Ok(())
}

#[test]
fn metadata_if_exists_sync_methods_return_none_only_for_missing_path() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "file.txt";

    assert!(file.metadata_if_exists_sync()?.is_none());
    assert!(file.symlink_metadata_if_exists_sync()?.is_none());
    file.write_sync(b"content")?;
    assert_eq!(file.metadata_if_exists_sync()?.map(|metadata| metadata.len()), Some(7));
    assert_eq!(
        file.symlink_metadata_if_exists_sync()?.map(|metadata| metadata.len()),
        Some(7)
    );

    Ok(())
}

#[cfg(unix)]
#[test]
fn dangling_symlink_has_symlink_metadata_but_no_metadata() -> Result<()> {
    let temp_dir = tempdir()?;
    let link = path!(&temp_dir) / "link";
    link.symlink_to_sync(path!(&temp_dir) / "missing")?;

    assert!(link.metadata_if_exists_sync()?.is_none());
    assert!(
        link.symlink_metadata_if_exists_sync()?
            .is_some_and(|metadata| metadata.file_type().is_symlink())
    );

    Ok(())
}

#[test]
fn touch_sync_creates_missing_file_without_creating_parents() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "touched.txt";

    file.touch_sync()?;

    assert!(file.is_file_sync()?);
    assert_eq!(file.metadata_sync()?.len(), 0);
    assert!((path!(&temp_dir) / "missing" / "touched.txt").touch_sync().is_err());
    Ok(())
}

#[test]
fn touch_sync_updates_mtime_without_truncating() -> Result<()> {
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
fn touch_sync_preserves_concurrently_written_content() -> Result<()> {
    let directory = tempdir()?;
    for index in 0..32 {
        let file = path!(&directory) / format!("touch-{index}.txt");
        let barrier = Barrier::new(5);

        thread::scope(|scope| -> Result<()> {
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
fn touch_sync_creates_target_of_dangling_symlink() -> Result<()> {
    let directory = tempdir()?;
    let target = path!(&directory) / "target.txt";
    let link = path!(&directory) / "link.txt";
    link.symlink_to_sync(&target)?;

    link.touch_sync()?;

    assert!(link.is_symlink_sync()?);
    assert!(target.is_file_sync()?);
    Ok(())
}

#[test]
fn empty_dir_sync_creates_missing_directory_with_parents() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "missing" / "nested";

    path.empty_dir_sync()?;

    assert!(path.is_dir_sync()?);
    assert_eq!(path.read_dir_names_lossy_sync()?.len(), 0);
    Ok(())
}

#[test]
fn empty_dir_sync_removes_contents_but_keeps_directory() -> Result<()> {
    let temp_dir = tempdir()?;
    let directory = path!(&temp_dir) / "target";
    fs::create_dir_all(&directory / "subdir" / "deep")?;
    fs::write(&directory / "file.txt", "content")?;
    fs::write(&directory / "subdir" / "deep" / "nested.txt", "nested")?;

    directory.empty_dir_sync()?;

    assert!(directory.is_dir_sync()?);
    assert_eq!(directory.read_dir_names_lossy_sync()?.len(), 0);
    Ok(())
}

#[cfg(unix)]
#[test]
fn empty_dir_sync_removes_symlinks_without_removing_targets() -> Result<()> {
    let directory = tempdir()?;
    let external = tempdir()?;
    let target_file = path!(&external) / "keep.txt";
    let target_dir = path!(&external) / "keep-dir";
    let nested_file = &target_dir / "keep.txt";
    fs::write(&target_file, b"preserved")?;
    fs::create_dir(&target_dir)?;
    fs::write(&nested_file, b"nested content")?;

    (path!(&directory) / "file-link").symlink_to_sync(&target_file)?;
    (path!(&directory) / "dir-link").symlink_to_sync(&target_dir)?;
    (path!(&directory) / "dangling-link").symlink_to_sync(path!(&external) / "missing")?;

    let path = path!(&directory);
    path.empty_dir_sync()?;

    assert_eq!(path.read_dir_paths_sync()?.len(), 0);
    assert_eq!(fs::read(&target_file)?, b"preserved");
    assert_eq!(fs::read(&nested_file)?, b"nested content");
    Ok(())
}

#[cfg(windows)]
#[test]
fn empty_dir_sync_removes_windows_directory_symlinks() -> Result<()> {
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

#[test]
fn create_parent_dir_all_sync_creates_missing_ancestors() -> Result<()> {
    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "nested" / "parent" / "file.txt";

    file.create_parent_dir_all_sync()?;
    file.create_parent_dir_all_sync()?;

    assert!(
        file.parent()
            .ok_or_else(|| anyhow!("expected a parent"))?
            .is_dir_sync()?
    );

    assert!(!file.try_exists_sync()?);
    Ok(())
}

#[test]
fn create_parent_dir_all_sync_without_parent_is_a_no_op() -> Result<()> {
    path!("").create_parent_dir_all_sync()?;
    Ok(())
}

#[test]
fn write_atomic_sync_replaces_existing_file_and_leaves_no_temporary_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "config.json";
    path.write_sync(b"old")?;

    path.write_atomic_sync(b"new")?;

    assert_eq!(path.read_sync()?, b"new");
    assert_eq!(path!(&temp_dir).read_dir_names_lossy_sync()?, ["config.json"]);
    Ok(())
}

#[test]
fn write_atomic_with_sync_prepares_empty_sibling_temporary_file() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "secret.key";

    path.write_atomic_with_sync(b"secret", |temp_path| {
        assert_ne!(temp_path, &path);
        assert_eq!(temp_path.parent(), path.parent());
        assert_eq!(temp_path.metadata_sync()?.len(), 0);
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

#[cfg(unix)]
#[test]
fn write_atomic_sync_keeps_existing_destination_mode() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "config.json";
    path.write_sync(b"old")?;
    path.chmod_sync(0o644)?;

    path.write_atomic_sync(b"new")?;
    assert_eq!(path.metadata_sync()?.permissions().mode() & 0o777, 0o644);

    path.write_atomic_with_sync(b"newer", |temp_path| temp_path.chmod_sync(0o600))?;
    assert_eq!(path.metadata_sync()?.permissions().mode() & 0o777, 0o600);
    Ok(())
}

#[test]
fn write_atomic_with_sync_removes_temporary_file_and_keeps_old_content_on_failure() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "config.json";
    path.write_sync(b"old")?;

    let error = path
        .write_atomic_with_sync(b"new", |_| Err(io::Error::other("prepare failed")))
        .err()
        .ok_or_else(|| anyhow!("expected prepare error"))?;

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(path.read_sync()?, b"old");
    assert_eq!(path!(&temp_dir).read_dir_names_lossy_sync()?, ["config.json"]);
    Ok(())
}

#[test]
fn write_atomic_with_sync_removes_temporary_file_on_panic() -> Result<()> {
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
    assert_eq!(path!(&temp_dir).read_dir_names_lossy_sync()?.len(), 0);
    Ok(())
}

#[cfg(unix)]
#[test]
fn write_atomic_sync_accepts_maximum_length_file_name() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "a".repeat(255);

    path.write_atomic_sync(b"long name")?;

    assert_eq!(path.read_sync()?, b"long name");
    Ok(())
}

#[test]
fn write_atomic_sync_rejects_path_without_file_name() -> Result<()> {
    let error = path!("/")
        .write_atomic_sync(b"data")
        .err()
        .ok_or_else(|| anyhow!("expected missing file name error"))?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    Ok(())
}

#[test]
fn persist_to_sync_replaces_destination_and_consumes_source() -> Result<()> {
    let temp_dir = tempdir()?;
    let src = path!(&temp_dir) / "prepared.tmp";
    let dest = path!(&temp_dir) / "dest.txt";
    src.write_sync(b"new")?;
    dest.write_sync(b"old")?;

    let persisted = src.persist_to_sync(&dest)?;

    assert_eq!(persisted, dest);
    assert!(!src.try_exists_sync()?);
    assert_eq!(dest.read_sync()?, b"new");
    Ok(())
}

#[test]
fn json_round_trips_through_unsized_value() -> Result<()> {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    let temp_dir = tempdir()?;
    let file = path!(&temp_dir) / "data.json";
    let original = [
        TestData {
            name: "first".to_owned(),
            value: 1,
        },
        TestData {
            name: "second".to_owned(),
            value: 2,
        },
    ];

    file.write_json_pretty_sync(&original[..])?;

    let loaded: Vec<TestData> = file.read_json_sync()?;
    assert_eq!(loaded, original);
    Ok(())
}

#[test]
fn read_json_sync_wraps_missing_file_as_io_error() -> Result<()> {
    let temp_dir = tempdir()?;
    let missing = path!(&temp_dir) / "missing.json";

    let error = missing
        .read_json_sync::<serde_json::Value>()
        .err()
        .ok_or_else(|| anyhow!("expected missing file error"))?;

    assert!(error.is_io());
    assert_eq!(error.io_error_kind(), Some(io::ErrorKind::NotFound));
    Ok(())
}

#[test]
fn write_json_pretty_sync_wraps_missing_parent_as_io_error() -> Result<()> {
    let temp_dir = tempdir()?;
    let file_path = path!(&temp_dir) / "missing" / "data.json";

    let error = file_path
        .write_json_pretty_sync(&[1, 2, 3])
        .err()
        .ok_or_else(|| anyhow!("expected missing parent error"))?;

    assert!(error.is_io());
    assert_eq!(error.io_error_kind(), Some(io::ErrorKind::NotFound));
    assert!(!(path!(&temp_dir) / "missing").try_exists_sync()?);
    Ok(())
}

#[test]
fn read_dir_helpers_collect_all_entries() -> Result<()> {
    let temp_dir = tempdir()?;
    let directory = path!(&temp_dir);
    fs::write(&directory / "a.txt", "")?;
    fs::write(&directory / "b.txt", "")?;
    fs::create_dir(&directory / "subdir")?;

    let mut names = directory.read_dir_names_lossy_sync()?;
    names.sort();
    assert_eq!(names, ["a.txt", "b.txt", "subdir"]);

    let mut paths = directory.read_dir_paths_sync()?;
    paths.sort();
    assert_eq!(
        paths,
        [&directory / "a.txt", &directory / "b.txt", &directory / "subdir"]
    );

    let mut lazy_paths = directory
        .read_dir_sync()?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;

    lazy_paths.sort();
    assert_eq!(lazy_paths, paths);

    let mut entries = directory.read_dir_entries_sync()?;
    entries.sort_by_key(PathEntry::file_name);
    assert_eq!(
        entries.iter().map(PathEntry::path).collect::<Vec<_>>(),
        [&directory / "a.txt", &directory / "b.txt", &directory / "subdir"],
    );

    let is_dir: Vec<_> = entries
        .iter()
        .map(|entry| entry.file_type().map(|file_type| file_type.is_dir()))
        .collect::<io::Result<_>>()?;

    assert_eq!(is_dir, [false, false, true]);
    Ok(())
}

#[cfg(unix)]
#[test]
fn special_file_type_checks_identify_only_their_type() -> Result<()> {
    use std::{
        ffi::CString,
        io,
        os::unix::{
            ffi::OsStrExt,
            net::UnixListener,
        },
    };

    let temp_dir = tempdir()?;
    let fifo = path!(&temp_dir) / "fifo";
    let fifo_c = CString::new(fifo.as_os_str().as_bytes())?;
    // SAFETY: `fifo_c` is a valid NUL-terminated string that outlives the call.
    if unsafe { libc::mkfifo(fifo_c.as_ptr(), 0o600) } != 0 {
        return Err(io::Error::last_os_error().into());
    }

    let socket = path!(&temp_dir) / "socket";
    let _listener = UnixListener::bind(&socket)?;
    let char_device = path!("/dev/null");
    let regular = path!(&temp_dir) / "file";
    regular.write_sync(b"")?;

    assert!(fifo.is_fifo_sync()? && !fifo.is_socket_sync()?);
    assert!(socket.is_socket_sync()? && !socket.is_fifo_sync()?);
    assert!(char_device.is_char_device_sync()? && !char_device.is_block_device_sync()?);
    assert!(!regular.is_fifo_sync()? && !regular.is_socket_sync()? && !regular.is_char_device_sync()?);
    assert!(!regular.is_block_device_sync()?);
    assert_eq!(
        (path!(&temp_dir) / "missing")
            .is_fifo_sync()
            .map_err(|error| error.kind()),
        Err(io::ErrorKind::NotFound)
    );

    Ok(())
}

#[cfg(unix)]
#[test]
fn write_atomic_sync_replaces_symlink_and_creates_new_files_as_0600() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir()?;
    let target = path!(&temp_dir) / "target.txt";
    let link = path!(&temp_dir) / "link.txt";
    target.write_sync(b"old")?;
    link.symlink_to_sync(&target)?;

    link.write_atomic_sync(b"new")?;
    assert!(!link.is_symlink_sync()?);
    assert_eq!(link.read_sync()?, b"new");
    assert_eq!(target.read_sync()?, b"old");

    let fresh = path!(&temp_dir) / "fresh.txt";
    fresh.write_atomic_sync(b"x")?;
    assert_eq!(fresh.metadata_sync()?.permissions().mode() & 0o777, 0o600);
    Ok(())
}

#[cfg(windows)]
#[test]
fn write_atomic_sync_replaces_destination_held_open_by_reader() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "config.json";
    path.write_sync(b"old")?;
    // std opens files with delete sharing, which the replace relies on.
    let reader = fs::File::open(&path)?;

    path.write_atomic_sync(b"new")?;
    drop(reader);

    assert_eq!(path.read_sync()?, b"new");
    Ok(())
}

#[cfg(unix)]
#[test]
fn write_atomic_sync_replaces_a_self_referencing_symlink() -> Result<()> {
    let temp_dir = tempdir()?;
    let dest = path!(&temp_dir) / "dest";
    dest.symlink_to_sync("dest")?;

    dest.write_atomic_sync(b"new")?;

    assert!(!dest.is_symlink_sync()?);
    assert_eq!(dest.read_sync()?, b"new");
    Ok(())
}

#[cfg(unix)]
#[test]
fn persist_to_sync_rejects_a_symlink_source() -> Result<()> {
    let temp_dir = tempdir()?;
    let data = path!(&temp_dir) / "data";
    data.write_sync(b"new")?;
    let source = path!(&temp_dir) / "staged";
    source.symlink_to_sync("data")?;
    let dest = path!(&temp_dir) / "dest";
    dest.write_sync(b"old")?;

    assert_eq!(
        source.persist_to_sync(&dest).map_err(|error| error.kind()),
        Err(io::ErrorKind::InvalidInput)
    );

    assert!(source.is_symlink_sync()?);
    assert_eq!(dest.read_sync()?, b"old");
    Ok(())
}

#[cfg(unix)]
#[test]
fn write_atomic_sync_fails_before_replacing_when_parent_is_unreadable() -> Result<()> {
    let temp_dir = tempdir()?;
    let parent = path!(&temp_dir) / "dir";
    parent.create_dir_sync()?;
    let dest = &parent / "dest";
    dest.write_sync(b"old")?;
    parent.chmod_sync(0o300)?;
    // Root can open the directory anyway, so only check the ordering when the open really fails.
    let result = fs::File::open(&parent).is_err().then(|| dest.write_atomic_sync(b"new"));
    parent.chmod_sync(0o700)?;

    if let Some(result) = result {
        assert_eq!(
            result.map_err(|error| error.kind()),
            Err(io::ErrorKind::PermissionDenied)
        );

        assert_eq!(dest.read_sync()?, b"old");
        assert_eq!(parent.read_dir_names_lossy_sync()?, ["dest"]);
    }

    Ok(())
}

#[test]
fn failed_replace_keeps_destination_and_cleans_up() -> Result<()> {
    let temp_dir = tempdir()?;
    let directory = path!(&temp_dir);
    let dest = &directory / "dest";
    dest.create_dir_sync()?;
    let marker = &dest / "keep";
    marker.write_sync(b"old")?;
    let source = &directory / "source";
    source.write_sync(b"new")?;

    assert!(dest.write_atomic_sync(b"new").is_err());
    assert!(source.persist_to_sync(&dest).is_err());

    assert_eq!(source.read_sync()?, b"new");
    assert_eq!(marker.read_sync()?, b"old");
    let mut names = directory.read_dir_names_lossy_sync()?;
    names.sort();
    assert_eq!(names, ["dest", "source"]);
    Ok(())
}

#[cfg(windows)]
#[test]
fn write_atomic_sync_gives_up_after_retrying_a_destination_without_delete_sharing() -> Result<()> {
    use std::{
        os::windows::fs::OpenOptionsExt,
        time::Duration,
    };

    // FILE_SHARE_READ | FILE_SHARE_WRITE, without FILE_SHARE_DELETE.
    const SHARE_READ_WRITE: u32 = 0x1 | 0x2;

    let temp_dir = tempdir()?;
    let dest = path!(&temp_dir) / "dest";
    dest.write_sync(b"old")?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARE_READ_WRITE)
        .open(&dest)?;

    let started = std::time::Instant::now();
    assert!(dest.write_atomic_sync(b"new").is_err());
    assert!(started.elapsed() >= Duration::from_millis(900));
    drop(lock);

    assert_eq!(dest.read_sync()?, b"old");
    assert_eq!(path!(&temp_dir).read_dir_names_lossy_sync()?, ["dest"]);
    dest.write_atomic_sync(b"new")?;
    assert_eq!(dest.read_sync()?, b"new");
    Ok(())
}

#[test]
fn read_dir_if_exists_sync_returns_none_only_for_missing_directory() -> Result<()> {
    let temp_dir = tempdir()?;
    let directory = path!(&temp_dir);
    (&directory / "a.txt").write_sync(b"")?;

    assert!((&directory / "missing").read_dir_if_exists_sync()?.is_none());
    let paths = directory
        .read_dir_if_exists_sync()?
        .into_iter()
        .flatten()
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;

    assert_eq!(paths, [&directory / "a.txt"]);
    Ok(())
}

#[test]
fn atomic_writer_publishes_streamed_contents_only_on_commit() -> Result<()> {
    let temp_dir = tempdir()?;
    let dest = path!(&temp_dir) / "dest";
    dest.write_sync(b"old")?;

    let mut writer = dest.atomic_writer_sync()?;
    io::copy(&mut &b"streamed"[..], &mut writer)?;
    assert_eq!(dest.read_sync()?, b"old");
    writer.commit()?;

    assert_eq!(dest.read_sync()?, b"streamed");
    assert_eq!(path!(&temp_dir).read_dir_names_lossy_sync()?, ["dest"]);
    Ok(())
}

#[test]
fn atomic_writer_accepts_contents_written_by_path() -> Result<()> {
    let temp_dir = tempdir()?;
    let dest = path!(&temp_dir) / "dest";

    let writer = dest.atomic_writer_sync()?;
    assert_eq!(writer.path().parent(), dest.parent());
    writer.path().write_sync(b"external")?;
    writer.commit()?;

    assert_eq!(dest.read_sync()?, b"external");
    Ok(())
}

#[test]
fn atomic_writer_dropped_without_commit_keeps_destination() -> Result<()> {
    let temp_dir = tempdir()?;
    let dest = path!(&temp_dir) / "dest";
    dest.write_sync(b"old")?;

    let mut writer = dest.atomic_writer_sync()?;
    writer.write_all(b"abandoned")?;
    drop(writer);

    assert_eq!(dest.read_sync()?, b"old");
    assert_eq!(path!(&temp_dir).read_dir_names_lossy_sync()?, ["dest"]);
    Ok(())
}

#[test]
fn atomic_writer_commit_noclobber_refuses_existing_destination() -> Result<()> {
    let temp_dir = tempdir()?;
    let dest = path!(&temp_dir) / "dest";

    let mut writer = dest.atomic_writer_sync()?;
    writer.write_all(b"first")?;
    writer.commit_noclobber()?;
    assert_eq!(dest.read_sync()?, b"first");

    let mut writer = dest.atomic_writer_sync()?;
    writer.write_all(b"second")?;
    assert_eq!(
        writer.commit_noclobber().map_err(|error| error.kind()),
        Err(io::ErrorKind::AlreadyExists)
    );

    assert_eq!(dest.read_sync()?, b"first");
    assert_eq!(path!(&temp_dir).read_dir_names_lossy_sync()?, ["dest"]);
    Ok(())
}

#[test]
fn write_json_pretty_atomic_sync_round_trips() -> Result<()> {
    let temp_dir = tempdir()?;
    let path = path!(&temp_dir) / "config.json";

    path.write_json_pretty_atomic_sync(&[1, 2, 3])?;
    assert_eq!(path.read_json_sync::<Vec<i32>>()?, [1, 2, 3]);
    path.write_json_pretty_atomic_sync(&[4])?;
    assert_eq!(path.read_json_sync::<Vec<i32>>()?, [4]);
    Ok(())
}

#[cfg(unix)]
#[test]
fn atomic_writer_keeps_temp_writable_by_path_and_applies_read_only_mode_at_commit() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir()?;
    let dest = path!(&temp_dir) / "secret";
    dest.write_sync(b"old")?;
    dest.chmod_sync(0o400)?;

    let writer = dest.atomic_writer_sync()?;
    writer.path().write_sync(b"new")?;
    writer.commit()?;

    assert_eq!(dest.read_sync()?, b"new");
    assert_eq!(dest.metadata_sync()?.permissions().mode() & 0o777, 0o400);
    Ok(())
}

#[cfg(unix)]
#[test]
fn atomic_writer_keeps_mode_set_by_the_caller() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir()?;
    let dest = path!(&temp_dir) / "config";
    dest.write_sync(b"old")?;
    dest.chmod_sync(0o644)?;

    let writer = dest.atomic_writer_sync()?;
    writer.path().chmod_sync(0o640)?;
    writer.commit()?;

    assert_eq!(dest.metadata_sync()?.permissions().mode() & 0o777, 0o640);
    Ok(())
}
