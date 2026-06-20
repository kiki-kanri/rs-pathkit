use std::{
    ffi::OsString,
    fs::{
        DirEntry,
        FileType,
        Metadata,
    },
    io,
};

use crate::Path;

/// A wrapper around [`std::fs::DirEntry`] whose path accessors return [`Path`].
///
/// This keeps directory traversal in the `pathkit` fluent API while preserving
/// access to common `DirEntry` metadata helpers.
#[derive(Debug)]
pub struct PathEntry(DirEntry);

impl From<DirEntry> for PathEntry {
    #[inline]
    fn from(entry: DirEntry) -> Self {
        Self::new(entry)
    }
}

impl PathEntry {
    /// Creates a new `PathEntry` from a standard directory entry.
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
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata()
    }

    /// Returns the file type for this entry.
    pub fn file_type(&self) -> io::Result<FileType> {
        self.0.file_type()
    }

    /// Returns a reference to the underlying standard directory entry.
    #[inline]
    pub fn as_dir_entry(&self) -> &DirEntry {
        &self.0
    }

    /// Converts this wrapper into the underlying standard directory entry.
    #[inline]
    pub fn into_dir_entry(self) -> DirEntry {
        self.0
    }
}

#[cfg(test)]
mod tests {
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
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn path_entry_path_returns_pathkit_path() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("example.txt");
        write(&file_path, b"content")?;

        let entry = read_dir(dir.path())?
            .next()
            .ok_or_else(|| anyhow!("expected one directory entry"))??;

        let path_entry = PathEntry::from(entry);
        let path: Path = path_entry.path();

        assert_eq!(path, Path::new(file_path));
        assert_eq!(path_entry.file_name(), OsString::from("example.txt"));
        assert!(path_entry.file_type()?.is_file());
        assert!(path_entry.metadata()?.is_file());

        Ok(())
    }

    #[test]
    fn path_entry_exposes_raw_dir_entry_interop() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("raw.txt");
        write(&file_path, b"content")?;

        let entry = read_dir(dir.path())?
            .next()
            .ok_or_else(|| anyhow!("expected one directory entry"))??;

        let path_entry = PathEntry::from(entry);
        assert_eq!(path_entry.as_dir_entry().path(), file_path);

        let entry = path_entry.into_dir_entry();
        assert_eq!(entry.path(), file_path);

        Ok(())
    }
}
