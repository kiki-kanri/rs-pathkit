use std::{
    borrow::Borrow,
    ffi::OsStr,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    ops::Deref,
    path::{
        Path as StdPath,
        PathBuf,
    },
};

use super::core::Path;

impl AsRef<StdPath> for Path {
    #[inline]
    fn as_ref(&self) -> &StdPath {
        &self.0
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

impl Display for Path {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_string_lossy())
    }
}

impl From<&StdPath> for Path {
    #[inline]
    fn from(path: &StdPath) -> Self {
        Self(path.to_path_buf())
    }
}

impl From<&str> for Path {
    #[inline]
    fn from(path: &str) -> Self {
        Self(PathBuf::from(path))
    }
}

impl AsRef<OsStr> for Path {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.0.as_os_str()
    }
}

/// An owned string-to-path conversion that consumes the string without accessing the filesystem.
///
/// # Examples
///
/// ```rust
/// use pathkit::Path;
///
/// let path = Path::from(String::from("config.json"));
/// assert_eq!(path.as_path(), std::path::Path::new("config.json"));
/// ```
impl From<String> for Path {
    #[inline]
    fn from(path: String) -> Self {
        Self(PathBuf::from(path))
    }
}

/// A consuming conversion into an owned, lossy Unicode path string.
///
/// Replaces invalid Unicode with the replacement character (`U+FFFD`). Use
/// `std::path::PathBuf::from(path)` instead to transfer the native path data losslessly.
///
/// # Examples
///
/// ```rust
/// use pathkit::path;
///
/// let text = String::from(path!("config.json"));
/// assert_eq!(text, "config.json");
/// ```
impl From<Path> for String {
    #[inline]
    fn from(path: Path) -> Self {
        path.to_string_lossy().into_owned()
    }
}

/// A borrowed UTF-8 string view of a path without copying its data.
///
/// The view borrows the original path. Unlike conversion into [`String`], this does
/// not replace invalid Unicode; [`std::path::Path::to_str`] provides a fallible alternative.
///
/// # Panics
///
/// Panics if the path is not valid UTF-8.
///
/// # Examples
///
/// ```rust
/// use pathkit::path;
///
/// let path = path!("config.json");
/// let text: &str = path.as_ref();
/// assert_eq!(text, "config.json");
/// ```
impl AsRef<str> for Path {
    #[inline]
    fn as_ref(&self) -> &str {
        match self.to_str() {
            Some(path) => path,
            None => panic!("Path contains non-UTF-8 data and cannot be borrowed as str"),
        }
    }
}

/// A borrowed [`std::path::PathBuf`] view without copying or converting native path data.
///
/// The view borrows the original path. It does not consume the path or allow mutation
/// of the underlying buffer.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
///
/// use pathkit::path;
///
/// let path = path!("config.json");
/// let buffer: &PathBuf = path.as_ref();
/// assert_eq!(buffer, &PathBuf::from("config.json"));
/// ```
impl AsRef<PathBuf> for Path {
    #[inline]
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl From<Path> for PathBuf {
    #[inline]
    fn from(path: Path) -> Self {
        path.0
    }
}

impl AsRef<Path> for Path {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Borrow,
        ffi::OsStr,
        path::{
            Path as StdPath,
            PathBuf,
        },
    };

    use super::Path;
    use crate::path;

    #[test]
    fn test_std_path_reference_traits() {
        let path = path!("/test/path");
        let expected = StdPath::new("/test/path");

        let as_ref: &StdPath = path.as_ref();
        let borrowed: &StdPath = path.borrow();
        let dereferenced: &StdPath = &path;

        assert_eq!(as_ref, expected);
        assert_eq!(borrowed, expected);
        assert_eq!(dereferenced, expected);
    }

    #[test]
    fn test_display_formats_lossy_path() {
        let path = path!("/test/path");

        assert_eq!(path.to_string(), "/test/path");
        assert_eq!(format!("{path}"), "/test/path");
        assert_eq!(format!("Path: {}", path!("file.txt")), "Path: file.txt");
    }

    #[test]
    fn test_path_conversions() {
        let std_path = StdPath::new("/test/path");
        let from_std_path: Path = Path::from(std_path);
        assert_eq!(from_std_path.to_str(), Some("/test/path"));

        let from_pathbuf = path!(PathBuf::from("/test/path"));
        assert_eq!(from_pathbuf.to_path_buf(), PathBuf::from("/test/path"));

        let path = path!("/test/path");
        let pathbuf: PathBuf = PathBuf::from(path.clone());
        let string: String = String::from(path);

        assert_eq!(pathbuf, PathBuf::from("/test/path"));
        assert_eq!(string, "/test/path");
        assert_eq!(Path::from("/test/path").to_str(), Some("/test/path"));
        assert_eq!(Path::from(String::from("/test/path")).to_str(), Some("/test/path"));
    }

    #[test]
    fn test_string_views() {
        let path = path!("/test/文件.txt");
        let ascii = path!("/test/path");
        let borrowed_str: &str = ascii.as_ref();

        assert_eq!(path.to_string_lossy(), "/test/文件.txt");
        assert_eq!(path.to_str(), Some("/test/文件.txt"));
        assert_eq!(borrowed_str, "/test/path");
    }

    #[cfg(unix)]
    #[test]
    #[should_panic(expected = "Path contains non-UTF-8 data and cannot be borrowed as str")]
    fn test_as_ref_str_panics_on_non_utf8_path() {
        use std::{
            ffi::OsString,
            os::unix::ffi::OsStringExt,
        };

        let path = path!(OsString::from_vec(vec![0xff]));
        let _: &str = path.as_ref();
    }

    #[test]
    fn test_os_and_pathbuf_views() {
        let path = path!("/test/path");
        let os_str: &OsStr = path.as_ref();
        let pathbuf: &PathBuf = path.as_ref();

        assert_eq!(path.as_os_str(), OsStr::new("/test/path"));
        assert_eq!(os_str, OsStr::new("/test/path"));
        assert_eq!(pathbuf, &PathBuf::from("/test/path"));
    }

    #[test]
    fn test_path_as_ref_self() {
        let path = path!("/test/path");
        let path_ref: &Path = path.as_ref();

        assert_eq!(path_ref, &path);
    }
}
