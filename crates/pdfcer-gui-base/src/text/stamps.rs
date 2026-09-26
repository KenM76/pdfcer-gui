//! # `text::stamps` — the words for custom stamp collections
//!
//! Two surfaces, one vocabulary: the **Save as stamp collection** window that
//! authors one, and the **Document Properties** section that discloses one
//! when the open file already is one. They share this catalog because they
//! share the concept, and a feature that calls the same thing a *category*
//! in one place and a *set* in the other has already lost the operator.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/stamps.md`.

use crate::stamps::Adjustment;

// ─────────────────────────────────────────────────────────────────────────
// The model's own defaults
// ─────────────────────────────────────────────────────────────────────────

/// The display name a page gets before the operator types one.
#[must_use]
pub fn default_stamp_name(page_number: usize) -> String {
    format!("Stamp {page_number}")
}

/// The filename stem used when the category cannot make one.
#[must_use]
pub const fn default_category_file_stem() -> &'static str {
    "Stamps"
}

// ─────────────────────────────────────────────────────────────────────────
// The command
// ─────────────────────────────────────────────────────────────────────────

/// The ribbon command's label.
#[must_use]
pub const fn command_label() -> &'static str {
    "Save as stamp collection"
}

/// The ribbon command's tooltip.
#[must_use]
pub const fn command_tooltip() -> &'static str {
    "Turn this document's pages into named stamps that appear in Acrobat's \
     stamp menu — one page per stamp, one file per category."
}

// ─────────────────────────────────────────────────────────────────────────
// The window
// ─────────────────────────────────────────────────────────────────────────

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Save as stamp collection"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "Each page becomes one stamp. Name them below, give the set a category, \
     and save it where Acrobat keeps stamps — they appear in its stamp menu \
     the next time it starts."
}

/// The line under [`intro`] when the open document already **is** a stamp
/// collection.
#[must_use]
pub const fn reopened_note() -> &'static str {
    "This document is already a stamp collection — the names below are the \
     ones it carries."
}

/// The category field's label.
#[must_use]
pub const fn category_label() -> &'static str {
    "Category"
}

/// The category field's hint.
#[must_use]
pub const fn category_hint() -> &'static str {
    "The heading this set appears under in Acrobat's stamp menu."
}

/// The heading over the per-page rows.
#[must_use]
pub fn stamps_heading(count: usize) -> String {
    if count == 1 {
        "1 stamp".to_owned()
    } else {
        format!("{count} stamps")
    }
}

/// The label on a row's page number.
#[must_use]
pub fn page_label(page_number: usize) -> String {
    format!("Page {page_number}")
}

/// The tooltip on a row's include checkbox.
#[must_use]
pub const fn include_hint() -> &'static str {
    "Uncheck to leave this page out of the collection. It stays in your \
     document either way — nothing here changes the file you have open."
}

/// The commit button.
#[must_use]
pub const fn save_button() -> &'static str {
    "Save collection…"
}

/// The cancel button.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The native save dialog's title.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Save stamp collection"
}

/// Why the Save button is greyed, when it is.
///
/// **R9**: greying is for *temporarily* unavailable, and both of these are
/// fixed by typing. Both are explained on hover, which is what this is for.
#[must_use]
pub const fn blocked_no_stamps() -> &'static str {
    "Every page is unchecked. A collection needs at least one stamp."
}

/// The second refusal — see [`blocked_no_stamps`].
#[must_use]
pub const fn blocked_no_category() -> &'static str {
    "Type a category name. Acrobat uses it as the heading for this set."
}

// ─────────────────────────────────────────────────────────────────────────
// The disclosures — R8b rule 4, off-canvas, before the write
// ─────────────────────────────────────────────────────────────────────────

/// The heading over the adjustment list.
#[must_use]
pub const fn adjustments_heading() -> &'static str {
    "What pdfcer changed"
}

/// The explanation under [`adjustments_heading`].
#[must_use]
pub const fn adjustments_intro() -> &'static str {
    "Each stamp is stored under a hidden identifier as well as the name you \
     typed. Your names are written exactly as you typed them; these are the \
     identifiers pdfcer had to adjust."
}

/// One adjustment, as a sentence.
///
/// Takes the page number rather than the index — this is read by a person
/// beside a row that says `Page 3`.
#[must_use]
pub fn adjustment(page_number: usize, adjustment: &Adjustment) -> String {
    match adjustment {
        Adjustment::CharactersRemoved(1) => {
            format!("Page {page_number}: one character left out of the identifier.")
        }
        Adjustment::CharactersRemoved(n) => {
            format!("Page {page_number}: {n} characters left out of the identifier.")
        }
        // The one adjustment with a consequence rather than a tidiness
        // reason, so it is the one that says WHY at length. A `#` marks a
        // stamp Acrobat expects to rewrite its own text; pdfcer writing one
        // would promise a recalculation that never happens.
        Adjustment::DynamicMarkerRemoved => format!(
            "Page {page_number}: the leading # was removed. Acrobat reads it as \
             a stamp that rewrites its own text when placed, which pdfcer does \
             not make."
        ),
        Adjustment::MadeUnique(name) => format!(
            "Page {page_number}: another stamp already had that name, so the \
             identifier became {name}. The name you typed is unchanged."
        ),
        Adjustment::Truncated => {
            format!("Page {page_number}: the identifier was shortened.")
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The outcome
// ─────────────────────────────────────────────────────────────────────────

/// The status line after a collection is written.
#[must_use]
pub fn saved(count: usize, path: &str) -> String {
    let stamps = if count == 1 {
        "1 stamp".to_owned()
    } else {
        format!("{count} stamps")
    };
    format!("Saved {stamps} to {path}. Restart Acrobat to see them in its stamp menu.")
}

/// The disclosure when the engine named fewer stamps than were planned.
#[must_use]
pub fn short_by(missing: usize) -> String {
    format!(
        "pdfcer wrote {missing} fewer stamps than planned. This is a fault in \
         pdfcer, not in your document — the collection is incomplete."
    )
}

/// A failed write, framed by stage, with the engine's own detail.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("The stamp collection could not be written. {detail}")
}

// ─────────────────────────────────────────────────────────────────────────
// Document Properties — reading a collection someone else wrote
// ─────────────────────────────────────────────────────────────────────────

/// The Document Properties section heading.
#[must_use]
pub const fn properties_heading() -> &'static str {
    "Stamp collection"
}

/// The category row's label in Document Properties.
#[must_use]
pub const fn properties_category_label() -> &'static str {
    "Category"
}

/// What a collection with no `/Info` `/Title` shows for its category.
#[must_use]
pub const fn properties_no_category() -> &'static str {
    "None — Acrobat would list these without a heading"
}

/// The stamp-count row's label.
#[must_use]
pub const fn properties_count_label() -> &'static str {
    "Stamps"
}

/// The stamp-count row's value.
#[must_use]
pub fn properties_count(total: usize, dynamic: usize) -> String {
    let stamps = if total == 1 {
        "1 stamp".to_owned()
    } else {
        format!("{total} stamps")
    };
    match dynamic {
        0 => stamps,
        d if d == total => format!("{stamps}, all of them dynamic"),
        d => format!("{stamps}, {d} of them dynamic"),
    }
}

/// The explanation of what a dynamic stamp is, shown when there is one.
#[must_use]
pub const fn properties_dynamic_note() -> &'static str {
    "A dynamic stamp rewrites its own text when Acrobat places it — a date, a \
     name, a time. Its page here shows the text it was designed with."
}

/// One stamp's row in Document Properties.
#[must_use]
pub fn properties_stamp(display: &str, internal: &str) -> String {
    if display.is_empty() {
        internal.to_owned()
    } else {
        display.to_owned()
    }
}

/// The page a stamp names, for its row's second column.
#[must_use]
pub fn properties_stamp_page(page_number: usize) -> String {
    format!("page {page_number}")
}

/// The second column when a stamp names no page of **this** document.
#[must_use]
pub const fn properties_stamp_no_page() -> &'static str {
    "names no page in this document"
}

/// The second column when the **page tree** could not be read.
#[must_use]
pub const fn properties_stamp_page_unreadable() -> &'static str {
    "page not known"
}

/// The one sentence that names why no stamp in this collection has a page.
#[must_use]
pub fn properties_page_tree_unreadable(why: &str) -> String {
    format!(
        "Which page each stamp names could not be worked out: this document's page tree could \
         not be read ({why}). The names above come from the file's name tree and are unaffected."
    )
}

// ─────────────────────────────────────────────────────────────────────────
// The gallery — the operator's own stamps, beside the standard ones
// ─────────────────────────────────────────────────────────────────────────

/// The heading over one collection's stamps in the Markup ▸ Stamp gallery.
#[must_use]
pub fn gallery_category(name: &str) -> String {
    name.to_owned()
}

/// The fallback heading for a collection whose file names no category.
#[must_use]
pub const fn gallery_category_unnamed() -> &'static str {
    "Stamps"
}

/// The pre-commit sentence for a **dynamic** stamp, shown in the dialog
/// beside the gallery once one is chosen.
#[must_use]
pub const fn gallery_dynamic_note() -> &'static str {
    "This stamp was made to fill in its own date and name when it is placed. \
     pdfcer places the artwork as it was drawn, so any date or name on it will \
     be the one its author typed."
}

// ─────────────────────────────────────────────────────────────────────────
// Placing one — the two DECLINES, then the four disclosures
// ─────────────────────────────────────────────────────────────────────────

/// **Why a placement never happened** — the reason half of
/// `pdfcer_gui::app::status::decline::Declined::CustomStampUnavailable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomStampUnavailable {
    /// The collection file would not open at all: moved, renamed, deleted, or
    /// no longer a PDF this engine will load.
    Unreadable,
    /// The collection opened, but no longer has the page the gallery offered —
    /// `pdfcer_core::edit::EditError::SourcePageOutOfRange`.
    ///
    /// This is the failure the `pdfcer_gui::dialogs::textannot` window's
    /// `library` field predicts in its own doc comment: the list is scanned
    /// when the window opens, and **Acrobat rewrites that folder while pdfcer
    /// is running**. A collection that lost a stamp between the scan and the
    /// drop lands here. It is rare and it is real, and the engine gave it its
    /// own variant precisely so that a shell could say *which* document the
    /// missing page belongs to rather than guess.
    PageGone,
}

/// The sentence for one [`CustomStampUnavailable`].
#[must_use]
pub const fn place_declined(why: CustomStampUnavailable) -> &'static str {
    match why {
        CustomStampUnavailable::Unreadable => place_source_unreadable(),
        CustomStampUnavailable::PageGone => place_source_page_gone(),
    }
}

/// The collection opened but no longer holds the page the gallery offered.
#[must_use]
pub const fn place_source_page_gone() -> &'static str {
    "That stamp could not be placed: it is no longer in the collection it \
     came from. The collection has been changed since this window was \
     opened — close it and open it again to see the stamps that are there \
     now."
}

/// The collection file would not open when the operator dropped the stamp.
#[must_use]
pub const fn place_source_unreadable() -> &'static str {
    "That stamp could not be placed: its collection file could not be \
     opened. It may have been moved, renamed or deleted since this window \
     was opened — close it and open it again to see the stamps that are \
     there now."
}

/// **The artwork was stretched to fit the rectangle that was dragged.**
#[must_use]
pub fn placed_distorted(scale_x: f64, scale_y: f64) -> String {
    // Guarded against a zero or negative factor, which the engine cannot
    // produce from a normalised `/Rect` and a non-empty box, but which would
    // print `inf%` if it ever did.
    let ratio = if scale_y > 0.0 && scale_x > 0.0 {
        scale_x / scale_y
    } else {
        1.0
    };
    let (percent, direction) = if ratio >= 1.0 {
        ((ratio - 1.0) * 100.0, "wider")
    } else {
        ((1.0 / ratio - 1.0) * 100.0, "taller")
    };
    format!(
        "Stamp placed, and stretched: the rectangle you drew is a different \
         shape from the stamp, so the artwork is {percent:.0}% {direction} than \
         it was drawn. Drawing a rectangle with the same proportions places it \
         undistorted."
    )
}

/// The dynamic stamp's words were frozen at design time — said again, after
/// the fact.
#[must_use]
pub const fn placed_dynamic() -> &'static str {
    "The date and name on this stamp are the ones its author typed when the \
     stamp was made, not today's."
}

/// Form-field widgets on the stamp's own page did not travel with the artwork.
#[must_use]
pub fn placed_widgets_ignored(count: usize) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!(
        "The stamp's page carried {count} form field{plural}, which are not \
         part of its artwork and were not placed."
    )
}

/// Annotations on the stamp's own page did not travel with the artwork.
#[must_use]
pub fn placed_annotations_ignored(count: usize) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!(
        "The stamp's page carried {count} annotation{plural}, which are not \
         part of its artwork and were not placed."
    )
}
