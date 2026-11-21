//! # `canvas::textedit::reface` — a key the run's font lacks goes into the
//! draft anyway, planned to be set in the nearest face that has it
//!
//! [`plan`] decides at the keystroke and remembers the face; [`take`] hands
//! the plan to the commit action. The commit drops the keys the run has come
//! to take since (a whole-line face change). A key no face takes is left to
//! [`super::refused`].

use pdfcer_gui_base::editmodel::reface::Reface;
use std::collections::BTreeSet;

use pdfcer_core::text_edit::format::RunRepertoire;
use pdfcer_gui_base::editmodel::{disposition, fallbackface, nearface};

use super::{Anchor, Draft};
use crate::app::state::OpenDoc;
use crate::panels::properties::face::{FaceChoice, FaceOrigin};

const KEY: &str = "textedit-reface"; // ui-text-exempt: a memory key, never displayed.

/// The face planned for the open draft's foreign keys.
#[derive(Clone)]
struct Planned {
    page: usize,
    run: usize,
    /// The revision the faces were read at.
    epoch: u64,
    /// Every key planned so far, in the order first typed.
    chars: Vec<char>,
    /// The face nearest the run's own among those taking every key.
    face: FaceChoice,
}

fn read(ctx: &egui::Context, page: usize, run: usize) -> Option<Planned> {
    ctx.data(|d| d.get_temp::<Planned>(egui::Id::new(KEY)))
        .filter(|p| p.page == page && p.run == run)
}

/// Forget the plan; called when the draft closes.
pub(super) fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Planned>(egui::Id::new(KEY)));
}

/// Whether a plan is held for `run` on `page`.
pub(super) fn is_planned(ctx: &egui::Context, page: usize, run: usize) -> bool {
    read(ctx, page, run).is_some()
}

/// The selector of the face planned for `run` on `page`, which the preview
/// hands the engine as its fallback face.
pub(super) fn face(ctx: &egui::Context, page: usize, run: usize) -> Option<String> {
    read(ctx, page, run).map(|p| p.face.selector)
}

/// Plan `missing` for the draft's run. Answers the face's name when a face
/// takes every key planned so far; the caller then inserts them.
pub(super) fn plan(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &Draft,
    missing: &[char],
    base_font: &str,
) -> Option<String> {
    let Anchor::Run { run, .. } = &draft.anchor else {
        return None;
    };
    let held = read(ctx, draft.page, *run);
    let mut chars = held.as_ref().map_or_else(Vec::new, |p| p.chars.clone());
    for c in missing {
        if !chars.contains(c) {
            chars.push(*c);
        }
    }
    if let Some(held) = held
        && held.epoch == doc.edit_epoch
        && held.chars == chars
    {
        return Some(held.face.label);
    }
    let face = nearest(doc, draft.page, *run, &chars, base_font)?;
    let line = trace_line(draft.page, *run, &chars, &face);
    crate::diag::trace(|| line);
    let label = face.label.clone();
    let planned = Planned {
        page: draft.page,
        run: *run,
        epoch: doc.edit_epoch,
        chars,
        face,
    };
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), planned));
    Some(label)
}

/// The plan for the commit: the planned keys the draft still holds, or
/// `None` when it holds none.
pub(super) fn take(ctx: &egui::Context, draft: &Draft) -> Option<Reface> {
    let Anchor::Run { run, .. } = &draft.anchor else {
        return None;
    };
    let held = read(ctx, draft.page, *run)?;
    let chars: Vec<char> = held
        .chars
        .iter()
        .copied()
        .filter(|c| draft.text.contains(*c))
        .collect();
    (!chars.is_empty()).then(|| Reface {
        face: held.face.selector.clone(),
        label: held.face.label.clone(),
        chars,
    })
}

/// The face nearest `base_font` among those that take `chars`, when the
/// engine's fallback sets every one of them in it for the run.
fn nearest(
    doc: &OpenDoc,
    page: usize,
    run: usize,
    chars: &[char],
    base_font: &str,
) -> Option<FaceChoice> {
    pick(doc, page, run, chars, base_font).filter(|face| engine_takes(doc, page, run, face, chars))
}

/// Whether the engine, given `face` as the fallback, sets every one of
/// `chars` in it for the run (`EditSession::run_repertoire_with`).
fn engine_takes(doc: &OpenDoc, page: usize, run: usize, face: &FaceChoice, chars: &[char]) -> bool {
    let Some(pin) = super::pin::resolve(doc, page, run) else {
        return false;
    };
    let options = super::installed::augmented(doc, disposition::typing())
        .with_fallback(fallbackface::named(&face.selector));
    doc.session
        .run_repertoire_with(page, "", Some(pin.span), &options)
        .is_ok_and(|rep: RunRepertoire| {
            let in_face: &BTreeSet<char> = &rep.via_fallback;
            chars.iter().all(|c| in_face.contains(c))
        })
}

/// The face nearest `base_font` among those the font preflight says take
/// every one of `chars`.
fn pick(
    doc: &OpenDoc,
    page: usize,
    run: usize,
    chars: &[char],
    base_font: &str,
) -> Option<FaceChoice> {
    let candidate: String = chars.iter().collect();
    let read = super::pin::inspect(doc, page, run)?;
    let preflight = super::pin::font_preflight(doc, page, &read, Some(&candidate))?;
    let faces = crate::panels::properties::face::choices(Some(&preflight));
    let candidates: Vec<_> = faces
        .iter()
        .map(|f| nearface::Candidate {
            base_font: &f.label,
            on_page: f.origin == FaceOrigin::OnThisPage,
        })
        .collect();
    faces
        .get(nearface::nearest(base_font, &candidates)?)
        .cloned()
}

fn trace_line(page: usize, run: usize, chars: &[char], face: &FaceChoice) -> String {
    let named: Vec<String> = chars
        .iter()
        .map(|c| format!("U+{:04X}", u32::from(*c)))
        .collect();
    format!(
        "text-edit-reface-planned page={page} run={run} characters={} face={} route=engine", // ui-text-exempt: diagnostic trace
        named.join(","),
        face.selector
    )
}
