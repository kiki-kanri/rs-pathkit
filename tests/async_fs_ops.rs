use anyhow::Result;
use pathkit::{
    AsyncFsOps,
    Path,
};
use tempfile::NamedTempFile;
use tokio::fs;

#[cfg(unix)]
#[tokio::test]
async fn chmod() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let temp_file = NamedTempFile::new()?;
    let file_path = Path::new(temp_file.path());
    file_path.chmod(0o744).await?;
    assert_eq!(
        fs::metadata(file_path.clone()).await?.permissions().mode() & 0o777,
        0o744
    );

    file_path.chmod(0o700).await?;
    assert_eq!(
        fs::metadata(file_path.clone()).await?.permissions().mode() & 0o777,
        0o700
    );

    return Ok(());
}

#[tokio::test]
async fn remove_file() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let file_path = Path::new(temp_file.path());

    assert!(file_path.exists().await?);
    file_path.remove_file().await?;
    assert!(!file_path.exists().await?);

    Ok(())
}
