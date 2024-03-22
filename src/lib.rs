use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::sync::Arc;

use async_fs::File;
use async_mutex::Mutex;
use async_walkdir::WalkDir;
use async_zip::base::write::ZipFileWriter;
use async_zip::error::Result;
use async_zip::{Compression, ZipEntryBuilder, ZipString};
use futures_lite::stream::StreamExt;
use futures_lite::AsyncReadExt;
use rocket::tokio;

static LOCK: Mutex<()> = Mutex::new(());

async fn zip_dir<P: AsRef<Path>>(
    wk: &mut WalkDir,
    prefix: P,
    writer: File,
    method: Compression,
) -> Result<()>
where
    P: AsRef<OsStr>,
{
    let zip = Arc::new(Mutex::new(ZipFileWriter::new(writer)));
    let mut handles = vec![];
    while let Some(entry) = wk.next().await {
        if let Ok(entry) = entry {
            let path = entry.path();
            let name = ZipString::from(
                path.strip_prefix(Path::new(&prefix))
                    .unwrap()
                    .to_str()
                    .unwrap(),
            );
            if path.is_file() {
                let zip = zip.clone();
                let handle = tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    let mut f = File::open(&path).await.unwrap();
                    f.read_to_end(&mut buffer).await.unwrap();
                    zip.lock()
                        .await
                        .write_entry_whole(
                            ZipEntryBuilder::new(name, method)
                                .unix_permissions(0o755)
                                .build(),
                            &buffer,
                        )
                        .await
                        .expect("can't write to zip");
                });
                handles.push(handle);
            }
        }
    }

    for handle in handles {
        handle
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
    }

    Arc::try_unwrap(zip)
        .map_err(|_| io::Error::from(io::ErrorKind::Other))?
        .into_inner()
        .close()
        .await?;
    Ok(())
}

pub async fn zip_all<P: AsRef<Path> + AsRef<OsStr>>(dir_path: P) -> io::Result<()> {
    let file = File::create("new.zip").await?;
    let mut walk_dir = WalkDir::new(&dir_path);
    zip_dir(&mut walk_dir, dir_path, file, Compression::Deflate)
        .await
        .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

    Ok(())
}

pub async fn generate_zip() -> io::Result<()> {
    let (_l, locked) = match LOCK.try_lock() {
        Some(l) => (l, false),
        None => (LOCK.lock().await, true),
    };
    if locked {
        return Ok(());
    }
    tokio::spawn(async {
        zip_all("data").await.expect("failed zip_all");
        async_fs::rename("new.zip", "newrgb.zip")
            .await
            .expect("failed rename");
    })
    .await
    .map_err(|_| io::Error::from(io::ErrorKind::Other))
}
