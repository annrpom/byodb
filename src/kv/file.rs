use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

/// Atomically writes `data` to `path`.
//
// Achieves both reader-writer and power-loss atomicity by:
//
// 1. Writing to a sibling temp file in the same directory.
// 2. `fsync`ing the temp file to flush data to disk.
// 3. Renaming onto `path` — atomic on POSIX filesystems (and NTFS).
// 4. `fsync`ing the parent directory to make the rename durable.
//
// TODO: remove this once page writes are implemented.
#[allow(dead_code)]
pub(crate) fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let tmp_path = dir.join(tmp_name(path));

    {
        let mut tmp = File::create(&tmp_path)?;
        tmp.write_all(data)?;
        tmp.sync_all()?;
    }

    fs::rename(&tmp_path, path)?;
    fsync_dir(dir)?;

    Ok(())
}

fn tmp_name(path: &Path) -> String {
    format!(
        ".tmp.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("data")
    )
}

#[cfg(unix)]
fn fsync_dir(dir: &Path) -> io::Result<()> {
    File::open(dir)?.sync_all()
}

#[cfg(not(unix))]
fn fsync_dir(_dir: &Path) -> io::Result<()> {
    // NTFS journals the rename; no explicit dir fsync needed.
    Ok(())
}
