//! Path construction from native path expressions and formatted string literals.

/// Creates an owned [`Path`](crate::Path) from a path expression or formatted string literal.
///
/// `path!(expression)` passes the expression to [`Path::new`](crate::Path::new), which
/// copies its path data without accessing the filesystem. An owned expression is consumed;
/// a borrowed expression leaves its source available. A non-literal expression accepts
/// an optional trailing comma.
///
/// `path!("format string", arguments...)` uses [`format!`] syntax, including captured
/// variables. A string literal always uses this formatting form, even without arguments;
/// literal braces must be escaped as `{{` and `}}`. Expression operands must implement
/// `AsRef<std::path::Path>`; formatting arguments follow [`format!`] requirements.
///
/// # Panics
///
/// The formatting form panics if an argument's formatting implementation panics or
/// returns a formatting error.
///
/// # Examples
///
/// ```rust
/// use pathkit::{
///     Path,
///     path,
/// };
///
/// let source = String::from("config.json");
/// let direct = path!(&source);
/// assert_eq!(direct, Path::new("config.json"));
/// assert_eq!(source, "config.json");
///
/// let name = "config";
/// let formatted = path!("{name}.{}", "json");
/// assert_eq!(formatted, direct);
/// assert_eq!(path!("{{config}}.json"), Path::new("{config}.json"));
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
