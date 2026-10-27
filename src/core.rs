//! Owned paths and lexical path transformations.
//!
//! Filesystem operations are provided by [`crate::SyncFsOps`] and, with the
//! `async-fs-ops` feature, `AsyncFsOps`.

use std::{
    ffi::{
        OsStr,
        OsString,
    },
    io::Result as IoResult,
    path::{
        Path as StdPath,
        PathBuf,
    },
};

use derive_more::{
    AsRef,
    From,
    Into,
};
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
/// Formatting with [`std::fmt::Display`] replaces invalid Unicode with the replacement
/// character.
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, AsRef, From, Into)]
#[as_ref(StdPath, OsStr, PathBuf)]
#[from(&str, String, OsString, PathBuf)]
#[serde(transparent)]
pub struct Path(pub(crate) PathBuf);

impl Path {
    /// Creates an owned copy of a path without accessing the filesystem.
    ///
    /// Accepts any value implementing `AsRef<std::path::Path>`, including native OS strings,
    /// standard paths, and [`Path`] values. The input is borrowed to copy its path data;
    /// passing an owned value consumes that argument but does not reuse its buffer.
    /// [`Path::from`] takes ownership of a [`String`], [`std::ffi::OsString`], or
    /// [`std::path::PathBuf`] buffer without copying.
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
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn absolutize(&self) -> IoResult<Self> {
        Ok(Self(self.0.absolutize()?.into_owned()))
    }

    /// Returns a new lexically normalized path using `cwd` as the base for relative paths.
    ///
    /// Uses [`path_absolutize::Absolutize::absolutize_from`] without accessing the filesystem
    /// or resolving symlinks. Supply an absolute `cwd` to obtain an absolute result;
    /// a relative base does not establish an absolute working directory. The original
    /// path is unchanged.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pathkit::path;
    ///
    /// let base = std::env::current_dir()?;
    /// let resolved = path!("config.json").absolutize_from(&base);
    /// assert_eq!(resolved, path!(base.join("config.json")));
    /// # Ok::<(), std::io::Error>(())
    /// ```
    #[must_use]
    pub fn absolutize_from(&self, cwd: impl AsRef<StdPath>) -> Self {
        Self(self.0.absolutize_from(cwd).into_owned())
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
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn absolutize_virtually(&self, virtual_root: impl AsRef<StdPath>) -> IoResult<Self> {
        Ok(Self(self.0.absolutize_virtually(virtual_root)?.into_owned()))
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
        Self(self.0.join(path))
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
        Self(self.0.with_added_extension(extension))
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
        Self(self.0.with_extension(extension))
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
        Self(self.0.with_file_name(file_name))
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

    use anyhow::{
        Result,
        anyhow,
    };

    use super::*;

    #[test]
    fn test_new() {
        // Test with &str
        let path = crate::path!("/test/path");
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with String
        let path = crate::path!(String::from("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with PathBuf
        let path = crate::path!(PathBuf::from("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with OsStr
        let path = crate::path!(OsStr::new("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with OsString
        let path = crate::path!(OsString::from("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with StdPath
        let path = crate::path!(StdPath::new("/test/path"));
        assert_eq!(path.to_str(), Some("/test/path"));

        // Test with pathkit::Path reference
        let original = crate::path!("/test/path");
        let path = crate::path!(&original);
        assert_eq!(path.to_str(), Some("/test/path"));
    }

    #[test]
    fn test_as_path() {
        let path = crate::path!("/test/path");
        let std_path: &StdPath = path.as_path();
        assert_eq!(std_path, StdPath::new("/test/path"));
    }

    #[test]
    fn test_to_path_buf() {
        let path = crate::path!("/test/path");
        let path_buf = path.to_path_buf();
        assert_eq!(path_buf, PathBuf::from("/test/path"));
    }

    #[test]
    fn test_join() {
        let path = crate::path!("{MAIN_SEPARATOR}base");
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
        let subpath = crate::path!("subpath");
        assert_eq!(
            path.join(subpath).to_str(),
            Some(format!("{MAIN_SEPARATOR}base{MAIN_SEPARATOR}subpath").as_str())
        );
    }

    #[test]
    fn test_parent() {
        let path = crate::path!("/base/subdir/file.txt");
        assert_eq!(
            path.parent().and_then(|parent| parent.to_str().map(str::to_owned)),
            Some(String::from("/base/subdir"))
        );

        // Test root path
        let path = crate::path!("/");
        assert!(path.parent().is_none());

        // A one-component relative path has an empty parent
        let path = crate::path!("file.txt");
        assert_eq!(path.parent(), Some(crate::path!("")));
    }

    #[test]
    fn test_with_extension() {
        let path = crate::path!("/path/to/file.txt");
        assert_eq!(path.with_extension("md").to_str(), Some("/path/to/file.md"));

        // Test adding extension to file without extension
        let path = crate::path!("/path/to/file");
        assert_eq!(path.with_extension("txt").to_str(), Some("/path/to/file.txt"));

        // Test replacing extension
        let path = crate::path!("/path/to/file.txt");
        assert_eq!(path.with_extension("json").to_str(), Some("/path/to/file.json"));
    }

    #[test]
    fn test_with_file_name() {
        let path = crate::path!("/path/to/file.txt");
        let renamed: Path = path.with_file_name("other.md");

        let expected = format!("/path/to{MAIN_SEPARATOR}other.md");
        assert_eq!(renamed.to_str(), Some(expected.as_str()));
    }

    #[test]
    fn test_with_added_extension() {
        let path = crate::path!("/path/to/file.tar.gz");
        let rotated: Path = path.with_added_extension("1");

        assert_eq!(rotated.to_str(), Some("/path/to/file.tar.gz.1"));

        let path = crate::path!("/path/to/file");
        assert_eq!(path.with_added_extension("txt").to_str(), Some("/path/to/file.txt"));
    }

    #[test]
    fn test_absolute_path_functionality() -> Result<()> {
        // Test absolutize
        let path = crate::path!(".");
        let absolute = path.absolutize()?;
        assert!(absolute.is_absolute());

        Ok(())
    }

    #[test]
    fn test_absolutize_from() {
        let path = crate::path!("subdir");
        let absolute = path.absolutize_from(format!("{MAIN_SEPARATOR}base"));
        // Check that the result ends with the joined path (handles platform-specific separators)
        assert!(
            absolute
                .to_str()
                .is_some_and(|path| path.ends_with(&format!("{MAIN_SEPARATOR}subdir")))
        );
    }

    #[test]
    fn test_absolutize_virtually() -> Result<()> {
        let path = crate::path!("subdir/file.txt");
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
}
