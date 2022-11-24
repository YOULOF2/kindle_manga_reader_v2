use sysinfo::{DiskExt, System, SystemExt};

use std::{error::Error, fmt, path::PathBuf};
pub struct KindleMount {
    pub point: PathBuf,
    pub available_space: u64,
}

#[derive(Debug)]
pub struct KindleNotFoundError;

impl Error for KindleNotFoundError {}

impl fmt::Display for KindleNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}



pub fn get_mount() -> Result<KindleMount, KindleNotFoundError> {
    let sys = System::new_all();

    let mut mount_point = PathBuf::new();
    let mut available_space = 0;
    let mut disk_found = false;

    for disk in sys.disks() {
        if disk.name() == "Kindle" {
            mount_point = disk.mount_point().to_owned();
            available_space = disk.available_space();
            disk_found = true;
        }
    }

    if !disk_found {
        return Err(KindleNotFoundError);
    }

    Ok(KindleMount {
        point: mount_point,
        available_space: available_space,
    })
}
