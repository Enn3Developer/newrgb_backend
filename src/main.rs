use actix_files::NamedFile;
use actix_web::{get, App, HttpServer, Result};
use std::io;

#[get("/")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("newrgb.zip")?)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let ip = if cfg!(debug_assertions) {
        "127.0.0.1"
    } else {
        "0.0.0.0"
    };
    HttpServer::new(|| App::new().service(index))
        .bind((ip, 6969))?
        .run()
        .await
}
