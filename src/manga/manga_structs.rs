use crate::{
    make_mobi,
    utils::{get_json, resize_image_to_a4},
};
use std::fs;
use std::fs::File;
use std::path::PathBuf;

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

impl CanDownload for MangaVolume {
    fn download_images(&self) -> Vec<PathBuf> {
        let volume_images: Vec<Vec<PathBuf>> = self
            .chapters
            .iter()
            .map(|chapter| chapter.download_images())
            .collect();
        volume_images.concat()
    }
    fn to_mobi(&self) -> PathBuf {
        let mut images = self.download_images();
        images.push(PathBuf::from("assets\\endofthisvolume.png"));

        make_mobi::make_volume(
            &images,
            &self.manga_title,
            &self.title,
            &String::from("KindleMangaReader"),
        )
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

impl CanDownload for MangaChapter {
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
                println!("Writing image to {:?}", file_path);

                let mut file = File::create(&file_path).unwrap();

                reqwest::blocking::get(url)
                    .unwrap()
                    .copy_to(&mut file)
                    .unwrap();

                resize_image_to_a4(&file_path);

                fs::canonicalize(file_path).unwrap()
            })
            .collect();
        image_file_paths
    }

    fn to_mobi(&self) -> PathBuf {
        let mut images = self.download_images();
        images.push(PathBuf::from("assets\\endofthischapter.png"));

        make_mobi::make_chapter(
            &images,
            &self.manga_title,
            &self.volume_title,
            &self.title,
            &String::from("KindleMangaReader"),
        )
    }
}

// ─── Traits ──────────────────────────────────────────────────────────────────
pub trait CanDownload {
    fn download_images(&self) -> Vec<PathBuf>;
    fn to_mobi(&self) -> PathBuf;
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
