//! # `text::print::impose` — several pages per sheet, and booklets
//!
//! The words for `pdfcer_gui::dialogs::print::impose`. Acrobat's names are
//! kept (*Multiple*, *Booklet*, *Pages per sheet*, *Page order*, *Binding*).

use pdfcer_print::imposition::{Binding, BookletSubset, PageOrder};

/// The sizing choice for several pages per sheet.
#[must_use]
pub const fn sizing_mode_multiple() -> &'static str {
    "Multiple"
}

/// Hover on Multiple.
#[must_use]
pub const fn sizing_mode_multiple_tooltip() -> &'static str {
    "Print several pages on each sheet."
}

/// The sizing choice for a booklet.
#[must_use]
pub const fn sizing_mode_booklet() -> &'static str {
    "Booklet"
}

/// Hover on Booklet.
#[must_use]
pub const fn sizing_mode_booklet_tooltip() -> &'static str {
    "Print two pages on each side of each sheet, ordered so the stack folds \
     in half into a booklet that reads in order."
}

/// Label on the pages-per-sheet choice.
#[must_use]
pub const fn pages_per_sheet() -> &'static str {
    "Pages per sheet"
}

/// Label on the page-order choice.
#[must_use]
pub const fn page_order() -> &'static str {
    "Page order"
}

/// One page order's name.
#[must_use]
pub const fn order(order: PageOrder) -> &'static str {
    match order {
        PageOrder::Horizontal => "Horizontal",
        PageOrder::HorizontalReversed => "Horizontal reversed",
        PageOrder::Vertical => "Vertical",
        PageOrder::VerticalReversed => "Vertical reversed",
    }
}

/// The border checkbox.
#[must_use]
pub const fn border() -> &'static str {
    "Print page border"
}

/// The auto-rotate checkbox.
#[must_use]
pub const fn rotate() -> &'static str {
    "Auto-rotate pages"
}

/// Hover on auto-rotate.
#[must_use]
pub const fn rotate_tooltip() -> &'static str {
    "Turn a page a quarter turn clockwise when it then prints larger."
}

/// Label on the binding choice.
#[must_use]
pub const fn binding_label() -> &'static str {
    "Binding"
}

/// One binding's name.
#[must_use]
pub const fn binding(binding: Binding) -> &'static str {
    match binding {
        Binding::Left => "Left",
        Binding::Right => "Right",
        Binding::LeftTall => "Left (tall)",
        Binding::RightTall => "Right (tall)",
    }
}

/// Hover on the binding choice.
#[must_use]
pub const fn binding_tooltip() -> &'static str {
    "Left and Right fold down the middle of a sideways sheet; the tall \
     choices fold across the middle of an upright one. Right is for \
     right-to-left reading."
}

/// Label on the sides choice.
#[must_use]
pub const fn subset_label() -> &'static str {
    "Sides"
}

/// One sides choice's name.
#[must_use]
pub const fn subset(subset: BookletSubset) -> &'static str {
    match subset {
        BookletSubset::BothSides => "Both sides",
        BookletSubset::FrontOnly => "Front side only",
        BookletSubset::BackOnly => "Back side only",
    }
}

/// Label before the booklet's sheet range.
#[must_use]
pub const fn sheets_from() -> &'static str {
    "Sheets"
}

/// Between the two ends of the sheet range.
#[must_use]
pub const fn sheets_to() -> &'static str {
    "to"
}

/// Hover on the sheet range.
#[must_use]
pub const fn sheets_tooltip() -> &'static str {
    "Physical sheets of the booklet, counted from the outside in. Sheet 1 \
     carries the cover. Pages are chosen on the left as usual."
}

/// What Multiple made.
#[must_use]
pub fn nup_caption(rows: usize, columns: usize, sheets: usize, turned: usize) -> String {
    let sheets_word = if sheets == 1 { "sheet" } else { "sheets" };
    let mut s = format!("{columns} × {rows} pages per sheet, {sheets} {sheets_word}.");
    if turned > 0 {
        let pages = if turned == 1 { "page" } else { "pages" };
        s.push_str(&format!(" {turned} {pages} turned to fit."));
    }
    s
}

/// What Booklet made.
#[must_use]
pub fn booklet_caption(printed: (usize, usize), total: usize, blanks: usize) -> String {
    let mut s = if printed == (1, total) {
        let word = if total == 1 { "sheet" } else { "sheets" };
        format!("{total} {word}, folded once.")
    } else {
        format!("Sheets {}–{} of {total}.", printed.0, printed.1)
    };
    if blanks > 0 {
        let word = if blanks == 1 { "page is" } else { "pages are" };
        s.push_str(&format!(
            " {blanks} blank {word} added: a folded booklet needs a multiple of four."
        ));
    }
    s
}

/// What to do between the two passes of a one-sided booklet, if anything.
#[must_use]
pub const fn booklet_flip(subset: BookletSubset) -> Option<&'static str> {
    match subset {
        BookletSubset::BothSides => Some("Needs two-sided printing, flipped on the short edge."),
        BookletSubset::FrontOnly => Some(
            "Then put the printed stack back in the tray, turned over, \
             without reordering it, and print Back side only.",
        ),
        BookletSubset::BackOnly => Some(
            "Load the stack printed with Front side only, turned over and in \
             the same order, before printing.",
        ),
    }
}

/// Why the pages could not be laid out.
#[must_use]
pub fn impose_refused(reason: &str) -> String {
    format!("Can't lay the pages out: {reason}.")
}

/// The Position tab while Multiple or Booklet is on.
#[must_use]
pub const fn impose_positions_fixed() -> &'static str {
    "Each page's position is set by the layout chosen on the Pages tab."
}

/// The commit's refusal when an imposed sheet is too large to draw.
#[must_use]
pub const fn impose_sheet_too_large() -> &'static str {
    "A sheet is too large to render at this resolution. Lower the resolution \
     and try again."
}
