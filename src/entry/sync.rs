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
