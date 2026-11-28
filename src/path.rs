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

/// An owned filesystem path.
///
/// Dereferences to [`std::path::Path`]; inherent methods that share a name with a std method
/// shadow it and return an owned [`Path`]. Equality, hashing, and ordering follow [`PathBuf`], not
/// filesystem identity.
///
/// Serde delegates to [`PathBuf`], so serializing a non-UTF-8 path fails. [`fmt::Display`]
/// replaces invalid Unicode with `U+FFFD`.
///
/// The `/` operator joins like [`Path::join`]: an absolute right-hand side replaces the base.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, AsRef, From, Into)]
#[as_ref(StdPath, OsStr, PathBuf)]
#[from(&str, &OsStr, &StdPath, &PathBuf, String, std::ffi::OsString, PathBuf)]
#[serde(transparent)]
pub struct Path(PathBuf);

impl Path {
    /// Copies the path data; use [`Path::from`] to take ownership of an existing buffer.
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

    pub fn as_path(&self) -> &StdPath {
        &self.0
    }

    #[must_use]
    pub fn join(&self, path: impl AsRef<StdPath>) -> Self {
        Self(self.0.join(path))
    }

    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(Self::new)
    }

    /// # Panics
    ///
    /// Panics if `extension` contains a path separator, as the std method does.
    #[must_use]
    pub fn with_added_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self {
        Self(self.0.with_added_extension(extension))
    }

    /// # Panics
    ///
    /// Panics if `extension` contains a path separator, as the std method does.
    #[must_use]
    pub fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self {
        Self(self.0.with_extension(extension))
    }

    #[must_use]
    pub fn with_file_name<S: AsRef<OsStr>>(&self, file_name: S) -> Self {
        Self(self.0.with_file_name(file_name))
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl Borrow<StdPath> for Path {
    fn borrow(&self) -> &StdPath {
        &self.0
    }
}

impl Deref for Path {
    type Target = StdPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0.display(), f)
    }
}

impl<P: AsRef<StdPath>> Div<P> for Path {
    type Output = Path;

    /// Reuses the left-hand buffer.
    fn div(mut self, rhs: P) -> Path {
        self.0.push(rhs);
        self
    }
}

impl<P: AsRef<StdPath>> Div<P> for &Path {
    type Output = Path;

    fn div(self, rhs: P) -> Path {
        self.join(rhs)
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
        path::MAIN_SEPARATOR,
    };

    use super::*;
    use crate::path;

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
    fn owned_and_borrowed_div_join_without_changing_the_base() {
        let base = path!("base");
        let expected = Path::from(format!("base{MAIN_SEPARATOR}sub"));

        assert_eq!(&base / "sub", expected);
        assert_eq!(base.clone() / path!("sub"), expected);
        assert_eq!(base, path!("base"));
    }

    #[test]
    fn div_with_absolute_operand_replaces_base() {
        let absolute = if cfg!(windows) { r"C:\abs" } else { "/abs" };
        assert_eq!(path!("base") / absolute, path!(absolute));
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

    #[test]
    fn absolutize_from_handles_empty_path_and_relative_base() {
        let base = if cfg!(windows) { r"C:\base" } else { "/base" };
        assert_eq!(path!("").absolutize_from(base), path!(base));
        assert_eq!(path!("a/../b").absolutize_from("rel"), path!("rel") / "b");
        assert!(!path!("x").absolutize_from("rel").is_absolute());
    }
}
