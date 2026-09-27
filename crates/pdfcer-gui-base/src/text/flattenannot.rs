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

/// What one annotation left in place by a page-wide burn is, in the words the
/// "kept" sentence counts it under.
#[must_use]
pub const fn kept_kind(reason: AnnotFlattenRefusalReason) -> &'static str {
    match reason {
        AnnotFlattenRefusalReason::Widget => "form field",
        AnnotFlattenRefusalReason::Link => "link",
        AnnotFlattenRefusalReason::Redact => "redaction mark",
        AnnotFlattenRefusalReason::FileAttachment => "attached file",
        AnnotFlattenRefusalReason::Media => "sound or video",
        AnnotFlattenRefusalReason::Locked => "locked markup",
        AnnotFlattenRefusalReason::HasAction => "markup that runs an action",
        AnnotFlattenRefusalReason::Hidden | AnnotFlattenRefusalReason::NoView => "hidden markup",
        AnnotFlattenRefusalReason::NoRotateOnRotatedPage => "upright markup on a rotated page",
        _ => "markup pdfcer cannot burn",
    }
}

/// The sentence naming what a page-wide burn left in place, given each kept
/// annotation's reason, or `None` when it left nothing worth naming. Pop-ups are not counted: each goes with the
/// markup that owns it.
#[must_use]
pub fn kept(reasons: impl IntoIterator<Item = AnnotFlattenRefusalReason>) -> Option<String> {
    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    for reason in reasons {
        if matches!(reason, AnnotFlattenRefusalReason::Popup) {
            continue;
        }
        let kind = kept_kind(reason);
        match counts.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, n)) => *n += 1,
            None => counts.push((kind, 1)),
        }
    }
    if counts.is_empty() {
        return None;
    }
    let parts: Vec<String> = counts.iter().map(|(k, n)| format!("{k} ({n})")).collect();
    Some(format!("Left as they were: {}.", parts.join(", ")))
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
    fn kept_counts_by_kind_and_ignores_popups() {
        let skipped = [
            AnnotFlattenRefusalReason::Link,
            AnnotFlattenRefusalReason::Popup,
            AnnotFlattenRefusalReason::Link,
            AnnotFlattenRefusalReason::Widget,
        ];
        assert_eq!(
            kept(skipped).as_deref(),
            Some("Left as they were: link (2), form field (1).")
        );
        assert_eq!(kept([AnnotFlattenRefusalReason::Popup]), None);
        assert_eq!(kept([]), None);
    }

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
