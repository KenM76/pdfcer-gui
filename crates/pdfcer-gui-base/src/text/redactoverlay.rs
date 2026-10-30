//! # `text::redactoverlay` — how each redaction's overlay mark was drawn
//!
//! The words for `redact::disclosures::overlays`: the six overlay counts of
//! `pdfcer_core::redact::RedactionReport`, one sentence each, said only when
//! non-zero.

/// Overlays cut back to the page edge.
#[must_use]
pub fn overlay_trimmed(n: u64) -> String {
    format!(
        "{n} redaction mark(s) reach past the edge of the page: the box drawn over them stops at \
         the edge. Everything under them is removed either way."
    )
}

/// Overlays not drawn because the region lies wholly off the page.
#[must_use]
pub fn overlay_off_page(n: u64) -> String {
    format!(
        "{n} redaction mark(s) lie wholly outside the page: what was under them is removed, and \
         no box is drawn for them."
    )
}

/// Regions left transparent: the mark names no fill.
#[must_use]
pub fn overlay_transparent(n: u64) -> String {
    format!(
        "{n} redaction mark(s) name no fill colour: the area they cover is left blank, with no \
         box drawn over it."
    )
}

/// A stored overlay appearance pdfcer could not draw.
#[must_use]
pub fn overlay_ro_not_drawn(n: u64) -> String {
    format!(
        "{n} redaction mark(s) carry their own overlay picture, which pdfcer cannot draw: a \
         plain box is drawn instead."
    )
}

/// A fill colour given in grey or CMYK numbers.
#[must_use]
pub fn overlay_fill_not_rgb(n: u64) -> String {
    format!(
        "{n} redaction mark(s) give their fill as grey or CMYK numbers where an RGB colour is \
         expected: the colour they name is used."
    )
}

/// Overlay text printed over the cleared area.
#[must_use]
pub fn overlay_text_burned(n: u64) -> String {
    format!(
        "{n} redaction mark(s) carry their own overlay text, which is printed over the cleared area."
    )
}
