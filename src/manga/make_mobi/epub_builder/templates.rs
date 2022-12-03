// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use once_cell::sync::Lazy;

use crate::assets::templates;

pub static IBOOKS: &[u8] = templates::IBOOKS_XML;
pub static CONTAINER: &[u8] = templates::CONTAINER_XML;

pub static TOC_NCX: Lazy<::mustache::Template> = Lazy::new(|| {
    ::mustache::compile_str(templates::TOC_NCX).expect("error compiling 'toc.ncx' template'")
});

pub mod v2 {
    use once_cell::sync::Lazy;

    use crate::assets::templates;

    pub static CONTENT_OPF: Lazy<::mustache::Template> = Lazy::new(|| {
        ::mustache::compile_str(templates::v2::CONTENT_OPF)
            .expect("error compiling 'content.opf' (for EPUB 2.0) template")
    });
    pub static NAV_XHTML: Lazy<::mustache::Template> = Lazy::new(|| {
        ::mustache::compile_str(templates::v2::NAV_XHTML)
            .expect("error compiling 'nav.xhtml' (for EPUB 2.0) template")
    });
}
pub mod v3 {
    use once_cell::sync::Lazy;

    use crate::assets::templates;

    pub static CONTENT_OPF: Lazy<::mustache::Template> = Lazy::new(|| {
        ::mustache::compile_str(templates::v3::CONTENT_OPF)
            .expect("error compiling 'content.opf' (for EPUB 3.0) template")
    });
    pub static NAV_XHTML: Lazy<::mustache::Template> = Lazy::new(|| {
        ::mustache::compile_str(templates::v3::NAV_XHTML)
            .expect("error compiling 'nav.xhtml' (for EPUB 3.0) template")
    });
}
