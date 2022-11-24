// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use once_cell::sync::Lazy;

pub static IBOOKS: &[u8] = include_bytes!("./epub_templates/ibooks.xml");
pub static CONTAINER: &[u8] = include_bytes!("./epub_templates/container.xml");

pub static TOC_NCX: Lazy<::mustache::Template> = Lazy::new(|| {
    ::mustache::compile_str(include_str!("./epub_templates/toc.ncx"))
        .expect("error compiling 'toc.ncx' template'")
});

pub mod v2 {
    use once_cell::sync::Lazy;

    pub static CONTENT_OPF: Lazy<::mustache::Template> = Lazy::new(|| {
        ::mustache::compile_str(include_str!("./epub_templates/v2/content.opf"))
            .expect("error compiling 'content.opf' (for EPUB 2.0) template")
    });
    pub static NAV_XHTML: Lazy<::mustache::Template> = Lazy::new(|| {
        ::mustache::compile_str(include_str!("./epub_templates/v2/nav.xhtml"))
            .expect("error compiling 'nav.xhtml' (for EPUB 2.0) template")
    });
}
pub mod v3 {
    use once_cell::sync::Lazy;

    pub static CONTENT_OPF: Lazy<::mustache::Template> = Lazy::new(|| {
        ::mustache::compile_str(include_str!("./epub_templates/v3/content.opf"))
            .expect("error compiling 'content.opf' (for EPUB 3.0) template")
    });
    pub static NAV_XHTML: Lazy<::mustache::Template> = Lazy::new(|| {
        ::mustache::compile_str(include_str!("./epub_templates/v3/nav.xhtml"))
            .expect("error compiling 'nav.xhtml' (for EPUB 3.0) template")
    });
}
