//! What the Export image window says about separation plates.

/// The choice, shown under EMF.
#[must_use]
pub const fn checkbox() -> &'static str {
    "One file per ink (plates)"
}

/// What the choice does.
#[must_use]
pub const fn hint() -> &'static str {
    "Each ink on the page (cyan, magenta, yellow, black, then every spot colour) \
     is written as its own metafile, named after the page and the ink."
}

/// The receipt's first line for a separated page.
#[must_use]
pub fn wrote(page_number: usize, plates: usize, first: &str) -> String {
    format!("Page {page_number} written as {plates} plate file(s), beginning with {first}.")
}

/// One plate's line: what stayed lines and what had to be pixels.
#[must_use]
pub fn plate(ink: &str, ops: usize, rasters: usize) -> String {
    if rasters == 0 {
        format!("The {ink} plate: {ops} drawing operations, all of them real lines.")
    } else {
        format!(
            "The {ink} plate: {ops} drawing operations, plus {rasters} part(s) that had to \
             go in as pictures."
        )
    }
}

/// A page the engine could not separate.
#[must_use]
pub fn composite(page_number: usize) -> String {
    format!(
        "Page {page_number} could not be separated into inks, so it was written as one \
         ordinary metafile."
    )
}

/// A result from the engine this version cannot write.
#[must_use]
pub fn unsupported(page_number: usize) -> String {
    format!(
        "Page {page_number}'s plates came back in a form this version cannot write, so nothing more was written."
    )
}

#[cfg(test)]
mod tests {
    use super::plate;

    #[test]
    fn a_plate_line_names_its_pictures_only_when_it_has_some() {
        assert!(plate("Cyan", 3, 0).contains("all of them real lines"));
        assert!(plate("Spot", 3, 2).contains("2 part(s)"));
    }
}
