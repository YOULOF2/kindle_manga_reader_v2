use std::{path::PathBuf, process::Command};

use crate::assets::KINDLEGEN_PATH;

pub fn convert(epub_path: &PathBuf, mobi_path: &String) {
    let log_name = "log.log";
    let log = std::fs::File::create(log_name).expect("failed to open log");

    Command::new(KINDLEGEN_PATH)
        .arg(epub_path)
        .arg("-c0")
        .arg("-o")
        .stdout(log)
        .arg(mobi_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    // .stdout(Stdio::null())
    // .stderr(Stdio::null()).spawn().unwrap();
}
