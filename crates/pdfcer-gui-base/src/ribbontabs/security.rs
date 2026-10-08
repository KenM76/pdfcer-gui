//! The **Security** tab — *who may open, change or trust this file, and what
//! must never leave it?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ribbontabs/security.md`.

use super::{command, group, large};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

use crate::modecapability::EDIT_CONTENT_CONDITION as EDIT_CONTENT;

/// The Security tab.
pub fn tab() -> Tab {
    Tab::new("security", crate::text::security::tab_security())
        .with_question(ribbon::question_security())
        .with_groups([
            // ---------------------------------------------------------------
            // Security — what the file allows, who it is from, and its
            // passwords. A save transform, not an undoable content edit.
            // ---------------------------------------------------------------
            group(
                "security",
                crate::text::protect::group_file_security(),
                [
                    large("file.purge_password_values"),
                    large("file.encrypt"),
                    large("file.allow_rc4_edits").shown_when("doc.rc4"),
                    large("file.unlock").shown_when("doc.locked"),
                    large("file.permissions"),
                    large("file.sign").provided_by("signing"),
                    large("file.add_validation_evidence").provided_by("signing"),
                    large("file.add_archive_timestamp").provided_by("timestamp"),
                ],
            ),
            // ---------------------------------------------------------------
            // Protect — mark, then apply. Shown only where page content may be
            // edited, so in Edit mode alone.
            // ---------------------------------------------------------------
            group(
                "protect",
                ribbon::group_edit_protect(),
                [
                    // Large — the mockup's `Redact` big, with the two
                    // qualified redaction verbs in a column beside it. First
                    // in the group already.
                    large("edit.redact").shown_when(EDIT_CONTENT),
                    // Between mark-by-search and Apply, which is the order an
                    // operator works in: find what you can find, mark what you
                    // cannot, then apply once. Putting it after Apply would put
                    // a marking verb on the far side of the destructive one.
                    command("edit.redact_selection").shown_when(EDIT_CONTENT),
                    // The census, BETWEEN the marking verbs and Apply, and
                    // the position is the same argument as its neighbour's
                    // taken one step further. The operator's sequence is: find
                    // what you can find, mark what you cannot, **check what you
                    // could not have known about**, then apply once. It is the
                    // last thing to do before the irreversible one, because it
                    // is the only one that can tell you the document was not
                    // yet clean.
                    //
                    // Not Large. `edit.redact` is the group's big button
                    // because it is where an operator starts; this is where
                    // they finish, and a second large control in a three-deep
                    // column would make the group read as two features.
                    command("edit.offpage").shown_when(EDIT_CONTENT),
                    command("edit.redact_apply").shown_when(EDIT_CONTENT),
                ],
            ),
        ])
}
