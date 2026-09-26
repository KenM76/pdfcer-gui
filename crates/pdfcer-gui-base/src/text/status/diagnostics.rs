//! # `text::status::diagnostics` — **the narrator's whole vocabulary**
//!
//!
//! ## The seam is a consumer, not an alphabet
//!
//! A catalog area in this crate is keyed by **the surface it serves**, and
//! every function here is read by exactly one: `app::status::notes`, whose
//! `findings` table pairs a renderer counter with one of these sentences and
//! whose `notes_line` joins the results. Nothing else in the shell calls any
//! of them, and the Render-diagnostics dialog reaches them only *through*
//! `findings`, deliberately — two tables would agree on the day they were
//! written and disagree the first time an eleventh counter was added to one.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/status/diagnostics.md`.

/// The disclosure control's label, closed and open.
#[must_use]
pub fn diagnostics_toggle(open: bool) -> &'static str {
    if open {
        "⏷ Render notes"
    } else {
        "⏵ Render notes"
    }
}

/// Hover text for the disclosure.
#[must_use]
pub fn diagnostics_tooltip() -> &'static str {
    "What pdfcer had to substitute or leave out when it drew this page. \
     These are facts about the renderer, not faults in your document."
}

/// Shown when the page drew with nothing substituted and nothing skipped.
#[must_use]
pub fn diagnostics_clean() -> &'static str {
    "Drawn with nothing substituted or left out"
}

/// Glyphs painted from a **bundled** substitute face.
#[must_use]
pub fn diagnostics_glyphs_substituted(n: usize) -> String {
    if n == 1 {
        "1 glyph drawn with a bundled substitute face".to_owned()
    } else {
        format!("{n} glyphs drawn with a bundled substitute face")
    }
}

/// Glyphs painted from an **operator-supplied** face.
#[must_use]
pub fn diagnostics_glyphs_supplied(n: usize) -> String {
    if n == 1 {
        "1 glyph drawn from a supplied font".to_owned()
    } else {
        format!("{n} glyphs drawn from a supplied font")
    }
}

/// Glyphs that had no shape at all — `.notdef`, or nothing painted.
#[must_use]
pub fn diagnostics_glyphs_notdef(n: usize) -> String {
    if n == 1 {
        "1 glyph with no shape available".to_owned()
    } else {
        format!("{n} glyphs with no shape available")
    }
}

/// Whole fonts whose machinery this build does not implement; their text was
/// **skipped**, not approximated.
#[must_use]
pub fn diagnostics_fonts_skipped(n: usize) -> String {
    if n == 1 {
        "text from 1 font not drawn".to_owned()
    } else {
        format!("text from {n} fonts not drawn")
    }
}

/// Images that could not be drawn at all.
#[must_use]
pub fn diagnostics_images_skipped(n: usize) -> String {
    if n == 1 {
        "1 image not drawn".to_owned()
    } else {
        format!("{n} images not drawn")
    }
}

/// Annotations the file carries that pdfcer drew **nothing** for.
#[must_use]
pub fn diagnostics_annots_no_appearance(n: usize) -> String {
    if n == 1 {
        "1 annotation carries no appearance and is not drawn".to_owned()
    } else {
        format!("{n} annotations carry no appearance and are not drawn")
    }
}

/// Operators recognised but not yet implemented.
#[must_use]
pub fn diagnostics_ops_deferred(n: usize) -> String {
    if n == 1 {
        "1 drawing operator not yet implemented".to_owned()
    } else {
        format!("{n} drawing operators not yet implemented")
    }
}

/// Operators not recognised at all.
#[must_use]
pub fn diagnostics_ops_unknown(n: usize) -> String {
    if n == 1 {
        "1 unrecognised drawing operator".to_owned()
    } else {
        format!("{n} unrecognised drawing operators")
    }
}

/// Optional-content sections that were hidden and therefore not drawn.
#[must_use]
pub fn diagnostics_layers_hidden(n: usize) -> String {
    if n == 1 {
        "1 hidden layer section not drawn".to_owned()
    } else {
        format!("{n} hidden layer sections not drawn")
    }
}

/// `/Contents` entries that named an object the file does not contain.
#[must_use]
pub fn diagnostics_contents_missing(n: usize) -> String {
    if n == 1 {
        "1 content stream missing from the file".to_owned()
    } else {
        format!("{n} content streams missing from the file")
    }
}

/// The page carried no `/Resources` dictionary, and pdfcer supplied one.
#[must_use]
pub const fn diagnostics_resources_defaulted() -> &'static str {
    "this page names no resources of its own — an empty set was assumed"
}

/// Join the notes into the single line the disclosure shows.
#[must_use]
pub fn diagnostics_join(parts: &[String]) -> String {
    parts.join(" · ")
}
