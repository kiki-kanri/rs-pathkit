//! Core path operations module
//!
//! This module provides the main `Path` struct which wraps `std::path::PathBuf`
//! and provides extended functionality similar to Python's pathlib.

use std::{
    ffi::OsStr,
    fs::canonicalize,
    path::{
        Path as StdPath,
        PathBuf,
    },
};

use anyhow::Result;
use path_absolutize::Absolutize;
use serde::{
    Deserialize,
    Serialize,
};

/// A wrapper around `std::path::PathBuf` that provides extended path operations.
///
/// `Path` is similar to Python's `pathlib.Path`, providing an object-oriented
/// interface for path manipulation. It wraps `std::path::PathBuf` and implements
/// various traits for seamless interoperability with the standard library.
///
/// # Features
///
/// - **Serde Support**: Can be serialized and deserialized
/// - **Trait Implementations**: Implements `AsRef`, `Borrow`, `Deref`, `Display`, `From`
/// - **Path Joining**: Supports the `/` operator via the `Div` trait
///
/// # Example
///
/// ```rust
/// use pathkit::Path;
///
/// // Create a path
/// let path = Path::new("/home/user/project");
///
/// // Join paths
/// let config = path.join("config.json");
///
/// // Use / operator (note: this consumes the path)
/// let nested = Path::new("/home/user") / "project" / "subdir";
///
/// // Get path components
/// let parent = path.parent();
/// let file_name = path.file_name();
/// let extension = path.extension();
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Path(pub(crate) PathBuf);

impl Path {
    /// Creates a new `Path` from a given path.
    ///
    /// This method accepts any type that can be converted to `PathBuf`,
    /// including `&str`, `String`, `PathBuf`, and `&Path`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pathkit::Path;
    ///
    /// // From &str
    /// let path = Path::new("/test/path");
    ///
    /// // From String
    /// let path = Path::new(String::from("/test/path"));
    ///
    /// // From PathBuf
    /// use std::path::PathBuf;
    /// let path = Path::new(PathBuf::from("/test/path"));
    /// ```
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self(path.into())
    }

    /// Converts the path to an absolute path.
    ///
    /// This uses the `path-absolutize` crate which handles edge cases
    /// like converting relative paths to absolute paths.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use pathkit::Path;
    ///
    /// let path = Path::new("relative/path");
    /// let absolute = path.absolutize()?;
    /// assert!(absolute.is_absolute());
    /// ```
    pub fn absolutize(&self) -> Result<Self> {
        Ok(Self::new(self.0.absolutize()?))
    }

    /// Converts the path to an absolute path, using the given directory as base.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use pathkit::Path;
    ///
    /// let path = Path::new("relative/path");
    /// let absolute = path.absolutize_from("/custom/cwd")?;
    /// ```
    pub fn absolutize_from(&self, cwd: impl AsRef<StdPath>) -> Result<Self> {
        Ok(Self::new(self.0.absolutize_from(cwd)?))
    }

    /// Converts a relative path to an absolute path with a virtual root.
    ///
    /// This is useful for testing or sandboxed environments where you want
    /// to treat a directory as the root.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use pathkit::Path;
    ///
    /// let path = Path::new("subdir/file.txt");
    /// let absolute = path.absolutize_virtually("/virtual/root")?;
    /// assert_eq!(absolute.to_str(), Some("/virtual/root/subdir/file.txt"));
    /// ```
    pub fn absolutize_virtually(&self, virtual_root: impl AsRef<StdPath>) -> Result<Self> {
        Ok(Self::new(self.0.absolutize_virtually(virtual_root)?))
    }

    /// Returns a reference to the underlying `std::path::Path`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pathkit::Path;
    ///
    /// let path = Path::new("/test/path");
    /// let std_path = path.as_path();
    /// assert_eq!(std_path, std::path::Path::new("/test/path"));
    /// ```
    pub fn as_path(&self) -> &StdPath {
        &self.0
    }

    /// Returns the canonical form of the path.
    ///
    /// This resolves symlinks and normalizes the path. Unlike `absolutize`,
    /// this requires the path to exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use pathkit::Path;
    ///
    /// let path = Path::new(".");
    /// let canonical = path.canonicalize()?;
    /// assert!(canonical.is_absolute());
    /// ```
    pub fn canonicalize(&self) -> Result<Self, std::io::Error> {
        canonicalize(&self.0).map(Self::new)
    }

    /// Joins this path with another path.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::MAIN_SEPARATOR;
    ///
    /// use pathkit::Path;
    ///
    /// let path = Path::new(&format!("{0}base", MAIN_SEPARATOR));
    /// let joined = path.join(&format!("subdir{0}file.txt", MAIN_SEPARATOR));
    /// assert_eq!(joined.to_str(), Some(format!("{0}base{0}subdir{0}file.txt", MAIN_SEPARATOR).as_str()));
    /// ```
    pub fn join(&self, path: impl AsRef<StdPath>) -> Self {
        Self::new(self.0.join(path))
    }

    /// Returns the parent directory of this path.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pathkit::Path;
    ///
    /// let path = Path::new("/base/subdir/file.txt");
    /// assert_eq!(path.parent().unwrap().to_str(), Some("/base/subdir"));
    ///
    /// // Root path has no parent
    /// let root = Path::new("/");
    /// assert!(root.parent().is_none());
    /// ```
    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(Self::new)
    }

    /// Converts this path to a `PathBuf`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pathkit::Path;
    /// use std::path::PathBuf;
    ///
    /// let path = Path::new("/test/path");
    /// let buf: PathBuf = path.to_path_buf();
    /// assert_eq!(buf, PathBuf::from("/test/path"));
    /// ```
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }

    /// Returns a new path with a different file extension.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pathkit::Path;
    ///
    /// // Replace extension
    /// let path = Path::new("/path/to/file.txt");
    /// assert_eq!(path.with_extension("md").to_str(), Some("/path/to/file.md"));
    ///
    /// // Add extension to file without one
    /// let path = Path::new("/path/to/file");
    /// assert_eq!(path.with_extension("txt").to_str(), Some("/path/to/file.txt"));
    /// ```
    pub fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self {
        Self::new(self.0.with_extension(extension))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsStr,
        path::MAIN_SEPARATOR,
    };

    use super::*;

    #[test]
    fn test_new() {
        // Test with &str
        let path = Path::new("/test/path");
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with String
        let path = Path::new(String::from("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with PathBuf
        let path = Path::new(PathBuf::from("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));
    }

    #[test]
    fn test_as_path() {
        let path = Path::new("/test/path");
        let std_path: &std::path::Path = path.as_path();
        assert_eq!(std_path, std::path::Path::new("/test/path"));
    }

    #[test]
    fn test_to_path_buf() {
        let path = Path::new("/test/path");
        let path_buf = path.to_path_buf();
        assert_eq!(path_buf, PathBuf::from("/test/path"));
    }

    #[test]
    fn test_join() {
        let path = Path::new(format!("{0}base", MAIN_SEPARATOR));
        assert_eq!(
            path.join("subdir").to_str(),
            Some(format!("{0}base{0}subdir", MAIN_SEPARATOR).as_str())
        );

        // On Windows, join doesn't treat "/" as separator, so we use sep for proper path construction
        assert_eq!(
            path.join(format!("subdir{0}file.txt", MAIN_SEPARATOR)).to_str(),
            Some(format!("{0}base{0}subdir{0}file.txt", MAIN_SEPARATOR).as_str())
        );

        // Join with Path
        let subpath = Path::new("subpath");
        assert_eq!(
            path.join(subpath).to_str(),
            Some(format!("{0}base{0}subpath", MAIN_SEPARATOR).as_str())
        );
    }

    #[test]
    fn test_parent() {
        let path = Path::new("/base/subdir/file.txt");
        assert_eq!(path.parent().unwrap().to_str(), Some("/base/subdir"));

        // Test root path
        let path = Path::new("/");
        assert!(path.parent().is_none());

        // Test relative path with no parent
        let path = Path::new("file.txt");
        assert!(path.parent().is_some());
    }

    #[test]
    fn test_with_extension() {
        let path = Path::new("/path/to/file.txt");
        assert_eq!(path.with_extension("md").to_str(), Some("/path/to/file.md"));

        // Test adding extension to file without extension
        let path = Path::new("/path/to/file");
        assert_eq!(path.with_extension("txt").to_str(), Some("/path/to/file.txt"));

        // Test replacing extension
        let path = Path::new("/path/to/file.txt");
        assert_eq!(path.with_extension("json").to_str(), Some("/path/to/file.json"));
    }

    #[test]
    fn test_is_absolute() {
        // On Unix, /absolute/path is absolute; on Windows, only C:\path or \\server\share are absolute
        #[cfg(not(windows))]
        {
            let path = Path::new("/absolute/path");
            assert!(path.is_absolute());
        }

        let relative_path = Path::new("relative/path");
        assert!(!relative_path.is_absolute());

        #[cfg(windows)]
        {
            // Windows-style absolute paths
            let path = Path::new("C:\\absolute\\path");
            assert!(path.is_absolute());
        }
    }

    #[test]
    fn test_is_relative() {
        // On Unix, /absolute/path is absolute; on Windows, /path is treated as relative
        // since it doesn't have a drive letter
        #[cfg(not(windows))]
        {
            let path = Path::new("/absolute/path");
            assert!(!path.is_relative());
        }

        let relative_path = Path::new("relative/path");
        assert!(relative_path.is_relative());

        #[cfg(windows)]
        {
            // Windows-style absolute paths are not relative
            let path = Path::new("C:\\absolute\\path");
            assert!(!path.is_relative());
        }
    }

    #[test]
    fn test_file_name() {
        let path = Path::new("/path/to/file.txt");
        assert_eq!(path.file_name(), Some(OsStr::new("file.txt")));

        // Note: std::path::Path ignores trailing slash and returns the last component
        let path = Path::new("/path/to/");
        assert_eq!(path.file_name(), Some(OsStr::new("to")));

        let path = Path::new("/");
        assert_eq!(path.file_name(), None);
    }

    #[test]
    fn test_file_stem() {
        let path = Path::new("file.txt");
        assert_eq!(path.file_stem(), Some(OsStr::new("file")));

        let path = Path::new(".hidden");
        assert_eq!(path.file_stem(), Some(OsStr::new(".hidden")));

        let path = Path::new("file.tar.gz");
        assert_eq!(path.file_stem(), Some(OsStr::new("file.tar")));
    }

    #[test]
    fn test_extension() {
        let path = Path::new("file.txt");
        assert_eq!(path.extension(), Some(OsStr::new("txt")));

        let path = Path::new("file.tar.gz");
        assert_eq!(path.extension(), Some(OsStr::new("gz")));

        let path = Path::new("file");
        assert_eq!(path.extension(), None);

        let path = Path::new("/path/to.");
        assert_eq!(path.extension(), Some(OsStr::new("")));
    }

    #[test]
    fn test_starts_with() {
        let path = Path::new("/path/to/file");
        // starts_with is part of std::path::Path, available through Deref
        assert!(path.starts_with("/path"));
        assert!(path.starts_with(std::path::Path::new("/path")));
        assert!(!path.starts_with("/other"));
    }

    #[test]
    fn test_ends_with() {
        let path = Path::new("/path/to/file.txt");
        assert!(path.ends_with("file.txt"));
        assert!(path.ends_with(std::path::Path::new("to/file.txt")));
        assert!(!path.ends_with("other.txt"));
    }

    #[test]
    fn test_components() {
        let path = Path::new("/path/to/file.txt");
        let components: Vec<_> = path.components().collect();
        assert!(components.len() >= 3);
    }

    #[test]
    fn test_iter() {
        let path = Path::new("/path/to/file.txt");
        let iter = path.iter();
        let components: Vec<_> = iter.collect();
        assert!(!components.is_empty());
    }

    // Skip contains test - Path doesn't have this method
    // Skip set_file_name test - Path doesn't have this method
    // Skip set_extension test - Path doesn't have this method
    // Skip pop test - Path doesn't have this method
    // Skip push test - Path doesn't have this method

    #[test]
    fn test_absolute_path_functionality() -> Result<()> {
        // Test absolutize
        let path = Path::new(".");
        let absolute = path.absolutize()?;
        assert!(absolute.is_absolute());

        Ok(())
    }

    #[test]
    fn test_absolutize_from() -> Result<()> {
        let path = Path::new("subdir");
        let absolute = path.absolutize_from(format!("{0}base", MAIN_SEPARATOR))?;
        // Check that the result ends with the joined path (handles platform-specific separators)
        assert!(absolute
            .to_str()
            .unwrap()
            .ends_with(&format!("{}subdir", MAIN_SEPARATOR)));

        Ok(())
    }

    #[test]
    fn test_absolutize_virtually() -> Result<()> {
        let path = Path::new("subdir/file.txt");
        let absolute = path.absolutize_virtually("/virtual")?;
        // Check that the result contains the virtual root and subdir (handles platform-specific separators)
        let abs_str = absolute.to_str().unwrap();
        assert!(abs_str.contains("virtual"));
        assert!(abs_str.contains("subdir"));
        assert!(abs_str.contains("file.txt"));

        Ok(())
    }

    #[test]
    fn test_canonicalize() -> Result<()> {
        let path = Path::new(".");
        let canonical = path.canonicalize()?;
        assert!(canonical.is_absolute());

        Ok(())
    }

    #[test]
    fn test_display() {
        let path = Path::new("/test/path");
        assert_eq!(format!("{}", path), "/test/path");
    }

    #[test]
    fn test_debug() {
        let path = Path::new("/test/path");
        let debug_str = format!("{:?}", path);
        assert!(debug_str.contains("Path"));
    }

    #[test]
    fn test_eq() {
        let path1 = Path::new("/test/path");
        let path2 = Path::new("/test/path");
        let path3 = Path::new("/other/path");

        assert_eq!(path1, path2);
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_hash() {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{
                Hash,
                Hasher,
            },
        };

        let path1 = Path::new("/test/path");
        let path2 = Path::new("/test/path");

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        path1.hash(&mut hasher1);
        path2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_ord() {
        use std::cmp::Ordering;

        let path1 = Path::new("/a/b");
        let path2 = Path::new("/a/c");

        assert_eq!(path1.cmp(&path2), Ordering::Less);
        assert_eq!(path2.cmp(&path1), Ordering::Greater);
        assert_eq!(path1.cmp(&path1), Ordering::Equal);
    }

    #[test]
    fn test_from_pathbuf() {
        let pathbuf = PathBuf::from("/test/path");
        let path = Path::new(pathbuf);
        assert_eq!(path.to_path_buf(), PathBuf::from("/test/path"));
    }

    #[test]
    fn test_from_std_path() {
        let std_path = std::path::Path::new("/test/path");
        let path = Path::from(std_path);
        assert_eq!(path.to_str(), Some("/test/path"));
    }
}
