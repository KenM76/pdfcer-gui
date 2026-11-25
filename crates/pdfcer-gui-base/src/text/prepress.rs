//! # `text::prepress` — every word the prepress windows say
//!
//! Consumed by `pdfcer_gui::dialogs::prepress` and the report it shows,
//! written by `pdfcer_gui::app::actions::prepress`.

/// Fix hairlines' title.
#[must_use]
pub const fn hairlines_title() -> &'static str {
    "Fix hairlines"
}

/// What Fix hairlines does.
#[must_use]
pub const fn hairlines_intro() -> &'static str {
    "Every line that would print thinner than the width below, or with width 0, is widened to exactly that width. Nothing else moves. One Ctrl+Z undoes the whole run."
}

/// The minimum-width field's label.
#[must_use]
pub const fn hairlines_min() -> &'static str {
    "Thinnest line to print"
}

/// What the minimum means.
#[must_use]
pub const fn hairlines_min_hover() -> &'static str {
    "Measured along the line's thinnest axis on the printed page, in points (1/72 inch). 0.25 pt is a common press minimum."
}

/// Flatten transparency's title.
#[must_use]
pub const fn flatten_title() -> &'static str {
    "Flatten transparency"
}

/// What Flatten transparency does.
#[must_use]
pub const fn flatten_intro() -> &'static str {
    "Redraws transparent areas as opaque artwork that prints the same, for presses and standards that cannot take transparency. What cannot be computed as vector shapes is drawn as a picture at the resolution below. One Ctrl+Z undoes the whole run."
}

/// Keep text live.
#[must_use]
pub const fn flatten_keep_text() -> &'static str {
    "Keep text outside the flattened areas as text"
}

/// Keep strokes stroked.
#[must_use]
pub const fn flatten_keep_strokes() -> &'static str {
    "Keep redrawn lines as lines"
}

/// What clearing keep-strokes does.
#[must_use]
pub const fn flatten_keep_strokes_hover() -> &'static str {
    "Clear this to draw each redrawn line as a filled outline of its stroke."
}

/// The resolution field's label.
#[must_use]
pub const fn flatten_ppi() -> &'static str {
    "Resolution for areas drawn as pictures"
}

/// The resolution's range.
#[must_use]
pub fn flatten_ppi_hover(min: f64, max: f64) -> String {
    format!("Pixels per inch, {min} to {max}.")
}

/// Convert colours' title.
#[must_use]
pub const fn colours_title() -> &'static str {
    "Convert colours"
}

/// What Convert colours does.
#[must_use]
pub const fn colours_intro() -> &'static str {
    "Converts the chosen colours, in fills, lines, text, pictures and shadings, to the destination profile's colours. One Ctrl+Z undoes the whole run."
}

/// The destination combo box's label.
#[must_use]
pub const fn colours_destination() -> &'static str {
    "Convert to"
}

/// The document's own output intent, as a destination.
#[must_use]
pub const fn colours_output_intent() -> &'static str {
    "The document's output intent"
}

/// The closed combo box with nothing chosen.
#[must_use]
pub const fn choose_profile() -> &'static str {
    "Choose a profile"
}

/// An installed profile that could not be read; `why` is the system's reason.
#[must_use]
pub fn profile_unreadable(path: &str, why: &str) -> String {
    format!("Could not read the profile {path}: {why}")
}

/// No destination exists.
#[must_use]
pub const fn no_profiles() -> &'static str {
    "No output profile is installed and the document has no output intent, so there is nothing to convert to."
}

/// The source group's heading.
#[must_use]
pub const fn colours_sources() -> &'static str {
    "Convert these colours"
}

/// Source switch: device RGB.
#[must_use]
pub const fn source_device_rgb() -> &'static str {
    "RGB"
}

/// Source switch: ICC RGB.
#[must_use]
pub const fn source_icc_rgb() -> &'static str {
    "RGB with a profile"
}

/// Source switch: Lab.
#[must_use]
pub const fn source_lab() -> &'static str {
    "Lab"
}

/// Source switch: calibrated RGB.
#[must_use]
pub const fn source_cal_rgb() -> &'static str {
    "Calibrated RGB"
}

/// Source switch: grey.
#[must_use]
pub const fn source_grey() -> &'static str {
    "Grey"
}

/// Why grey is off by default.
#[must_use]
pub const fn source_grey_hover() -> &'static str {
    "Off by default: grey usually means to print on the black plate as it is."
}

/// Keep pure black.
#[must_use]
pub const fn keep_black() -> &'static str {
    "Keep pure black on the black plate"
}

/// What keep pure black means.
#[must_use]
pub const fn keep_black_hover() -> &'static str {
    "Pure black goes to black ink alone rather than through the profile, which would make it a rich black of all four inks."
}

/// Black point compensation.
#[must_use]
pub const fn bpc() -> &'static str {
    "Black point compensation"
}

/// The intent combo box's label.
#[must_use]
pub const fn intent() -> &'static str {
    "Rendering intent"
}

/// Each object's own intent.
#[must_use]
pub const fn intent_objects_own() -> &'static str {
    "Each object's own"
}

/// Perceptual.
#[must_use]
pub const fn intent_perceptual() -> &'static str {
    "Perceptual"
}

/// Relative colorimetric.
#[must_use]
pub const fn intent_relative() -> &'static str {
    "Relative colorimetric"
}

/// Saturation.
#[must_use]
pub const fn intent_saturation() -> &'static str {
    "Saturation"
}

/// Absolute colorimetric.
#[must_use]
pub const fn intent_absolute() -> &'static str {
    "Absolute colorimetric"
}

/// Embed the destination as the output intent.
#[must_use]
pub const fn embed_intent() -> &'static str {
    "Make this profile the document's output intent"
}

/// Convert to PDF/X's title.
#[must_use]
pub const fn pdfx_title() -> &'static str {
    "Convert to PDF/X"
}

/// What Convert to PDF/X does.
#[must_use]
pub const fn pdfx_intro() -> &'static str {
    "Flattens transparency where the standard forbids it, converts colours to the output profile, sets trim boxes and the output intent, then checks the result. If it still fails, every change is undone. One Ctrl+Z undoes a conversion that stays."
}

/// The level combo box's label.
#[must_use]
pub const fn pdfx_level() -> &'static str {
    "Standard"
}

/// The output-profile combo box's label.
#[must_use]
pub const fn pdfx_profile() -> &'static str {
    "Output profile"
}

/// The printing-condition field's label.
#[must_use]
pub const fn pdfx_condition() -> &'static str {
    "Printing condition"
}

/// Run.
#[must_use]
pub const fn apply() -> &'static str {
    "Apply"
}

/// Close.
#[must_use]
pub const fn close() -> &'static str {
    "Close"
}

/// Apply greyed: no page.
#[must_use]
pub const fn names_no_page() -> &'static str {
    "Choose at least one page."
}

/// Apply greyed: no colour.
#[must_use]
pub const fn names_no_colour() -> &'static str {
    "Choose at least one kind of colour to convert."
}

/// Apply greyed: a number out of range.
#[must_use]
pub const fn out_of_range() -> &'static str {
    "The number is out of range."
}

/// The report's heading.
#[must_use]
pub const fn report_heading() -> &'static str {
    "What changed"
}

/// The report's heading after a refusal.
#[must_use]
pub const fn refused_heading() -> &'static str {
    "Nothing changed"
}

/// Hairlines widened.
#[must_use]
pub fn hairlines_widened(n: usize, pages: usize, zero: usize, min: f64) -> String {
    format!("Widened {n} line(s) on {pages} page(s) to {min} pt; {zero} of them had width 0.")
}

/// No hairline found.
#[must_use]
pub fn hairlines_none(min: f64) -> String {
    format!("No line on these pages prints thinner than {min} pt.")
}

/// A shared drawing rewritten.
#[must_use]
pub fn form_reach(form: &str, invocations: usize) -> String {
    format!(
        "Shared drawing {form} was changed in place; it is drawn {invocations} time(s), and every one changed."
    )
}

/// A page the verb could not read.
#[must_use]
pub fn page_skipped(page: usize, why: &str) -> String {
    format!("Page {page} was not changed: {why}")
}

/// A page with no transparency.
#[must_use]
pub fn flatten_page_clean(page: usize) -> String {
    format!("Page {page}: no transparency.")
}

/// A page left untouched though it uses transparency.
#[must_use]
pub fn flatten_page_left(page: usize) -> String {
    format!("Page {page}: uses transparency, left as it was.")
}

/// A page flattened.
#[must_use]
pub fn flatten_page_done(
    page: usize,
    rebuilt: bool,
    vector: usize,
    raster: usize,
    text: usize,
    strokes: usize,
) -> String {
    let how = if rebuilt {
        "redrawn"
    } else {
        "covered where transparent"
    };
    format!(
        "Page {page}: {how}; {vector} area(s) as vector shapes, {raster} as pictures, {text} text run(s) and {strokes} line(s) turned to outlines."
    )
}

/// One of the flattener's own notes about a page.
#[must_use]
pub fn flatten_page_note(page: usize, note: &str) -> String {
    format!("Page {page}: {note}")
}

/// Spot inks redrawn in process colour.
#[must_use]
pub fn flatten_spots(page: usize, inks: &str) -> String {
    format!("Page {page}: spot inks redrawn in process colour: {inks}.")
}

/// Annotations the flatten left alone.
#[must_use]
pub fn flatten_annots_left(page: usize, n: usize) -> String {
    format!(
        "Page {page}: {n} transparent annotation(s) left as they are; flatten annotations first to include them."
    )
}

/// Objects converted.
#[must_use]
pub fn colours_changed(n: usize) -> String {
    format!("Changed {n} object(s).")
}

/// One source family's counts.
#[must_use]
pub fn colours_source(
    name: &str,
    paints: usize,
    images: usize,
    lookups: usize,
    shadings: usize,
    gamut: usize,
) -> String {
    format!(
        "{name}: {paints} paint(s), {images} picture(s), {lookups} palette(s), {shadings} shading(s); {gamut} out of the destination's gamut."
    )
}

/// One object not converted.
#[must_use]
pub fn colours_unconverted(object: &str, why: &str) -> String {
    format!("Not converted ({object}): {why}")
}

/// An output intent added.
#[must_use]
pub const fn intent_added() -> &'static str {
    "Added the profile as the document's output intent."
}

/// An output intent replaced.
#[must_use]
pub fn intent_replaced(previous: &str) -> String {
    format!("Replaced the output intent {previous}.")
}

/// Nothing converted.
#[must_use]
pub const fn colours_none() -> &'static str {
    "No colour of the chosen kinds was found on these pages."
}

/// The PDF/X result.
#[must_use]
pub fn pdfx_result(level: &str, before: &str, after: &str, claimed: bool) -> String {
    let claim = if claimed {
        "The file now claims it."
    } else {
        "The file does not claim it."
    };
    format!("{level}: was {before}, now {after}. {claim}")
}

/// A PDF/X attempt kept partially.
#[must_use]
pub const fn pdfx_kept_partial() -> &'static str {
    "The conversion did not reach conformance; its changes were kept as you asked."
}

/// A PDF/X attempt reverted.
#[must_use]
pub const fn pdfx_reverted() -> &'static str {
    "The conversion did not reach conformance, so every change was undone."
}

/// A finding still open after PDF/X conversion.
#[must_use]
pub fn pdfx_finding(rule: &str, message: &str) -> String {
    format!("Still open: {rule}: {message}")
}

/// The disclosures heading.
#[must_use]
pub const fn notes_heading() -> &'static str {
    "Notes"
}

/// A verdict word, as the reports print it.
#[must_use]
pub fn verdict_word(token: &str) -> &'static str {
    match token {
        "conforms" => "conforms",
        "fails" => "does not conform",
        _ => "cannot tell",
    }
}
