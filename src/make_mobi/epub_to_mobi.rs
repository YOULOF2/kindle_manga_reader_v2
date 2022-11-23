use std::process::{Command, ExitStatus};

pub fn convert(epub_path: &String, mobi_path: &String) -> ExitStatus {
    Command::new("src\\make_mobi\\kindlegen.exe").arg(epub_path).arg("-c0").arg("-o").arg(mobi_path).status().unwrap()
}