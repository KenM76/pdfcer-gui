//! The snapshot region's commands. Each is greyed until a box is drawn, and
//! says so on hover.

use super::CommandText;

/// `edit.delete_in_region` — delete the objects the region takes.
#[must_use]
pub const fn edit_delete_in_region() -> CommandText {
    CommandText::new(
        "Delete in region",
        "Delete the objects on the page inside the region, or touching it if \
         Delete takes is set to Touching. Undo brings them back. Greyed until you \
         draw a marquee first.",
    )
}

/// `edit.redact_region` — mark the region for redaction.
#[must_use]
pub const fn edit_redact_region() -> CommandText {
    CommandText::new(
        "Redact region",
        "Mark the region for redaction. Anything it covers or crosses goes when \
         you apply redactions; nothing is removed before then. Greyed until you draw \
         a marquee first.",
    )
}

/// `edit.redact_outside_region` — mark everything but the region.
#[must_use]
pub const fn edit_redact_outside_region() -> CommandText {
    CommandText::new(
        "Redact outside region\u{2026}",
        "Mark everything on the page outside the region for redaction, on this \
         page or a range of pages the same size. A window asks first. Greyed until \
         you draw a marquee first.",
    )
}

/// `pages.crop_to_region` — open Crop seeded with the region.
#[must_use]
pub const fn pages_crop_to_region() -> CommandText {
    CommandText::new(
        "Crop to region\u{2026}",
        "Open Crop with the region as the crop, to check or adjust before it is \
         applied. Crop hides what is outside; it does not remove it. Greyed until you \
         draw a marquee first.",
    )
}
