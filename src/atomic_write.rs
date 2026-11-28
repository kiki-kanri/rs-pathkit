use std::{
    fs::{
        self,
        File,
        OpenOptions,
    },
    io::{
        self,
        Write,
    },
    path::Path as StdPath,
    process,
    sync::atomic::{
        AtomicU64,
        Ordering,
    },
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use crate::Path;

const TEMP_FILE_ATTEMPTS: u32 = 64;
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn write_atomic(
    dest: &Path,
    contents: &[u8],
    prepare: impl FnOnce(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let (temp_path, mut file) = create_temp_sibling(dest)?;
    let mut cleanup = RemoveOnDrop(Some(&temp_path));
    // Keep an existing regular file's permission bits instead of the temporary file's 0600; a
    // symlink is replaced rather than followed, so its target's mode is not consulted.
    #[cfg(unix)]
    if let Some(metadata) = crate::sync_fs_ops::none_if_not_found(fs::symlink_metadata(dest))?
        && metadata.is_file()
    {
        use std::os::unix::fs::PermissionsExt;

        file.set_permissions(fs::Permissions::from_mode(metadata.permissions().mode() & 0o777))?;
    }

    prepare(&temp_path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    drop(file);
    replace_durably(&temp_path, dest)?;
    cleanup.0 = None;
    Ok(())
}

pub(crate) fn persist(source: &StdPath, dest: &StdPath) -> io::Result<()> {
    // Renaming a symlink would publish the link, not the file that was just synced.
    if !fs::symlink_metadata(source)?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "persist source is not a regular file",
        ));
    }

    // Windows requires write access to flush file buffers.
    #[cfg(windows)]
    let file = OpenOptions::new().write(true).open(source)?;
    #[cfg(not(windows))]
    let file = File::open(source)?;

    file.sync_all()?;
    drop(file);
    replace_durably(source, dest)
}

/// Removes the temporary file on errors and unwinding unless disarmed.
struct RemoveOnDrop<'a>(Option<&'a Path>);

impl Drop for RemoveOnDrop<'_> {
    fn drop(&mut self) {
        if let Some(path) = self.0 {
            let _ = fs::remove_file(path);
        }
    }
}

fn create_temp_sibling(path: &Path) -> io::Result<(Path, File)> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    if path.file_name().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "atomic write path has no file name",
        ));
    }

    let parent = parent_or_current_dir(path);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.subsec_nanos());

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);

    for _ in 0..TEMP_FILE_ATTEMPTS {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        // Keep the name short and independent of the destination to stay within name length limits.
        let temp_path = Path::from(parent.join(format!(".pathkit-{:x}-{nanos:x}-{counter:x}.tmp", process::id())));
        match options.open(&temp_path) {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {},
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "failed to create a unique temporary file for atomic write",
    ))
}

fn parent_or_current_dir(path: &StdPath) -> &StdPath {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| StdPath::new("."))
}

fn replace_durably(source: &StdPath, dest: &StdPath) -> io::Result<()> {
    // Open the directory first so a missing read permission fails before anything is replaced.
    #[cfg(unix)]
    let dest_dir = File::open(parent_or_current_dir(dest))?;
    rename_retrying(source, dest)?;

    #[cfg(unix)]
    dest_dir.sync_all()?;
    Ok(())
}

#[cfg(not(windows))]
fn rename_retrying(source: &StdPath, dest: &StdPath) -> io::Result<()> {
    fs::rename(source, dest)
}

/// Retries while another process (typically antivirus or the search indexer) briefly holds the
/// destination or the freshly written source open, which surfaces as a sharing, lock, or
/// access-denied error. A permanent access-denied error is returned after the retry budget.
#[cfg(windows)]
fn rename_retrying(source: &StdPath, dest: &StdPath) -> io::Result<()> {
    use std::{
        thread,
        time::Duration,
    };

    use windows_sys::Win32::Foundation::{
        ERROR_ACCESS_DENIED,
        ERROR_LOCK_VIOLATION,
        ERROR_SHARING_VIOLATION,
    };

    const RETRY_BUDGET: Duration = Duration::from_secs(1);
    const MAX_DELAY: Duration = Duration::from_millis(250);

    let mut delay = Duration::from_millis(10);
    let mut waited = Duration::ZERO;
    loop {
        let Err(error) = fs::rename(source, dest) else {
            return Ok(());
        };

        let transient = matches!(
            error.raw_os_error().and_then(|code| u32::try_from(code).ok()),
            Some(ERROR_ACCESS_DENIED | ERROR_LOCK_VIOLATION | ERROR_SHARING_VIOLATION)
        );

        if !transient || waited >= RETRY_BUDGET {
            return Err(error);
        }

        thread::sleep(delay);
        waited += delay;
        delay = (delay * 2).min(MAX_DELAY);
    }
}
