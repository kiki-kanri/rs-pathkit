use std::{
    ffi::OsString,
    fs::{
        FileType,
        Metadata,
    },
    io,
};

use tokio::fs::DirEntry;

use crate::Path;

/// A wrapper around [`tokio::fs::DirEntry`] whose path accessors return [`Path`].
///
/// This is the asynchronous counterpart to [`PathEntry`](crate::PathEntry).
#[derive(Debug)]
pub struct AsyncPathEntry(DirEntry);

impl From<DirEntry> for AsyncPathEntry {
    #[inline]
    fn from(entry: DirEntry) -> Self {
        Self::new(entry)
    }
}

impl AsyncPathEntry {
    /// Creates a new `AsyncPathEntry` from a Tokio directory entry.
    #[inline]
    pub fn new(entry: DirEntry) -> Self {
        Self(entry)
    }

    // Public methods

    /// Returns the full path to this entry as a [`Path`].
    #[inline]
    pub fn path(&self) -> Path {
        Path::new(self.0.path())
    }

    /// Returns this entry's file name.
    #[inline]
    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }

    /// Returns metadata for this entry.
    pub async fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata().await
    }

    /// Returns the file type for this entry.
    pub async fn file_type(&self) -> io::Result<FileType> {
        self.0.file_type().await
    }

    /// Returns a reference to the underlying Tokio directory entry.
    #[inline]
    pub fn as_dir_entry(&self) -> &DirEntry {
        &self.0
    }

    /// Converts this wrapper into the underlying Tokio directory entry.
    #[inline]
    pub fn into_dir_entry(self) -> DirEntry {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use anyhow::{
        Result,
        anyhow,
    };
    use tempfile::tempdir;
    use tokio::fs::{
        read_dir,
        write,
    };

    use super::*;

    #[tokio::test]
    async fn async_path_entry_path_returns_pathkit_path() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("async-example.txt");
        write(&file_path, b"content").await?;

        let mut entries = read_dir(dir.path()).await?;
        let entry = entries
            .next_entry()
            .await?
            .ok_or_else(|| anyhow!("expected one async directory entry"))?;

        let path_entry = AsyncPathEntry::from(entry);
        let path: Path = path_entry.path();

        assert_eq!(path, Path::new(file_path));
        assert_eq!(path_entry.file_name(), OsString::from("async-example.txt"));
        assert!(path_entry.file_type().await?.is_file());
        assert!(path_entry.metadata().await?.is_file());

        Ok(())
    }

    #[tokio::test]
    async fn async_path_entry_exposes_raw_dir_entry_interop() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("async-raw.txt");
        write(&file_path, b"content").await?;

        let mut entries = read_dir(dir.path()).await?;
        let entry = entries
            .next_entry()
            .await?
            .ok_or_else(|| anyhow!("expected one async directory entry"))?;

        let path_entry = AsyncPathEntry::from(entry);
        assert_eq!(path_entry.as_dir_entry().path(), file_path);

        let entry = path_entry.into_dir_entry();
        assert_eq!(entry.path(), file_path);

        Ok(())
    }
}
