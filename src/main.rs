use kindle_manga_reader_v2::{manga, CanDownload};

fn main() {
    let manga = manga::get_by_id("5a90308a-8b12-4a4d-9c6d-2487028fe319");
    println!("{:#?}", manga.volumes[0].to_mobi())
}