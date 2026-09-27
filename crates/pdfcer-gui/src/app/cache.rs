//! # `app::cache` — the per-document caches
//!
//! The caches are `pdfcer_gui_base::doccache`; the methods that fill them are
//! `pdfcer_gui_base::opendoc::cache`.

pub use pdfcer_gui_base::doccache::{
    FontCache, FormRunCache, LinkCache, PageObjectCache, PageTextCache,
};
pub use pdfcer_gui_base::opendoc::cache::*;
