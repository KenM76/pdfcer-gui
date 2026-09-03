//! # `app::dispatch::fonts` — the Tools tab's two font commands
//!
//! Split out of [`super`] under **R2** on 2026-08-28, when that file crossed
//! 1,500 lines for the fifth time.
//!
//! ## ★★ The seam, and why it is a subject rather than a size
//!
//! Both commands here open a **confirmation window over a plan**, and both can
//! answer *"there is nothing to do"* — which is what gives them a dispatch shape
//! nothing else in this shell has. Every other window-opening arm is one line,
//! `self.dialogs.open_x(&self.status)`, because a document is the only input
//! and opening always succeeds. These two **decline with a sentence**, and a
//! decline that is not recorded is an operator pressing a button and seeing
//! nothing happen.
//!
//! ★ They are otherwise mirror images, and the asymmetry is worth stating
//! because it explains why only one of them was blocked on a preference:
//! **embedding needs an operand from outside the document** — a font file on
//! the operator's disk, found through a folder list they maintain, because
//! `pdfcer-core` *"never goes looking"* — and **removal needs none**, since it
//! deletes what the document already carries.
//!
//! ## ★ The harness seam lives here, not in the preference
//!
//! [`folders`] appends a directory named by an environment variable, so
//! `ui-verify` can supply the one input an operator supplies through Settings
//! without the harness rewriting `userdata/preferences.txt` — a file that
//! belongs to whoever is running the build, and that a check has no business
//! editing. It is the same seam `PDFCER_DIAG_SAVE_PATH` is, for the same reason
//! `save_copy`'s header gives: the alternative is a check that mutates
//! persisted state and leaves it mutated.

use std::path::PathBuf;

use crate::app::prefs::Prefs;
use crate::app::state::Status;
use crate::dialogs::DialogsState;

/// A font folder supplied by the harness, in addition to the operator's.
///
/// ★ **Additional, never a replacement.** A variable that *replaced* the
/// preference would let a check pass on a build whose preference plumbing was
/// broken end to end — the harness would be testing its own environment
/// variable. Appending means the operator's folders are still read on the same
/// run, so the check exercises the real path and only adds to its input.
// ui-text-exempt: an environment variable name, never displayed.
const FONT_DIR_ENV: &str = "PDFCER_DIAG_FONT_DIR";

/// Whether this file owns `id`.
///
/// `pub(crate)` for [`super::routes::handles`]' reason: `shell::commands::reach`'s
/// reachability checker must be able to evaluate every guard arm it finds, and
/// a guard it cannot evaluate is a place commands could hide from the check
/// that exists to find them.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(id, "tools.embed_fonts" | "tools.unembed_fonts")
}

/// Where pdfcer may look for a donor font on this run.
///
/// The operator's list first, then anything the harness named. Order is search
/// order and the first match wins — see [`crate::app::prefs::fonts`] — so the
/// operator's own folders take precedence over a harness's, which is the only
/// ordering that keeps a driven run honest about what a real one would do.
#[must_use]
pub(crate) fn folders(prefs: &Prefs) -> Vec<PathBuf> {
    // ★★★ The operator's own folders, then this computer's if they asked for
    // them (`OPERATOR_REQUESTS.md` O50), then anything the harness named.
    //
    // `search_path` owns the second step, including the rule that a folder
    // already listed by hand is not added twice -- so an operator who typed
    // `C:\Windows\Fonts` into the list and then ticked the box does not spend
    // two of their sixteen slots saying one thing.
    let mut out = crate::app::prefs::fonts::search_path(&prefs.font_folders, prefs.use_os_fonts);
    if let Ok(extra) = std::env::var(FONT_DIR_ENV) {
        // Semicolon-separated, matching the platform's own `PATH` convention
        // rather than inventing one. A colon would be ambiguous with a drive
        // letter on the platform this ships on.
        for part in extra.split(';') {
            if let Some(path) = crate::app::prefs::fonts::parse_one(part) {
                crate::app::prefs::fonts::add(&mut out, &path);
            }
        }
    }
    out
}

/// Dispatch a font command.
///
/// ★★★ **`tools.embed_fonts` — registered, drawn on the Tools tab and inert for
/// the whole life of the project.** Wired 2026-08-28.
///
/// Its `SCAFFOLDED` reason quoted a premise that had expired — *"at S3 `Action`
/// carries zoom and page navigation and nothing else"* — and the entry itself
/// flagged that. Re-deriving it turned up a **second, unrecorded** dependency
/// that was the real one: `EmbedRequest::supplied` is a donor map *"the shell
/// resolved for it"*, and pdfcer never goes looking. So the command was blocked
/// on a font-folder preference that did not exist until the same day, and that
/// dependency was in neither register.
///
/// ⇒ **A blocker can be correct for the wrong reason.** It is the least visible
/// of the five ways this project's scaffold list has gone wrong: nothing about
/// such an entry looks stale, and the only thing that finds it is asking what
/// the verb's own *request struct* requires rather than whether the verb exists.
///
/// ## ★ It can decline with a sentence, and the sentence is recorded
///
/// A document whose fonts are all embedded is the **normal** case, not an
/// error, and opening a window to say so would be a modal an operator has to
/// dismiss to learn they did not need it. So the construction declines, and the
/// decline goes to `record_note` — the same channel a refused clipboard cut
/// uses, for the same reason: the operator still believes the gesture worked,
/// and silence is what would leave them believing it.
pub(crate) fn dispatch(id: &str, dialogs: &mut DialogsState, status: &Status, prefs: &Prefs) {
    // The epoch is read before a window is built, so a decline is stamped with
    // the revision the operator is looking at. Nothing here edits, so it cannot
    // move underneath.
    let Status::Open(doc) = status else {
        return;
    };
    let epoch = doc.edit_epoch;
    // ui-text-exempt: registered command ids, never displayed.
    let (declined, detail) = match id {
        "tools.embed_fonts" => {
            let folders = folders(prefs);
            let note = dialogs.open_embed_fonts(status, &folders);
            (
                note,
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("embed-fonts-declined folders={}", folders.len()),
            )
        }
        // ★★ **`tools.unembed_fonts` — the LAST scaffolded command the font
        // work reached, and the only one of the ten whose recorded blocker was
        // TRUE.**
        //
        // It said the confirmation window this needs does not exist, because
        // *"three of unembedding's four consequences are invisible on the
        // canvas"*. It did not, and they are. `dialogs::unembed` is that
        // window; there is now a fourth consequence in it that nobody had
        // written down.
        //
        // => A blocker naming a SURFACE THAT DOES NOT EXIST is the strong kind.
        // It cannot go stale by accident, because nothing makes a window appear
        // except somebody building one. That is the distinction worth keeping
        // after an audit found six of eleven entries wrong: the register is not
        // noise, it is unevenly reliable, and the reliable entries are the ones
        // whose truth condition is inside this repository.
        "tools.unembed_fonts" => (
            dialogs.open_unembed_fonts(status),
            // ui-text-exempt: diagnostic trace, never displayed.
            "unembed-fonts-declined".to_owned(),
        ),
        _ => return,
    };
    if let Some(note) = declined {
        crate::diag::trace(|| format!("{detail} detail=nothing-to-open"));
        crate::app::actions::record_note(epoch, note);
    }
}
