#[macro_use]
extern crate rocket;

use rocket::fs::NamedFile;
use rocket::response::Redirect;
use rocket::{Build, Rocket};
use std::io;
use std::path::Path;

#[get("/")]
async fn index() -> Redirect {
    Redirect::to(uri!("/newrgb.zip"))
}

#[get("/newrgb.zip")]
async fn newrgb() -> io::Result<NamedFile> {
    if !Path::new("newrgb.zip").exists() {
        newrgb_backend::generate_zip().await?
    }
    NamedFile::open("newrgb.zip").await
}

#[get("/generate_zip")]
async fn generate_zip() -> io::Result<&'static str> {
    newrgb_backend::generate_zip().await?;
    Ok("Done")
}

#[get("/background/<i>")]
async fn background(i: usize) -> io::Result<NamedFile> {
    NamedFile::open(format!("background/{i}.png")).await
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![index, generate_zip, background, newrgb])
}
