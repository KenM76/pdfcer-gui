//! The status-bar sentences for Markup ▸ Comments ▸ *Store appearances*,
//! which writes the picture pdfcer draws for an appearance-less markup into
//! the file (`EditSession::bake_missing_appearances`).

use pdfcer_core::edit::{BakeAppearancesOutcome, SkippedAppearance};

/// The disclosure after a store: what changed, then what pdfcer settled
/// rather than read, then what it left without a picture.
#[must_use]
pub fn stored(o: &BakeAppearancesOutcome) -> Vec<String> {
    let mut lines = Vec::new();
    if o.baked.is_empty() {
        lines.push(nothing_stored().to_owned());
    } else {
        lines.push(format!(
            "Stored a picture for {} annotation(s); other viewers now show them as pdfcer draws \
             them. Ctrl+Z undoes it.",
            o.baked.len()
        ));
    }
    let moved = o.baked.iter().filter(|b| b.rect_changed).count();
    if moved > 0 {
        lines.push(format!(
            "{moved} of them had their box widened to hold the picture (an arrowhead or a thick \
             line reached past it)."
        ));
    }
    lines.extend(o.baked.iter().flat_map(|b| b.notes.iter().cloned()));
    lines.extend(kept(&o.skipped));
    lines
}

/// Nothing in the file lacked a picture pdfcer could store.
#[must_use]
pub const fn nothing_stored() -> &'static str {
    "No annotation needed a stored picture, so nothing was changed."
}

/// The annotations left without a picture, with the first one's reason.
#[must_use]
pub fn kept(skipped: &[SkippedAppearance]) -> Option<String> {
    let first = skipped.first()?;
    Some(format!(
        "{} annotation(s) kept without a stored picture; the first, a {}, because {}.",
        skipped.len(),
        first.subtype,
        first.reason
    ))
}
