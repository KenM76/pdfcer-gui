//! # `text::redactremoved` — the words themselves, not the count of them
//!
//! Consumed by `crate::redact::disclosures::removed_text`, drawn
//! under `pdfcer_gui::text::redact::will_remove_heading` beneath the counts.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/redactremoved.md`.

/// The most entries drawn before the list is cut short.
pub const MAX_ENTRIES: usize = 200;

/// The most characters drawn from any one entry.
pub const MAX_CHARS: usize = 240;

/// The heading over the list.
#[must_use]
pub fn removed_text_heading() -> &'static str {
    "The text inside the marked regions, decoded as it was removed:"
}

/// The sentence under the heading that says what one entry is.
#[must_use]
pub fn removed_text_lead() -> &'static str {
    "One line per marked region, run together in the order it was drawn. Regions that removed identical text appear once, so there are never more lines than marks and there are often fewer."
}

/// One entry, quoted, truncated on a `char` boundary if it is enormous.
#[must_use]
pub fn removed_text_entry(text: &str) -> String {
    let count = text.chars().count();
    if count <= MAX_CHARS {
        return format!("\u{201c}{text}\u{201d}");
    }
    let shown: String = text.chars().take(MAX_CHARS).collect();
    let dropped = count - MAX_CHARS;
    format!(
        "\u{201c}{shown}\u{201d} \u{2014} and {dropped} more character(s) in this one region, not shown"
    )
}

/// The line that closes a list cut short by [`MAX_ENTRIES`].
#[must_use]
pub fn removed_text_more(hidden: usize) -> String {
    format!(
        "\u{2026} and {hidden} further region(s) whose text is not listed here. All of them will be removed; only the listing stops."
    )
}

/// Marks are about to be applied and none of them holds any text.
#[must_use]
pub fn removed_text_none() -> &'static str {
    "No text is inside the marked regions. Whatever they cover is drawn some other way \u{2014} line work, a picture, or blank paper \u{2014} and the counts above say what happens to it."
}

/// Character codes are going and the report carries no text for them.
#[must_use]
pub fn removed_text_undecodable(glyphs: u64) -> String {
    format!(
        "{glyphs} character(s) will be deleted from the page content and pdfcer can list none of them: the engine reported the removal without reporting any text for it. They are removed either way \u{2014} what is missing is the listing, not the removal."
    )
}
