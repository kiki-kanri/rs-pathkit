use std::ops::Div;

use super::core::Path;

impl Div<&Path> for &Path {
    type Output = Path;

    fn div(self, rhs: &Path) -> Self::Output {
        self.join(rhs)
    }
}

impl Div<&Path> for Path {
    type Output = Path;

    fn div(self, rhs: &Path) -> Self::Output {
        self.join(rhs)
    }
}

impl Div<Path> for &Path {
    type Output = Path;

    fn div(self, rhs: Path) -> Self::Output {
        self.join(&rhs)
    }
}

impl Div<Path> for Path {
    type Output = Path;

    fn div(self, rhs: Path) -> Self::Output {
        self.join(&rhs)
    }
}

impl<T: AsRef<str>> Div<T> for &Path {
    type Output = Path;

    fn div(self, rhs: T) -> Self::Output {
        self.join(rhs.as_ref())
    }
}

impl<T: AsRef<str>> Div<T> for Path {
    type Output = Path;

    fn div(self, rhs: T) -> Self::Output {
        self.join(rhs.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::Path;

    // Test Div<&Path> for &Path
    #[test]
    fn test_div_ref_path_ref_path() {
        let path = Path::new("/test/path");
        let subpath = Path::new("subpath");
        let result = &path / &subpath;
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<&Path> for Path (owned)
    #[test]
    fn test_div_ref_path_owned_path() {
        let path = Path::new("/test/path");
        let subpath = Path::new("subpath");
        let result = &path / subpath;
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<Path> for &Path
    #[test]
    fn test_div_owned_path_ref_path() {
        let path = Path::new("/test/path");
        let subpath = Path::new("subpath");
        let result = path / &subpath;
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<Path> for Path
    #[test]
    fn test_div_owned_path_owned_path() {
        let path = Path::new("/test/path");
        let subpath = Path::new("subpath");
        let result = path / subpath;
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<&str> for &Path
    #[test]
    fn test_div_ref_path_ref_str() {
        let path = Path::new("/test/path");
        let subpath = "subpath";
        let result = &path / subpath;
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<String> for &Path
    #[test]
    fn test_div_ref_path_string() {
        let path = Path::new("/test/path");
        let subpath = String::from("subpath");
        let result = &path / subpath;
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<&String> for &Path
    #[test]
    fn test_div_ref_path_ref_string() {
        let path = Path::new("/test/path");
        let subpath = &String::from("subpath");
        let result = &path / subpath;
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<&str> for owned Path
    #[test]
    fn test_div_owned_path_ref_str() {
        let path = Path::new("/test/path");
        let result = path / "subpath";
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<String> for owned Path
    #[test]
    fn test_div_owned_path_string() {
        let path = Path::new("/test/path");
        let result = path / String::from("subpath");
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test Div<&String> for owned Path
    #[test]
    fn test_div_owned_path_ref_string() {
        let path = Path::new("/test/path");
        let subpath = &String::from("subpath");
        let result = path / subpath;
        assert_eq!(result, Path::new("/test/path/subpath"));
    }

    // Test multiple divisions
    #[test]
    fn test_multiple_div() {
        let path = Path::new("/test");
        let result = path / "path" / "to" / "file.txt";
        assert_eq!(result, Path::new("/test/path/to/file.txt"));
    }

    // Test division with empty string
    #[test]
    fn test_div_empty_string() {
        let path = Path::new("/test/path");
        let result = path / "";
        assert_eq!(result, Path::new("/test/path"));
    }

    // Test division with path separator in string
    #[test]
    fn test_div_with_separator() {
        let path = Path::new("/test/path");
        let result = path / "to/file.txt";
        assert_eq!(result, Path::new("/test/path/to/file.txt"));
    }

    // Test division preserves original path (owned)
    #[test]
    fn test_div_preserves_original_owned() {
        let original = Path::new("/test/path");
        let _result = original.clone() / "subpath";
        assert_eq!(original, Path::new("/test/path"));
    }

    // Test division preserves original path (reference)
    #[test]
    fn test_div_preserves_original_ref() {
        let original = Path::new("/test/path");
        let _result = &original / "subpath";
        assert_eq!(original, Path::new("/test/path"));
    }
}
