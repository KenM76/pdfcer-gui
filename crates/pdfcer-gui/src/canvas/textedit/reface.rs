//! # `canvas::textedit::reface` — a key the run's font lacks goes into the
//! draft anyway, planned to be set in the nearest face that has it
//!
//! [`plan`] decides at the keystroke and remembers the face; [`take`] hands
//! the plan to the commit action. The commit drops the keys the run has come
//! to take since (a whole-line face change). A key no face takes is left to
//! [`super::refused`].

use pdfcer_gui_base::editmodel::nearface;
use pdfcer_gui_base::editmodel::reface::Reface;

use super::{Anchor, Draft};
use crate::app::state::OpenDoc;
use crate::panels::properties::face::{FaceChoice, FaceOrigin};

/// How many stand-in characters are carried; the face must take each.
const PLACEHOLDERS: usize = 3;

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
    /// Characters of the run's own text that can stand in for the keys.
    placeholders: Vec<char>,
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

/// Plan `missing` for the draft's run. Answers the face's name when a face
/// takes every key planned so far; the caller then inserts them.
pub(super) fn plan(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &Draft,
    missing: &[char],
    base_font: &str,
) -> Option<String> {
    let Anchor::Run { run, original } = &draft.anchor else {
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
    let mut placeholders: Vec<char> = Vec::new();
    for c in original.chars() {
        if c.is_alphanumeric() && !chars.contains(&c) && !placeholders.contains(&c) {
            placeholders.push(c);
        }
    }
    placeholders.truncate(PLACEHOLDERS);
    let face = nearest(doc, draft.page, *run, &chars, &placeholders, base_font)?;
    let line = trace_line(draft.page, *run, &chars, &face);
    crate::diag::trace(|| line);
    let label = face.label.clone();
    let planned = Planned {
        page: draft.page,
        run: *run,
        epoch: doc.edit_epoch,
        chars,
        placeholders,
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
        placeholders: held.placeholders.clone(),
    })
}

/// The face nearest `base_font` among those taking `chars` and every
/// placeholder (the first segment's neighbours are in the run's font, but a
/// placeholder token is moved to the face before it is overwritten).
fn nearest(
    doc: &OpenDoc,
    page: usize,
    run: usize,
    chars: &[char],
    placeholders: &[char],
    base_font: &str,
) -> Option<FaceChoice> {
    if placeholders.is_empty() {
        return None;
    }
    let candidate: String = chars.iter().chain(placeholders).collect();
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
        "text-edit-reface-planned page={page} run={run} characters={} face={}", // ui-text-exempt: diagnostic trace
        named.join(","),
        face.selector
    )
}
