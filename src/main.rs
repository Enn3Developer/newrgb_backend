use actix_files::NamedFile;
use actix_web::{get, App, HttpServer, Responder, Result};
use std::io;
use std::path::Path;

#[get("/")]
async fn index() -> Result<NamedFile> {
    if !Path::new("newrgb.zip").exists() {
        newrgb_backend::generate_zip().await
    }
    Ok(NamedFile::open("newrgb.zip")?)
}

#[get("/generate_zip")]
async fn generate_zip() -> impl Responder {
    newrgb_backend::generate_zip().await;
    "Done"
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let ip = if cfg!(debug_assertions) {
        "127.0.0.1"
    } else {
        "0.0.0.0"
    };
    HttpServer::new(|| App::new().service(index).service(generate_zip))
        .bind((ip, 6969))?
        .run()
        .await
}
