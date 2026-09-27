//! # `app::state` — one open document
//!
//! The record and its behaviour are `pdfcer_gui_base::opendoc`.

pub use pdfcer_gui_base::opendoc::*;

#[cfg(test)]
pub(crate) use pdfcer_gui_base::opendoc::fixtures::{
    CONTRADICTS_ITSELF, FOUR_PAGES, ORPHAN_WIDGET, RECOVERED_NO_LOSSES, RECOVERED_WITH_LOSSES,
    ROTATED_TEXT, SIGNED_TWO_PAGES, THREE_TEXT_FIELDS, open_fixture, open_local_fixture,
};
