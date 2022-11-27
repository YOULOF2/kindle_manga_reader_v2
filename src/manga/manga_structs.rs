use crate::manga::common::{get_json, Outputfile};
use crate::manga::make_mobi;
use crate::assets::image_paths;

use image::{imageops, DynamicImage};
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
    pub cover_url: VolumeCoverImage,
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

        volume_images.insert(0, self.download_cover());

        volume_images
    }

    fn download_cover(&self) -> PathBuf {
        fn internal_download_cover(cover_url: String) -> PathBuf {
            let cover_file_name = cover_url.split('/').last().unwrap();
            println!("{:?}", cover_file_name);

            let file_path = PathBuf::from(format!("temp\\{}", cover_file_name));

            let mut file = File::create(&file_path).unwrap();

            reqwest::blocking::get(&cover_url)
                .unwrap()
                .copy_to(&mut file)
                .unwrap();

            resize_image_to_a4(&file_path);

            fs::canonicalize(&file_path).unwrap()
        }

        fn add_overlay(
            mut base: DynamicImage,
            overlay_image: DynamicImage,
            output_path: PathBuf,
        ) -> PathBuf {
            imageops::overlay(&mut base, &overlay_image, 0, 0);
            base.save(&output_path).unwrap();
            output_path
        }

        match &self.cover_url {
            VolumeCoverImage::Found(image_url) => internal_download_cover(image_url.to_string()),
            VolumeCoverImage::NotFound(image_url) => {
                let image_path = internal_download_cover(image_url.to_string());
                add_overlay(
                    image::open(&image_path).unwrap(),
                    image::open(image_paths::VOLUME_COVER_NOT_FOUND).unwrap(),
                    image_path,
                )
            }
        }
    }

    pub fn to_mobi(&self) -> Outputfile {
        //! 1. Downloads the volume images
        //! 2. Adds the end of volume image
        //! 3. Converts it to mobi
        //!
        //! Returns `Outputfile` with `path` (mobi path) and `size` (mobi file size),
        //!  `manga_title` (manga title), `volume_title` (volume title) and `chapter_title` as None

        let mut images = self.download_images();

        images.push(fs::canonicalize(PathBuf::from(image_paths::END_OF_VOLUME)).unwrap());

        let mobi_file = make_mobi::make_volume(
            &images,
            &self.manga_title,
            &self.title,
            &String::from("KindleMangaReader"),
        );

        println!("{:?}", mobi_file);

        let mobi_size = mobi_file.metadata().unwrap().len().to_owned();

        Outputfile {
            content_type: String::from("volume"),
            manga_title: self.manga_title.clone(),
            volume_title: self.title.clone(),
            chapter_title: None,
            path: fs::canonicalize(mobi_file).unwrap(),
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

        let base_url = chapter_data["baseUrl"].as_str().unwrap().to_owned();
        let chapter_hash = chapter_data["chapter"]["hash"].as_str().unwrap().to_owned();

        // vector of all join handles
        let mut join_handles = vec![];

        for image in chapter_data["chapter"]["dataSaver"]
            .as_array()
            .unwrap()
            .clone()
        {
            // 👇 to stop the borrow checker from complaining
            let local_base_url = base_url.clone();
            let local_chapter_hash = chapter_hash.clone();

            // Spawns a new thread with a closure that, creates the image urls
            // and then downloads the images,
            // and then resizes them to a4
            let join_handle = thread::spawn(move || {
                let file_name = image.as_str().unwrap();

                let url = format!(
                    "{}/data-saver/{}/{}",
                    local_base_url, local_chapter_hash, file_name
                );

                let file_path = PathBuf::from(format!("temp\\{}", file_name));

                let mut file = File::create(&file_path).unwrap();

                reqwest::blocking::get(url)
                    .unwrap()
                    .copy_to(&mut file)
                    .unwrap();

                let cannon_file_path = fs::canonicalize(file_path).unwrap();

                resize_image_to_a4(&cannon_file_path);

                cannon_file_path
            });

            join_handles.push(join_handle);
            // image_file_paths.push((image, base_url, chapter_hash))
        }

        // join the threads and get the image file path as the output
        let image_file_paths: Vec<PathBuf> = join_handles
            .into_iter()
            .map(|handler| handler.join().unwrap())
            .collect();

        image_file_paths
    }

    pub fn to_mobi(&self) -> Outputfile {
        //! 1. Downloads the chapter images
        //! 2. Adds the end of chapter image
        //! 3. Converts it to mobi
        //!
        //! Returns `Outputfile` with `path` (mobi path) and `size` (mobi file size),
        //!  `manga_title` (manga title), `volume_title` (volume title) and `chapter_title` (chapter title)

        let mut images = self.download_images();

        images.push(fs::canonicalize(PathBuf::from(image_paths::END_OF_CHAPTER)).unwrap());

        let mobi_file = make_mobi::make_chapter(
            &images,
            &self.manga_title,
            &self.volume_title,
            &self.title,
            &String::from("KindleMangaReader"),
        );

        let mobi_size = mobi_file.metadata().unwrap().len().to_owned();

        Outputfile {
            content_type: String::from("chapter"),
            manga_title: self.manga_title.clone(),
            volume_title: self.volume_title.clone(),
            chapter_title: Some(self.title.clone()),
            path: fs::canonicalize(mobi_file).unwrap(),
            size: mobi_size,
        }
    }
}

// ─── Functions ───────────────────────────────────────────────────────────────

// Resize image to have A4 page size
use std::io::BufWriter;
use std::num::NonZeroU32;

use fast_image_resize as fr;
use image::codecs::png::PngEncoder;
use image::{io::Reader as ImageReader, ColorType, ImageEncoder};

use std::time::Instant;

pub fn resize_image_to_a4(image_path: &PathBuf) {
    let now = Instant::now();
    let opened_image = ImageReader::open(image_path).unwrap().decode().unwrap();

    let width = NonZeroU32::new(opened_image.width()).unwrap();
    let height = NonZeroU32::new(opened_image.height()).unwrap();

    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        opened_image.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .unwrap();

    // Multiple RGB channels of source image by alpha channel
    // (not required for the Nearest algorithm)
    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .unwrap();

    // height of the destination image
    let hszize =
        ((opened_image.height() as f64) * (2480.0 / (opened_image.width() as f64))).round() as u32;

    // Create container for data of destination image
    let dst_width = NonZeroU32::new(2480).unwrap();
    let dst_height = NonZeroU32::new(hszize).unwrap();
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    // Get mutable view of destination image data
    let mut dst_view = dst_image.view_mut();

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::CatmullRom));
    resizer.resize(&src_image.view(), &mut dst_view).unwrap();

    // Divide RGB channels of destination image by alpha
    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    // Write destination image as PNG-file
    let mut result_buf = BufWriter::new(Vec::new());
    PngEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            dst_width.get(),
            dst_height.get(),
            ColorType::Rgba8,
        )
        .unwrap();

    let elapsed = now.elapsed();
    println!("time to resize image is: {:.2?}", elapsed.as_secs());
}

// ─── Enums ───────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub enum VolumeCoverImage {
    Found(String),
    NotFound(String),
}
