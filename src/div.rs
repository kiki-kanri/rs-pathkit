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

    // Test Div<&Path> for &Path
    #[test]
    fn test_div_ref_path_ref_path() {
        let path = path!("/test/path");
        let subpath = path!("subpath");
        let result = &path / &subpath;
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<&Path> for Path (owned)
    #[test]
    fn test_div_ref_path_owned_path() {
        let path = path!("/test/path");
        let subpath = path!("subpath");
        let result = &path / subpath;
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<Path> for &Path
    #[test]
    fn test_div_owned_path_ref_path() {
        let path = path!("/test/path");
        let subpath = path!("subpath");
        let result = path / &subpath;
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<Path> for Path
    #[test]
    fn test_div_owned_path_owned_path() {
        let path = path!("/test/path");
        let subpath = path!("subpath");
        let result = path / subpath;
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<&str> for &Path
    #[test]
    fn test_div_ref_path_ref_str() {
        let path = path!("/test/path");
        let subpath = "subpath";
        let result = &path / subpath;
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<String> for &Path
    #[test]
    fn test_div_ref_path_string() {
        let path = path!("/test/path");
        let subpath = String::from("subpath");
        let result = &path / subpath;
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<&String> for &Path
    #[test]
    fn test_div_ref_path_ref_string() {
        let path = path!("/test/path");
        let subpath = &String::from("subpath");
        let result = &path / subpath;
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<&str> for owned Path
    #[test]
    fn test_div_owned_path_ref_str() {
        let path = path!("/test/path");
        let result = path / "subpath";
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<String> for owned Path
    #[test]
    fn test_div_owned_path_string() {
        let path = path!("/test/path");
        let result = path / String::from("subpath");
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test Div<&String> for owned Path
    #[test]
    fn test_div_owned_path_ref_string() {
        let path = path!("/test/path");
        let subpath = &String::from("subpath");
        let result = path / subpath;
        assert_eq!(result, path!("/test/path/subpath"));
    }

    // Test multiple divisions
    #[test]
    fn test_multiple_div() {
        let path = path!("/test");
        let result = path / "path" / "to" / "file.txt";
        assert_eq!(result, path!("/test/path/to/file.txt"));
    }

    // Test division with empty string
    #[test]
    fn test_div_empty_string() {
        let path = path!("/test/path");
        let result = path / "";
        assert_eq!(result, path!("/test/path"));
    }

    // Test division with path separator in string
    #[test]
    fn test_div_with_separator() {
        let path = path!("/test/path");
        let result = path / "to/file.txt";
        assert_eq!(result, path!("/test/path/to/file.txt"));
    }

    // Test division preserves original path (owned)
    #[test]
    fn test_div_preserves_original_owned() {
        let original = path!("/test/path");
        let _result = original.clone() / "subpath";
        assert_eq!(original, path!("/test/path"));
    }

    // Test division preserves original path (reference)
    #[test]
    fn test_div_preserves_original_ref() {
        let original = path!("/test/path");
        let _result = &original / "subpath";
        assert_eq!(original, path!("/test/path"));
    }
}
