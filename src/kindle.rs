use std::{
    error::Error,
    fmt,
    fs::{self, read_to_string},
    io::Write,
    path::Path,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_json;
use sysinfo::{DiskExt, System, SystemExt};

use crate::manga::Outputfile;
use crate::que;
// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct KindleNotFoundError;

impl Error for KindleNotFoundError {}

impl fmt::Display for KindleNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

// ─── Serde Structs ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct OnDeviceFile {
    pub r#type: String,
    pub manga_title: String,
    pub volume_title: String,
    pub chapter_title: Option<String>,
    pub file_name: String,
    pub file_size: u64,
}

impl OnDeviceFile {
    pub fn new() -> OnDeviceFile {
        OnDeviceFile {
            r#type: String::new(),
            manga_title: String::new(),
            volume_title: String::new(),
            chapter_title: Some(String::new()),
            file_name: String::new(),
            file_size: 0,
        }
    }
}

impl Default for OnDeviceFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
struct OnDeviceFiles {
    files: Vec<OnDeviceFile>,
}

impl OnDeviceFiles {
    pub fn new() -> OnDeviceFiles {
        OnDeviceFiles {
            files: vec![OnDeviceFile::new()],
        }
    }
}

// ─── Kindle Struct ───────────────────────────────────────────────────────────

pub struct Mount {
    sys: System,
    mount_point: PathBuf,
    pub is_connected: bool,
    pub available_space: u64,
}

impl Default for Mount {
    fn default() -> Self {
        Self::new()
    }
}

// Private
impl Mount {
    fn create_kmr2_file(&self) {
        let serialized = serde_json::to_string(&OnDeviceFiles::new()).unwrap();

        let kmr_data_file_path = Path::new(&self.mount_point).join("kmr2.json");

        let mut kmr_data_file = fs::File::create(&kmr_data_file_path).unwrap();

        kmr_data_file.write_all(serialized.as_bytes()).unwrap();
    }

    fn add_to_kmr2_file(&self, output_file: &Outputfile) {
        let kmr_data_file_path = Path::new(&self.mount_point).join("kmr2.json");

        let serialized = read_to_string(&kmr_data_file_path).unwrap();

        let mut data: OnDeviceFiles = serde_json::from_str(&serialized).unwrap();

        // Remove empty OnDeviceFile
        let empty_data_index = data.files.iter().position(|x| x.r#type.eq(""));
        if let Some(index) = empty_data_index {
            data.files.remove(index);
        }

        let file_data = OnDeviceFile {
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
            file_size: output_file.size,
        };

        // check if the data is already on the kindle, and if there are no duplicates, add the file data
        let duplicate_index = data
            .files
            .iter()
            .position(|x| x.file_name.eq(&file_data.file_name));
        if duplicate_index.is_none() {
            data.files.push(file_data);

            let serialized = serde_json::to_string(&data).unwrap();

            fs::write(kmr_data_file_path, serialized).unwrap();
        }
    }

    fn remove_from_kmr2_file(&self, file: &OnDeviceFile) {
        let kmr_data_file_path = Path::new(&self.mount_point).join("kmr2.json");

        let serialized = read_to_string(&kmr_data_file_path).unwrap();

        let mut data: OnDeviceFiles = serde_json::from_str(&serialized).unwrap();

        // Remove OnDevice file from data
        let item_index = data
            .files
            .iter()
            .position(|x| x.file_name.eq(&file.file_name));
        if let Some(index) = item_index {
            data.files.remove(index);
        }

        let serialized = serde_json::to_string(&data).unwrap();

        fs::write(kmr_data_file_path, serialized).unwrap();
    }

    fn does_kmr2_exist(&self) -> bool {
        Path::new(&self.mount_point).join("kmr2.json").exists()
    }
}

// Public
impl Mount {
    pub fn new() -> Mount {
        let mount_point = PathBuf::new();
        let available_space = 0;
        let kindle_found = false;
        let sys = System::new_all();

        Mount {
            sys,
            mount_point,
            available_space,
            is_connected: kindle_found,
        }
    }

    pub fn scan(&mut self) -> bool {
        self.sys.refresh_disks_list();

        for disk in self.sys.disks() {
            if disk.name() == "Kindle" {
                self.mount_point = disk.mount_point().to_owned();
                self.available_space = disk.available_space();
                self.is_connected = true;
                break;
            } else {
                self.mount_point = PathBuf::new();
                self.available_space = 0;
                self.is_connected = false;
            }
        }

        self.is_connected
    }

    // , output_file: Outputfile
    pub fn send_to_kindle(&self, output_file: &Outputfile) -> Result<(), KindleNotFoundError> {
        if !self.is_connected {
            return Err(KindleNotFoundError);
        }

        if !self.does_kmr2_exist() {
            self.create_kmr2_file();
        }

        // check if there is available space on the kindle
        if (self.available_space + 100) > output_file.size {
            self.add_to_kmr2_file(output_file);

            let documents_path = Path::new(&self.mount_point)
                .join("documents")
                .join(output_file.path.file_name().unwrap());

            fs::copy(&output_file.path, documents_path).unwrap();
        } else {
            que::add(output_file);
        }

        Ok(())
    }

    pub fn on_device_manga(&self) -> Result<Vec<OnDeviceFile>, KindleNotFoundError> {
        if !self.is_connected {
            return Err(KindleNotFoundError);
        }

        if !self.does_kmr2_exist() {
            self.create_kmr2_file();
        }

        let kmr_data_file_path = Path::new(&self.mount_point).join("kmr2.json");

        let serialized = read_to_string(&kmr_data_file_path).unwrap();

        let data: OnDeviceFiles = serde_json::from_str(&serialized).unwrap();

        Ok(data.files)
    }

    pub fn remove_manga(&self, file: &OnDeviceFile) {
        self.remove_from_kmr2_file(file);

        fs::remove_file(
            Path::new(&self.mount_point)
                .join("documents")
                .join(file.file_name.clone()),
        )
        .unwrap();
    }
}
