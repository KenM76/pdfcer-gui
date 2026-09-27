//! # `text::import_text` — every word the **Import text as pages** window says
//!
//!
//! > *"also the engine can export PDFs as text. we should have export/import
//! > for that."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/import_text.md`.

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Import text as pages"
}

/// The standing sentence at the top of the window.
///
#[must_use]
pub const fn standing_note() -> &'static str {
    "This makes new pages and adds them to the document — it does not change any page you \
     already have. The words are set in one font at one size, so this is a readable, \
     searchable copy of the text and not a copy of its original layout."
}

/// The heading over the sheet controls.
#[must_use]
pub const fn sheet_heading() -> &'static str {
    "The new pages"
}

/// The heading over the type controls.
#[must_use]
pub const fn type_heading() -> &'static str {
    "The words"
}

/// The heading over the position radios.
#[must_use]
pub const fn where_heading() -> &'static str {
    "Where they go"
}

/// The sheet-size chooser's label.
#[must_use]
pub const fn size_label() -> &'static str {
    "Sheet size"
}

/// The margin field's label.
#[must_use]
pub const fn margin_label() -> &'static str {
    "Margin"
}

/// The typeface chooser's label.
#[must_use]
pub const fn face_label() -> &'static str {
    "Font"
}

/// The size spinner's label.
#[must_use]
pub const fn size_pt_label() -> &'static str {
    "Size"
}

/// The sentence under the font chooser.
#[must_use]
pub const fn face_note() -> &'static str {
    "These fonts are built into every PDF reader, so the file stays small and opens anywhere. \
     A character none of them can write will stop the import and be named, rather than being \
     silently dropped."
}

/// Where the pages land — before the page on screen.
#[must_use]
pub const fn before_current() -> &'static str {
    "Before this page"
}

/// After the page on screen. The default.
#[must_use]
pub const fn after_current() -> &'static str {
    "After this page"
}

/// Before every existing page.
#[must_use]
pub const fn at_start() -> &'static str {
    "At the start"
}

/// After every existing page.
#[must_use]
pub const fn at_end() -> &'static str {
    "At the end"
}

/// The commit button.
#[must_use]
pub const fn import() -> &'static str {
    "Import"
}

/// Cancel.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// **The label for one Standard-14 face**, as the chooser shows it.
#[must_use]
pub const fn face_name(face: pdfcer_core::fontdata::Std14) -> &'static str {
    match face {
        pdfcer_core::fontdata::Std14::HelveticaBold => "Helvetica Bold",
        pdfcer_core::fontdata::Std14::TimesRoman => "Times Roman",
        pdfcer_core::fontdata::Std14::TimesBold => "Times Bold",
        pdfcer_core::fontdata::Std14::Courier => "Courier",
        _ => "Helvetica",
    }
}

/// The window's title bar: the act, then the file it is about.
#[must_use]
pub fn window_title_for(name: &str) -> String {
    format!("{} \u{2014} {name}", window_title())
}
