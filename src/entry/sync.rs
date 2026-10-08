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

/// A directory entry with owned [`Path`] access and native file-name and metadata queries.
///
/// Owns a [`std::fs::DirEntry`]. Paths and names preserve native encoding; metadata
/// and file-type queries do not follow symlinks. Resource retention follows the
/// underlying directory entry.
///
/// On Unix, retaining an entry can keep its directory's file descriptor open even
/// after the directory iterator is dropped.
#[derive(Debug)]
pub struct PathEntry(DirEntry);

impl From<DirEntry> for PathEntry {
    #[inline]
    fn from(entry: DirEntry) -> Self {
        Self::new(entry)
    }
}

impl PathEntry {
    /// Creates a directory entry by taking ownership of a [`std::fs::DirEntry`].
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
    /// Leaves the entry available for further queries. Follows [`std::fs::DirEntry::metadata`].
    ///
    /// # Errors
    ///
    /// Propagates filesystem errors if the underlying query requires I/O and fails, such
    /// as when the entry is removed or permissions prevent reading its metadata.
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata()
    }

    /// Returns the directory entry's file type without following symlinks.
    ///
    /// Leaves the entry available for further queries. Follows [`std::fs::DirEntry::file_type`].
    ///
    /// # Errors
    ///
    /// Propagates filesystem errors if the underlying query requires I/O and fails, such
    /// as when the entry is removed or permissions prevent reading its metadata.
    pub fn file_type(&self) -> io::Result<FileType> {
        self.0.file_type()
    }

    /// Returns a borrowed view of the underlying [`std::fs::DirEntry`].
    ///
    /// Does not consume the entry or query the filesystem.
    #[inline]
    pub fn as_dir_entry(&self) -> &DirEntry {
        &self.0
    }

    /// Consumes this entry and returns its underlying [`std::fs::DirEntry`].
    ///
    /// Transfers ownership without querying the filesystem.
    #[inline]
    pub fn into_dir_entry(self) -> DirEntry {
        self.0
    }
}
