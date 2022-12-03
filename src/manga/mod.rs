mod common;
mod make_mobi;
mod manga_structs;

pub use self::manga_structs::MangaSeries;
pub use common::Outputfile;

use self::common::get_json;
use self::manga_structs::{MangaChapter, MangaVolume, VolumeCoverImage};

use std::{error::Error, fmt};

#[derive(Debug)]
pub struct MangaNotFound;

impl Error for MangaNotFound {}

impl fmt::Display for MangaNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Manga is not found")
    }
}

/// Get the manga by id and return a `MangaSeries`
pub fn get_manga_by_id(manga_id: &str) -> Result<MangaSeries, MangaNotFound> {
    if manga_id.trim().is_empty() {
        return Err(MangaNotFound);
    }

    let manga_details_data = get_json(format!("https://api.mangadex.org/manga/{}", manga_id));

    if manga_details_data["result"]
        .to_string()
        .replace('"', "")
        .eq("error")
    {
        return Err(MangaNotFound);
    }

    let manga_title = manga_details_data["data"]["attributes"]["title"]["en"]
        .to_string()
        .replace('"', "");

    let manga_description = manga_details_data["data"]["attributes"]["description"]["en"]
        .to_string()
        .replace('"', "");

    let manga_demographic = manga_details_data["data"]["attributes"]["publicationDemographic"]
        .to_string()
        .replace('"', "");

    let manga_status = manga_details_data["data"]["attributes"]["status"]
        .to_string()
        .replace('"', "");

    let manga_year = manga_details_data["data"]["attributes"]["year"]
        .to_string()
        .replace('"', "");

    let manga_tags: Vec<String> = manga_details_data["data"]["attributes"]["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tag| {
            tag["attributes"]["name"]["en"]
                .to_string()
                .replace('\"', "")
        })
        .collect();

    let mut manga_cover_url = String::new();
    for relationship_data in manga_details_data["data"]["relationships"]
        .as_array()
        .unwrap()
    {
        if relationship_data["type"].eq("cover_art") {
            let manga_cover_data = get_json(format!(
                "https://api.mangadex.org/cover/{}",
                relationship_data["id"].to_string().replace('"', "")
            ));

            manga_cover_url = format!(
                "https://uploads.mangadex.org/covers/{}/{}",
                &manga_id, manga_cover_data["data"]["attributes"]["fileName"]
            )
            .replace('"', "");
        }
    }

    let all_manga_volume_covers = get_json(
        format!("https://api.mangadex.org/cover?limit=100&manga%5B%5D={}&order%5BcreatedAt%5D=asc&order%5BupdatedAt%5D=asc&order%5Bvolume%5D=asc", 
        &manga_id
    ));

    // Get aggregated manga data
    let aggregated_manga_data = get_json(format!(
        "https://api.mangadex.org/manga/{}/aggregate?translatedLanguage%5B%5D=en",
        manga_id
    ));

    let manga_volume = &aggregated_manga_data["volumes"];

    let mut manga_volumes: Vec<MangaVolume> = Vec::new();

    for (volume_title, volume_data) in manga_volume.as_object().unwrap() {
        let mut chapters: Vec<MangaChapter> = Vec::new();

        /* -------------------- Sort The Chapters By Their Number ------------------- */
        let mut chapter_titles: Vec<f32> = Vec::new();

        for (chapter_title, _) in volume_data["chapters"].as_object().unwrap() {
            chapter_titles.push(chapter_title.parse().unwrap_or(1.0));
        }

        chapter_titles.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let chapter_titles: Vec<String> = chapter_titles
            .iter()
            .map(|chapter| chapter.to_string())
            .collect();

        for chapter_title in chapter_titles {
            for (internal_chapter_title, chapter_data) in
                volume_data["chapters"].as_object().unwrap()
            {
                if internal_chapter_title.eq(&chapter_title) {
                    chapters.push(MangaChapter {
                        id: chapter_data["id"].to_string().replace('"', ""),
                        title: chapter_data["chapter"].to_string().replace('"', ""),
                        volume_title: volume_title.to_owned(),
                        manga_title: manga_title.to_owned().replace('"', ""),
                    })
                }
            }
        }
        /* ----------------------------------- end ---------------------------------- */

        let internal_volume_title = if volume_title.eq("none") {
            String::from("UnGrouped")
        } else {
            volume_title.to_owned()
        };

        let mut volume_cover_url = String::new();
        let mut volume_cover_url_type: VolumeCoverImage = VolumeCoverImage::Found(String::new());
        for cover in all_manga_volume_covers["data"].as_array().unwrap() {
            if cover[0]["attributes"]["volume"].eq(volume_title) {
                let hash_cover_filename = cover[0]["attributes"]["fileName"]
                    .to_string()
                    .replace('"', "");

                volume_cover_url = format!(
                    "https://mangadex.org/covers/{}/{}",
                    manga_id, hash_cover_filename
                );
                volume_cover_url_type = VolumeCoverImage::Found(volume_cover_url.to_owned());
            }

            if volume_cover_url.eq(&String::new()) {
                volume_cover_url = manga_cover_url.to_owned();
                volume_cover_url_type = VolumeCoverImage::NotFound(volume_cover_url.to_owned());
            }
        }
        manga_volumes.push(MangaVolume {
            title: internal_volume_title.to_owned().replace('"', ""),
            manga_title: manga_title.to_owned().replace('"', ""),
            cover_url: volume_cover_url_type,
            chapters,
        });
    }

    // ─── Sort The Volumes ────────────────────────────────────────────────

    let mut sorted_volumes: Vec<MangaVolume> = Vec::new();

    let mut volume_titles_as_floats: Vec<f32> = Vec::new();
    for volume in manga_volumes.iter() {
        match volume.title.parse::<f32>() {
            Ok(item) => volume_titles_as_floats.push(item),
            Err(_) => {
                sorted_volumes.push(volume.clone());
                continue;
            }
        }
    }

    volume_titles_as_floats.sort_by(|a, b| b.partial_cmp(a).unwrap());
    for volume_title_as_float in volume_titles_as_floats {
        for volume in manga_volumes.iter() {
            if volume.title == volume_title_as_float.to_string() {
                sorted_volumes.insert(0, volume.clone())
            }
        }
    }
    // ─────────────────────────────────────────────────────────────────────

    Ok(MangaSeries {
        id: manga_id.to_string(),
        title: manga_title,
        description: manga_description,
        demographic: manga_demographic,
        status: manga_status,
        year: manga_year,
        tags: manga_tags,
        cover_url: manga_cover_url,
        volumes: sorted_volumes,
    })
}
