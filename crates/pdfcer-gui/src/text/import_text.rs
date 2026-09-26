//! # `text::import_text` — every word the **Import text as pages** window says
//!
//!
//! > *"also the engine can export PDFs as text. we should have export/import
//! > for that."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/import_text.md`.

/// The window's title.
///
/// *"as pages"* in the title as well as on the command, because a window that
/// has been open for a minute is the only thing on screen and its title is the
/// last statement of what is about to happen.
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
///
/// **One margin, not four.** `PageTemplate` carries four and this window
/// offers one, which is a deliberate narrowing: an operator importing a text
/// file wants a readable page, and four spinners is a form to fill in rather
/// than a decision to make. The engine's four are still set — all to this
/// number — so nothing is lost that a later ask could not add.
#[must_use]
pub const fn margin_label() -> &'static str {
    "Margin"
}

/// Points, as a suffix.
#[must_use]
pub const fn points_suffix() -> &'static str {
    " pt"
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
///
/// The one warning in the window, and it earns its place: `place_text`
/// embeds nothing (R79), so a file containing a character none of the Standard
/// 14 can write is **refused entirely** rather than imported with gaps. That is
/// the correct behaviour and it is also the one an operator will not predict,
/// because every other program on his machine would have substituted a font.
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
///
/// The engine's `/BaseFont` name with its hyphen opened out — `Times-Roman`
/// becomes *Times Roman* — and nothing else. These are the names printed in
/// every PDF reader's font panel and written into the `/BaseFont` key, so an
/// operator checking what a page uses meets the same word pdfcer wrote.
/// `new_document::size_name` makes the same choice about sheet names.
///
/// ⚠ **The `_` arm answers Helvetica, and that is a real hazard rather than a
/// tidy default**: a face added to `dialogs::import_text`'s `FACES` without an
/// arm here would silently show *Helvetica* in the chooser beside the real
/// Helvetica. `dialogs::import_text::tests::every_offered_face_has_its_own_label`
/// is what notices — it asserts the labels are distinct rather than that they
/// are correct, which is the property a test can actually hold.
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
///
/// The file's name after an em dash, which is this program's title
/// convention — the window says what it does first, because that is what an
/// operator alt-tabbing back to it needs, and names the subject second.
#[must_use]
pub fn window_title_for(name: &str) -> String {
    format!("{} \u{2014} {name}", window_title())
}
