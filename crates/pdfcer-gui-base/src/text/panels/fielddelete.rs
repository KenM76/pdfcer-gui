//! The Properties panel's delete buttons when several form fields are
//! selected.

/// Deletes every selected field whole.
#[must_use]
pub fn delete_fields(n: usize) -> String {
    format!("Delete {n} fields")
}

/// What deleting the selected fields takes with them.
#[must_use]
pub fn delete_fields_hover() -> String {
    "Removes every selected field from the form, with all of its boxes on every page. One \
     Undo puts them all back."
        .to_owned()
}

/// Deletes only the selected boxes, as the Delete key does.
#[must_use]
pub fn delete_boxes(n: usize) -> String {
    format!("Delete {n} selected boxes")
}

/// What deleting only the selected boxes does, and when it does more.
#[must_use]
pub fn delete_boxes_hover() -> String {
    "Removes only the selected boxes. A field drawn elsewhere stays in the form; a field \
     whose last box goes leaves it. One Undo puts them all back."
        .to_owned()
}
