// get json from url and return serde_json::Value
use serde_json;
pub fn get_json(url: String) -> serde_json::Value {
    reqwest::blocking::get(url).unwrap().json::<serde_json::Value>().unwrap()
}

// Resize image to have A4 page size
use image::{imageops::FilterType, io::Reader as ImageReader};
use std::path::PathBuf;
pub fn resize_image_to_a4(image_path: &PathBuf) -> () {
    println!("Starting to resize image");
    let opened_image = ImageReader::open(image_path).unwrap().decode().unwrap();

    let hszize =
        (
            (opened_image.height() as f64) * (2480.0 / (opened_image.width() as f64))
        ).round() as u32;

    let resized_image = opened_image.resize(2480, hszize, FilterType::Lanczos3);

    resized_image.save(image_path).unwrap();
    println!("Finsihed resizeing image");
}
