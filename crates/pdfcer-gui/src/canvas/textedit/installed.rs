//! # `canvas::textedit::installed` — every typed edit may extend the run's
//! embedded subset from the same face in the font folders
//!
//! Contract: [`augmented`] is applied to the options of every commit, preview
//! and keystroke query of a typed edit, so the three agree on which keys the
//! subset can take. The folders are this computer's font folder plus the
//! operator's own (`app::dispatch::fonts::augment_folders`). [`laddered`] is
//! applied only to a commit the operator made by pressing a workaround offer;
//! `docs/modules/pdfcer-gui-base/editmodel/replacementfaces.md` says why.

use pdfcer_core::text_edit::EditOptions;
use pdfcer_gui_base::editmodel::installedfaces::{self, InstalledFaces};
use pdfcer_gui_base::editmodel::replacementfaces;

use crate::app::actions::Action;
use crate::app::prefs::Prefs;
use crate::app::state::OpenDoc;

/// `options`, able to extend the run's subset from a same-name installed face.
#[must_use]
pub fn augmented(doc: &OpenDoc, options: EditOptions) -> EditOptions {
    installedfaces::augmenting(options, warm(&doc.prefs))
}

/// `options` with the same folders as the replacement-face ladder's source.
#[must_use]
pub fn laddered(doc: &OpenDoc, options: EditOptions) -> EditOptions {
    let folders = crate::app::dispatch::fonts::augment_folders(&doc.prefs);
    replacementfaces::laddered(options, replacementfaces::for_folders(&folders))
}

/// The commit that types `character` at the caret of the open draft on
/// `page`'s `run`, with the replacement-face ladder to set it in an installed
/// face. `None` when no draft is open on that run, a face is already planned
/// for it (`super::reface`), or there are no font folders.
#[must_use]
pub fn letter_commit(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    run: usize,
    character: char,
) -> Option<Action> {
    let draft = super::read(ctx)?;
    let super::Anchor::Run { run: at, original } = &draft.anchor else {
        return None;
    };
    if draft.page != page || *at != run || super::reface::face(ctx, page, run).is_some() {
        return None;
    }
    if crate::app::dispatch::fonts::augment_folders(&doc.prefs).is_empty() {
        return None;
    }
    let mut text: Vec<char> = draft.text.chars().collect();
    text.insert(draft.caret.min(text.len()), character);
    Some(Action::CommitTextEdit {
        page,
        run,
        original: original.clone(),
        replacement: text.into_iter().collect(),
        reface: None,
        workarounds: true,
    })
}

/// The installed faces for `prefs`' folders, indexed in the background the
/// first time they are asked for.
#[must_use]
pub fn warm(prefs: &Prefs) -> &'static InstalledFaces {
    installedfaces::for_folders(&crate::app::dispatch::fonts::augment_folders(prefs))
}
