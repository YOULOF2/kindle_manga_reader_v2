pub mod templates {
    pub const CONTAINER_XML: &[u8] = include_bytes!("../assets/templates/container.xml");

    pub const IBOOKS_XML: &[u8] = include_bytes!("../assets/templates/ibooks.xml");

    pub const COVER_HTML: &str = include_str!("../assets/templates/cover.html");

    pub const PAGE_HTML: &str = include_str!("../assets/templates/page.html");

    pub const TOC_NCX: &str = include_str!("../assets/templates/toc.ncx");

    pub mod v2 {
        pub const CONTENT_OPF: &str = include_str!("../assets/templates/v2/content.opf");

        pub const NAV_XHTML: &str = include_str!("../assets/templates/v2/nav.xhtml");
    }

    pub mod v3 {
        pub const CONTENT_OPF: &str = include_str!("../assets/templates/v2/content.opf");

        pub const NAV_XHTML: &str = include_str!("../assets/templates/v2/nav.xhtml");
    }
}

pub mod image_paths {
    pub const END_OF_CHAPTER: &str = "assets\\endofthischapter.png";

    pub const END_OF_VOLUME: &str = "assets\\endofthisvolume.png";

    pub const VOLUME_COVER_NOT_FOUND: &str = "assets\\volcovernotfound.png";
}

pub mod que {
    pub const QUE_DB: &str = "assets\\que\\que_db.json";

    pub const QUE_FOLDER: &str = "assets\\que";
}

pub const KINDLEGEN_PATH: &str = "assets\\kindlegen.exe";
