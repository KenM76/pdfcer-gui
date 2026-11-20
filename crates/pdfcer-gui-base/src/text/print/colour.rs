//! # `text::print::colour` — the words for the Print dialog's colour choice

/// Above the three choices.
#[must_use]
pub const fn heading() -> &'static str {
    "Colour"
}

/// As the document is coloured.
#[must_use]
pub const fn colour() -> &'static str {
    "In colour"
}

/// Composite grey.
#[must_use]
pub const fn grey() -> &'static str {
    "In grey (greyscale)"
}

/// Every mark solid black.
#[must_use]
pub const fn black() -> &'static str {
    "Black only"
}

/// One grey sheet per ink.
#[must_use]
pub const fn separations() -> &'static str {
    "Separations (one sheet per ink)"
}

/// Under the choices while separations are chosen.
#[must_use]
pub const fn separations_note() -> &'static str {
    "Each page prints as one grey sheet per ink: cyan, magenta, yellow and \
     black, then each spot colour, with the ink named on the sheet. Plate greys \
     approximate ink density. The preview's plates switch shows them one ink \
     at a time. N-up, booklet and poster sheets print in colour."
}

/// After a job printed as separations. `inks` are as the file spells them.
#[must_use]
pub fn plates_sent(pages: usize, plates: usize, inks: &[String]) -> String {
    let what = if pages == 1 {
        "1 page"
    } else {
        &format!("{pages} pages")
    };
    format!(
        "{what} printed as {plates} separation sheets ({}). Plate greys approximate ink density.",
        inks.join(", ")
    )
}

/// After a job some of whose pages could not be separated.
#[must_use]
pub fn plates_refused(pages: usize) -> String {
    if pages == 1 {
        "1 page was too large to separate in memory and printed in colour.".to_owned()
    } else {
        format!("{pages} pages were too large to separate in memory and printed in colour.")
    }
}

/// Under the choices while black only is chosen.
#[must_use]
pub const fn black_note() -> &'static str {
    "Everything that is not white prints solid black, including pale fills and photographs."
}
