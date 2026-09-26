//! # `text::panels::textannotstyle` — the words for restyling a mark that
//! carries WORDS
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/textannotstyle.md`.

/// **Why a text box's appearance is not changed here — and it is NOT
/// because no verb exists.**
#[must_use]
pub const fn markup_text_box_not_restylable() -> &'static str {
    "pdfcer does not change a text box's colour here: redrawing it would lay the words out as \
     one line and push anything past the first sentence outside the box. You can still move it, \
     resize it, delete it, and edit the words it carries."
}

/// The sentence under the sticky-note and stamp style rows.
#[must_use]
pub const fn markup_text_annot_note() -> &'static str {
    "A note or stamp always has a colour, so it can be changed but not taken away again. \
     Transparency is not offered for these two."
}

/// What the icon chooser shows for a note whose `/Name` pdfcer does not model.
#[must_use]
pub const fn markup_icon_foreign() -> &'static str {
    "Not one of these"
}

/// **The file's own icon name, shown as the entry it is** — for a `/Name`
/// §12.5.6.4 permits and pdfcer does not model.
#[must_use]
pub fn markup_icon_foreign_named(name: &str) -> String {
    format!("\"{name}\"")
}

/// **The note under a foreign icon** — and it says something different
/// from what it said yesterday.
#[must_use]
pub const fn markup_icon_foreign_note() -> &'static str {
    "This note uses an icon pdfcer does not draw. The name is kept exactly as it is in the file \
     — including when you change the colour — but pdfcer draws its own sticky-note symbol for \
     it, so it will not look the way it does in the program that made it."
}

/// The label on the stamp's label-size control.
#[must_use]
pub const fn stamp_text_size_label() -> &'static str {
    "Text size"
}

/// The unit suffix inside the stamp label-size spinner.
#[must_use]
pub const fn stamp_text_size_suffix() -> &'static str {
    " pt"
}

/// The label on the chooser for what happens when the resized label no longer
/// fits the stamp's box.
#[must_use]
pub const fn stamp_fit_label() -> &'static str {
    "If it does not fit"
}

/// `StampFit::GrowToText`, for the chooser.
#[must_use]
pub const fn stamp_fit_grow() -> &'static str {
    "Make the stamp wider"
}

/// `StampFit::ShrinkToBox`, for the chooser.
#[must_use]
pub const fn stamp_fit_shrink() -> &'static str {
    "Shrink the words to fit"
}

/// `StampFit::ClipToBox`, for the chooser.
#[must_use]
pub const fn stamp_fit_clip() -> &'static str {
    "Cut the words off at the edge"
}

/// **One policy, as the chooser lists it** — the dispatcher the two surfaces
/// share.
#[must_use]
pub const fn stamp_fit_option(fit: pdfcer_core::annot_author::StampFit) -> &'static str {
    use pdfcer_core::annot_author::StampFit;
    match fit {
        StampFit::GrowToText => stamp_fit_grow(),
        StampFit::ShrinkToBox => stamp_fit_shrink(),
        StampFit::ClipToBox => stamp_fit_clip(),
        _ => stamp_fit_unknown(),
    }
}

/// A fit policy this build has no words for. See [`stamp_fit_option`].
#[must_use]
pub const fn stamp_fit_unknown() -> &'static str {
    "A fit rule this build does not know"
}

/// ⚠ **Shown when the stamp's `/DA` is present and unreadable** —
/// `StampSizeSource::DaUnreadable`.
#[must_use]
pub const fn stamp_size_da_unreadable() -> &'static str {
    "This stamp declares a text size that pdfcer cannot read, so the size shown was measured \
     from the stamp's own picture instead. Setting a size here will replace what the file \
     declares."
}

/// **The stamp's label was drawn smaller than asked for** —
/// `StampLabelFit::LabelShrunk`.
#[must_use]
pub fn stamp_label_shrunk(drawn: f64, requested: f64) -> String {
    format!(
        "The stamp's words were shrunk to {drawn:.0} pt to fit its box — you asked for \
         {requested:.0} pt."
    )
}

/// ⚠ **Characters the operator typed are not on the page** —
/// `StampLabelFit::LabelClipped`.
#[must_use]
pub fn stamp_label_clipped(hidden: usize) -> String {
    format!(
        "{hidden} character(s) of this stamp's words are cut off by its box — the words are \
         centred, so the loss is split between both ends."
    )
}

/// **The stamp's box was widened to hold the label** —
/// `StampLabelFit::BoxGrown`.
#[must_use]
pub fn stamp_label_box_grown(width: f64) -> String {
    format!("The stamp was widened to {width:.0} pt so its words fit.")
}

/// ⚠ **The stamp's words were fitted in a way this build has no words for** —
/// a `StampLabelFit` variant added to `pdfcer-core` after this build was
/// linked.
#[must_use]
pub const fn stamp_label_fit_unknown() -> &'static str {
    "pdfcer had to adjust this stamp's words to fit its box, in a way this version cannot \
     describe. Check the stamp."
}
