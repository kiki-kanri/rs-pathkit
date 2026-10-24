use std::{
    borrow::Borrow,
    ffi::{
        OsStr,
        OsString,
    },
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

use crate::core::Path;

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

/// An owned native-string-to-path conversion that consumes the string without copying
/// or accessing the filesystem.
///
/// # Examples
///
/// ```rust
/// use std::ffi::OsString;
///
/// use pathkit::Path;
///
/// let path = Path::from(OsString::from("config.json"));
/// assert_eq!(path.as_path(), std::path::Path::new("config.json"));
/// ```
impl From<OsString> for Path {
    #[inline]
    fn from(path: OsString) -> Self {
        Self(PathBuf::from(path))
    }
}

/// An owned path conversion that takes the [`std::path::PathBuf`] buffer without copying
/// or accessing the filesystem.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
///
/// use pathkit::Path;
///
/// let path = Path::from(PathBuf::from("config.json"));
/// assert_eq!(path.as_path(), std::path::Path::new("config.json"));
/// ```
impl From<PathBuf> for Path {
    #[inline]
    fn from(path: PathBuf) -> Self {
        Self(path)
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
        ffi::{
            OsStr,
            OsString,
        },
        path::{
            Path as StdPath,
            PathBuf,
        },
    };

    use super::Path;

    #[test]
    fn test_std_path_reference_traits() {
        let path = crate::path!("/test/path");
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
        let path = crate::path!("/test/path");

        assert_eq!(path.to_string(), "/test/path");
        assert_eq!(format!("{path}"), "/test/path");
        assert_eq!(format!("Path: {}", crate::path!("file.txt")), "Path: file.txt");
    }

    #[test]
    fn test_path_conversions() {
        let std_path = StdPath::new("/test/path");
        let expected = PathBuf::from("/test/path");

        assert_eq!(Path::from(std_path).as_path(), std_path);
        assert_eq!(Path::from("/test/path").as_path(), std_path);
        assert_eq!(Path::from(String::from("/test/path")).as_path(), std_path);
        assert_eq!(Path::from(OsString::from("/test/path")).as_path(), std_path);
        assert_eq!(Path::from(expected.clone()).as_path(), std_path);
        assert_eq!(PathBuf::from(crate::path!("/test/path")), expected);
    }

    #[test]
    fn test_string_views() {
        let path = crate::path!("/test/文件.txt");

        assert_eq!(path.to_str(), Some("/test/文件.txt"));
        assert_eq!(path.to_string_lossy(), "/test/文件.txt");
    }

    #[cfg(unix)]
    #[test]
    fn test_non_utf8_path_preserves_native_data() {
        use std::os::unix::ffi::{
            OsStrExt,
            OsStringExt,
        };

        let bytes = b"/test/\xff".to_vec();
        let path = Path::from(OsString::from_vec(bytes.clone()));
        let os_str: &OsStr = path.as_ref();

        assert_eq!(path.to_str(), None);
        assert_eq!(path.to_string(), "/test/\u{fffd}");
        assert_eq!(os_str.as_bytes(), bytes.as_slice());
        assert_eq!(PathBuf::from(path).into_os_string().into_vec(), bytes);
    }

    #[test]
    fn test_os_and_pathbuf_views() {
        let path = crate::path!("/test/path");
        let os_str: &OsStr = path.as_ref();
        let pathbuf: &PathBuf = path.as_ref();

        assert_eq!(path.as_os_str(), OsStr::new("/test/path"));
        assert_eq!(os_str, OsStr::new("/test/path"));
        assert_eq!(pathbuf, &PathBuf::from("/test/path"));
    }

    #[test]
    fn test_path_as_ref_self() {
        let path = crate::path!("/test/path");
        let path_ref: &Path = path.as_ref();

        assert_eq!(path_ref, &path);
    }
}
