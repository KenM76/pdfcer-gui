//! # `text::settings::ocrlayer` — the copy for the recognised-text colour
//!
//! One setting, `OPERATOR_REQUESTS.md` **O229**, and its own file because it
//! belongs to a feature rather than to a standard: everything else in the
//! *Drawing the page* group answers a question about rendering, and this
//! answers *what colour do I want the X-ray in*.
//!
//! The rule this copy follows is [`super`]'s: every setting states a **title**,
//! what happens if you **never touch it**, and what it **costs or does not
//! affect**.
//!
//! ★ The radius line does the heavy lifting here and is the one an operator
//! could not guess. Choosing a colour for text that the file itself renders as
//! nothing sounds like it must be changing the document; it is not, and saying
//! so is the difference between a setting somebody uses and one they leave
//! alone in case it does something.

/// **The colour the recognised text is drawn in** — O229.
///
/// Worded around the scan rather than around the layer: the operator is
/// picking a colour they can *tell apart from the drawing underneath*, which
/// is the whole of the decision they are making.
#[must_use]
pub const fn ocr_colour_title() -> &'static str {
    "Colour of the recognised text"
}

/// What happens if you never touch it.
#[must_use]
pub const fn ocr_colour_silence() -> &'static str {
    "Recognised text is drawn in magenta, which is a colour a scanned drawing almost never contains, so it stands out against black line work and against blue or red CAD linework alike."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn ocr_colour_radius() -> &'static str {
    "On screen only, and only while the text layer is switched on. The recognised text itself is invisible in the file — this colour is never printed, never exported and never saved into the document."
}

/// The swatch's own label.
#[must_use]
pub const fn ocr_colour_label() -> &'static str {
    "Recognised text"
}

/// The note under the swatch.
#[must_use]
pub const fn ocr_colour_note() -> &'static str {
    "Pick a colour that contrasts with your scans. Turn the text layer on from View to see it."
}

/// The control that puts the colour back, shown only when it has been moved.
#[must_use]
pub const fn ocr_colour_reset() -> &'static str {
    "Back to magenta"
}
