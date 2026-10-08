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

/// A directory entry with owned [`Path`] access and native file-name and metadata queries.
///
/// Owns a [`tokio::fs::DirEntry`]. Paths and names preserve native encoding; metadata
/// and file-type queries do not follow symlinks. Resource retention follows the
/// underlying directory entry.
///
/// Available with the `async-fs-ops` feature. Metadata and file-type I/O requires a
/// Tokio runtime with blocking-task support when the underlying query submits work.
#[derive(Debug)]
pub struct AsyncPathEntry(DirEntry);

impl From<DirEntry> for AsyncPathEntry {
    #[inline]
    fn from(entry: DirEntry) -> Self {
        Self::new(entry)
    }
}

impl AsyncPathEntry {
    /// Creates a directory entry by taking ownership of a [`tokio::fs::DirEntry`].
    ///
    /// Does not enumerate the directory or query additional metadata.
    #[inline]
    pub fn new(entry: DirEntry) -> Self {
        Self(entry)
    }

    // Public methods

    /// Returns an owned path by combining the iterator's directory path and the entry's name.
    ///
    /// The result is not necessarily absolute: a relative directory path produces a
    /// relative entry path. Does not resolve symlinks or check that the entry still exists.
    #[inline]
    pub fn path(&self) -> Path {
        Path::new(self.0.path())
    }

    /// Returns an owned copy of the entry's file name without its directory path.
    ///
    /// Preserves the platform-native encoding without converting it to a Unicode string.
    #[inline]
    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }

    /// Returns metadata for the directory entry without following symlinks.
    ///
    /// Leaves the entry available for further queries. Follows [`tokio::fs::DirEntry::metadata`].
    ///
    /// # Errors
    ///
    /// Propagates filesystem errors if the underlying query requires I/O and fails, such
    /// as when the entry is removed or permissions prevent reading its metadata.
    ///
    /// # Panics
    ///
    /// Panics if the query submits blocking work while polled outside a Tokio runtime.
    ///
    /// # Cancellation safety
    ///
    /// Dropping the future discards its result. Metadata I/O already submitted to Tokio's
    /// blocking pool can continue; the entry remains usable for later queries.
    pub async fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata().await
    }

    /// Returns the directory entry's file type without following symlinks.
    ///
    /// Leaves the entry available for further queries. Follows [`tokio::fs::DirEntry::file_type`].
    ///
    /// # Errors
    ///
    /// Propagates filesystem errors if the underlying query requires I/O and fails, such
    /// as when the entry is removed or permissions prevent reading its metadata.
    ///
    /// # Panics
    ///
    /// Panics if the query submits blocking work while polled outside a Tokio runtime.
    ///
    /// # Cancellation safety
    ///
    /// Dropping the future discards its result. Metadata I/O already submitted to Tokio's
    /// blocking pool can continue; the entry remains usable for later queries.
    pub async fn file_type(&self) -> io::Result<FileType> {
        self.0.file_type().await
    }

    /// Returns a borrowed view of the underlying [`tokio::fs::DirEntry`].
    ///
    /// Does not consume the entry or query the filesystem.
    #[inline]
    pub fn as_dir_entry(&self) -> &DirEntry {
        &self.0
    }

    /// Consumes this entry and returns its underlying [`tokio::fs::DirEntry`].
    ///
    /// Transfers ownership without querying the filesystem.
    #[inline]
    pub fn into_dir_entry(self) -> DirEntry {
        self.0
    }
}
