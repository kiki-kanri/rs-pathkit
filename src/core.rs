//! Owned paths and lexical path transformations.
//!
//! Filesystem access is limited to operations that explicitly require it, such as
//! [`Path::canonicalize`].

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

/// An owned filesystem path with path transformations and filesystem extension traits.
///
/// The path retains the platform-native representation of [`std::path::PathBuf`].
/// Construction and lexical transformations do not require the path to exist and do
/// not modify the filesystem. Borrowed transformations leave the original value unchanged.
///
/// Dereferencing exposes [`std::path::Path`] methods. Equality, hashing, and ordering
/// follow [`std::path::PathBuf`] semantics, not filesystem identity or natural sorting.
/// The `/` operator returns a joined path; an owned operand is consumed, while a borrowed
/// operand remains available.
///
/// Serde serialization and deserialization use the underlying [`std::path::PathBuf`]
/// representation; serialization fails if the native path is not valid UTF-8.
/// Converting into [`String`] or formatting with [`std::fmt::Display`] replaces invalid
/// Unicode with the replacement character. In contrast, borrowing
/// through `AsRef<str>` panics if the path is not valid UTF-8; use
/// [`std::path::Path::to_str`] to check or [`Self::as_path`] to preserve native encoding.
///
/// # Examples
///
/// ```rust
/// use pathkit::path;
///
/// let root = path!("project");
/// let config = &root / "config" / "app.json";
/// assert_eq!(config, root.join("config").join("app.json"));
/// assert_eq!(root, path!("project"));
///
/// let mut paths = vec![path!("b"), path!("a")];
/// paths.sort();
/// assert_eq!(paths, vec![path!("a"), path!("b")]);
/// ```
///
/// # See also
///
/// - [`crate::SyncFsOps`]
/// - [`crate::PathEntry`]
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Path(pub(crate) PathBuf);

impl Path {
    /// Creates an owned copy of a path without accessing the filesystem.
    ///
    /// Accepts any value implementing `AsRef<std::path::Path>`, including native OS strings,
    /// standard paths, and [`Path`] values. The input is borrowed to copy its path data;
    /// passing an owned value consumes that argument but does not reuse its buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::Path;
    ///
    /// let path = Path::new("project/config.json");
    /// assert_eq!(path.as_path(), std::path::Path::new("project/config.json"));
    /// ```
    #[inline]
    pub fn new(path: impl AsRef<StdPath>) -> Self {
        Self(path.as_ref().to_path_buf())
    }

    /// Returns a new lexically normalized absolute path using the current working directory.
    ///
    /// Resolves `.` and `..` through [`path_absolutize::Absolutize::absolutize`] without
    /// resolving symlinks or requiring the path to exist. The original path is unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error if obtaining the current working directory fails for a relative path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let path = path!("relative/file.txt");
    /// let absolute = path.absolutize()?;
    /// assert!(absolute.is_absolute());
    /// assert_eq!(path, path!("relative/file.txt"));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn absolutize(&self) -> Result<Self> {
        Ok(Self::new(self.0.absolutize()?))
    }

    /// Returns a new lexically normalized path using `cwd` as the base for relative paths.
    ///
    /// Uses [`path_absolutize::Absolutize::absolutize_from`] without accessing the filesystem
    /// or resolving symlinks. Supply an absolute `cwd` to obtain an absolute result;
    /// a relative base does not establish an absolute working directory. The original
    /// path is unchanged.
    ///
    /// Although the return type is [`anyhow::Result`], this implementation always returns `Ok`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let base = std::env::current_dir()?;
    /// let resolved = path!("config.json").absolutize_from(&base)?;
    /// assert_eq!(resolved, path!(base.join("config.json")));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn absolutize_from(&self, cwd: impl AsRef<StdPath>) -> Result<Self> {
        Ok(Self::new(self.0.absolutize_from(cwd)))
    }

    /// Returns a new lexically normalized path relative to a virtual root.
    ///
    /// Uses [`path_absolutize::Absolutize::absolutize_virtually`]. A relative `virtual_root`
    /// is resolved against the current working directory. Relative paths are normalized
    /// before being joined to that root; absolute paths are checked using the upstream
    /// platform-specific root check. The original path is unchanged.
    ///
    /// This operation neither resolves symlinks nor creates a filesystem sandbox. It must
    /// not be used as a security boundary or as proof that filesystem access stays within
    /// `virtual_root`.
    ///
    /// # Errors
    ///
    /// Propagates errors from resolving the root or normalizing the path. Returns an error
    /// when the upstream root check rejects an absolute path or a Windows drive prefix.
    /// On Windows, checks requiring UTF-8 also fail for non-UTF-8 paths or roots.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let root = std::env::current_dir()?;
    /// let resolved = path!("config.json").absolutize_virtually(&root)?;
    /// assert_eq!(resolved, path!(root.join("config.json")));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn absolutize_virtually(&self, virtual_root: impl AsRef<StdPath>) -> Result<Self> {
        Ok(Self::new(self.0.absolutize_virtually(virtual_root)?))
    }

    /// Returns a borrowed view of the underlying standard path without copying its data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let path = path!("config.json");
    /// assert_eq!(path.as_path(), std::path::Path::new("config.json"));
    /// ```
    #[inline]
    pub fn as_path(&self) -> &StdPath {
        &self.0
    }

    /// Returns a new absolute path with symlinks resolved by the filesystem.
    ///
    /// Uses [`std::fs::canonicalize`], leaving the original value unchanged. Unlike
    /// [`Self::absolutize`], this operation requires the path to exist. On Windows, the
    /// result uses the standard library's extended-length path representation.
    ///
    /// # Errors
    ///
    /// Propagates filesystem errors, including missing path components, permission errors,
    /// and non-directory components before the end of the path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let directory = tempfile::tempdir()?;
    /// let canonical = path!(directory.path()).canonicalize()?;
    /// assert!(canonical.is_absolute());
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn canonicalize(&self) -> Result<Self, std::io::Error> {
        canonicalize(&self.0).map(Self::new)
    }

    /// Returns a new path with `path` joined according to standard path semantics.
    ///
    /// Leaves the original value unchanged. An absolute argument replaces the base;
    /// Windows rooted paths and drive prefixes follow [`std::path::PathBuf::push`].
    /// This is a lexical operation, not a check that the result stays within the base.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let base = path!("project");
    /// let joined = base.join("config.json");
    /// assert_eq!(
    ///     joined.as_path(),
    ///     std::path::Path::new("project").join("config.json")
    /// );
    ///
    /// assert_eq!(base, path!("project"));
    /// ```
    ///
    /// # See also
    ///
    /// - [`std::path::Path::join`]
    #[inline]
    #[must_use]
    pub fn join(&self, path: impl AsRef<StdPath>) -> Self {
        Self::new(self.0.join(path))
    }

    /// Returns an owned parent path, or `None` if no parent exists.
    ///
    /// Leaves the original value unchanged and follows [`std::path::Path::parent`].
    /// A root or empty path has no parent; a one-component relative path has an empty
    /// parent path rather than `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// assert_eq!(
    ///     path!("project/config.json").parent(),
    ///     Some(path!("project"))
    /// );
    ///
    /// assert_eq!(path!("config.json").parent(), Some(path!("")));
    /// assert_eq!(path!("").parent(), None);
    /// ```
    #[inline]
    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(Self::new)
    }

    /// Returns an owned copy of the underlying [`std::path::PathBuf`].
    ///
    /// Leaves the original value unchanged. To transfer the buffer instead of copying it,
    /// consume the path with `PathBuf::from(path)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    ///
    /// use pathkit::path;
    ///
    /// let path = path!("config.json");
    /// assert_eq!(path.to_path_buf(), PathBuf::from("config.json"));
    /// ```
    #[inline]
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }

    /// Returns a new path with `extension` appended to its full file name.
    ///
    /// Leaves the original value unchanged. Unlike [`Self::with_extension`], this keeps
    /// any existing extension. An empty extension or a path without a file name leaves
    /// the path unchanged, following [`std::path::Path::with_added_extension`].
    ///
    /// # Panics
    ///
    /// Panics if `extension` contains a path separator recognized by the current platform.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let path = path!("app.log");
    /// assert_eq!(path.with_added_extension("1"), path!("app.log.1"));
    /// assert_eq!(path, path!("app.log"));
    /// ```
    #[inline]
    #[must_use]
    pub fn with_added_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self {
        Self::new(self.0.with_added_extension(extension))
    }

    /// Returns a new path with its final extension replaced by `extension`.
    ///
    /// Leaves the original value unchanged. An empty extension removes the final extension;
    /// a path without a file stem is unchanged, following [`std::path::Path::with_extension`].
    ///
    /// # Panics
    ///
    /// Panics if `extension` contains a path separator recognized by the current platform.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// assert_eq!(
    ///     path!("archive.tar.gz").with_extension("xz"),
    ///     path!("archive.tar.xz")
    /// );
    ///
    /// assert_eq!(path!("config.json").with_extension(""), path!("config"));
    /// ```
    #[inline]
    #[must_use]
    pub fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self {
        Self::new(self.0.with_extension(extension))
    }

    /// Returns a new path with its file name replaced by `file_name`.
    ///
    /// Leaves the original value unchanged. If there is no file name, appends `file_name`.
    /// The argument is interpreted as a path, not validated as a single component;
    /// separators, rooted paths, and drive prefixes follow [`std::path::Path::with_file_name`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let path = path!("project/config.json");
    /// assert_eq!(
    ///     path.with_file_name("settings.json"),
    ///     path!("project").join("settings.json")
    /// );
    ///
    /// assert_eq!(path, path!("project/config.json"));
    /// ```
    #[inline]
    #[must_use]
    pub fn with_file_name<S: AsRef<OsStr>>(&self, file_name: S) -> Self {
        Self::new(self.0.with_file_name(file_name))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::{
            OsStr,
            OsString,
        },
        path::MAIN_SEPARATOR,
    };

    use anyhow::anyhow;

    use super::*;
    use crate::path;

    #[test]
    fn test_new() {
        // Test with &str
        let path = path!("/test/path");
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with String
        let path = path!(String::from("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with PathBuf
        let path = path!(PathBuf::from("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with OsStr
        let path = path!(OsStr::new("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with OsString
        let path = path!(OsString::from("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with StdPath
        let path = path!(StdPath::new("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with pathkit::Path reference
        let original = path!("/test/path");
        let path = path!(&original);
        assert_eq!(path.to_str(), Some("/test/path"));
    }

    #[test]
    fn test_as_path() {
        let path = path!("/test/path");
        let std_path: &std::path::Path = path.as_path();
        assert_eq!(std_path, std::path::Path::new("/test/path"));
    }

    #[test]
    fn test_to_path_buf() {
        let path = path!("/test/path");
        let path_buf = path.to_path_buf();
        assert_eq!(path_buf, PathBuf::from("/test/path"));
    }

    #[test]
    fn test_join() {
        let path = path!("{MAIN_SEPARATOR}base");
        assert_eq!(
            path.join("subdir").to_str(),
            Some(format!("{MAIN_SEPARATOR}base{MAIN_SEPARATOR}subdir").as_str())
        );

        // On Windows, join doesn't treat "/" as separator, so we use sep for proper path construction
        assert_eq!(
            path.join(format!("subdir{MAIN_SEPARATOR}file.txt")).to_str(),
            Some(format!("{MAIN_SEPARATOR}base{MAIN_SEPARATOR}subdir{MAIN_SEPARATOR}file.txt").as_str())
        );

        // Join with Path
        let subpath = path!("subpath");
        assert_eq!(
            path.join(subpath).to_str(),
            Some(format!("{MAIN_SEPARATOR}base{MAIN_SEPARATOR}subpath").as_str())
        );
    }

    #[test]
    fn test_parent() {
        let path = path!("/base/subdir/file.txt");
        assert_eq!(
            path.parent().and_then(|parent| parent.to_str().map(str::to_owned)),
            Some(String::from("/base/subdir"))
        );

        // Test root path
        let path = path!("/");
        assert!(path.parent().is_none());

        // Test relative path with no parent
        let path = path!("file.txt");
        assert!(path.parent().is_some());
    }

    #[test]
    fn test_with_extension() {
        let path = path!("/path/to/file.txt");
        assert_eq!(path.with_extension("md").to_str(), Some("/path/to/file.md"));

        // Test adding extension to file without extension
        let path = path!("/path/to/file");
        assert_eq!(path.with_extension("txt").to_str(), Some("/path/to/file.txt"));

        // Test replacing extension
        let path = path!("/path/to/file.txt");
        assert_eq!(path.with_extension("json").to_str(), Some("/path/to/file.json"));
    }

    #[test]
    fn test_with_file_name() {
        let path = path!("/path/to/file.txt");
        let renamed: Path = path.with_file_name("other.md");

        let expected = format!("/path/to{MAIN_SEPARATOR}other.md");
        assert_eq!(renamed.to_str(), Some(expected.as_str()));
    }

    #[test]
    fn test_with_added_extension() {
        let path = path!("/path/to/file.tar.gz");
        let rotated: Path = path.with_added_extension("1");

        assert_eq!(rotated.to_str(), Some("/path/to/file.tar.gz.1"));

        let path = path!("/path/to/file");
        assert_eq!(path.with_added_extension("txt").to_str(), Some("/path/to/file.txt"));
    }

    #[test]
    fn test_is_absolute() {
        // On Unix, /absolute/path is absolute; on Windows, only C:\path or \\server\share are absolute
        #[cfg(not(windows))]
        {
            let path = path!("/absolute/path");
            assert!(path.is_absolute());
        }

        let relative_path = path!("relative/path");
        assert!(!relative_path.is_absolute());

        #[cfg(windows)]
        {
            // Windows-style absolute paths
            let path = path!("C:\\absolute\\path");
            assert!(path.is_absolute());
        }
    }

    #[test]
    fn test_is_relative() {
        // On Unix, /absolute/path is absolute; on Windows, /path is treated as relative
        // since it doesn't have a drive letter
        #[cfg(not(windows))]
        {
            let path = path!("/absolute/path");
            assert!(!path.is_relative());
        }

        let relative_path = path!("relative/path");
        assert!(relative_path.is_relative());

        #[cfg(windows)]
        {
            // Windows-style absolute paths are not relative
            let path = path!("C:\\absolute\\path");
            assert!(!path.is_relative());
        }
    }

    #[test]
    fn test_file_name() {
        let path = path!("/path/to/file.txt");
        assert_eq!(path.file_name(), Some(OsStr::new("file.txt")));

        // Note: std::path::Path ignores trailing slash and returns the last component
        let path = path!("/path/to/");
        assert_eq!(path.file_name(), Some(OsStr::new("to")));

        let path = path!("/");
        assert_eq!(path.file_name(), None);
    }

    #[test]
    fn test_file_stem() {
        let path = path!("file.txt");
        assert_eq!(path.file_stem(), Some(OsStr::new("file")));

        let path = path!(".hidden");
        assert_eq!(path.file_stem(), Some(OsStr::new(".hidden")));

        let path = path!("file.tar.gz");
        assert_eq!(path.file_stem(), Some(OsStr::new("file.tar")));
    }

    #[test]
    fn test_extension() {
        let path = path!("file.txt");
        assert_eq!(path.extension(), Some(OsStr::new("txt")));

        let path = path!("file.tar.gz");
        assert_eq!(path.extension(), Some(OsStr::new("gz")));

        let path = path!("file");
        assert_eq!(path.extension(), None);

        let path = path!("/path/to.");
        assert_eq!(path.extension(), Some(OsStr::new("")));
    }

    #[test]
    fn test_starts_with() {
        let path = path!("/path/to/file");
        // starts_with is part of std::path::Path, available through Deref
        assert!(path.starts_with("/path"));
        assert!(path.starts_with(std::path::Path::new("/path")));
        assert!(!path.starts_with("/other"));
    }

    #[test]
    fn test_ends_with() {
        let path = path!("/path/to/file.txt");
        assert!(path.ends_with("file.txt"));
        assert!(path.ends_with(std::path::Path::new("to/file.txt")));
        assert!(!path.ends_with("other.txt"));
    }

    #[test]
    fn test_path_component_iteration() {
        let path = path!("/path/to/file.txt");
        let components: Vec<_> = path.components().collect();
        let iter_components: Vec<_> = path.iter().collect();

        assert!(components.len() >= 3);
        assert_ne!(iter_components, [] as [&OsStr; 0]);
    }

    // Skip contains test - Path doesn't have this method
    // Skip set_file_name test - Path doesn't have this method
    // Skip set_extension test - Path doesn't have this method
    // Skip pop test - Path doesn't have this method
    // Skip push test - Path doesn't have this method

    #[test]
    fn test_absolute_path_functionality() -> Result<()> {
        // Test absolutize
        let path = path!(".");
        let absolute = path.absolutize()?;
        assert!(absolute.is_absolute());

        Ok(())
    }

    #[test]
    fn test_absolutize_from() -> Result<()> {
        let path = path!("subdir");
        let absolute = path.absolutize_from(format!("{MAIN_SEPARATOR}base"))?;
        // Check that the result ends with the joined path (handles platform-specific separators)
        assert!(
            absolute
                .to_str()
                .is_some_and(|path| path.ends_with(&format!("{MAIN_SEPARATOR}subdir")))
        );

        Ok(())
    }

    #[test]
    fn test_absolutize_virtually() -> Result<()> {
        let path = path!("subdir/file.txt");
        let absolute = path.absolutize_virtually("/virtual")?;
        // Check that the result contains the virtual root and subdir (handles platform-specific separators)
        let abs_str = absolute
            .to_str()
            .ok_or_else(|| anyhow!("absolute path should be valid UTF-8"))?;

        assert!(abs_str.contains("virtual"));
        assert!(abs_str.contains("subdir"));
        assert!(abs_str.contains("file.txt"));

        Ok(())
    }

    #[test]
    fn test_canonicalize() -> Result<()> {
        let path = path!(".");
        let canonical = path.canonicalize()?;
        assert!(canonical.is_absolute());

        Ok(())
    }

    #[test]
    fn test_display() {
        let path = path!("/test/path");
        assert_eq!(format!("{path}"), "/test/path");
    }

    #[test]
    fn test_from_pathbuf() {
        let pathbuf = PathBuf::from("/test/path");
        let path = path!(pathbuf);
        assert_eq!(path.to_path_buf(), PathBuf::from("/test/path"));
    }

    #[test]
    fn test_from_std_path() {
        let std_path = std::path::Path::new("/test/path");
        let path = Path::from(std_path);
        assert_eq!(path.to_str(), Some("/test/path"));
    }
}
