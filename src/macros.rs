/// Creates a [`Path`](crate::Path) from a path expression or a [`format!`] string.
///
/// A string literal always takes the [`format!`] form, even without arguments, so literal braces
/// must be escaped as `{{` and `}}`. Any other expression goes through
/// [`Path::new`](crate::Path::new).
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
        $crate::Path::from(format!($fmt $(, $($arg)*)?))
    };
    ($path:expr $(,)?) => {
        $crate::Path::new($path)
    };
}
