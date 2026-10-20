use std::ops::Div;

use crate::core::Path;

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

/// A path-joining operator with a string operand and an owned path result.
///
/// Borrows both operands without changing the base path. Uses [`Path::join`], including
/// its rooted-path and absolute-path replacement rules.
impl Div<&str> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &str) -> Self::Output {
        self.join(rhs)
    }
}

/// A path-joining operator with a string operand and an owned path result.
///
/// Borrows both operands without changing the base path. Uses [`Path::join`], including
/// its rooted-path and absolute-path replacement rules.
impl Div<&String> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &String) -> Self::Output {
        self.join(rhs.as_str())
    }
}

/// A path-joining operator with a string operand and an owned path result.
///
/// Borrows the base and consumes the string without changing the base path. Uses [`Path::join`], including
/// its rooted-path and absolute-path replacement rules.
impl Div<String> for &Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: String) -> Self::Output {
        self.join(rhs.as_str())
    }
}

/// A path-joining operator with a string operand and an owned path result.
///
/// Consumes the base and borrows the string. Uses [`Path::join`], including its rooted-path and
/// absolute-path replacement rules; this does not rename or otherwise modify the filesystem.
impl Div<&str> for Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &str) -> Self::Output {
        self.join(rhs)
    }
}

/// A path-joining operator with a string operand and an owned path result.
///
/// Consumes the base and borrows the string. Uses [`Path::join`], including its rooted-path and
/// absolute-path replacement rules; this does not rename or otherwise modify the filesystem.
impl Div<&String> for Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: &String) -> Self::Output {
        self.join(rhs.as_str())
    }
}

/// A path-joining operator with a string operand and an owned path result.
///
/// Consumes both operands. Uses [`Path::join`], including its rooted-path and
/// absolute-path replacement rules; this does not rename or otherwise modify the filesystem.
impl Div<String> for Path {
    type Output = Path;

    #[inline]
    fn div(self, rhs: String) -> Self::Output {
        self.join(rhs.as_str())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_div_accepts_path_variants() {
        let base = crate::path!("/test/path");
        let subpath = crate::path!("subpath");
        let expected = crate::path!("/test/path/subpath");

        assert_eq!(&base / &subpath, expected);
        assert_eq!(&base / subpath.clone(), expected);
        assert_eq!(base.clone() / &subpath, expected);
        assert_eq!(base / subpath, expected);
    }

    #[test]
    fn test_div_accepts_string_variants() {
        let base = crate::path!("/test/path");
        let subpath = String::from("subpath");
        let expected = crate::path!("/test/path/subpath");

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
            crate::path!("/test") / "path" / "to" / "file.txt",
            crate::path!("/test/path/to/file.txt")
        );

        assert_eq!(crate::path!("/test/path") / "", crate::path!("/test/path"));
        assert_eq!(
            crate::path!("/test/path") / "to/file.txt",
            crate::path!("/test/path/to/file.txt")
        );
    }

    #[test]
    fn test_div_preserves_original_path() {
        let original = crate::path!("/test/path");

        let owned_result = original.clone() / "subpath";
        let borrowed_result = &original / "subpath";

        assert_eq!(owned_result, borrowed_result);
        assert_eq!(original, crate::path!("/test/path"));
    }
}
