// get json from url and return serde_json::Value
use serde_json;
pub fn get_json(url: String) -> serde_json::Value {
    reqwest::blocking::get(url).unwrap().json::<serde_json::Value>().unwrap()
}


use std::path::PathBuf;
#[derive(Debug, Clone)]
pub struct Outputfile {
    ///content type
    pub content_type: String,

    /// manga name
    pub manga_title: String,

    /// volume title
    pub volume_title: String,

    /// chapter title, can be none for volumes
    pub chapter_title: Option<String>,

    /// file path
    pub path: PathBuf,

    /// file size
    pub size: u64,
}
