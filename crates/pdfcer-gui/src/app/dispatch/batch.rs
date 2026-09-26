//! # `app::dispatch::batch` — the Tools ▸ Batch band's arms
//!
//! One command today, `tools.merge_files`, and it is here rather than inline in
//! [`crate::app::dispatch`] under **R2**: that file is at 1,400 of the 1,500
//! ceiling and this codebase's comment density means an inline arm carrying its
//! own argument would consume most of the remaining headroom. It is the seventh
//! such split and the reasoning is the one the six before it recorded.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/batch.md`.

use crate::app::PdfcerApp;
use crate::app::actions::Action;

/// Whether this module owns `id`.
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(id, "tools.merge_files")
}

/// Do whatever this build does about a Batch command.
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

    // The suggested destination sits **beside the first source**, which is
    // the only folder pdfcer has any evidence about. `Combined.pdf` names the
    // result rather than the verb, on `save_copy_suffix`'s rule.
    //
    // And it can never be one of the sources, which is the guarantee
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
