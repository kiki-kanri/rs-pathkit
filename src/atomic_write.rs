use std::{
    fs::{
        self,
        File,
        OpenOptions,
    },
    io,
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

/// Creates the temporary sibling for `dest`, plus the permission bits an existing regular file at
/// `dest` has on Unix, to be applied with [`InheritedMode::apply`].
pub(crate) fn stage(dest: &Path) -> io::Result<(File, TempGuard, Option<InheritedMode>)> {
    let (temp_path, file) = create_temp_sibling(dest)?;
    let guard = TempGuard {
        path: temp_path,
        armed: true,
    };

    // A symlink is replaced rather than followed, so its target's mode is not consulted.
    #[cfg(unix)]
    let inherited = match crate::sync_fs_ops::none_if_not_found(fs::symlink_metadata(dest))? {
        Some(metadata) if metadata.is_file() => {
            use std::os::unix::fs::PermissionsExt;

            Some(InheritedMode {
                created: file.metadata()?.permissions(),
                inherited: fs::Permissions::from_mode(metadata.permissions().mode() & 0o777),
            })
        },
        _ => None,
    };

    #[cfg(not(unix))]
    let inherited = None;

    Ok((file, guard, inherited))
}

/// Permission bits to carry over from the replaced file. Applied late, so a temporary file written
/// by path stays writable even when the destination is read-only.
#[derive(Debug)]
pub(crate) struct InheritedMode {
    created: fs::Permissions,
    inherited: fs::Permissions,
}

impl InheritedMode {
    /// Applies the inherited bits unless the caller already changed the temporary file's mode.
    pub(crate) fn apply(self, file: &File) -> io::Result<()> {
        if file.metadata()?.permissions() == self.created {
            file.set_permissions(self.inherited)?;
        }

        Ok(())
    }
}

pub(crate) fn publish_replacing(file: File, guard: TempGuard, dest: &StdPath) -> io::Result<()> {
    publish(file, guard, dest, rename_retrying)
}

pub(crate) fn publish_noclobber(file: File, guard: TempGuard, dest: &StdPath) -> io::Result<()> {
    publish(file, guard, dest, |temp, dest| {
        // A hard link fails with `AlreadyExists` instead of replacing.
        fs::hard_link(temp, dest)?;
        // The destination is already published, so a leftover temporary name is not an error.
        let _ = fs::remove_file(temp);
        Ok(())
    })
}

/// Syncs and closes `file`, then publishes the staged file at `dest` with `place`.
fn publish(
    file: File,
    mut guard: TempGuard,
    dest: &StdPath,
    place: impl FnOnce(&StdPath, &StdPath) -> io::Result<()>,
) -> io::Result<()> {
    // Close before any early return so the guard's cleanup never races an open handle.
    let synced = file.sync_all();
    drop(file);
    synced?;

    place_durably(&guard.path, dest, |temp, dest| {
        place(temp, dest)?;
        // Once published, the temporary name may be reused by another writer; never remove it again.
        guard.armed = false;
        Ok(())
    })
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
    place_durably(source, dest, rename_retrying)
}

/// Removes the temporary file on errors and unwinding unless disarmed.
#[derive(Debug)]
pub(crate) struct TempGuard {
    pub(crate) path: Path,
    armed: bool,
}

impl Drop for TempGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
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

/// Runs `place`, then syncs `dest`'s directory on Unix.
fn place_durably(
    source: &StdPath,
    dest: &StdPath,
    place: impl FnOnce(&StdPath, &StdPath) -> io::Result<()>,
) -> io::Result<()> {
    // Open the directory first so a missing read permission fails before anything is replaced.
    #[cfg(unix)]
    let dest_dir = File::open(parent_or_current_dir(dest))?;
    place(source, dest)?;

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
