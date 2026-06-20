#![cfg(feature = "async-fs-ops")]

use std::ffi::OsString;

use anyhow::{
    Result,
    anyhow,
};
use pathkit::{
    AsyncPathEntry,
    path,
};
use tempfile::tempdir;
use tokio::fs::{
    read_dir,
    write,
};

#[tokio::test]
async fn async_path_entry_path_returns_pathkit_path() -> Result<()> {
    let dir = tempdir()?;
    let file_path = path!(&dir) / "async-example.txt";
    write(&file_path, b"content").await?;

    let mut entries = read_dir(&dir).await?;
    let entry = entries
        .next_entry()
        .await?
        .ok_or_else(|| anyhow!("expected one async directory entry"))?;

    let path_entry = AsyncPathEntry::from(entry);
    let path = path_entry.path();

    assert_eq!(path, file_path);
    assert_eq!(path_entry.file_name(), OsString::from("async-example.txt"));
    assert!(path_entry.file_type().await?.is_file());
    assert!(path_entry.metadata().await?.is_file());

    Ok(())
}

#[tokio::test]
async fn async_path_entry_exposes_raw_dir_entry_interop() -> Result<()> {
    let dir = tempdir()?;
    let file_path = path!(&dir) / "async-raw.txt";
    write(&file_path, b"content").await?;

    let mut entries = read_dir(&dir).await?;
    let entry = entries
        .next_entry()
        .await?
        .ok_or_else(|| anyhow!("expected one async directory entry"))?;

    let path_entry = AsyncPathEntry::from(entry);
    assert_eq!(path!(path_entry.as_dir_entry().path()), file_path);

    let entry = path_entry.into_dir_entry();
    assert_eq!(path!(entry.path()), file_path);

    Ok(())
}
