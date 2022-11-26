use kindle_manga_reader_v2::{get_manga_by_id, kindle};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let start = Instant::now();

    let main_thread_handle = thread::spawn(|| {
        let manga_series = get_manga_by_id("129c90ca-b997-4789-a748-e8765bc67a65");
        let first_volume = &manga_series.volumes[0].chapters[1];
        println!("{:#?}", first_volume);
        let chapter = first_volume.to_mobi();

        let mut kindle = kindle::Mount::new();
        loop {
            println!("-------");
            // println!("{:#?}", chapter);
            if kindle.scan() {
                println!("Kindle has been found!");
                kindle.send_to_kindle(chapter).unwrap();
                break;
            } else {
                println!("The kindle is not connected")
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    main_thread_handle.join().unwrap();

    let end = start.elapsed();
    println!("it took {} minutes", end.as_secs()/60);
}
