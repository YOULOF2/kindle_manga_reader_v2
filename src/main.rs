use kindle_manga_reader_v2::manga;

use kindle_manga_reader_v2::kindle;

fn main() {
    let manga_series = manga::get_by_id("129c90ca-b997-4789-a748-e8765bc67a65");
    let chapter = manga_series.volumes.last().unwrap().chapters.last().unwrap().to_mobi();
    let chapter_size = chapter.size;

    println!("{:#?}", kindle::get_mount().unwrap().available_space > chapter_size);
}
