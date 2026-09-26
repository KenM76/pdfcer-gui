//! # `app::actions::stamps` — the apply half of `Save as stamp collection…`
//!
//! `OPERATOR_REQUESTS.md` **O169**. The dialog collected the plan; this module
//! asks where the file goes, calls [`crate::stamps::write::build_and_write`],
//! and says on the status bar what actually happened.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/stamps.md`.

use std::path::PathBuf;

use crate::app::files::{self, Picked};
use crate::app::state::OpenDoc;
use crate::stamps::Plan;
use crate::text::stamps as t;

/// **Write `plan` out as an Acrobat stamp collection.**
pub(super) fn collection(doc: &mut OpenDoc, plan: &Plan) {
    // ⚠ Should be unreachable: the dialog greys Save on every `Blocker` and
    // explains each on hover. Kept because "the button was
    // greyed" is a claim about a different module, and an action that trusts
    // its caller's UI is an action that writes an untitled collection the day
    // somebody adds a keyboard shortcut for it.
    if let Some(blocker) = plan.blocker() {
        let reason = match blocker {
            crate::stamps::Blocker::NoStamps => t::blocked_no_stamps(),
            crate::stamps::Blocker::NoCategory => t::blocked_no_category(),
        };
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("stamp-collection-declined reason={blocker:?}")
        });
        super::record_note(doc.edit_epoch, reason.to_owned());
        return;
    }

    let suggested = suggested_path(doc, plan);
    let target = match files::pick_save_path(&suggested, t::save_dialog_title()) {
        Picked::Path(path) => path,
        // A cancelled save is a complete, correct, uninteresting outcome —
        // `save_copy`'s wording and its reasoning.
        Picked::Cancelled => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                "stamp-collection-cancelled".to_owned()
            });
            return;
        }
        Picked::Unavailable => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                "stamp-collection-unavailable reason=no-picker-in-this-build".to_owned()
            });
            return;
        }
    };

    // See the module header, point 2. The failure is deliberately ignored:
    // if the folder cannot be made, the write below fails with the operating
    // system's own sentence about the actual path, which is a better message
    // than anything this line could invent about a directory.
    if let Some(parent) = target.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // The operator's settings travel whole, not a pre-built options struct.
    // `crate::stamps::write` needs both funnels — it opens a session and it
    // serialises one — and handing it the settings is what lets it take both
    // rather than one. See that function's doc comment.
    match crate::stamps::write::build_and_write(&doc.session.view(), plan, &doc.settings, &target) {
        Ok(written) => {
            // The receipt goes first — `record_notes`' own rule: *"the first
            // sentence is the one an operator reads if they read only one."*
            let mut notes = vec![t::saved(
                written.stamps_named,
                &target.display().to_string(),
            )];
            // ⚠ Should be unreachable, and printed loudly if it is not. See
            // `crate::text::stamps::short_by`: a shortfall means this shell
            // handed the engine a name list and a page list that disagreed,
            // which is the one defect in this feature with no visible symptom.
            let planned = plan.included();
            if written.stamps_named < planned {
                notes.push(t::short_by(planned - written.stamps_named));
            }
            super::record_notes(doc.edit_epoch, notes);
        }
        Err(failure) => {
            super::record_note(doc.edit_epoch, t::write_failed(failure.detail()));
        }
    }
}

/// The folder and filename the picker opens on.
fn suggested_path(doc: &OpenDoc, plan: &Plan) -> PathBuf {
    let name =
        crate::stamps::folder::suggested_file_name(&plan.category, t::default_category_file_stem());
    if let Some(dir) = crate::stamps::folder::user_stamps_dir() {
        return dir.join(name);
    }
    doc.stored_under()
        .and_then(|source| source.parent().map(std::path::Path::to_path_buf))
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plan with a category, over a two-page document.
    fn plan(category: &str) -> Plan {
        Plan::new(2, category, &[], t::default_stamp_name)
    }

    #[test]
    fn the_suggested_name_comes_from_the_category() {
        // Not from the document's filename, which is the mistake the shape of
        // every other `suggested_path` in this crate invites: the category is
        // what a person browsing the stamps folder is looking for, and the
        // document a collection was cut from may be called `Sheet1.pdf`.
        let name = crate::stamps::folder::suggested_file_name(
            &plan("Signatures").category,
            t::default_category_file_stem(),
        );
        assert_eq!(name, "Signatures.pdf");
    }

    #[test]
    fn a_category_with_a_path_separator_still_makes_a_filename() {
        // `Approved / Rejected` is an entirely reasonable thing to type into a
        // free-text field, and it is not a legal Windows filename.
        let name = crate::stamps::folder::suggested_file_name(
            "Approved / Rejected",
            t::default_category_file_stem(),
        );
        assert!(!name.contains('/'), "got {name:?}");
        assert!(name.ends_with(".pdf"), "got {name:?}");
    }
}
