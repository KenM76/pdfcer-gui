//! # `app::dispatch::batch` — the Tools ▸ Batch band's arms
//!
//! One command today, `tools.merge_files`, and it is here rather than inline in
//! [`crate::app::dispatch`] under **R2**: that file is at 1,400 of the 1,500
//! ceiling and this codebase's comment density means an inline arm carrying its
//! own argument would consume most of the remaining headroom. It is the seventh
//! such split and the reasoning is the one the six before it recorded.
//!
//! ## ★★★ What this closes, and what it says about the check that missed it
//!
//! `OPERATOR_REQUESTS.md` row **O68**: *"the Merge files and Split files
//! buttons don't do anything."*
//!
//! They were registered, drawn on the Tools tab with icons and tooltips, and
//! had no arm anywhere in the dispatcher. A press fell to the catch-all and
//! traced `command-unimplemented`, which no operator can see.
//!
//! There **is** a gate for exactly this — `every_registered_command_is_routed_
//! or_argued` — and it did not fire, because both ids were entered on the
//! `SCAFFOLDED` allow-list with a written paragraph beside them. That is the
//! finding worth more than the fix:
//!
//! > **An allow-list whose entries are prose can only ever force an
//! > explanation, never a fix.**
//!
//! The check can prove an entry is registered, has no arm, and that its reason
//! is long enough and does not merely restate the id. It cannot prove the
//! reason is *true*. An audit on 2026-08-28 found six of eleven entries wrong;
//! `tools.merge_files` was the seventh and survived that audit, sitting
//! immediately above the entry that was retired for naming a missing **host** —
//! which is what its own reason did.
//!
//! ⇒ The runtime replacement is in `tools/ui-verify`: press every registered
//! id and fail on any `command-unimplemented` line. That is a claim about the
//! running program, and no paragraph can satisfy it.
//!
//! ## The two pickers, and why there is no dialog between them
//!
//! Choose the sources, choose the destination, done. A *Combine Files* has
//! nothing left to ask — it takes every page of every source, in the order
//! given — which is the same argument `pages.merge_into` makes for opening no
//! dialog at all. A window offering options nobody has asked for would be
//! ceremony, and the two things an operator might eventually want (reorder the
//! sources, take a subset of a source's pages) are features with their own
//! designs rather than defaults this verb is missing.

use crate::app::PdfcerApp;
use crate::app::actions::Action;

/// Whether this module owns `id`.
///
/// The membership half of the membership-test guard pattern; see
/// `shell::commands::reach::guards`' paragraph on `pages::handles` for why this
/// shape is tolerated and what mitigates it. The mitigation is honoured below:
/// [`dispatch`]'s fall-through is `unreachable!` naming the id, so a member of
/// this set missing from that match panics loudly in a developer build rather
/// than silently doing nothing.
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(id, "tools.merge_files")
}

/// Do whatever this build does about a Batch command.
///
/// # ★ Both pickers run HERE, during dispatch, between frames
///
/// The same position `file.open`'s does, and for the same reason: an `rfd`
/// modal opened from inside an `egui` layout closure blocks the frame it is
/// being drawn in, leaving the window half-painted underneath a dialog the
/// operator cannot dismiss to finish it. Dispatch is outside the layout for
/// every route that reaches this arm — ribbon, QAT, chord — and the one
/// exception in this shell (`PdfcerApp::central` dispatching canvas context-menu
/// tokens from inside `CentralPanel::show`) cannot reach a Tools-tab command.
///
/// `actions` is taken and unused, deliberately: it keeps this arm's signature
/// identical to its six siblings, so a reader comparing them does not have to
/// work out whether the difference means anything. A merge raises no `Action`
/// because it changes no document — see `app::actions::merge`'s header.
pub(crate) fn dispatch(app: &mut PdfcerApp, id: &str, _actions: &mut [Action]) {
    match id {
        "tools.merge_files" => merge_files(app),
        // See `handles`: loud in a developer build, and never reached in a
        // release one because the guard and this match state the same set.
        other => unreachable!(
            // ui-text-exempt: a developer-build panic message; never rendered.
            "batch::dispatch reached with an id it does not handle: {other}"
        ),
    }
}

/// `tools.merge_files` — ask for the sources, ask where it goes, write it.
///
/// # ★★ Nothing is gated on a document being open, and that is deliberate
///
/// This is one of the handful of commands live with an empty window, and it
/// belongs there: it produces a document **from files on disk**, so requiring
/// one to be open first would be a precondition with no reason behind it. An
/// operator who has just launched pdfcer in order to combine four drawings is
/// exactly the person this command is for.
///
/// The consequence is recorded rather than hidden: with nothing open there is
/// no status row to put a sentence on, so a merge from an empty window reports
/// only to the trace. See `app::actions::merge`'s note on the gap.
fn merge_files(app: &mut PdfcerApp) {
    let sources = crate::app::files::pick_merge_sources();
    if sources.is_empty() {
        // Cancelled, or a build with no picker. Nothing is traced beyond the
        // fact: a cancelled Combine is a complete, correct, uninteresting
        // outcome, exactly as a cancelled Open is.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "merge-files-cancelled reason=no-sources-chosen".to_owned()
        });
        return;
    }

    // ★ The suggested destination sits **beside the first source**, which is
    // the only folder pdfcer has any evidence about. `Combined.pdf` names the
    // result rather than the verb, on `save_copy_suffix`'s rule.
    //
    // ★★ And it can never be one of the sources, which is the guarantee
    // `pick_save_path`'s docs ask every caller for: `Combined.pdf` is a
    // constant, and a source that happens to be called `Combined.pdf` would
    // have to be chosen again by hand at the picker. That is the difference
    // between a suggestion an operator may accept without reading and one that
    // could destroy an input.
    let suggested = sources[0].parent().map_or_else(
        || std::path::PathBuf::from(crate::text::files::merge_target_name()),
        |dir| dir.join(crate::text::files::merge_target_name()),
    );

    let crate::app::files::Picked::Path(target) = crate::app::files::pick_save_path(
        &suggested,
        crate::text::files::merge_target_dialog_title(),
    ) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "merge-files-cancelled reason=no-destination-chosen".to_owned()
        });
        return;
    };

    crate::app::actions::merge::write_merge(&app.status, &sources, &target);
}
