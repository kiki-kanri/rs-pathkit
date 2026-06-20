//! Convenience macros for creating [`Path`](crate::Path) values.

/// Creates a [`Path`](crate::Path), with optional [`format!`]-style interpolation.
///
/// `path!` accepts either a direct path expression or a string literal using the
/// same formatting syntax as [`format!`].
///
/// # Examples
///
/// ```rust
/// use pathkit::{Path, path};
///
/// let direct = path!(String::from("/tmp/example"));
/// assert_eq!(direct, Path::new("/tmp/example"));
///
/// let name = "config";
/// let formatted = path!("/tmp/{name}.{}", "json");
/// assert_eq!(formatted, Path::new("/tmp/config.json"));
/// ```
#[macro_export]
macro_rules! path {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::Path::new(format!($fmt $(, $($arg)*)?))
    };
    ($path:expr $(,)?) => {
        $crate::Path::new($path)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn path_macro_creates_path_from_literal() {
        assert_eq!(path!("/tmp/example").to_str(), Some("/tmp/example"));
    }

    #[test]
    fn path_macro_supports_format_arguments() {
        let dir = "configs";
        let extension = "json";

        assert_eq!(
            path!("/tmp/{dir}/app.{extension}").to_str(),
            Some("/tmp/configs/app.json")
        );

        assert_eq!(
            path!("/tmp/{}/app.{}", dir, extension).to_str(),
            Some("/tmp/configs/app.json")
        );
    }

    #[test]
    fn path_macro_accepts_direct_path_expression() {
        assert_eq!(path!(String::from("/tmp/example")).to_str(), Some("/tmp/example"));
    }
}
