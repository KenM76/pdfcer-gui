//! # The phone layout's bars (`MOBILE_DESIGN.md` §3)
//!
//! Drawn in place of the ribbon, the document tabs and the docks when the
//! window is narrower than [`egui_shell::ribbon::phone::PHONE_MAX_WIDTH`].
//! [`PdfcerApp::ribbon_band`] builds one `Ribbon` for both layouts — the
//! conditions, rect reports, glyphs and custom-item renderer are the ribbon's
//! own — and hands it here when the window is a phone's; only the placement
//! differs.

use egui_shell::ribbon::phone::PhoneText;

use crate::app::PdfcerApp;
use crate::app::state::Status;
use crate::text::phone as t;

/// The app bar's title and the page line under it.
pub(super) struct Heading {
    title: String,
    subtitle: String,
}

impl PdfcerApp {
    /// The active document's [`Heading`], or the no-document title.
    pub(super) fn phone_heading(&self) -> Heading {
        let title = self
            .active_path()
            .map(|path| crate::text::doctabs::tab_label(path, false))
            .unwrap_or_else(|| t::NO_DOCUMENT.to_owned());
        let subtitle = match &self.status {
            Status::Open(doc) if !doc.pages.is_empty() => {
                t::page_subtitle(doc.view.page_index + 1, doc.pages.len())
            }
            _ => String::new(),
        };
        Heading { title, subtitle }
    }
}

/// The text the phone bars draw that is not a command's: the heading, the
/// overflow button's name, and the All tools cell.
pub(super) fn text(heading: &Heading) -> PhoneText<'_> {
    PhoneText {
        title: &heading.title,
        subtitle: &heading.subtitle,
        more: t::MORE,
        all_tools: t::ALL_TOOLS,
        all_tools_icon: Some(pdfcer_gui_base::icons::Icon::Grid.name()),
    }
}
