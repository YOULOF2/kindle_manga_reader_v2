mod manga_structs;
mod make_mobi;
mod common;

use self::common::get_json;
use self::manga_structs::{MangaChapter, MangaSeries, MangaVolume};

// Should return Result<MangaData::MangaSeries, Box<dyn std::error::Error>>
pub fn get_by_id(manga_id: &str) -> MangaSeries {
    let manga_details_data = get_json(format!("https://api.mangadex.org/manga/{}", manga_id));

    let manga_title = manga_details_data["data"]["attributes"]["title"]["en"].to_string().replace("\"", "");

    let manga_description =
        manga_details_data["data"]["attributes"]["description"]["en"].to_string().replace("\"", "");

    let manga_demographic =
        manga_details_data["data"]["attributes"]["publicationDemographic"].to_string().replace("\"", "");

    let manga_status = manga_details_data["data"]["attributes"]["status"].to_string().replace("\"", "");

    let manga_year = manga_details_data["data"]["attributes"]["year"].to_string().replace("\"", "");

    let manga_tags: Vec<String> = manga_details_data["data"]["attributes"]["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tag| tag["attributes"]["name"]["en"].to_string().replace("\"", ""))
        .collect();

    let mut manga_cover_url = String::new();
    for internal_relationship in manga_details_data["data"]["relationships"]
        .as_array()
        .iter()
    {
        for relationship_data in internal_relationship.iter() {
            if relationship_data["type"].eq("cover_art") {
                println!(
                    "{}",
                    format!(
                        "https://api.mangadex.org/cover/{}",
                        relationship_data["id"].to_string().replace("\"", "")
                    )
                );
                let manga_cover_data = get_json(format!(
                    "https://api.mangadex.org/cover/{}",
                    relationship_data["id"].to_string().replace("\"", "")
                ));

                manga_cover_url = format!(
                    "https://uploads.mangadex.org/covers/{}/{}",
                    manga_cover_data["data"]["id"].to_string(),
                    manga_cover_data["data"]["attributes"]["fileName"].to_string()
                )
                .replace("\"", "");
            }
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
            chapter_titles.push(chapter_title.parse().unwrap());
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
                        id: chapter_data["id"].to_string().replace("\"", ""),
                        title: chapter_data["chapter"].to_string().replace("\"", ""),
                        volume_title: volume_title.to_owned(),
                        manga_title: manga_title.to_owned().replace("\"", ""),
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
        for cover in all_manga_volume_covers["data"].as_array() {
            if cover[0]["attributes"]["volume"].eq(volume_title) {
                let hash_cover_filename = cover[0]["attributes"]["fileName"].to_string().replace("\"", "");

                volume_cover_url = format!(
                    "https://mangadex.org/covers/{}/{}",
                    manga_id, hash_cover_filename
                );
            }

            if volume_cover_url.eq(&String::new()) {
                volume_cover_url = manga_cover_url.to_owned();
            }
        }

        manga_volumes.push(MangaVolume {
            title: internal_volume_title.to_owned().replace("\"", ""),
            manga_title: manga_title.to_owned().replace("\"", ""),
            cover_url: volume_cover_url,
            chapters: chapters,
        });
    }

    MangaSeries {
        id: manga_id.to_string(),
        title: manga_title,
        description: manga_description,
        demographic: manga_demographic,
        status: manga_status,
        year: manga_year,
        tags: manga_tags,
        cover_url: manga_cover_url,
        volumes: manga_volumes,
    }
}