use pathkit::path;

#[test]
fn paths_can_be_sorted() {
    let mut paths = vec![path!("c"), path!("a/b"), path!("a")];

    paths.sort();

    assert_eq!(paths, vec![path!("a"), path!("a/b"), path!("c")]);
}

#[test]
fn ordering_matches_std_path() {
    let paths = ["", ".", "a", "a/", "a/./b", "a/b", "a/../b", "a-b"];

    for left in paths {
        for right in paths {
            let left = path!(left);
            let right = path!(right);
            let expected = left.as_path().cmp(right.as_path());

            assert_eq!(left.cmp(&right), expected);
            assert_eq!(left.partial_cmp(&right), Some(expected));
            assert_eq!(left == right, expected.is_eq());
        }
    }
}

#[cfg(unix)]
#[test]
fn ordering_preserves_non_utf8_paths() {
    use std::{
        ffi::OsStr,
        os::unix::ffi::OsStrExt,
    };

    let left = path!(OsStr::from_bytes(b"a\xfe"));
    let right = path!(OsStr::from_bytes(b"a\xff"));

    assert!(left < right);
    assert_eq!(left.cmp(&right), left.as_path().cmp(right.as_path()));
    assert_eq!(left.partial_cmp(&right), Some(left.cmp(&right)));
}
