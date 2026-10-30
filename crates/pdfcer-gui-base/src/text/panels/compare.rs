//! `text::panels::compare` — the Compare panel's sentences: the older file,
//! the options, the list of changes and what the comparison could not see.

use crate::text::plural;

/// Heading over the older version's file name.
#[must_use]
pub fn older_heading() -> &'static str {
    "Older version"
}

/// Shown before an older version has been chosen.
#[must_use]
pub fn older_none() -> &'static str {
    "None chosen. The open document is the newer version."
}

/// The button that picks the older version.
#[must_use]
pub fn choose() -> &'static str {
    "Choose\u{2026}"
}

/// Its hover.
#[must_use]
pub fn choose_hover() -> &'static str {
    "Pick the earlier version of this document to compare against."
}

/// Title of the file picker.
#[must_use]
pub fn pick_title() -> &'static str {
    "Choose the older version to compare against"
}

/// The text option.
#[must_use]
pub fn opt_text() -> &'static str {
    "Text"
}

/// Its hover.
#[must_use]
pub fn opt_text_hover() -> &'static str {
    "Words inserted, deleted or replaced. Text that only moved to another \
     line or page is not a change."
}

/// The formatting option.
#[must_use]
pub fn opt_formatting() -> &'static str {
    "Text formatting"
}

/// Its hover.
#[must_use]
pub fn opt_formatting_hover() -> &'static str {
    "Also list words that stayed the same but changed size, weight or colour."
}

/// The appearance option.
#[must_use]
pub fn opt_appearance() -> &'static str {
    "Appearance"
}

/// Its hover.
#[must_use]
pub fn opt_appearance_hover() -> &'static str {
    "Draw both versions and outline every area that looks different: \
     drawings, images and text alike. Pages are paired by how they look."
}

/// Label of the older version's page range.
#[must_use]
pub fn old_pages() -> &'static str {
    "Older pages"
}

/// Label of the newer version's page range.
#[must_use]
pub fn new_pages() -> &'static str {
    "This document's pages"
}

/// Hover of both page-range fields.
#[must_use]
pub fn pages_hover() -> &'static str {
    "Pages to compare, such as 1-3, 5. Leave empty for all pages."
}

/// A page range that does not parse.
#[must_use]
pub fn pages_invalid(which: &str) -> String {
    format!("{which}: that is not a page range in this document.")
}

/// The button that runs the comparison.
#[must_use]
pub fn run() -> &'static str {
    "Compare"
}

/// Its hover.
#[must_use]
pub fn run_hover() -> &'static str {
    "Compare the older version with this document, including edits not yet saved."
}

/// Hover of the greyed Compare button before an older version is chosen.
#[must_use]
pub fn run_needs_older() -> &'static str {
    "Choose the older version first."
}

/// Hover of the greyed Compare button with no comparison ticked.
#[must_use]
pub fn run_needs_option() -> &'static str {
    "Tick Text or Appearance."
}

/// Shown while the comparison runs.
#[must_use]
pub fn running() -> &'static str {
    "Comparing\u{2026}"
}

/// The older file would not open.
#[must_use]
pub fn older_unreadable(why: &str) -> String {
    format!("The older version could not be opened: {why}")
}

/// This document could not be read for the comparison.
#[must_use]
pub fn newer_unreadable(why: &str) -> String {
    format!("This document could not be read for the comparison: {why}")
}

/// A side forbids copying its text.
#[must_use]
pub fn copy_forbidden(older: bool) -> &'static str {
    if older {
        "The older version's security settings do not allow copying its \
         text, so its text cannot be compared."
    } else {
        "This document's security settings do not allow copying its text, \
         so its text cannot be compared."
    }
}

/// The text comparison failed for another reason.
#[must_use]
pub fn text_failed(why: &str) -> String {
    format!("The text could not be compared: {why}")
}

/// The comparison's worker ended without an answer.
#[must_use]
pub fn worker_lost() -> &'static str {
    "the comparison stopped unexpectedly"
}

/// The appearance comparison failed.
#[must_use]
pub fn appearance_failed(why: &str) -> String {
    format!("The appearance could not be compared: {why}")
}

/// Nothing differs.
#[must_use]
pub fn identical() -> &'static str {
    "No differences found."
}

/// The count line over the list.
#[must_use]
pub fn counts(text: usize, regions: usize, pages: usize) -> String {
    let mut parts = Vec::new();
    if text > 0 {
        parts.push(plural(text, "text change", "text changes"));
    }
    if regions > 0 {
        parts.push(plural(regions, "changed area", "changed areas"));
    }
    if pages > 0 {
        parts.push(plural(
            pages,
            "added or removed page",
            "added or removed pages",
        ));
    }
    parts.join(", ")
}

/// Kind label of an insertion.
#[must_use]
pub fn kind_inserted() -> &'static str {
    "Inserted"
}

/// Kind label of a deletion.
#[must_use]
pub fn kind_deleted() -> &'static str {
    "Deleted"
}

/// Kind label of a replacement.
#[must_use]
pub fn kind_replaced() -> &'static str {
    "Replaced"
}

/// Kind label of a formatting change.
#[must_use]
pub fn kind_formatting() -> &'static str {
    "Formatting"
}

/// Kind label of a changed area.
#[must_use]
pub fn kind_area() -> &'static str {
    "Appearance"
}

/// Kind label of a page only in this document.
#[must_use]
pub fn kind_page_added() -> &'static str {
    "Page added"
}

/// Kind label of a page only in the older version.
#[must_use]
pub fn kind_page_removed() -> &'static str {
    "Page removed"
}

/// A replacement's text, older then newer.
#[must_use]
pub fn replaced_line(old: &str, new: &str) -> String {
    format!("\u{201c}{old}\u{201d} became \u{201c}{new}\u{201d}")
}

/// A quoted stretch of words.
#[must_use]
pub fn quoted(words: &str) -> String {
    format!("\u{201c}{words}\u{201d}")
}

/// What a formatting change changed.
#[must_use]
pub fn formatting_line(words: &str, what: &[String]) -> String {
    format!("\u{201c}{words}\u{201d}: {}", what.join(", "))
}

/// The size part of a formatting change.
#[must_use]
pub fn fmt_size(old: f32, new: f32) -> String {
    format!("size {old:.1} to {new:.1} pt")
}

/// The weight part of a formatting change.
#[must_use]
pub fn fmt_weight(old: u16, new: u16) -> String {
    format!("weight {old} to {new}")
}

/// The colour part of a formatting change.
#[must_use]
pub fn fmt_colour() -> &'static str {
    "colour"
}

/// A changed area's line.
#[must_use]
pub fn area_line(old_page: usize, pixels: u64) -> String {
    format!("Looks different from older page {old_page} ({pixels} pixels)")
}

/// A page only in this document.
#[must_use]
pub fn page_added_line() -> &'static str {
    "No page of the older version looks like it."
}

/// A page only in the older version.
#[must_use]
pub fn page_removed_line(old_page: usize) -> String {
    format!("Older page {old_page} has no matching page here.")
}

/// A page the appearance comparison paired out of order.
#[must_use]
pub fn page_moved(old_page: usize, new_page: usize) -> String {
    format!("Older page {old_page} looks most like page {new_page} here, so they were compared.")
}

/// A pair the appearance comparison could not draw.
#[must_use]
pub fn pair_failed(new_page: usize, why: &str) -> String {
    format!("Page {new_page} could not be drawn for the comparison: {why}")
}

/// A pair whose pages differ in size.
#[must_use]
pub fn size_differs(new_page: usize) -> String {
    format!(
        "Page {new_page} is a different size from its older page; the areas are measured from the top-left corner."
    )
}

/// The row's page column.
#[must_use]
pub fn page_cell(page: usize) -> String {
    format!("p. {page}")
}

/// Hover of a row that jumps.
#[must_use]
pub fn row_hover() -> &'static str {
    "Show this change on the page."
}

/// Hover of a row whose words are only in the older version.
#[must_use]
pub fn row_older_only() -> &'static str {
    "These words are only in the older version, so there is nothing to show here."
}

/// The hover of a deleted-text row: what a click does and where it lands.
#[must_use]
pub fn deleted_row_hover() -> String {
    format!("{} {}", row_hover(), deleted_hover())
}

/// Hover addition for a deletion: where its mark goes.
#[must_use]
pub fn deleted_hover() -> &'static str {
    "Shown where the words came in the page's reading order, which on a \
     drawing can be away from where they were drawn."
}

/// Text that could not be read on some pages.
#[must_use]
pub fn unreadable(old: usize, new: usize) -> String {
    format!(
        "Text could not be read on {old} older and {new} newer page(s); \
         their words are missing from the comparison."
    )
}

/// The diff hit its work limit.
#[must_use]
pub fn truncated() -> &'static str {
    "The text differs so much that part of it is listed as one large replacement."
}

/// How the appearance comparison measured.
#[must_use]
pub fn appearance_basis(dpi: f32, threshold: u8) -> String {
    format!(
        "Appearance compared at {dpi:.0} dpi; a pixel counts as changed when \
         it differs by more than {threshold} of 255."
    )
}

/// No pages could be paired by appearance.
#[must_use]
pub fn nothing_matched() -> &'static str {
    "No page of one version looked like any page of the other, so no areas could be compared."
}

/// Results belong to another open document.
#[must_use]
pub fn other_document(name: &str) -> String {
    format!("These results are for {name}. Switch to it to show or mark them.")
}

/// The document changed since the comparison.
#[must_use]
pub fn stale() -> &'static str {
    "This document has changed since the comparison. Compare again to mark the changes."
}

/// The summary-page option.
#[must_use]
pub fn opt_summary() -> &'static str {
    "Add a summary page"
}

/// Its hover.
#[must_use]
pub fn opt_summary_hover() -> &'static str {
    "Insert a page before page 1 listing the counts and the colour of each kind of change."
}

/// The button that marks the changes as comments.
#[must_use]
pub fn mark() -> &'static str {
    "Mark changes as comments"
}

/// Its hover.
#[must_use]
pub fn mark_hover() -> &'static str {
    "Add a comment for every change to this document: blue for inserted, \
     orange for replaced or changed areas, red for deleted, purple for \
     formatting. One undo removes them all."
}

/// Hover of the greyed mark button when nothing differs.
#[must_use]
pub fn mark_nothing() -> &'static str {
    "There are no changes to mark."
}

/// The labels the summary page names the two versions by.
#[must_use]
pub fn label_older(name: &str) -> String {
    format!("Older: {name}")
}

/// The newer label.
#[must_use]
pub fn label_newer(name: &str) -> String {
    format!("Newer: {name}")
}

/// A change the engine could not mark.
#[must_use]
pub fn unmarked(count: usize) -> String {
    format!("{count} change(s) could not be marked; the comparison panel still lists them.")
}

/// Marking stopped after some marks were written.
#[must_use]
pub fn mark_failed(why: &str) -> String {
    format!(
        "Marking stopped part way, and the marks made so far were kept: {why}. One Undo removes them."
    )
}

/// Status line after marking.
#[must_use]
pub fn marked(annotations: usize) -> String {
    plural(
        annotations,
        "comment added for the comparison",
        "comments added for the comparison",
    )
}

/// The note on an appearance mark.
#[must_use]
pub fn area_note(old_page: usize, pixels: u64) -> String {
    format!("Changed appearance (compared with older page {old_page}; {pixels} pixels)")
}

/// The note on an added page's frame.
#[must_use]
pub fn page_added_note() -> &'static str {
    "Inserted page: no page of the older version looks like it."
}

/// The appearance comparison's paragraph on the summary page: how many
/// areas changed, how each is marked, then every `note` the panel lists.
#[must_use]
pub fn summary_note(areas: usize, notes: &[String]) -> String {
    let mut text = format!(
        "Appearance comparison: {areas} changed area(s), each an orange cloud with a note; a blue frame marks a page with no match in the older version."
    );
    for note in notes {
        text.push('\n');
        text.push_str(note);
    }
    text
}
