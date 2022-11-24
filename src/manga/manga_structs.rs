use crate::{manga::get_json, manga::make_mobi};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::thread;

// ─── Mangaseries ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct MangaSeries {
    pub id: String,
    pub title: String,
    pub description: String,
    pub demographic: String,
    pub status: String,
    pub year: String,
    pub tags: Vec<String>,
    pub cover_url: String,
    pub volumes: Vec<MangaVolume>,
}

// ─── Mangavolume ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct MangaVolume {
    pub title: String,
    pub manga_title: String,
    pub cover_url: String,
    pub chapters: Vec<MangaChapter>,
}

impl MangaVolume {
    fn download_images(&self) -> Vec<PathBuf> {
        let volume_images: Vec<Vec<PathBuf>> = self
            .chapters
            .iter()
            .map(|chapter| chapter.download_images())
            .collect();
        let mut volume_images = volume_images.concat();
        volume_images.insert(0, self.dowload_cover());
        volume_images
    }

    fn dowload_cover(&self) -> PathBuf {
        let cover_file_name = self.cover_url.split("/").last().unwrap();

        let file_path = PathBuf::from(format!("temp\\{}", cover_file_name));

        let mut file = File::create(&file_path).unwrap();

        reqwest::blocking::get(&self.cover_url)
            .unwrap()
            .copy_to(&mut file)
            .unwrap();

        resize_image_to_a4(&file_path);

        fs::canonicalize(file_path).unwrap()
    }

    pub fn to_mobi(&self) -> Outputfile {
        //! 1. Downloads the volume images
        //! 2. Adds the end of volume image
        //! 3. Converts it to mobi
        //!
        //! Returns `Outputfile` with `path` (mobi path) and `size` (mobi file size)

        let mut images = self.download_images();

        images.push(PathBuf::from("assets\\endofthisvolume.png"));

        let mobi_file = make_mobi::make_volume(
            &images,
            &self.manga_title,
            &self.title,
            &String::from("KindleMangaReader"),
        );

        let mobi_size = mobi_file.metadata().unwrap().len().to_owned();

        Outputfile {
            path: mobi_file,
            size: mobi_size,
        }
    }
}

// ─── Mangachapter ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct MangaChapter {
    pub id: String,
    pub title: String,
    pub volume_title: String,
    pub manga_title: String,
}

impl MangaChapter {
    fn download_images(&self) -> Vec<PathBuf> {
        let chapter_data = get_json(format!(
            "https://api.mangadex.org/at-home/server/{}",
            self.id
        ));

        let base_url = chapter_data["baseUrl"].as_str().unwrap();
        let chapter_hash = chapter_data["chapter"]["hash"].as_str().unwrap();

        // Convert chapter data to request urls
        let image_file_paths: Vec<PathBuf> = chapter_data["chapter"]["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|wraped_file_name| {
                let file_name = wraped_file_name.as_str().unwrap();

                let url = format!("{}/data/{}/{}", base_url, chapter_hash, file_name);

                let file_path = PathBuf::from(format!("temp\\{}", file_name));

                let mut file = File::create(&file_path).unwrap();

                reqwest::blocking::get(url)
                    .unwrap()
                    .copy_to(&mut file)
                    .unwrap();

                fs::canonicalize(file_path).unwrap()
            })
            .collect();

        let mut children = vec![];
        for image in image_file_paths.clone() {
            children.push(
                thread::spawn(move || {
                    resize_image_to_a4(&image)
                })
            )
        }

        for child in children {
            // Wait for the thread to finish. Returns a result.
            let _ = child.join();
        }

        image_file_paths
    }

    pub fn to_mobi(&self) -> Outputfile {
        //! 1. Downloads the chapter images
        //! 2. Adds the end of chapter image
        //! 3. Converts it to mobi
        //!
        //! Returns `Outputfile` with `path` (mobi path) and `size` (mobi file size)

        let mut images = self.download_images();

        images.push(PathBuf::from("assets\\endofthischapter.png"));

        let mobi_file = make_mobi::make_chapter(
            &images,
            &self.manga_title,
            &self.volume_title,
            &self.title,
            &String::from("KindleMangaReader"),
        );

        let mobi_size = mobi_file.metadata().unwrap().len().to_owned();

        Outputfile {
            path: mobi_file,
            size: mobi_size,
        }
    }
}

// ─── Outputfile ──────────────────────────────────────────────────────────────

pub struct Outputfile {
    /// file path
    pub path: PathBuf,

    /// file size
    pub size: u64,
}

// ─── Functions ───────────────────────────────────────────────────────────────

// Resize image to have A4 page size
use image::{imageops::FilterType, io::Reader as ImageReader};
pub fn resize_image_to_a4(image_path: &PathBuf) -> () {
    let opened_image = ImageReader::open(image_path).unwrap().decode().unwrap();

    let hszize =
        ((opened_image.height() as f64) * (2480.0 / (opened_image.width() as f64))).round() as u32;

    let resized_image = opened_image.resize(2480, hszize, FilterType::Lanczos3);

    resized_image.save(image_path).unwrap();
}

// ─── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    #[test]
    fn chapter_download_images() {
        let chapter = MangaChapter {
            id: String::from("eadf3eba-1023-4db5-86e4-158be4a1e78e"),
            title: String::new(),
            manga_title: String::new(),
            volume_title: String::new(),
        };
        let all_image_paths = chapter.download_images();

        for path in all_image_paths {
            assert!(Path::new(&path).exists());
            fs::remove_file(path).unwrap();
        }
    }
}
