//! # `text::linkedit` — the words for adding and editing a link
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/linkedit.md`.

/// The instruction on the Tool panel while the link box is being drawn.
#[must_use]
pub fn armed_instruction() -> &'static str {
    "Drag a box over what the link should cover, or click for a standard-size \
     one. Escape cancels."
}

/// The Add-link window's title.
#[must_use]
pub fn window_title() -> &'static str {
    "Add link"
}

/// Which page the new link sits on.
#[must_use]
pub fn sits_on(page_number: usize) -> String {
    format!("On page {page_number}.")
}

/// The "Goes to" heading.
#[must_use]
pub fn goes_to() -> &'static str {
    "Goes to"
}

/// The page choice.
#[must_use]
pub fn choice_page() -> &'static str {
    "A page of this document"
}

/// The web address choice.
#[must_use]
pub fn choice_web() -> &'static str {
    "A web address"
}

/// The named destination choice.
#[must_use]
pub fn choice_named() -> &'static str {
    "A named destination"
}

/// The page number field's label.
#[must_use]
pub fn page_label() -> &'static str {
    "Page"
}

/// What a page link does on arrival.
#[must_use]
pub fn page_note(page_count: usize) -> String {
    format!("Of {page_count}. Opens the whole page in the window.")
}

/// The hint inside the web address field.
#[must_use]
pub fn web_hint() -> &'static str {
    "https://"
}

/// The hint inside the destination name field.
#[must_use]
pub fn named_hint() -> &'static str {
    "Destination name"
}

/// The note under the destination name field.
#[must_use]
pub fn named_note() -> &'static str {
    "Type the name exactly as the document defines it. The link follows the \
     destination if its page moves."
}

/// The refusal while the web address is empty.
#[must_use]
pub fn needs_web() -> &'static str {
    "Type a web address."
}

/// The refusal while the web address has characters a link cannot carry.
#[must_use]
pub fn web_not_ascii() -> &'static str {
    "A web address must use plain ASCII characters. Percent-encode anything \
     else."
}

/// The refusal while the destination name is empty.
#[must_use]
pub fn needs_name() -> &'static str {
    "Type a destination name."
}

/// The border heading.
#[must_use]
pub fn border_heading() -> &'static str {
    "Border"
}

/// The visible border switch.
#[must_use]
pub fn border_visible() -> &'static str {
    "Draw a visible rectangle"
}

/// The visible border switch's tooltip.
#[must_use]
pub fn border_visible_tip() -> &'static str {
    "Off: the link is invisible on the page and prints as nothing; it is still \
     clickable in a viewer."
}

/// The width field's label.
#[must_use]
pub fn border_width() -> &'static str {
    "Width"
}

/// The width field's unit.
#[must_use]
pub fn width_suffix() -> &'static str {
    " pt"
}

/// The colour swatch's label.
#[must_use]
pub fn border_colour() -> &'static str {
    "Colour"
}

/// The dashed switch.
#[must_use]
pub fn border_dashed() -> &'static str {
    "Dashed"
}

/// The commit button in the Add-link window.
#[must_use]
pub fn add_button() -> &'static str {
    "Add link"
}

/// The cancel button.
#[must_use]
pub fn cancel_button() -> &'static str {
    "Cancel"
}

/// The Properties section's heading.
#[must_use]
pub fn section_heading() -> &'static str {
    "Link"
}

/// The Properties button that writes the destination.
#[must_use]
pub fn apply_target() -> &'static str {
    "Apply destination"
}

/// The Properties button that writes the border.
#[must_use]
pub fn apply_border() -> &'static str {
    "Apply border"
}

/// A locked link.
#[must_use]
pub fn locked() -> &'static str {
    "This link is locked by the file, so its destination and border cannot be \
     changed."
}

/// A link whose action this shell does not author, said before the
/// operator replaces it.
#[must_use]
pub fn other_action(kind: &str) -> String {
    if kind.is_empty() {
        "This link runs an action pdfcer does not edit. Applying a destination \
         below replaces it."
            .to_owned()
    } else {
        format!(
            "This link runs a {kind} action, which pdfcer does not edit. Applying \
             a destination below replaces it."
        )
    }
}

/// A link that goes nowhere.
#[must_use]
pub fn nowhere() -> &'static str {
    "This link goes nowhere yet."
}

/// The status line after a link is added.
#[must_use]
pub fn added(page_number: usize) -> String {
    format!("Added a link on page {page_number}.")
}

/// The status line after a link is re-pointed.
#[must_use]
pub fn retargeted() -> &'static str {
    "Changed where the link goes."
}

/// The disclosure when re-pointing removed an action.
#[must_use]
pub fn replaced_action(kind: &str) -> String {
    format!("Its previous {kind} action was removed.")
}

/// The status line after a border change.
#[must_use]
pub fn bordered(visible: bool) -> &'static str {
    if visible {
        "Changed the link's border."
    } else {
        "Removed the link's border."
    }
}

/// The disclosure when a border change replaced another program's drawing.
#[must_use]
pub fn appearance_replaced() -> &'static str {
    "The link's previous drawn appearance, made by another program, was \
     replaced."
}

/// `EditError::NamedDestinationNotFound`, worded.
#[must_use]
pub fn name_not_found(name: &str) -> String {
    format!("This document defines no destination named “{name}”.")
}

/// `EditError::LinkUriInvalid`, worded around the engine's reason.
#[must_use]
pub fn uri_invalid(reason: &str) -> String {
    format!("That web address cannot be used: {reason}.")
}

/// `EditError::LinkBorderWidthInvalid`, worded.
#[must_use]
pub fn width_invalid() -> &'static str {
    "A visible border needs a width above zero."
}

/// `EditError::EmptyGeometry`, worded for a link.
#[must_use]
pub fn no_area() -> &'static str {
    "The link's box has no area, so it cannot carry a visible border."
}
