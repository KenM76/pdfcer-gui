//! # `app::rc4` — edits to a document kept under RC4, on request only
//!
//! The engine refuses every edit to an RC4-encrypted document until the
//! session opts in (`EditSession::set_rc4_append(Rc4Append::Preserve)`). Opted
//! in, a save appends under the file's own RC4 key, so signatures stay valid,
//! and each edited object is encrypted again with the key stream its earlier
//! version used; the save reports how many (`rc4_keystream_reused`), and the
//! operator is told after every save.
//!
//! # Contract
//!
//! - `file.allow_rc4_edits` flips the policy for this open document only. It
//!   is a session setting, not an edit: no undo entry, no dirty marker, and a
//!   reopened file refuses again.
//! - Traces `rc4-append policy=preserve|refuse`, `rc4-disclosed` on open and
//!   `rc4-keystream-reused n=` after a save.

use std::sync::Arc;

use pdfcer_core::writer::Rc4Append;

use crate::app::state::OpenDoc;
use crate::text::rc4 as t;

/// The command that flips the policy.
pub const COMMAND: &str = "file.allow_rc4_edits"; // ui-text-exempt: command id, never displayed

/// The condition published while the open document uses RC4.
pub const CONDITION: &str = "doc.rc4"; // ui-text-exempt: condition name, never displayed

/// Whether `doc` is encrypted with RC4.
#[must_use]
pub fn uses_rc4(doc: &OpenDoc) -> bool {
    doc.session
        .document()
        .encryption()
        .is_some_and(|e| e.uses_rc4())
}

/// Whether the operator has allowed edits under RC4.
#[must_use]
pub fn allowed(doc: &OpenDoc) -> bool {
    doc.session.rc4_append() == Some(Rc4Append::Preserve)
}

/// Whether the engine will refuse every edit to `doc` for RC4 alone.
#[must_use]
pub fn refusing(doc: &OpenDoc) -> bool {
    uses_rc4(doc) && !allowed(doc)
}

/// Flip the policy and say what it now does.
pub fn toggle(doc: &mut OpenDoc) {
    let next = if allowed(doc) {
        Rc4Append::Refuse
    } else {
        Rc4Append::Preserve
    };
    doc.render_worker.cancel_and_wait();
    let Some(session) = Arc::get_mut(&mut doc.session) else {
        crate::diag::trace(|| "rc4-append-refused reason=session-borrowed".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return;
    };
    session.set_rc4_append(next);
    let preserve = next == Rc4Append::Preserve;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "rc4-append policy={}",
            if preserve { "preserve" } else { "refuse" }
        )
    });
    crate::app::actions::record_note(doc.edit_epoch, t::policy(preserve).to_owned());
}

/// The sentence a save under RC4 owes, or `None` for any other save.
#[must_use]
pub fn after_save(reused: Option<usize>) -> Option<String> {
    let n = reused?;
    crate::diag::trace(|| format!("rc4-keystream-reused n={n}")); // ui-text-exempt: diagnostic trace, never displayed
    Some(t::reused(n))
}

/// The open-time sentence, for an RC4 file that refuses edits.
#[must_use]
pub fn on_open(doc: &OpenDoc) -> Option<String> {
    if !refusing(doc) {
        return None;
    }
    crate::diag::trace(|| "rc4-disclosed".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
    Some(t::on_open().to_owned())
}

/// Publish [`CONDITION`], and the switch's pressed state when allowed.
pub fn publish(doc: &OpenDoc, set: &mut egui_shell::commands::ConditionSet) {
    if uses_rc4(doc) {
        set.set(CONDITION);
        if allowed(doc) {
            set.set(egui_shell::ribbon::selected_condition(COMMAND));
        }
    }
}
