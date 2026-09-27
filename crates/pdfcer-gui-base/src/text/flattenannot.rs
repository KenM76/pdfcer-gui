//! # `text::flattenannot` — what making a markup part of the page says
//!
//! The status-bar sentence after `EditSession::flatten_annotations` burns a
//! markup, and the decline when it refuses one.

use pdfcer_core::edit::{AnnotFlattenOutcome, AnnotFlattenRefusalReason};

/// Why a markup could not be made part of the page, one sentence per reason.
#[must_use]
pub const fn refused(reason: AnnotFlattenRefusalReason) -> &'static str {
    match reason {
        AnnotFlattenRefusalReason::NotIndirect => {
            "This markup is stored in a way pdfcer cannot remove by itself, so it was not made part of the page."
        }
        AnnotFlattenRefusalReason::Widget => {
            "This is a form field's box. Flatten the form instead; nothing was changed."
        }
        AnnotFlattenRefusalReason::Popup => {
            "This is a note's pop-up window. It goes with the markup it belongs to; nothing was changed."
        }
        AnnotFlattenRefusalReason::Link => {
            "This is a link. Making it part of the page would remove where it goes, so nothing was changed."
        }
        AnnotFlattenRefusalReason::Redact => {
            "This is a redaction mark. Apply it or delete it instead; nothing was changed."
        }
        AnnotFlattenRefusalReason::FileAttachment => {
            "This markup carries an attached file that would be lost, so nothing was changed."
        }
        AnnotFlattenRefusalReason::Media => {
            "This markup plays sound or video, which page drawing cannot, so nothing was changed."
        }
        AnnotFlattenRefusalReason::Locked => {
            "This markup is locked against deletion. Unlock it in its properties first; nothing was changed."
        }
        AnnotFlattenRefusalReason::HasAction => {
            "This markup does something when clicked, which would be lost, so nothing was changed."
        }
        AnnotFlattenRefusalReason::Hidden => {
            "This markup is hidden; making it part of the page would show it. Nothing was changed."
        }
        AnnotFlattenRefusalReason::NoView => {
            "This markup is not shown on screen; making it part of the page would show it. Nothing was changed."
        }
        AnnotFlattenRefusalReason::NoAppearance => {
            "This markup has no drawn appearance to put on the page, so nothing was changed."
        }
        AnnotFlattenRefusalReason::StateUnresolved => {
            "The file does not say which of this markup's looks is current, so nothing was changed."
        }
        AnnotFlattenRefusalReason::DegenerateAppearance => {
            "This markup's box has no area, so there is nothing to put on the page. Nothing was changed."
        }
        AnnotFlattenRefusalReason::NoRotateOnRotatedPage => {
            "This markup stays upright on a rotated page; as page drawing it would turn with the page. Nothing was changed."
        }
        _ => "pdfcer cannot make this markup part of the page, so nothing was changed.",
    }
}

/// The disclosure after a markup was burned into its page: what changed, then
/// the engine's own notes, most important first.
#[must_use]
pub fn flattened(o: &AnnotFlattenOutcome) -> Vec<String> {
    let mut lines = vec![format!(
        "Made {} markup(s) part of the page. They look the same and are no longer markups; Ctrl+Z undoes it.",
        o.flattened
    )];
    if o.layered > 0 {
        lines.push(format!(
            "{} of them stay on their layer and still hide with it.",
            o.layered
        ));
    }
    lines.extend(o.disclosures.iter().cloned());
    lines
}

/// The sentence when there was nothing to burn.
#[must_use]
pub const fn nothing_flattened() -> &'static str {
    "Nothing was made part of the page."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_known_reasons_read_as_distinct_sentences() {
        let all = [
            AnnotFlattenRefusalReason::NotIndirect,
            AnnotFlattenRefusalReason::Widget,
            AnnotFlattenRefusalReason::Popup,
            AnnotFlattenRefusalReason::Link,
            AnnotFlattenRefusalReason::Redact,
            AnnotFlattenRefusalReason::FileAttachment,
            AnnotFlattenRefusalReason::Media,
            AnnotFlattenRefusalReason::Locked,
            AnnotFlattenRefusalReason::HasAction,
            AnnotFlattenRefusalReason::Hidden,
            AnnotFlattenRefusalReason::NoView,
            AnnotFlattenRefusalReason::NoAppearance,
            AnnotFlattenRefusalReason::StateUnresolved,
            AnnotFlattenRefusalReason::DegenerateAppearance,
            AnnotFlattenRefusalReason::NoRotateOnRotatedPage,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(refused(*a), refused(*b), "{a:?} and {b:?} read the same");
            }
        }
    }
}
