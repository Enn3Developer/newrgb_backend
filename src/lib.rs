use actix_web::rt;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::sync::{Mutex, TryLockError, TryLockResult};

use walkdir::{DirEntry, WalkDir};
use zip::write::FileOptions;
use zip::CompressionMethod;

static LOCK: Mutex<()> = Mutex::new(());

async fn zip_dir<T, P: AsRef<Path>>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: P,
    writer: T,
    method: CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
    P: AsRef<OsStr>,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(&prefix)).unwrap();
        if path.is_file() {
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub async fn zip_all<P: AsRef<Path> + AsRef<OsStr>>(dir_path: P) {
    let file = File::create("new.zip").unwrap();
    let walk_dir = WalkDir::new(&dir_path);
    let it = walk_dir.into_iter();
    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        dir_path,
        file,
        CompressionMethod::Zstd,
    )
    .await
    .unwrap();
}

pub async fn generate_zip() {
    let (_l, locked) = match LOCK.try_lock() {
        Ok(l) => (l, false),
        Err(TryLockError::WouldBlock) => (LOCK.lock().unwrap(), true),
        Err(TryLockError::Poisoned(_)) => (LOCK.lock().unwrap(), false),
    };
    if locked {
        return;
    }
    rt::spawn(async {
        zip_all("data").await;
        fs::rename("new.zip", "newrgb.zip").unwrap();
    })
    .await
    .expect("zip_all failed");
}
