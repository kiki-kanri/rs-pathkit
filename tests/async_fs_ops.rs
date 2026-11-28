#![cfg(feature = "async-fs-ops")]

use std::{
    ffi::OsString,
    io,
};

use anyhow::{
    Result,
    anyhow,
};
use pathkit::{
    async_fs_ops::AsyncFsOps,
    path,
    sync_fs_ops::SyncFsOps,
};
use tempfile::tempdir;

fn expect_error<T, E>(result: Result<T, E>) -> Result<E> {
    result.err().ok_or_else(|| anyhow!("expected an error"))
}

#[tokio::test]
async fn rename_to_moves_file_and_returns_destination() -> Result<()> {
    let directory = tempdir()?;
    let source = path!(&directory) / "source.txt";
    let renamed = path!(&directory) / "renamed.txt";
    source.write(b"hello").await?;

    assert_eq!(source.rename_to(&renamed).await?, renamed);
    assert!(!source.try_exists().await?);
    assert_eq!(renamed.read().await?, b"hello");
    Ok(())
}

#[tokio::test]
async fn read_dir_entries_returns_usable_async_entries() -> Result<()> {
    let directory = tempdir()?;
    let file = path!(&directory) / "file.txt";
    file.write(b"content").await?;
    (path!(&directory) / "subdir").create_dir().await?;

    let mut entries = path!(&directory).read_dir_entries().await?;
    entries.sort_by_key(pathkit::async_fs_ops::AsyncPathEntry::file_name);
    assert_eq!(entries.len(), 2);

    let file_entry = &entries[0];
    assert_eq!(file_entry.file_name(), OsString::from("file.txt"));
    assert_eq!(file_entry.path(), file);
    assert!(file_entry.file_type().await?.is_file());
    assert_eq!(file_entry.metadata().await?.len(), 7);

    let directory_entry = &entries[1];
    assert_eq!(directory_entry.file_name(), OsString::from("subdir"));
    assert!(directory_entry.file_type().await?.is_dir());
    Ok(())
}

#[tokio::test]
async fn read_dir_names_and_paths_list_children() -> Result<()> {
    let directory = tempdir()?;
    (path!(&directory) / "a.txt").write(b"").await?;
    (path!(&directory) / "b").create_dir().await?;

    let mut names = path!(&directory).read_dir_names_lossy().await?;
    names.sort();
    assert_eq!(names, ["a.txt", "b"]);

    let mut paths = path!(&directory).read_dir_paths().await?;
    paths.sort();
    assert_eq!(paths, [path!(&directory) / "a.txt", path!(&directory) / "b"]);

    let mut read_dir = path!(&directory).read_dir().await?;
    let mut lazy_paths = Vec::new();
    while let Some(entry) = read_dir.next_entry().await? {
        lazy_paths.push(entry.path());
    }

    lazy_paths.sort();
    assert_eq!(lazy_paths, paths);
    assert!(read_dir.next_entry().await?.is_none());
    Ok(())
}

#[tokio::test]
async fn persist_to_replaces_destination() -> Result<()> {
    let directory = tempdir()?;
    let source = path!(&directory) / "prepared.tmp";
    let dest = path!(&directory) / "dest.txt";
    source.write(b"new").await?;
    dest.write(b"old").await?;

    assert_eq!(source.persist_to(&dest).await?, dest);

    assert!(!source.try_exists().await?);
    assert_eq!(dest.read().await?, b"new");
    Ok(())
}

#[tokio::test]
async fn write_atomic_with_prepare_panic_surfaces_as_io_error_and_temporary_file_removed() -> Result<()> {
    let directory = tempdir()?;
    let file = path!(&directory) / "config.json";
    file.write(b"old").await?;

    let error = expect_error(file.write_atomic_with(b"new", |_| panic!("prepare panicked")).await)?;

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(file.read().await?, b"old");
    assert_eq!(path!(&directory).read_dir_names_lossy().await?, ["config.json"]);
    Ok(())
}

#[tokio::test]
async fn if_exists_methods_return_none_or_false_for_missing_path() -> Result<()> {
    let directory = tempdir()?;
    let missing = path!(&directory) / "missing";

    assert!(missing.read_if_exists().await?.is_none());
    assert!(missing.read_to_string_if_exists().await?.is_none());
    assert!(missing.metadata_if_exists().await?.is_none());
    assert!(missing.symlink_metadata_if_exists().await?.is_none());
    assert!(!missing.remove_file_if_exists().await?);
    Ok(())
}

#[tokio::test]
async fn if_exists_methods_return_values_for_existing_path() -> Result<()> {
    let directory = tempdir()?;
    let file = path!(&directory) / "file.txt";
    file.write(b"content").await?;

    assert_eq!(file.read_if_exists().await?.as_deref(), Some(b"content".as_slice()));
    assert_eq!(file.read_to_string_if_exists().await?.as_deref(), Some("content"));
    assert_eq!(file.metadata_if_exists().await?.map(|metadata| metadata.len()), Some(7));
    assert_eq!(
        file.symlink_metadata_if_exists().await?.map(|metadata| metadata.len()),
        Some(7)
    );

    assert!(file.remove_file_if_exists().await?);
    assert!(!file.try_exists().await?);
    assert!(!file.remove_file_if_exists().await?);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn symlink_metadata_if_exists_returns_some_for_dangling_symlink() -> Result<()> {
    let directory = tempdir()?;
    let link = path!(&directory) / "link";
    link.symlink_to(path!(&directory) / "missing").await?;

    assert!(link.is_symlink().await?);
    assert!(link.metadata_if_exists().await?.is_none());
    assert!(
        link.symlink_metadata_if_exists()
            .await?
            .is_some_and(|metadata| metadata.file_type().is_symlink())
    );

    assert!(link.remove_file_if_exists().await?);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn if_exists_methods_propagate_errors_other_than_not_found() -> Result<()> {
    let directory = tempdir()?;
    let blocker = path!(&directory) / "blocker";
    blocker.write(b"file").await?;
    let child = &blocker / "child";

    let errors = [
        expect_error(child.read_if_exists().await)?,
        expect_error(child.read_to_string_if_exists().await)?,
        expect_error(child.metadata_if_exists().await)?,
        expect_error(child.symlink_metadata_if_exists().await)?,
        expect_error(child.remove_file_if_exists().await)?,
        expect_error(child.read_dir_if_exists().await)?,
    ];

    for error in errors {
        assert_eq!(error.kind(), io::ErrorKind::NotADirectory);
    }

    Ok(())
}

#[tokio::test]
async fn json_round_trips_unsized_values() -> Result<()> {
    let directory = tempdir()?;
    let file = path!(&directory) / "data.json";
    let values = vec![1, 2, 3];

    file.write_json_pretty(values.as_slice()).await?;

    assert_eq!(file.read_json::<Vec<i32>>().await?, values);
    Ok(())
}

#[tokio::test]
async fn read_json_missing_file_returns_wrapped_io_error() -> Result<()> {
    let directory = tempdir()?;
    let missing = path!(&directory) / "missing.json";

    let error = expect_error(missing.read_json::<serde_json::Value>().await)?;

    assert!(error.is_io());
    assert_eq!(error.io_error_kind(), Some(io::ErrorKind::NotFound));
    Ok(())
}

#[tokio::test]
async fn write_json_pretty_missing_parent_returns_wrapped_io_error() -> Result<()> {
    let directory = tempdir()?;
    let file = path!(&directory) / "missing" / "data.json";

    let error = expect_error(file.write_json_pretty(&[1, 2, 3]).await)?;

    assert!(error.is_io());
    assert_eq!(error.io_error_kind(), Some(io::ErrorKind::NotFound));
    Ok(())
}

#[tokio::test]
async fn create_parent_dir_all_creates_missing_ancestors() -> Result<()> {
    let directory = tempdir()?;
    let file = path!(&directory) / "nested" / "parent" / "file.txt";

    file.create_parent_dir_all().await?;
    file.create_parent_dir_all().await?;

    assert!((path!(&directory) / "nested" / "parent").is_dir().await?);
    assert!(!file.try_exists().await?);
    Ok(())
}

#[tokio::test]
async fn create_parent_dir_all_without_parent_is_noop() -> Result<()> {
    path!("").create_parent_dir_all().await?;
    Ok(())
}

#[tokio::test]
async fn link_methods_use_the_receiver_as_the_new_link() -> Result<()> {
    let directory = tempdir()?;
    let target = path!(&directory) / "target";
    let hard = path!(&directory) / "hard";
    target.write(b"first").await?;

    hard.hard_link_to(&target).await?;
    hard.write(b"second").await?;
    assert_eq!(target.read().await?, b"second");

    #[cfg(unix)]
    {
        let soft = path!(&directory) / "soft";
        soft.symlink_to("target").await?;
        assert_eq!(soft.read_link().await?, path!("target"));
        assert_eq!(soft.read().await?, b"second");
    }

    Ok(())
}

#[test]
fn dropping_an_in_flight_atomic_write_still_publishes_it() -> Result<()> {
    use std::{
        future::Future,
        task::{
            Context,
            Waker,
        },
        time::Duration,
    };

    let directory = tempdir()?;
    let dest = path!(&directory) / "dest";
    dest.write_sync(b"old")?;
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;

    runtime.block_on(async {
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel();
        let mut operation = Box::pin(dest.write_atomic_with(b"new", move |_| {
            started_tx.send(()).map_err(io::Error::other)?;
            resume_rx.recv_timeout(Duration::from_secs(5)).map_err(io::Error::other)
        }));

        assert!(
            operation
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );

        started_rx.recv_timeout(Duration::from_secs(5))?;
        drop(operation);
        resume_tx.send(())?;
        Ok::<(), anyhow::Error>(())
    })?;

    // Dropping the runtime waits for the blocking task to finish.
    drop(runtime);
    assert_eq!(dest.read_sync()?, b"new");
    assert_eq!(path!(&directory).read_dir_names_lossy_sync()?, ["dest"]);
    Ok(())
}

#[tokio::test]
async fn read_dir_if_exists_returns_none_for_missing_directory() -> Result<()> {
    let directory = tempdir()?;
    (path!(&directory) / "a.txt").write(b"").await?;

    assert!((path!(&directory) / "missing").read_dir_if_exists().await?.is_none());
    let mut read_dir = path!(&directory)
        .read_dir_if_exists()
        .await?
        .ok_or_else(|| anyhow!("expected an existing directory"))?;

    let entry = read_dir
        .next_entry()
        .await?
        .ok_or_else(|| anyhow!("expected one entry"))?;

    assert_eq!(entry.path(), path!(&directory) / "a.txt");
    Ok(())
}

#[tokio::test]
async fn write_json_pretty_atomic_round_trips() -> Result<()> {
    let directory = tempdir()?;
    let path = path!(&directory) / "config.json";

    path.write_json_pretty_atomic(&[1, 2, 3]).await?;

    assert_eq!(path.read_json::<Vec<i32>>().await?, [1, 2, 3]);
    assert_eq!(path!(&directory).read_dir_names_lossy().await?, ["config.json"]);
    Ok(())
}
