//! # `app::actions::stamps` — the apply half of `Save as stamp collection…`
//!
//! `OPERATOR_REQUESTS.md` **O169**. The dialog collected the plan; this module
//! asks where the file goes, calls [`crate::stamps::write::build_and_write`],
//! and says on the status bar what actually happened.
//!
//! ## Why this is its own module rather than an arm in `super::export`
//!
//! **R2**, arithmetic: `super::export` measured 1,266 lines before this work,
//! and the seam is real rather than a line count. Every other body in that file
//! is an *export* in the ordinary sense — the same pages, expressed in another
//! format, for a person or a downstream tool to read. This one **authors a new
//! kind of document**: the bytes it writes are a PDF whose structure means
//! something specific to a second application, and the reason it is hard is not
//! the encoding but the naming.
//!
//! ## ★★ The three things this module is responsible for, and nothing else
//!
//! 1. **The suggestion.** Acrobat's own user-stamps folder, discovered rather
//!    than hard-coded — see [`crate::stamps::folder`] for why `DC` is a version
//!    that will one day be something else, and why `Preflight Acrobat
//!    Continuous` sorts first among the folders beside it.
//! 2. **Creating the folder the operator named.** ★ Non-obvious and load
//!    bearing: `…\Adobe\Acrobat\DC\Stamps` **does not exist** until the day
//!    somebody makes their first custom stamp in Acrobat. Refusing to write
//!    there would make the feature fail for precisely the operator who has
//!    never made one — which is every operator this feature was built for.
//!    The directory is created only *after* he has accepted a path, so pdfcer
//!    never drops a folder into another application's preferences uninvited.
//! 3. **The receipt**, including the sentence that says **restart Acrobat**.
//!    Acrobat scans its stamps folder at startup and nowhere else; finding that
//!    out by experiment costs ten minutes of believing the feature is broken.
//!
//! Everything about *what the file contains* lives in [`crate::stamps`], which
//! is where it can be unit tested without a picker.

use std::path::PathBuf;

use crate::app::files::{self, Picked};
use crate::app::state::OpenDoc;
use crate::stamps::Plan;
use crate::text::stamps as t;

/// **Write `plan` out as an Acrobat stamp collection.**
///
/// The whole of `WriteAction::StampCollection`, and — like
/// `super::extract::extract` — a verb that goes nowhere near `vector_edit`:
/// nothing is mutated, so there is no worker to cancel, no epoch to bump and no
/// texture to drop. It is a **read** of the session that happens to produce a
/// file.
///
/// # ★ The view is the SESSION's, not the loaded file's
///
/// Decision 018, and the same choice `extract` makes. An operator who rotates
/// three sheets and then saves them as stamps must get the rotated sheets;
/// writing the file as it was opened would be a silent, plausible-looking wrong
/// answer, and this one would not surface until the stamp landed on somebody
/// else's drawing.
pub(super) fn collection(doc: &mut OpenDoc, plan: &Plan) {
    // ⚠ Should be unreachable: the dialog greys Save on exactly these two
    // conditions and explains both on hover. Kept because "the button was
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

    // ★ See the module header, point 2. The failure is deliberately ignored:
    // if the folder cannot be made, the write below fails with the operating
    // system's own sentence about the actual path, which is a better message
    // than anything this line could invent about a directory.
    if let Some(parent) = target.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // ★ The operator's settings travel whole, not a pre-built options struct.
    // `crate::stamps::write` needs BOTH funnels — it opens a session and it
    // serialises one — and handing it the settings is what lets it take both
    // rather than one. See that function's doc comment for the finding.
    match crate::stamps::write::build_and_write(&doc.session.view(), plan, &doc.settings, &target) {
        Ok(written) => {
            // ★ The receipt goes FIRST — `record_notes`' own rule: *"the first
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
///
/// Acrobat's user-stamps folder when one can be identified, and the document's
/// own folder when it cannot — the graceful nothing **R9** asks for rather than
/// a suggestion pointing at a path that does not exist on this platform.
///
/// The *name* is always [`crate::stamps::folder::suggested_file_name`]'s, and
/// deliberately readable: Acrobat writes an opaque key there
/// (`YTV_yyfVN1TzJ0_6oei-GB.pdf`, measured on this machine) and reads the
/// category from `/Info` `/Title`, so nothing depends on the filename and a
/// human browsing that folder gets `Signatures.pdf` instead.
fn suggested_path(doc: &OpenDoc, plan: &Plan) -> PathBuf {
    let name = crate::stamps::folder::suggested_file_name(&plan.category);
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
        Plan::new(2, category, &[])
    }

    #[test]
    fn the_suggested_name_comes_from_the_category() {
        // Not from the document's filename, which is the mistake the shape of
        // every other `suggested_path` in this crate invites: the category is
        // what a person browsing the stamps folder is looking for, and the
        // document a collection was cut from may be called `Sheet1.pdf`.
        let name = crate::stamps::folder::suggested_file_name(&plan("Signatures").category);
        assert_eq!(name, "Signatures.pdf");
    }

    #[test]
    fn a_category_with_a_path_separator_still_makes_a_filename() {
        // `Approved / Rejected` is an entirely reasonable thing to type into a
        // free-text field, and it is not a legal Windows filename.
        let name = crate::stamps::folder::suggested_file_name("Approved / Rejected");
        assert!(!name.contains('/'), "got {name:?}");
        assert!(name.ends_with(".pdf"), "got {name:?}");
    }
}
