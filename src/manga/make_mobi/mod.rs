use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

mod epub_builder;

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use handlebars::Handlebars;
use image::GenericImageView;
use serde_json::json;

mod epub_to_mobi;

// ─── Functions ───────────────────────────────────────────────────────────────

pub fn get_extension_from_filename(filename: &PathBuf) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

fn read_as_bytes(file: &PathBuf) -> Vec<u8> {
    fs::read(file).unwrap()
}

fn render_template(html_data: &str, data: serde_json::Value) -> String {
    let reg = Handlebars::new();
    reg.render_template(&html_data, &data).unwrap()
}

// ─── Make Epub ───────────────────────────────────────────────────────────────

// make chapter as epub
fn make_epub(
    images: &Vec<PathBuf>,
    epub_file_path: &String,
    author: &String,
    epub_title: &String,
) -> () {
    let css = r#"@charset "utf-8";a {text-decoration: none;}#toc ol {list-style-type: none;}img {display: block;width: 100%;object-fit: contain;}"#;

    let all_images = images.to_owned();
    // all_images.sort_by(|a, b| natord::compare(a.to_str().unwrap(), b.to_str().unwrap()));

    let mut epub = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();

    // Metadata
    epub.metadata("author", author).unwrap();
    epub.metadata("title", epub_title).unwrap();

    // stylesheet
    epub.stylesheet(css.as_bytes()).unwrap();

    // ─── Add Cover To Epub ───────────────────────────────────────────────

    // first item is cover image
    let cover_image = &all_images[0];

    // gets file extensions of cover image
    let file_extension = get_extension_from_filename(cover_image).unwrap();

    // load cover image and convert to bytes < &[u8] >
    let cover_as_bytes = &read_as_bytes(cover_image)[..];

    // add image to epub
    epub.add_cover_image(
        format!("image-0.{}", file_extension),
        cover_as_bytes,
        format!("image/{}", file_extension),
    )
    .unwrap();

    // opens and gets dimensions of cover image
    let (im_width, im_height) = image::open(cover_image).unwrap().dimensions();

    // render the fields in cover.html
    let binding = render_template(
        include_str!(".\\mobi_templates\\cover.html"),
        json!({"width": im_width, "height": im_height, "cover_path": format!("image-0.{}", file_extension)}),
    );

    // convert the rendered html string to bytes < &[u8] >
    let file_as_bytes = binding.as_bytes();

    // add cover.html to epub
    epub.add_content(
        EpubContent::new("cover.html", file_as_bytes)
            .title("Cover")
            .reftype(ReferenceType::Cover)
            .reftype(ReferenceType::Text),
    )
    .unwrap();

    // ─── Add Images To Epub ──────────────────────────────────────────────

    // all images except the first one
    let other_images = &all_images[1..];

    // for every image in other_images
    for (index, image) in other_images.iter().enumerate() {
        // get file extension of image
        let file_extension = get_extension_from_filename(image).unwrap();

        // convert image to bytes
        let image_as_bytes = &read_as_bytes(image)[..];

        // add image to epub
        epub.add_resource(
            format!("image-{}.{}", index + 1, file_extension),
            image_as_bytes,
            format!("image/{}", file_extension),
        )
        .unwrap();

        // get image width and height
        let (im_width, im_height) = image::open(image).unwrap().dimensions();

        // render template html to string
        let binding = render_template(
            include_str!(".\\mobi_templates\\page.html"),
            json!({"width": im_width, "height": im_height, "image": format!("image-{}.{}",index + 1, file_extension)}),
        );

        // convert html string to bytes
        let file_as_bytes = binding.as_bytes();

        // add chapter_{}.html to epub
        if index == 0 {
            epub.add_content(
                EpubContent::new(format!("page_{}.html", index + 1), file_as_bytes)
                    .title(format!("Page {}", index + 1))
                    .reftype(ReferenceType::Text),
            )
            .unwrap();
        } else {
            epub.add_content(
                EpubContent::new(format!("page_{}.html", index + 1), file_as_bytes)
                    .title(format!("Page {}", index + 1)),
            )
            .unwrap();
        }
    }

    let file = fs::File::create(&epub_file_path).unwrap();
    epub.generate(file).unwrap();
}

// ─── Public Methods ──────────────────────────────────────────────────────────

pub fn make_chapter(
    images: &Vec<PathBuf>,
    manga_title: &String,
    volume_title: &String,
    chapter_title: &String,
    author: &String,
) -> PathBuf {
    let ebook_title = format!(
        "{} volume {} chapter {}",
        manga_title, volume_title, chapter_title
    );

    let epub_file_path = format!("temp\\{}.epub", &ebook_title);

    make_epub(images, &epub_file_path, &author, &ebook_title);

    // Remove all images
    for file in images.iter() {
        if !file.file_name().unwrap().eq("endofthisvolume.png") && !file.file_name().unwrap().eq("endofthischapter.png") {
            fs::remove_file(file).unwrap();
        }
    }

    let mobi_file_name = format!(
        "{} volume {} chapter {}.mobi",
        manga_title, volume_title, chapter_title
    );

    epub_to_mobi::convert(&epub_file_path, &mobi_file_name);

    let mobi_file_path = PathBuf::from(format!("temp\\{}", &mobi_file_name));

    assert!(&mobi_file_path.is_file(), "mobi has not been created");

    // Remove epub
    fs::remove_file(epub_file_path).unwrap();

    mobi_file_path
}

pub fn make_volume(
    images: &Vec<PathBuf>,
    manga_title: &String,
    volume_title: &String,
    author: &String,
) -> PathBuf {
    let ebook_title = format!("{} volume {}", manga_title, volume_title);

    let epub_file_path = format!("temp\\{}.epub", &ebook_title);

    make_epub(images, &epub_file_path, &author, &ebook_title);

    // Remove all images
    for file in images.iter() {
        fs::remove_file(file).unwrap();
    }

    let mobi_file_name = format!("{}.mobi", ebook_title);

    epub_to_mobi::convert(&epub_file_path, &mobi_file_name);

    // Remove epub
    fs::remove_file(epub_file_path).unwrap();

    let mobi_file_path = PathBuf::from(format!("temp\\{}", mobi_file_name));

    assert!(&mobi_file_path.is_file(), "mobi has not been created");

    mobi_file_path
}
