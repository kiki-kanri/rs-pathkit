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
