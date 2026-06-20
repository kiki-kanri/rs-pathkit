use std::ops::Div;

use super::core::Path;

impl Div<&Path> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &Path) -> Self::Output {
        self.join(rhs)
    }
}

impl Div<&Path> for Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &Path) -> Self::Output {
        self.join(rhs)
    }
}

impl Div<Path> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: Path) -> Self::Output {
        self.join(&rhs)
    }
}

impl Div<Path> for Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: Path) -> Self::Output {
        self.join(&rhs)
    }
}

/// Division via `&str` — enabled by `AsRef<str>` on `Path`.
impl Div<&str> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &str) -> Self::Output {
        self.join(rhs)
    }
}

/// Division via `&String` — `&String` coerces to `&str` via Deref.
impl Div<&String> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &String) -> Self::Output {
        self.join(rhs.as_str())
    }
}

/// Division via `String` — enabled by `AsRef<str>` on `Path`.
impl Div<String> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: String) -> Self::Output {
        self.join(rhs.as_str())
    }
}

/// Division via `&str` — enabled by `AsRef<str>` on `Path`.
impl Div<&str> for Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &str) -> Self::Output {
        self.join(rhs)
    }
}

/// Division via `&String` — `&String` coerces to `&str` via Deref.
impl Div<&String> for Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &String) -> Self::Output {
        self.join(rhs.as_str())
    }
}

/// Division via `String` — enabled by `AsRef<str>` on `Path`.
impl Div<String> for Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: String) -> Self::Output {
        self.join(rhs.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::path;

    #[test]
    fn test_div_accepts_path_variants() {
        let base = path!("/test/path");
        let subpath = path!("subpath");
        let expected = path!("/test/path/subpath");

        assert_eq!(&base / &subpath, expected);
        assert_eq!(&base / subpath.clone(), expected);
        assert_eq!(base.clone() / &subpath, expected);
        assert_eq!(base / subpath, expected);
    }

    #[test]
    fn test_div_accepts_string_variants() {
        let base = path!("/test/path");
        let subpath = String::from("subpath");
        let expected = path!("/test/path/subpath");

        assert_eq!(&base / "subpath", expected);
        assert_eq!(&base / subpath.clone(), expected);
        assert_eq!(&base / &subpath, expected);
        assert_eq!(base.clone() / "subpath", expected);
        assert_eq!(base.clone() / subpath.clone(), expected);
        assert_eq!(base / &subpath, expected);
    }

    #[test]
    fn test_div_chains_and_accepts_embedded_separators() {
        assert_eq!(
            path!("/test") / "path" / "to" / "file.txt",
            path!("/test/path/to/file.txt")
        );

        assert_eq!(path!("/test/path") / "", path!("/test/path"));
        assert_eq!(path!("/test/path") / "to/file.txt", path!("/test/path/to/file.txt"));
    }

    #[test]
    fn test_div_preserves_original_path() {
        let original = path!("/test/path");

        let _owned_result = original.clone() / "subpath";
        let _borrowed_result = &original / "subpath";

        assert_eq!(original, path!("/test/path"));
    }
}
