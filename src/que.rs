use std::{
    error::Error,
    fmt,
    fs::{self, read_to_string},
    io::Write,
    path::Path,
    path::PathBuf,
    vec,
};

use serde::{Deserialize, Serialize};
use serde_json;

use crate::assets::que::{self, QUE_FOLDER};
use crate::kindle::Mount;
use crate::manga::Outputfile;

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct KindleNoSpaceAvailableError;

impl Error for KindleNoSpaceAvailableError {}

impl fmt::Display for KindleNoSpaceAvailableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is no space left on the kindle")
    }
}

// ─── Serde Structs ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct QueFile {
    pub r#type: String,
    pub manga_title: String,
    pub volume_title: String,
    pub chapter_title: Option<String>,
    pub file_name: String,
    pub size: u64,
}

impl QueFile {
    pub fn new() -> QueFile {
        QueFile {
            r#type: String::new(),
            manga_title: String::new(),
            volume_title: String::new(),
            chapter_title: Some(String::new()),
            file_name: String::new(),
            size: 0,
        }
    }

    pub fn to_output_file(&self) -> Outputfile {
        Outputfile {
            content_type: self.r#type.to_owned(),
            manga_title: self.manga_title.to_owned(),
            volume_title: self.volume_title.to_owned(),
            chapter_title: self.chapter_title.to_owned(),
            path: PathBuf::from(format!("{}\\{}", QUE_FOLDER, self.file_name)),
            size: self.size,
        }
    }
}

impl Default for QueFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
struct QueFiles {
    files: Vec<QueFile>,
}

impl QueFiles {
    pub fn new() -> QueFiles {
        QueFiles {
            files: vec![QueFile::new()],
        }
    }
}

// ─── Functions ───────────────────────────────────────────────────────────────

fn init_que_db() {
    if !Path::new(que::QUE_DB).exists() {
        let serialized = serde_json::to_string(&QueFiles::new()).unwrap();

        let mut que_db_data_file = fs::File::create(que::QUE_DB).unwrap();

        que_db_data_file.write_all(serialized.as_bytes()).unwrap();
    }
}

pub fn data() -> Vec<QueFile> {
    init_que_db();

    let serialized = read_to_string(que::QUE_DB).unwrap();

    let data: QueFiles = serde_json::from_str(&serialized).unwrap();

    data.files
}

pub fn add(output_file: &Outputfile) {
    init_que_db();

    // ─── Edit Que File ───────────────────────────────────────────────────

    let serialized = read_to_string(que::QUE_DB).unwrap();

    let mut data: QueFiles = serde_json::from_str(&serialized).unwrap();

    // Remove empty OnDeviceFile
    let empty_data_index = data.files.iter().position(|x| x.r#type.eq(""));
    if let Some(index) = empty_data_index {
        data.files.remove(index);
    }

    let file_data = QueFile {
        r#type: output_file.content_type.to_owned(),
        manga_title: output_file.manga_title.to_owned(),
        volume_title: output_file.volume_title.to_owned(),
        chapter_title: output_file.chapter_title.to_owned(),
        file_name: output_file
            .path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        size: output_file.size,
    };

    // check if the data is already in the que, and if there are no duplicates, add the file data
    let duplicate_index = data
        .files
        .iter()
        .position(|x| x.file_name.eq(&file_data.file_name));
    if duplicate_index.is_none() {
        data.files.push(file_data);

        let serialized = serde_json::to_string(&data).unwrap();

        fs::write(que::QUE_DB, serialized).unwrap();
    }

    // ─────────────────────────────────────────────────────────────────────
    fs::copy(
        &output_file.path,
        format!(
            "{}\\{}",
            que::QUE_FOLDER,
            &output_file.path.file_name().unwrap().to_str().unwrap()
        ),
    )
    .unwrap();
}

pub fn remove(que_file: &QueFile) {
    let serialized = read_to_string(que::QUE_DB).unwrap();

    let mut data: QueFiles = serde_json::from_str(&serialized).unwrap();

    let que_file_index = data
        .files
        .iter()
        .position(|x| x.file_name == que_file.file_name)
        .unwrap();

    data.files.remove(que_file_index);

    let serialized = serde_json::to_string(&data).unwrap();

    fs::write(que::QUE_DB, serialized).unwrap();

    fs::remove_file(format!("{}\\{}", QUE_FOLDER, que_file.file_name)).unwrap();
}

pub fn send_item_to_kindle(
    que_file: &QueFile,
    kindle: &Mount,
) -> Result<(), KindleNoSpaceAvailableError> {
    if que_file.size > kindle.available_space {
        return Err(KindleNoSpaceAvailableError);
    }

    kindle.send_to_kindle(&que_file.to_output_file()).unwrap();

    remove(que_file);

    Ok(())
}
