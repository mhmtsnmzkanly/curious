//! Lossless cold SQLite shards. Publish and verify the archive before removing raw bytes.
use crate::Result;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn is_archive(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "gz")
}
pub fn archive_path(raw: &Path) -> PathBuf {
    PathBuf::from(format!("{}.gz", raw.display()))
}

pub fn seal(raw: &Path) -> Result<PathBuf> {
    let parent = raw.parent().ok_or("archive needs a parent directory")?;
    let archive = archive_path(raw);
    let mut temporary = tempfile::Builder::new()
        .prefix(".curious-archive-")
        .tempfile_in(parent)?;
    let mut source = File::open(raw)?;
    let mut encoder =
        flate2::write::GzEncoder::new(temporary.as_file_mut(), flate2::Compression::fast());
    std::io::copy(&mut source, &mut encoder)?;
    encoder.finish()?.sync_all()?;
    // Verify byte-for-byte, including the gzip CRC/footer, before publishing.
    let mut source = File::open(raw)?;
    let mut decoded = flate2::read::GzDecoder::new(File::open(temporary.path())?);
    let mut a = [0u8; 65536];
    let mut b = [0u8; 65536];
    loop {
        let n = source.read(&mut a)?;
        if n == 0 {
            if decoded.read(&mut b[..1])? != 0 {
                return Err("archive length mismatch".into());
            }
            break;
        }
        decoded.read_exact(&mut b[..n])?;
        if a[..n] != b[..n] {
            return Err("archive verification failed".into());
        }
    }
    temporary.persist(&archive).map_err(|e| e.error)?;
    File::open(parent)?.sync_all()?;
    std::fs::remove_file(raw)?;
    File::open(parent)?.sync_all()?;
    Ok(archive)
}

fn unpack(archive: &Path, target: &Path) -> Result<()> {
    let mut source =
        flate2::read::GzDecoder::new(File::open(archive)?).take(2 * 1024 * 1024 * 1024 + 1);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    let bytes = std::io::copy(&mut source, &mut file)?;
    if bytes > 2 * 1024 * 1024 * 1024 {
        return Err("archive exceeds 2 GiB inspection limit".into());
    }
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

pub struct Reader {
    pub connection: rusqlite::Connection,
    _temporary: Option<tempfile::TempDir>,
}
impl std::ops::Deref for Reader {
    type Target = rusqlite::Connection;
    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
impl Reader {
    pub fn open(path: &Path) -> Result<Self> {
        let (target, temporary) = if is_archive(path) {
            let dir = tempfile::tempdir()?;
            let target = dir.path().join("history.sqlite");
            unpack(path, &target)?;
            (target, Some(dir))
        } else {
            (path.to_path_buf(), None)
        };
        let connection = rusqlite::Connection::open_with_flags(
            target,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        Ok(Self {
            connection,
            _temporary: temporary,
        })
    }
}

/// Call only while owning the run's writer lock. Incomplete outputs are never authoritative.
pub fn cleanup_pending(dir: &Path) -> Result<()> {
    for item in std::fs::read_dir(dir)? {
        let item = item?;
        if item.file_type()?.is_file()
            && item
                .file_name()
                .to_str()
                .is_some_and(|n| n.starts_with(".curious-archive-"))
        {
            std::fs::remove_file(item.path())?;
        }
    }
    Ok(())
}
