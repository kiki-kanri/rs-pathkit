use std::{
    borrow::Borrow,
    ffi::OsStr,
    fmt,
    io,
    ops::{
        Deref,
        Div,
    },
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

/// An owned filesystem path; filesystem operations live in [`crate::sync_fs_ops`] and, with
/// `async-fs-ops`, `async_fs_ops`.
///
/// Dereferences to [`std::path::Path`]; inherent methods that share a name with a std method
/// shadow it and return an owned [`Path`]. Equality, hashing, and ordering follow [`PathBuf`], not
/// filesystem identity.
///
/// Serde delegates to [`PathBuf`], so serializing a non-UTF-8 path fails. [`fmt::Display`]
/// replaces invalid Unicode with `U+FFFD`.
///
/// The `/` operator joins like [`Path::join`]: an absolute right-hand side replaces the base.
///
/// ```rust
/// use pathkit::path;
///
/// let root = path!("project");
/// let config = &root / "config" / "app.json";
/// assert_eq!(config, root.join("config").join("app.json"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, AsRef, From, Into)]
#[as_ref(StdPath, OsStr, PathBuf)]
#[from(&str, String, std::ffi::OsString, PathBuf)]
#[serde(transparent)]
pub struct Path(PathBuf);

impl Path {
    /// Copies the path data; use [`Path::from`] to take ownership of an existing buffer.
    #[inline]
    pub fn new(path: impl AsRef<StdPath>) -> Self {
        Self(path.as_ref().to_path_buf())
    }

    /// Wraps [`std::path::absolute`], so `..` stays on Unix; use [`Self::absolutize`] to resolve it
    /// lexically.
    pub fn absolute(&self) -> io::Result<Self> {
        std::path::absolute(&self.0).map(Self)
    }

    /// Makes the path absolute against the current working directory and resolves `.` and `..`
    /// lexically, without resolving symlinks or requiring the path to exist. `..` past the root stays
    /// at the root; an empty path resolves to the current working directory.
    ///
    /// Lexical `..` handling can differ from the filesystem when a preceding component is a
    /// symlink; use [`SyncFsOps::canonicalize_sync`](crate::sync_fs_ops::SyncFsOps::canonicalize_sync)
    /// when the path must exist and match the filesystem.
    ///
    /// # Errors
    ///
    /// Fails when the current working directory cannot be read for a relative path.
    pub fn absolutize(&self) -> io::Result<Self> {
        Ok(Self(self.0.absolutize()?.into_owned()))
    }

    /// Like [`Self::absolutize`], but resolves a relative path against `base` instead of the
    /// current working directory. `base` is used as given: a relative `base` gives a relative
    /// result, and its own `..` components are not resolved.
    #[must_use]
    pub fn absolutize_from(&self, base: impl AsRef<StdPath>) -> Self {
        Self(self.0.absolutize_from(base).into_owned())
    }

    #[inline]
    pub fn as_path(&self) -> &StdPath {
        &self.0
    }

    #[inline]
    #[must_use]
    pub fn join(&self, path: impl AsRef<StdPath>) -> Self {
        Self(self.0.join(path))
    }

    #[inline]
    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(Self::new)
    }

    /// # Panics
    ///
    /// Panics if `extension` contains a path separator, as the std method does.
    #[inline]
    #[must_use]
    pub fn with_added_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self {
        Self(self.0.with_added_extension(extension))
    }

    /// # Panics
    ///
    /// Panics if `extension` contains a path separator, as the std method does.
    #[inline]
    #[must_use]
    pub fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self {
        Self(self.0.with_extension(extension))
    }

    #[inline]
    #[must_use]
    pub fn with_file_name<S: AsRef<OsStr>>(&self, file_name: S) -> Self {
        Self(self.0.with_file_name(file_name))
    }
}

impl AsRef<Path> for Path {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl Borrow<StdPath> for Path {
    #[inline]
    fn borrow(&self) -> &StdPath {
        &self.0
    }
}

impl Deref for Path {
    type Target = StdPath;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Path {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0.display(), f)
    }
}

impl<P: AsRef<StdPath>> Div<P> for Path {
    type Output = Path;

    /// Reuses the left-hand buffer.
    #[inline]
    fn div(mut self, rhs: P) -> Path {
        self.0.push(rhs);
        self
    }
}

impl<P: AsRef<StdPath>> Div<P> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: P) -> Path {
        self.join(rhs)
    }
}

impl From<&StdPath> for Path {
    #[inline]
    fn from(path: &StdPath) -> Self {
        Self(path.to_path_buf())
    }
}

impl From<&PathBuf> for Path {
    #[inline]
    fn from(path: &PathBuf) -> Self {
        Self(path.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{
            BTreeMap,
            HashMap,
        },
        env,
        ffi::OsString,
        path::MAIN_SEPARATOR,
    };

    use super::*;
    use crate::path;

    #[test]
    fn parent_of_single_component_is_empty_not_none() {
        assert_eq!(path!("file.txt").parent(), Some(path!("")));
        assert_eq!(path!("").parent(), None);
    }

    #[test]
    fn absolute_resolves_relative_paths_and_rejects_empty() -> io::Result<()> {
        let absolute = path!("relative").absolute()?;
        assert!(absolute.is_absolute());
        assert!(absolute.ends_with("relative"));
        assert_eq!(
            path!("").absolute().map_err(|error| error.kind()),
            Err(io::ErrorKind::InvalidInput)
        );

        Ok(())
    }

    #[test]
    fn absolutize_resolves_dot_components_lexically() -> io::Result<()> {
        let base = if cfg!(windows) { r"C:\base" } else { "/base" };
        let expected = Path::new(base).join("b");

        assert_eq!(path!("a/../b").absolutize_from(base), expected);
        assert_eq!(
            path!("../../b").absolutize_from(base),
            Path::new(base).with_file_name("b")
        );

        assert_eq!(path!("a/../b").absolutize()?, path!(env::current_dir()?) / "b");
        Ok(())
    }

    #[test]
    fn div_accepts_any_path_like_operand() {
        let base = path!("base");
        let expected = Path::from(format!("base{MAIN_SEPARATOR}sub"));

        assert_eq!(&base / "sub", expected);
        assert_eq!(&base / String::from("sub"), expected);
        assert_eq!(&base / OsString::from("sub"), expected);
        assert_eq!(&base / PathBuf::from("sub"), expected);
        assert_eq!(&base / &path!("sub"), expected);
        assert_eq!(base.clone() / path!("sub"), expected);
        // `push("")` appends a trailing separator even though the paths compare equal.
        assert_eq!(
            (base.clone() / "").as_os_str(),
            OsStr::new(&format!("base{MAIN_SEPARATOR}"))
        );
        assert_eq!(base, path!("base"));
    }

    #[test]
    fn div_with_absolute_operand_replaces_base() {
        let absolute = if cfg!(windows) { r"C:\abs" } else { "/abs" };
        assert_eq!(path!("base") / absolute, path!(absolute));
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_path_is_preserved_and_displayed_lossily() {
        use std::os::unix::ffi::{
            OsStrExt,
            OsStringExt,
        };

        let bytes = b"/test/\xff".to_vec();
        let path = Path::from(OsString::from_vec(bytes.clone()));

        assert_eq!(path.to_str(), None);
        assert_eq!(path.to_string(), "/test/\u{fffd}");
        assert_eq!(path.as_os_str().as_bytes(), bytes.as_slice());
        assert_eq!(PathBuf::from(path).into_os_string().into_vec(), bytes);
    }

    #[cfg(unix)]
    #[test]
    fn serializing_non_utf8_path_fails() {
        use std::os::unix::ffi::OsStrExt;

        let path = path!(OsStr::from_bytes(b"\xff"));
        assert!(serde_json::to_string(&path).is_err());
    }

    #[test]
    fn serde_is_transparent_string_and_keeps_spelling() -> serde_json::Result<()> {
        let path = path!("a/./b/");
        assert_eq!(serde_json::to_string(&path)?, r#""a/./b/""#);
        assert_eq!(
            serde_json::from_str::<Path>(r#""a/./b/""#)?.as_os_str(),
            path.as_os_str()
        );

        Ok(())
    }

    #[test]
    fn display_honors_width_and_precision() {
        assert_eq!(format!("{:>6}", path!("ab")), "    ab");
        assert_eq!(format!("{:.2}", path!("abcd")), "ab");
    }

    #[test]
    fn std_path_can_look_up_owned_keys_by_component_equality() {
        let key = path!("a/./b/");
        let query = StdPath::new("a//b");

        assert_eq!(HashMap::from([(key.clone(), 1)]).get(query), Some(&1));
        assert_eq!(BTreeMap::from([(key, 1)]).get(query), Some(&1));
    }

    #[cfg(unix)]
    #[test]
    fn absolute_keeps_parent_components_unlike_absolutize() -> io::Result<()> {
        let path = path!("/a/../b/");

        assert_eq!(path.absolute()?.as_os_str(), OsStr::new("/a/../b/"));
        assert_eq!(path.absolutize()?.as_os_str(), OsStr::new("/b"));
        Ok(())
    }

    #[test]
    fn absolutize_from_handles_empty_path_and_relative_base() {
        let base = if cfg!(windows) { r"C:\base" } else { "/base" };
        assert_eq!(path!("").absolutize_from(base), path!(base));
        assert_eq!(path!("a/../b").absolutize_from("rel"), path!("rel") / "b");
        assert!(!path!("x").absolutize_from("rel").is_absolute());
    }

    #[cfg(windows)]
    #[test]
    fn windows_div_accepts_forward_slashes_and_keeps_drive_for_rooted_operand() {
        assert_eq!(path!(r"C:\base") / "a/b", path!(r"C:\base\a\b"));
        assert_eq!(path!(r"C:\base") / r"\abs", path!(r"C:\abs"));
        assert_eq!((path!(r"C:\base") / r"D:child").as_os_str(), OsStr::new(r"D:child"));
    }
}
