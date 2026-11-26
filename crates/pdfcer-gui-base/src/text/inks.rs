//! # `text::inks` — every word File ▸ Print ▸ Ink Manager says
//!
//! Consumed by `pdfcer_gui::dialogs::prepress::inks` and the report written
//! by `pdfcer_gui::app::actions::prepress`.

/// The window's title.
#[must_use]
pub const fn title() -> &'static str {
    "Ink Manager"
}

/// What the window does, and the two things it must disclose: the edits
/// change the file, and only inks named by a colour space are listed.
#[must_use]
pub const fn intro() -> &'static str {
    "Every ink a Separation or DeviceN colour space names, with where it is drawn. Choose how a spot ink prints: as itself, as another ink, or converted to process colours. These change the document itself, so the saved file carries the new inks; they are not a print-time setting. Plain CMYK paint is not listed unless a colour space names its plate. One Ctrl+Z undoes the whole run."
}

/// Shown when the document names no ink.
#[must_use]
pub const fn none() -> &'static str {
    "No colour space in this document names an ink."
}

/// Shown under the list when the engine could not read every ink.
#[must_use]
pub const fn incomplete() -> &'static str {
    "This list may be missing inks: a colour space names more inks than can be read, or sits too deep inside an object."
}

/// Column headings.
#[must_use]
pub const fn col_ink() -> &'static str {
    "Ink"
}

/// Column heading: process or spot.
#[must_use]
pub const fn col_kind() -> &'static str {
    "Type"
}

/// Column heading: the alternate spaces.
#[must_use]
pub const fn col_alternate() -> &'static str {
    "Alternate"
}

/// Column heading: where it is drawn.
#[must_use]
pub const fn col_pages() -> &'static str {
    "Pages"
}

/// Column heading: the operator's choice.
#[must_use]
pub const fn col_print_as() -> &'static str {
    "Prints as"
}

/// A process ink's type.
#[must_use]
pub const fn kind_process() -> &'static str {
    "Process"
}

/// A spot ink's type.
#[must_use]
pub const fn kind_spot() -> &'static str {
    "Spot"
}

/// What more than one alternate means.
#[must_use]
pub const fn alternates_disagree_hover() -> &'static str {
    "The colour spaces naming this ink disagree on how it looks without its own plate."
}

/// An ink defined but drawn nowhere.
#[must_use]
pub const fn not_drawn() -> &'static str {
    "not drawn"
}

/// The pages an ink is drawn on, and how many uses.
#[must_use]
pub fn pages(list: &str, uses: usize) -> String {
    format!("{list} ({uses} use(s))")
}

/// The choice that leaves a spot as it is.
#[must_use]
pub const fn keep() -> &'static str {
    "Itself"
}

/// The choice that converts a spot to process.
#[must_use]
pub const fn process() -> &'static str {
    "Process colours (CMYK)"
}

/// What converting does.
#[must_use]
pub const fn process_hover() -> &'static str {
    "Every colour drawn in this ink becomes CMYK through the ink's own alternate. Only an ink whose alternate is DeviceCMYK can be converted; any other is reported and left."
}

/// The choice that prints a spot as another ink.
#[must_use]
pub fn alias(to: &str) -> String {
    format!("As {to}")
}

/// What aliasing does.
#[must_use]
pub const fn alias_hover() -> &'static str {
    "Renames the ink in every colour space that names it, so it prints on the other ink's plate. Each space keeps its own on-screen colour."
}

/// Why Apply is greyed with nothing chosen.
#[must_use]
pub const fn nothing_chosen() -> &'static str {
    "Choose how at least one spot ink prints."
}

/// Why Apply is greyed when an alias lands on an ink that is itself aliased.
#[must_use]
pub fn chained(from: &str, to: &str) -> String {
    format!(
        "{from} prints as {to}, which is itself set to print as another ink. Pick the final ink for {from}."
    )
}

/// One alias that changed something.
#[must_use]
pub fn aliased(from: &str, to: &str, spaces: usize) -> String {
    format!("{from} now prints as {to}: {spaces} colour space(s) renamed.")
}

/// One alias that found nothing to change.
#[must_use]
pub fn alias_nothing(from: &str, to: &str) -> String {
    format!("{from} as {to}: nothing names {from}; nothing changed.")
}

/// An alias the engine refused after earlier ones had applied.
#[must_use]
pub fn alias_stopped(from: &str, to: &str, why: &str) -> String {
    format!("{from} as {to} was refused, and nothing after it ran: {why}")
}

/// The conversion's counts.
#[must_use]
pub fn converted(
    inks: &str,
    spaces: usize,
    paints: usize,
    images: usize,
    shadings: usize,
    lookups: usize,
) -> String {
    format!(
        "{inks} converted to process: {spaces} colour space(s), {paints} paint(s), {images} picture(s), {shadings} shading(s), {lookups} lookup table(s)."
    )
}

/// A conversion that found nothing to change.
#[must_use]
pub fn converted_nothing(inks: &str) -> String {
    format!("{inks}: nothing drawn in these inks could be converted; nothing changed.")
}

/// A conversion the engine refused after the aliases had applied.
#[must_use]
pub fn convert_stopped(why: &str) -> String {
    format!("Converting to process was refused: {why}")
}

/// Something an edit left as it was.
#[must_use]
pub fn refused(ink: &str, object: &str, why: &str) -> String {
    if object.is_empty() {
        format!("Left {ink}: {why}")
    } else {
        format!("Left {ink} in object {object}: {why}")
    }
}

/// A stream that no longer overprints.
#[must_use]
pub fn overprint_lost(object: &str) -> String {
    format!(
        "Object {object} painted a converted ink under overprint; the converted colour no longer overprints."
    )
}
