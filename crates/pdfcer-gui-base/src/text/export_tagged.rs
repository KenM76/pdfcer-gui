//! The words the Word and table exports add when the document is tagged:
//! whether its own structure was followed, and what that took.

fn plural(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 { one } else { many }
}

/// The tree was used. `percent` is the share of the text it owns, 0–100.
#[must_use]
pub fn followed(percent: u32) -> String {
    format!(
        "Headings, paragraphs, lists and tables were taken from the document's own tags, \
         which cover {percent}% of its text. The rest was laid out from the page."
    )
}

/// The tree exists and owns too little of the text to follow.
#[must_use]
pub fn too_little(percent: u32) -> String {
    format!(
        "The document is tagged, but its tags cover only {percent}% of its text, so \
         headings, paragraphs and tables were judged from the page instead."
    )
}

/// The tree exists and owns none of the text.
#[must_use]
pub const fn owns_nothing() -> &'static str {
    "The document is tagged, but its tags own none of its text, so headings, \
     paragraphs and tables were judged from the page instead."
}

/// Tagged content with no standard block type, written as paragraphs.
#[must_use]
pub fn as_paragraphs(count: usize) -> String {
    format!(
        "{count} tagged {} a type with no Word equivalent, so {} written as {}.",
        plural(count, "element has", "elements have"),
        plural(count, "it was", "they were"),
        plural(count, "a paragraph", "paragraphs"),
    )
}

/// Tables inside a table cell, flattened into that cell's text.
#[must_use]
pub fn nested_tables(count: usize) -> String {
    format!(
        "{count} {} inside a table cell {} flattened into the cell's text.",
        plural(count, "table", "tables"),
        plural(count, "was", "were"),
    )
}

/// Content tagged inside a table but outside its cells.
#[must_use]
pub fn stray_table_content(count: usize) -> String {
    format!(
        "{count} tagged {} inside a table but outside its cells {} written as {}.",
        plural(count, "element", "elements"),
        plural(count, "was", "were"),
        plural(count, "a paragraph", "paragraphs"),
    )
}

/// Tag references to content the page never marks.
#[must_use]
pub fn broken_references(count: usize) -> String {
    format!(
        "{count} {} in the document's tags {} to content that is not on the page; \
         nothing was written for {}.",
        plural(count, "reference", "references"),
        plural(count, "points", "point"),
        plural(count, "it", "them"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_agree_with_their_nouns() {
        assert!(as_paragraphs(1).contains("1 tagged element has"));
        assert!(as_paragraphs(3).contains("they were written as paragraphs"));
        assert!(broken_references(1).contains("1 reference in the document's tags points"));
        assert!(nested_tables(2).contains("2 tables inside a table cell were"));
    }
}
