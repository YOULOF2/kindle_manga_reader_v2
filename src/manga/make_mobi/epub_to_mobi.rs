use std::process::{Command, Stdio};

pub fn convert(epub_path: &String, mobi_path: &String) {
    Command::new("src\\manga\\make_mobi\\kindlegen.exe")
        .arg(epub_path)
        .arg("-c0")
        .arg("-o")
        .arg(mobi_path)
        // .status().unwrap();
        .stdout(Stdio::null())
        .stderr(Stdio::null()).spawn().unwrap();
        
}
