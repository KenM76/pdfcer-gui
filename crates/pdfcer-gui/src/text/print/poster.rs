//! # `text::print::poster` — poster printing, one page across many sheets
//!
//! The words for [`crate::dialogs::print::poster`]. Acrobat's option names
//! are kept (*Tile scale*, *Overlap*, *Cut marks*, *Labels*, *Tile only
//! large pages*) so an operator who knows that dialog finds the same words.

/// The choice between ordinary sizing and poster tiling, left half.
#[must_use]
pub const fn sizing_mode_size() -> &'static str {
    "Size"
}

/// The choice between ordinary sizing and poster tiling, right half.
#[must_use]
pub const fn sizing_mode_poster() -> &'static str {
    "Poster"
}

/// Hover on the Poster choice.
#[must_use]
pub const fn sizing_mode_poster_tooltip() -> &'static str {
    "Print one page across several sheets, to be trimmed and taped together."
}

/// Label on the poster magnification.
#[must_use]
pub const fn poster_tile_scale() -> &'static str {
    "Tile scale"
}

/// Hover on the poster magnification.
#[must_use]
pub const fn poster_tile_scale_tooltip() -> &'static str {
    "How big the assembled poster is. 100 % is the page's own size; 200 % \
     doubles its width and height, and takes about four times the sheets."
}

/// Label on the overlap field.
#[must_use]
pub const fn poster_overlap() -> &'static str {
    "Overlap"
}

/// Hover on the overlap field.
#[must_use]
pub const fn poster_overlap_tooltip() -> &'static str {
    "A band of the drawing printed on both neighbouring sheets, so they can \
     be lined up. It is cut off one of the two when assembling."
}

/// Suffix on millimetre fields.
#[must_use]
pub const fn mm_suffix() -> &'static str {
    " mm"
}

/// Cut marks checkbox.
#[must_use]
pub const fn poster_cut_marks() -> &'static str {
    "Cut marks"
}

/// Hover on the cut marks checkbox.
#[must_use]
pub const fn poster_cut_marks_tooltip() -> &'static str {
    "Ticks in the top and left margin of every sheet showing where to cut. \
     The drawing is shifted down and right to make room for them."
}

/// Labels checkbox.
#[must_use]
pub const fn poster_labels() -> &'static str {
    "Labels"
}

/// Hover on the labels checkbox.
#[must_use]
pub const fn poster_labels_tooltip() -> &'static str {
    "Prints the file name and each sheet's row and column in its top margin, \
     so the pile can be put back in order."
}

/// Tile only large pages checkbox.
#[must_use]
pub const fn poster_large_only() -> &'static str {
    "Tile only large pages"
}

/// Hover on the tile-only-large-pages checkbox.
#[must_use]
pub const fn poster_large_only_tooltip() -> &'static str {
    "Pages that fit on one sheet at this tile scale print on one sheet, \
     untiled. Unticked, every page is tiled."
}

/// What the poster comes to, for the page on the preview.
#[must_use]
pub fn poster_grid(rows: usize, columns: usize, poster_mm: (i64, i64)) -> String {
    let sheets = rows * columns;
    let noun = if sheets == 1 { "sheet" } else { "sheets" };
    format!(
        "{columns} across × {rows} down = {sheets} {noun}, assembling to {} × {} mm.",
        poster_mm.0, poster_mm.1
    )
}

/// The page on the preview fits one sheet and is not tiled.
#[must_use]
pub const fn poster_untiled() -> &'static str {
    "This page fits on one sheet and prints untiled."
}

/// The poster cannot be planned, with pdfcer-print's own reason.
#[must_use]
pub fn poster_refused(reason: &str) -> String {
    format!("This poster cannot be printed: {reason}")
}

/// A poster sheet's bitmap could not be allocated.
#[must_use]
pub const fn poster_sheet_too_large() -> &'static str {
    "A poster sheet is too large to render at this resolution. Lower the resolution and try again."
}

/// The Position tab while poster mode is on.
#[must_use]
pub const fn poster_positions_fixed() -> &'static str {
    "Poster sheets are placed by the tiling. Choose Size on the Pages tab to position pages yourself."
}
