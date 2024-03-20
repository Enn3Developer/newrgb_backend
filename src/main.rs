use actix_files::NamedFile;
use actix_web::{get, web, App, Error, HttpServer, Responder, Result};
use std::fmt::format;
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

#[get("/background/{i}.png")]
async fn background(i: web::Path<u32>) -> Result<NamedFile> {
    Ok(NamedFile::open(&format!("background/{i}.png"))?)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let ip = if cfg!(debug_assertions) {
        "127.0.0.1"
    } else {
        "0.0.0.0"
    };
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(generate_zip)
            .service(background)
    })
    .bind((ip, 6969))?
    .run()
    .await
}
