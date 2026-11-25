//! Settings text for what page labels do when pages move.

use crate::prefs::PageLabelPolicy;

/// The setting's name.
#[must_use]
pub const fn page_labels_title() -> &'static str {
    "Page labels when pages move"
}

/// What the standard leaves open.
#[must_use]
pub const fn page_labels_silence() -> &'static str {
    "A page can carry a label such as A-101 or iv instead of its number. The standard \
     says how labels are stored, but not what should happen to them when you move, \
     delete or insert pages."
}

/// What it affects.
#[must_use]
pub const fn page_labels_radius() -> &'static str {
    "Affects the file you save. Applies to the next page you move, delete or insert."
}

/// One option's name.
#[must_use]
pub const fn page_labels_label(choice: PageLabelPolicy) -> &'static str {
    match choice {
        PageLabelPolicy::FollowPages => "Each page keeps its label (pdfcer's default)",
        PageLabelPolicy::InSequence => "Numbering stays in page order",
    }
}

/// What one option does.
#[must_use]
pub const fn page_labels_note(choice: PageLabelPolicy) -> &'static str {
    match choice {
        PageLabelPolicy::FollowPages => {
            "Right for drawing sets: a sheet called A-103 is still A-103 after you \
             move it to the front, and deleting A-102 leaves A-101, A-103. Inserted \
             pages keep the labels they had in their own file."
        }
        PageLabelPolicy::InSequence => {
            "Right for books: page iv is whatever page is fourth. Deleting a page \
             renumbers the pages after it, and inserted pages continue the numbering \
             around them."
        }
    }
}
