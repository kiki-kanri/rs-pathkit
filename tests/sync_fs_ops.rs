use std::fs;

use anyhow::Result;
use pathkit::{
    Path,
    SyncFsOps,
};
use tempfile::NamedTempFile;

#[cfg(unix)]
#[test]
fn chmod_sync() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let temp_file = NamedTempFile::new()?;
    let file_path = Path::new(temp_file.path());
    file_path.chmod_sync(0o744)?;
    assert_eq!(fs::metadata(file_path.clone())?.permissions().mode() & 0o777, 0o744);

    file_path.chmod_sync(0o700)?;
    assert_eq!(fs::metadata(file_path.clone())?.permissions().mode() & 0o777, 0o700);

    Ok(())
}

#[test]
fn remove_file_sync() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = Path::new(temp_file.path());

    assert!(file_path.exists_sync()?);
    file_path.remove_file_sync()?;
    assert!(!file_path.exists_sync()?);

    Ok(())
}
