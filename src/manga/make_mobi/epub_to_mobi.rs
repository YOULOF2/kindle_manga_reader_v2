use std::{process::Command, path::PathBuf};

use crate::assets::KINDLEGEN_PATH;

pub fn convert(epub_path: &PathBuf, mobi_path: &String) {
    Command::new(KINDLEGEN_PATH)
        .arg(epub_path)
        .arg("-c0")
        .arg("-o")
        .arg(mobi_path)
        .status().unwrap();
        // .stdout(Stdio::null())
        // .stderr(Stdio::null()).spawn().unwrap();
        
}
