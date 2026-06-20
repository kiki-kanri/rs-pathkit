use std::{
    ffi::OsString,
    fs::{
        read_dir,
        write,
    },
};

use anyhow::{
    Result,
    anyhow,
};
use pathkit::{
    PathEntry,
    path,
};
use tempfile::tempdir;

#[test]
fn path_entry_path_returns_pathkit_path() -> Result<()> {
    let dir = tempdir()?;
    let file_path = path!(&dir) / "example.txt";
    write(&file_path, b"content")?;

    let entry = read_dir(&dir)?
        .next()
        .ok_or_else(|| anyhow!("expected one directory entry"))??;

    let path_entry = PathEntry::from(entry);
    let path = path_entry.path();

    assert_eq!(path, file_path);
    assert_eq!(path_entry.file_name(), OsString::from("example.txt"));
    assert!(path_entry.file_type()?.is_file());
    assert!(path_entry.metadata()?.is_file());

    Ok(())
}

#[test]
fn path_entry_exposes_raw_dir_entry_interop() -> Result<()> {
    let dir = tempdir()?;
    let file_path = path!(&dir) / "raw.txt";
    write(&file_path, b"content")?;

    let entry = read_dir(&dir)?
        .next()
        .ok_or_else(|| anyhow!("expected one directory entry"))??;

    let path_entry = PathEntry::from(entry);
    assert_eq!(path!(path_entry.as_dir_entry().path()), file_path);

    let entry = path_entry.into_dir_entry();
    assert_eq!(path!(entry.path()), file_path);

    Ok(())
}
