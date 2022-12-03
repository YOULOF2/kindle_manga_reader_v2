// ─── Local Library ───────────────────────────────────────────────────────────

use kindle_manga_reader_v2::kindle::OnDeviceFile;
use kindle_manga_reader_v2::que::QueFile;
use kindle_manga_reader_v2::{ascrii_art, cart, kindle, manga, que};

// ─── Ui Stuff ────────────────────────────────────────────────────────────────

use cursive_aligned_view::Alignable;

use cursive_tabs::TabPanel;

use cursive::{
    align::HAlign,
    theme::{BorderStyle, Palette},
    traits::*,
    views::{
        Button, Dialog, DummyView, EditView, LayerPosition, LinearLayout, NamedView, Panel,
        ProgressBar, ResizedView, SelectView, StackView, TextView,
    },
    Cursive,
};

use ansi_term::Colour;

// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let mut siv = cursive::default();

    // ─── Theme ───────────────────────────────────────────────────────────

    siv.set_theme(cursive::theme::Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: Palette::default().with(|palette| {
            use cursive::theme::BaseColor::*;
            use cursive::theme::PaletteColor::*;

            palette[Background] = Black.dark();
            palette[View] = Black.dark();
            palette[Primary] = White.light();
            palette[TitlePrimary] = Red.light();
            palette[Secondary] = White.light();
            palette[Highlight] = Magenta.light();
        }),
    });

    // ─── Global Callbacks ────────────────────────────────────────────────

    siv.add_global_callback('q', |s| {
        cart::delete_cart();
        s.quit()
    });

    siv.set_global_callback('m', |siv: &mut Cursive| {
        siv.call_on_name("content_panel", |view: &mut TabPanel| {
            view.set_active_tab("Manga").unwrap();
        });
    });

    siv.set_global_callback('k', |siv: &mut Cursive| {
        siv.call_on_name("content_panel", |view: &mut TabPanel| {
            view.set_active_tab("Kindle").unwrap();
        });
    });

    siv.set_global_callback('l', |siv: &mut Cursive| {
        siv.call_on_name("content_panel", |view: &mut TabPanel| {
            view.set_active_tab("Logger").unwrap();
        });
    });

    siv.set_global_callback('a', |siv: &mut Cursive| {
        siv.pop_layer();
        display_get_manga_id(siv);
    });

    // ─── Set Fps ─────────────────────────────────────────────────────────

    siv.set_fps(1);

    // ─── Display Content ─────────────────────────────────────────────────

    display_get_manga_id(&mut siv);

    // ─── Run Cursive ─────────────────────────────────────────────────────

    siv.run();
}

fn display_content(siv: &mut Cursive, manga_id: &str) {
    let title = TextView::new(ascrii_art::MAIN_TITLE)
        .with_name("main_title")
        .scrollable()
        .align_center();

    // ─── Display Manga Data ──────────────────────────────────────────────────────
    let get_manga = manga::get_manga_by_id(manga_id);

    if get_manga.is_err() {
        return display_get_manga_id(siv);
    }

    let manga = get_manga.unwrap();

    siv.set_user_data(manga);

    // ─── Volume Select View ──────────────────────────────────────────────────────

    fn display_volume_select(siv: &mut Cursive) -> SelectView<String> {
        let mut volume_select = SelectView::<String>::new()
            .on_select(|siv: &mut Cursive, item: &String| {
                for volume in siv
                    .user_data::<manga::MangaSeries>()
                    .unwrap()
                    .clone()
                    .volumes
                    .iter()
                {
                    if format!("v{}", volume.title).eq(item) {
                        siv.call_on_name("chapter_select", |view: &mut SelectView<String>| {
                            view.clear();

                            let all_cart_items = cart::get_cart();

                            for chapter in volume.chapters.iter() {
                                if all_cart_items
                                    .contains(&format!("{}-{}", volume.title, chapter.title))
                                {
                                    view.add_item(
                                        Colour::Green
                                            .bold()
                                            .paint(format!("Chapter {} (IN CART)", chapter.title))
                                            .to_string(),
                                        format!("{}-{}", volume.title, chapter.title),
                                    )
                                } else {
                                    view.add_item(
                                        format!("Chapter {}", chapter.title),
                                        format!("{}-{}", volume.title, chapter.title),
                                    );
                                }
                            }
                        });
                    }
                }
            })
            .on_submit(move |siv: &mut Cursive, item: &String| {
                for volume in siv
                    .user_data::<manga::MangaSeries>()
                    .unwrap()
                    .clone()
                    .volumes
                    .iter()
                {
                    if format!("v{}", volume.title).eq(item) {
                        siv.call_on_name("volume_select", |view: &mut SelectView<String>| {
                            let all_cart_items = cart::get_cart();

                            let selected_id = view.selected_id().unwrap();

                            view.remove_item(selected_id);

                            if !all_cart_items.contains(item) {
                                cart::add_to_cart(item);

                                view.insert_item(
                                    selected_id,
                                    Colour::Green
                                        .bold()
                                        .paint(format!("Volume {} (IN CART)", volume.title))
                                        .to_string(),
                                    item.to_string(),
                                );
                            } else {
                                cart::remove_from_cart(item);

                                view.insert_item(
                                    selected_id,
                                    format!("Volume {}", volume.title),
                                    item.to_string(),
                                );
                            }
                        });
                        update_cart_view(siv);
                    }
                }
            });

        for volume in siv
            .user_data::<manga::MangaSeries>()
            .unwrap()
            .volumes
            .iter()
        {
            volume_select.add_item(
                format!("Volume {}", volume.title),
                format!("v{}", volume.title).to_string(),
            );
        }

        volume_select
    }

    let volumes_list = Dialog::around(
        display_volume_select(siv)
            .with_name("volume_select")
            .full_height()
            .full_width()
            .scrollable(),
    )
    .title("Volume Select")
    .full_width();

    fn display_chapter_select() -> SelectView<String> {
        SelectView::<String>::new().on_submit(|siv: &mut Cursive, item: &String| {
            for volume in siv
                .user_data::<manga::MangaSeries>()
                .cloned()
                .unwrap()
                .volumes
                .iter()
            {
                for chapter in volume.chapters.iter() {
                    if format!("{}-{}", volume.title, chapter.title).eq(item) {
                        siv.call_on_name("chapter_select", |view: &mut SelectView<String>| {
                            let all_cart_items = cart::get_cart();
                            let selected_id = view.selected_id().unwrap();
                            view.remove_item(selected_id);
                            if !all_cart_items.contains(item) {
                                cart::add_to_cart(item);
                                view.insert_item(
                                    selected_id,
                                    Colour::Green
                                        .bold()
                                        .paint(format!("Chapter {} (IN CART)", chapter.title))
                                        .to_string(),
                                    item.to_string(),
                                );
                            } else {
                                cart::remove_from_cart(item);
                                view.insert_item(
                                    selected_id,
                                    format!("Chapter {}", chapter.title),
                                    item.to_string(),
                                );
                            }
                        });
                    }
                }
            }
            update_cart_view(siv);
        })
    }

    let chapter_select = Dialog::around(
        display_chapter_select()
            .with_name("chapter_select")
            .full_height()
            .full_width()
            .scrollable(),
    )
    .title("Chapter Select")
    .full_width();

    // ─── Cart ────────────────────────────────────────────────────────────

    fn update_cart_view(siv: &mut Cursive) {
        siv.call_on_name("cart_view", |view: &mut SelectView<String>| {
            view.clear();

            let cart_items = cart::get_cart();

            for cart_item in cart_items {
                if cart_item.contains('-') {
                    let split_text: Vec<&str> = cart_item.split('-').collect();
                    let (volume_number, chapter_number) = (split_text[0], split_text[1]);
                    view.add_item(
                        format!("Volume {} > Chapter {}", volume_number, chapter_number),
                        cart_item,
                    );
                } else {
                    let volume_number = cart_item.replace('v', "");
                    view.add_item(format!("Complete Volume {}", volume_number), cart_item);
                }
            }
        });

        let cart_view = siv.find_name::<SelectView<String>>("cart_view").unwrap();
        siv.call_on_name("cart_view_dialog", |view: &mut Dialog| {
            view.set_title(format!("Cart: {} items", cart_view.len()))
        });
    }

    let cart_view = Dialog::around(
        LinearLayout::vertical()
            .child(
                SelectView::<String>::new()
                    .with_name("cart_view")
                    .full_height()
                    .scrollable(),
            )
            .child(Dialog::around(Button::new(
                "Checkout",
                move |siv: &mut Cursive| {
                    let cart = cart::get_cart();
                    let number_of_volumes =
                        { cart.iter().filter(|item| !item.contains('v')).count() };

                    let number_of_chapters =
                        { cart.iter().filter(|item| item.contains('v')).count() };

                    siv.add_layer(
                        Dialog::text(format!(
                            "Send {} volumes and {} chapters to kindle?",
                            Colour::Cyan.bold().paint(number_of_volumes.to_string()),
                            Colour::Cyan.bold().paint(number_of_chapters.to_string())
                        ))
                        .button("Send to Kindle", |siv: &mut Cursive| {
                            siv.pop_layer();

                            siv.call_on_name(
                                "main_horizontal_stackview",
                                |view: &mut StackView| {
                                    view.move_to_front(LayerPosition::FromBack(0));
                                },
                            );

                            let mut volumes_to_get = vec![];
                            let mut chapters_to_get = vec![];

                            let cart = cart::get_cart();
                            for volume in siv
                                .user_data::<manga::MangaSeries>()
                                .unwrap()
                                .clone()
                                .volumes
                            {
                                for item in cart.iter() {
                                    if item.contains('v') {
                                        if volume.title.eq(&item.replace('v', "")) {
                                            volumes_to_get.push(volume.clone());
                                        }
                                    } else {
                                        let split_text: Vec<&str> = item.split('-').collect();

                                        if volume.title.eq(split_text[0]) {
                                            for chapter in volume.chapters.iter() {
                                                if chapter.title.eq(split_text[1]) {
                                                    chapters_to_get.push(chapter.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            siv.call_on_name("manga_progress_bar", |view: &mut ProgressBar| {
                                let mut max_counter = 0;
                                for _ in 0..(chapters_to_get.len() + volumes_to_get.len()) {
                                    max_counter += 5;
                                }
                                max_counter += 1;
                                view.set_max(max_counter);
                                view.start(|counter| {
                                    let mut files_to_send: Vec<manga::Outputfile> = Vec::new();

                                    for volume in volumes_to_get {
                                        files_to_send.push(volume.to_mobi(&counter));
                                    }
                                    for chapter in chapters_to_get {
                                        files_to_send.push(chapter.to_mobi(&counter));
                                    }

                                    let mut kindle = kindle::Mount::new();
                                    kindle.scan();
                                    for output_file in files_to_send {
                                        if kindle.is_connected {
                                            kindle.send_to_kindle(&output_file).unwrap();
                                        } else {
                                            que::add(&output_file);
                                        }
                                    }
                                    cart::delete_cart();
                                    counter.tick(1);
                                });
                            });
                        })
                        .button("Cancel", |siv: &mut Cursive| {
                            siv.pop_layer();
                        }),
                    );
                },
            ))),
    )
    .title("Cart: 0 items")
    .with_name("cart_view_dialog");

    // ─────────────────────────────────────────────────────────────────────────────

    let main_horizontal_stackview = StackView::new()
        .fullscreen_layer(
            Panel::new(
                LinearLayout::vertical()
                    .child(TextView::new(ascrii_art::PROCESSING_TEXT).h_align(HAlign::Center))
                    .child(DummyView)
                    .child(DummyView)
                    .child(DummyView)
                    .child(DummyView)
                    .child(Panel::new(
                        ProgressBar::new()
                            .with_name("manga_progress_bar")
                            .full_width(),
                    ))
                    .child(DummyView)
                    .child(DummyView)
                    .child(DummyView)
                    .child(DummyView)
                    .child(
                        Button::new(
                            Colour::Purple
                                .bold()
                                .paint("Remove Progress Bar")
                                .to_string(),
                            |siv: &mut Cursive| {
                                siv.call_on_name(
                                    "main_horizontal_stackview",
                                    |view: &mut StackView| {
                                        view.move_to_front(LayerPosition::FromBack(0));
                                    },
                                );
                                display_chapter_select();
                                display_volume_select(siv);
                                update_cart_view(siv);
                                siv.call_on_name("manga_progress_bar", |view: &mut ProgressBar| {
                                    view.set_value(0)
                                });
                            },
                        )
                        .full_width(),
                    )
                    .align_center(),
            )
            .full_screen(),
        )
        .fullscreen_layer(
            LinearLayout::horizontal()
                .child(
                    LinearLayout::vertical()
                        .child(volumes_list)
                        .child(cart_view),
                )
                .child(chapter_select)
                .with_name("main_manga_layer"),
        )
        .with_name("main_horizontal_stackview");

    let manga_panel = main_horizontal_stackview.with_name("Manga");

    // ─── Manga Data ──────────────────────────────────────────────────────

    let manga_data = Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(format!(
                "{}    {}",
                Colour::Purple.paint("Title:"),
                Colour::Cyan
                    .bold()
                    .paint(siv.user_data::<manga::MangaSeries>().unwrap().clone().title)
            )))
            .child(DummyView)
            .child(TextView::new(format!(
                "{}\n{}",
                Colour::Purple.paint("Description:"),
                Colour::Cyan.bold().paint(
                    siv.user_data::<manga::MangaSeries>()
                        .unwrap()
                        .clone()
                        .description
                )
            )))
            .child(DummyView)
            .child(
                LinearLayout::horizontal()
                    .child(TextView::new(format!(
                        "{}    {}",
                        Colour::Purple.paint("Demographic:"),
                        Colour::Cyan.bold().paint(
                            siv.user_data::<manga::MangaSeries>()
                                .unwrap()
                                .clone()
                                .demographic
                        )
                    )))
                    .child(TextView::new(format!(
                        "{}    {}",
                        Colour::Purple.paint("Status:"),
                        Colour::Cyan.bold().paint(
                            siv.user_data::<manga::MangaSeries>()
                                .unwrap()
                                .clone()
                                .status
                        )
                    )))
                    .child(TextView::new(format!(
                        "{}    {}",
                        Colour::Purple.paint("Year Released:"),
                        Colour::Cyan
                            .bold()
                            .paint(siv.user_data::<manga::MangaSeries>().unwrap().clone().year)
                    )))
                    .child(TextView::new(format!(
                        "{}    {}",
                        Colour::Purple.paint("Tags:"),
                        Colour::Cyan.bold().paint(
                            siv.user_data::<manga::MangaSeries>()
                                .unwrap()
                                .clone()
                                .tags
                                .join(" - ")
                        )
                    )))
                    .align_center(),
            ),
    )
    .title("Manga Details")
    .fixed_height(15);

    // ─── Kindle Panel ────────────────────────────────────────────────────
    fn update_kindle_select_view_dialog(siv: &mut Cursive) {
        siv.call_on_name("kindle_select_view_dialog", |view: &mut Dialog| {
            view.set_content(display_kindle_select_view_dialog());
        });
    }

    fn display_kindle_select_view_dialog() -> ResizedView<NamedView<SelectView<OnDeviceFile>>> {
        {
            let mut kindle_select_view = SelectView::new();
            let mut kindle = kindle::Mount::new();
            kindle.scan();
            if kindle.is_connected {
                if !kindle.on_device_manga().unwrap().is_empty() {
                    for manga in kindle.on_device_manga().unwrap() {
                        let manga_title_short = {
                            if manga.manga_title.len() > 90 {
                                format!("{}...", &manga.manga_title[..90])
                            } else {
                                manga.manga_title[..].to_string()
                            }
                        };
                        if manga.r#type.eq("volume") {
                            kindle_select_view.add_item(
                                format!("{} Volume {}", manga_title_short, manga.volume_title),
                                manga,
                            );
                        } else {
                            kindle_select_view.add_item(
                                format!(
                                    "{} Volume {} Chapter {}",
                                    manga_title_short,
                                    manga.volume_title,
                                    manga.chapter_title.clone().unwrap()
                                ),
                                manga,
                            );
                        }
                    }
                    kindle_select_view.set_on_select(|siv: &mut Cursive, manga: &OnDeviceFile| {
                        siv.call_on_name("kindle_select_view_info_dialog", |view: &mut Dialog| {
                            view.set_content(
                                LinearLayout::vertical()
                                    .child(TextView::new(format!(
                                        "{} {}",
                                        Colour::Purple.paint("Title:"),
                                        Colour::Cyan.bold().paint(manga.manga_title.clone())
                                    )))
                                    .child(DummyView)
                                    .child(TextView::new(format!(
                                        "{} {}",
                                        Colour::Purple.paint("Volume"),
                                        Colour::Cyan.bold().paint(manga.volume_title.clone())
                                    )))
                                    .child(DummyView)
                                    .child({
                                        if manga.chapter_title.clone().is_some() {
                                            LinearLayout::vertical()
                                                .child(TextView::new(format!(
                                                    "{} {}",
                                                    Colour::Purple.paint("Chapter"),
                                                    Colour::Cyan.bold().paint(
                                                        manga.chapter_title.clone().unwrap()
                                                    )
                                                )))
                                                .child(DummyView)
                                        } else {
                                            LinearLayout::vertical().child(TextView::new(""))
                                        }
                                    })
                                    .child(TextView::new(format!(
                                        "{}\n{}",
                                        Colour::Purple.paint("Filename:"),
                                        Colour::Cyan.bold().paint(manga.file_name.clone())
                                    )))
                                    .child(DummyView)
                                    .child(TextView::new(format!(
                                        "{} {} {}",
                                        Colour::Purple.paint("Filesize:"),
                                        Colour::Cyan.paint(
                                            (manga.file_size as f32 / 1024.0).round().to_string()
                                        ),
                                        Colour::Purple.paint("kilobytes")
                                    ))),
                            );
                        });
                    });
                    kindle_select_view.set_on_submit(|siv: &mut Cursive, manga: &OnDeviceFile| {
                        let cloned_manga = manga.to_owned();
                        siv.add_layer(
                            Dialog::around(TextView::new(format!(
                                "Remove\n{}\nfrom your device?",
                                Colour::Cyan
                                    .bold()
                                    .italic()
                                    .paint(cloned_manga.file_name.clone())
                            )))
                            .button(
                                Colour::Red.paint("REMOVE").to_string(),
                                move |siv: &mut Cursive| {
                                    let mut kindle = kindle::Mount::new();
                                    kindle.scan();
                                    if kindle.is_connected {
                                        kindle.remove_manga(&cloned_manga);

                                        update_kindle_select_view_dialog(siv);

                                        siv.pop_layer();
                                    }
                                },
                            )
                            .button("Cancel", |siv: &mut Cursive| {
                                siv.pop_layer();
                            })
                            .title("Are You Sure?")
                            .title_position(HAlign::Left),
                        )
                    });
                } else {
                    kindle_select_view.add_item("No Manga on Device", OnDeviceFile::new());
                }
            } else {
                kindle_select_view.add_item(
                    Colour::Red
                        .bold()
                        .underline()
                        .paint("KINDLE IS NOT CONNECTED. CLICK TO REFRESH")
                        .to_string(),
                    OnDeviceFile::new(),
                );
                kindle_select_view.set_on_submit(|siv: &mut Cursive, _file: &OnDeviceFile| {
                    update_kindle_select_view_dialog(siv);
                });
            };

            // Return configured SelectView
            kindle_select_view
                .with_name("kindle_select_view")
                .full_width()
        }
    }

    fn update_que_files_select_view_dailog(siv: &mut Cursive) {
        siv.call_on_name("que_files_select_view_dialog", |view: &mut Dialog| {
            view.set_content(display_kindle_select_view_dialog());
        });
    }

    fn display_que_files_select_view_dailog() -> SelectView<QueFile> {
        let mut in_que_manga_select_view = SelectView::new()
            .on_select(|siv: &mut Cursive, que_file: &QueFile| {
                siv.call_on_name("kindle_select_view_info_dialog", |view: &mut Dialog| {
                    view.set_content(
                        LinearLayout::vertical()
                            .child(TextView::new(format!(
                                "{} {}",
                                Colour::Purple.paint("Title:"),
                                Colour::Cyan.bold().paint(que_file.manga_title.clone())
                            )))
                            .child(DummyView)
                            .child(TextView::new(format!(
                                "{} {}",
                                Colour::Purple.paint("Volume"),
                                Colour::Cyan.bold().paint(que_file.volume_title.clone())
                            )))
                            .child(DummyView)
                            .child({
                                if que_file.chapter_title.clone().is_some() {
                                    LinearLayout::vertical()
                                        .child(TextView::new(format!(
                                            "{} {}",
                                            Colour::Purple.paint("Chapter"),
                                            Colour::Cyan
                                                .bold()
                                                .paint(que_file.chapter_title.clone().unwrap())
                                        )))
                                        .child(DummyView)
                                } else {
                                    LinearLayout::vertical().child(TextView::new(""))
                                }
                            })
                            .child(TextView::new(format!(
                                "{}\n{}",
                                Colour::Purple.paint("Filename:"),
                                Colour::Cyan.bold().paint(que_file.file_name.clone())
                            )))
                            .child(DummyView)
                            .child(TextView::new(format!(
                                "{} {} {}",
                                Colour::Purple.paint("Filesize:"),
                                Colour::Cyan
                                    .paint((que_file.size as f32 / 1024.0).round().to_string()),
                                Colour::Purple.paint("kilobytes")
                            ))),
                    );
                });
            })
            .on_submit(|siv: &mut Cursive, que_file: &QueFile| {
                let mut kindle = kindle::Mount::new();
                kindle.scan();
                if kindle.is_connected {
                    que::send_item_to_kindle(que_file, &kindle).unwrap();
                } else {
                    siv.add_layer(Dialog::info("KIndle is not Connected").title("(╯°□°）╯︵ ┻━┻"));
                }
            });

        let que_files = que::data();
        for que_file in que_files {
            if !que_file.manga_title.is_empty() {
                let manga_title_short = {
                    if que_file.manga_title.len() > 90 {
                        format!("{}...", &que_file.manga_title[..90])
                    } else {
                        que_file.manga_title[..].to_string()
                    }
                };
                if que_file.r#type.eq("volume") {
                    in_que_manga_select_view.add_item(
                        format!("{} Volume {}", manga_title_short, que_file.volume_title),
                        que_file,
                    );
                } else {
                    in_que_manga_select_view.add_item(
                        format!(
                            "{} Volume {} Chapter {}",
                            manga_title_short,
                            que_file.volume_title,
                            que_file.chapter_title.clone().unwrap()
                        ),
                        que_file,
                    );
                }
            }
        }

        in_que_manga_select_view
    }

    let kindle_panel = LinearLayout::horizontal()
        .child(
            LinearLayout::vertical()
                .child(
                    Dialog::around(display_kindle_select_view_dialog())
                        .title("Manga on Device")
                        .with_name("kindle_select_view_dialog")
                        .full_height()
                        .scrollable(),
                )
                .child(
                    Dialog::around(display_que_files_select_view_dailog())
                        .title("Manga in Que")
                        .with_name("que_files_select_view_dialog")
                        .full_height()
                        .scrollable(),
                ),
        )
        .child(
            Dialog::new()
                .title("Info")
                .with_name("kindle_select_view_info_dialog")
                .full_width(),
        )
        .full_height()
        .with_name("Kindle");

    siv.set_global_callback('r', |siv: &mut Cursive| {
        update_kindle_select_view_dialog(siv);
        update_que_files_select_view_dailog(siv);
    });

    // ─────────────────────────────────────────────────────────────

    let content_panel = TabPanel::new()
        .with_tab(kindle_panel)
        .with_tab(manga_panel)
        .with_name("content_panel");

    let layout = Dialog::around(
        LinearLayout::vertical()
            .child(title)
            .child(Dialog::text(format!(
                "Keyboard Shortcuts: ({})uit, ({})escan for kindle, lookup ({})nother manga, ({})anga tab, ({})indle tab, ({})ogger tab", 
                Colour::Blue.bold().paint("q"),
                Colour::Blue.bold().paint("r"),
                Colour::Blue.bold().paint("a"),
                Colour::Blue.bold().paint("m"),
                Colour::Blue.bold().paint("k"),
                Colour::Blue.bold().paint("l"),
                )).align_center())
            .child(manga_data)
            .child(content_panel),
    )
    .full_width()
    .full_height();

    siv.add_fullscreen_layer(layout);
}

fn display_get_manga_id(siv: &mut Cursive) {
    siv.pop_layer();

    let title = TextView::new(ascrii_art::MAIN_TITLE).align_top_center();

    let manga_id_dialog = Dialog::around(
        LinearLayout::vertical().child(title).child(
            Dialog::around(
                LinearLayout::vertical()
                    .child(DummyView)
                    .child(TextView::new("(っ ͡• ‿‿ ͡•)っ🎔").align_bottom_center())
                    .child(DummyView)
                    .child(
                        EditView::new()
                            .with_name("manga_id")
                            .fixed_width(50)
                            .align_center(),
                    )
                    .child(DummyView)
                    .child(
                        Button::new("Submit", |siv| {
                            let manga_id = siv
                                .call_on_name("manga_id", |view: &mut EditView| view.get_content())
                                .unwrap();

                            siv.call_on_name("submit_button", |view: &mut Button| {
                                view.set_enabled(false)
                            });

                            display_content(siv, &manga_id);
                        })
                        .with_name("submit_button")
                        .align_center(),
                    )
                    .child(DummyView)
                    .align_center(),
            )
            .title("Enter MangaDex Id")
            .align_center()
            .full_height(),
        ),
    )
    .full_screen()
    .with_name("manga_id_dialog");

    siv.add_fullscreen_layer(manga_id_dialog);
}
