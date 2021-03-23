#![feature(proc_macro_hygiene, decl_macro)]

use chrono::{DateTime, Utc};
use comrak::ComrakOptions;
use rocket_contrib::{serve::StaticFiles, templates::Template};
use serde::Serialize;
use std::{
    fs::{self, DirEntry},
    time::SystemTime,
};

#[macro_use]
extern crate rocket;

#[derive(Serialize)]
struct BlogEntry {
    title: String,
    description: String,
    path: String,
}

#[derive(Serialize)]
struct MainPageContext {
    entries: Vec<BlogEntry>,
}

#[get("/entries/<entry>")]
fn get_entry(entry: String) -> Template {
    let markdown = fs::read_to_string(format!("entries/{}", entry)).unwrap();

    let doc = comrak::markdown_to_html(&markdown, &ComrakOptions::default());

    Template::render("entry", doc)
}

#[get("/")]
fn index() -> Template {
    let mut files: Vec<DirEntry> = fs::read_dir("entries")
        .expect("entries folder not found")
        .map(|entry| entry.unwrap())
        .collect();

    files.sort_by(|a, b| {
        b.metadata()
            .unwrap()
            .created()
            .unwrap()
            .cmp(&a.metadata().unwrap().created().unwrap())
    });

    let mut entries = Vec::new();

    for entry_file in files {
        let contents = fs::read_to_string(entry_file.path()).unwrap();

        let title = contents
            .split("\n")
            .next()
            .unwrap()
            .replace("#", "")
            .trim()
            .to_string();

        let path = format!("entries/{}", entry_file.file_name().to_string_lossy());

        entries.push(BlogEntry {
            title,
            description: entry_file.path().to_string_lossy().to_string(),
            path,
        });
    }

    let context = MainPageContext { entries };

    Template::render("main-page", context)
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, get_entry])
        .mount("/", StaticFiles::from("./static"))
        .attach(Template::fairing())
        .launch();
}
