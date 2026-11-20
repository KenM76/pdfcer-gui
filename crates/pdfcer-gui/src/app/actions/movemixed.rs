//! # `app::actions::movemixed` — content, form fields and annotations moved as one step
//!
//! Commits [`VectorAction::MoveMixed`](super::VectorAction::MoveMixed): what
//! Align and Distribute raise when the selection holds form fields or
//! annotations, parts of several placed drawings, or parts beside page
//! objects, or leaves something selected out. The engine moves several objects
//! in one command per kind: one `move_objects_each` (when there is content),
//! one `move_objects_each_in_form` per drawing, one `move_widgets_each` for
//! every widget and one `move_annotations_each` for every markup, link and ce
//! dimension, then `coalesce_last` over the commands pushed.
//!
//! The engine re-places a ce dimension by the rule a pointer drag uses, so it
//! stays on what it measures: only its line and value move, and its box may
//! land short of where Align aimed it.
//!
//! ## A refusal does not roll the other kinds back
//!
//! Each batch verb is all-or-nothing within its kind. A kind that refused
//! moved nothing; what the other kinds applied stays applied and is folded
//! into one undo step, and the status line says which kind did not move and
//! why. A move of more commands than one undo step folds is refused before
//! anything moves.

use pdfcer_core::edit::{AnnotationBatchMove, CommandKind, EditError, EditSession};
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::annotselection::AnnotTarget;
use pdfcer_gui_base::docidentity::SelectedField;

use super::foldeach::{OnRefusal, Run, each, within_undo_depth};
use crate::app::state::OpenDoc;
use crate::text::panels::align as t;

/// What one Align or Distribute moves, each with its page-space delta.
pub(super) struct Moves<'a> {
    pub objects: &'a [(usize, f64, f64)],
    /// One list per placed drawing.
    pub leaves: &'a [Vec<(usize, f64, f64)>],
    /// Said first, whatever moves.
    pub notes: &'a [String],
    pub widgets: &'a [(SelectedField, f64, f64)],
    pub annots: &'a [(AnnotTarget, f64, f64)],
}

/// A [`VectorAction::MoveMixed`](super::VectorAction::MoveMixed) of form fields
/// and annotations alone, each moved by `(dx, dy)` in page space.
pub(crate) fn move_items(
    page: usize,
    widgets: impl IntoIterator<Item = SelectedField>,
    annots: impl IntoIterator<Item = AnnotTarget>,
    (dx, dy): (f64, f64),
    gesture: &'static str,
) -> super::Action {
    super::VectorAction::MoveMixed {
        page,
        objects: Vec::new(),
        leaves: Vec::new(),
        widgets: widgets.into_iter().map(|w| (w, dx, dy)).collect(),
        annots: annots.into_iter().map(|a| (a, dx, dy)).collect(),
        notes: Vec::new(),
        gesture,
    }
    .into()
}

/// Apply one [`VectorAction::MoveMixed`](super::VectorAction::MoveMixed).
pub(super) fn apply(doc: &mut OpenDoc, page: usize, moves: &Moves<'_>, gesture: &'static str) {
    let parts: usize = moves.leaves.iter().map(Vec::len).sum();
    let n = moves.objects.len() + parts + moves.widgets.len() + moves.annots.len();
    let mut report = Report::default();
    super::apply::vector_edit_on_page(doc, "move-mixed", page, n, |session| {
        let (said, done) = commit(session, page, moves)?;
        report = done;
        Ok::<_, EditError>(said)
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        let asked: Vec<String> = moves
            .widgets
            .iter()
            .map(|(f, dx, dy)| format!("{}#{}:{dx:.1},{dy:.1}", f.field, f.widget))
            .chain(
                moves
                    .annots
                    .iter()
                    .map(|(a, dx, dy)| format!("annot{}:{dx:.1},{dy:.1}", a.id.num)),
            )
            .collect();
        format!(
            "move-mixed-applied gesture={gesture} page={page} objects={}/{} parts={}/{} \
             drawings={}/{} widgets={}/{} annots={}/{} one_step={} notes={} moves={}",
            report.objects,
            moves.objects.len(),
            report.parts,
            parts,
            report.drawings,
            moves.leaves.len(),
            report.widgets,
            moves.widgets.len(),
            report.annots,
            moves.annots.len(),
            report.folded,
            moves.notes.len(),
            asked.join(";")
        )
    });
}

/// What [`commit`] managed.
#[derive(Default)]
struct Report {
    objects: usize,
    parts: usize,
    drawings: usize,
    widgets: usize,
    annots: usize,
    folded: bool,
}

/// What one batch verb did: how many targets moved, its disclosures, and
/// the refusal that kept some or all of that kind where they were.
#[derive(Default)]
struct Batch {
    moved: usize,
    told: Vec<String>,
    failure: Option<EditError>,
}

impl Batch {
    /// Commands the batch left on the undo stack: one, or none.
    fn pushed(&self) -> usize {
        usize::from(self.moved > 0)
    }

    fn refused(e: EditError) -> Self {
        Self {
            failure: Some(e),
            ..Self::default()
        }
    }
}

/// Each kind's run, in the order they were applied.
struct Runs {
    drawings: Run<EditError>,
    parts: usize,
    widgets: Batch,
    annots: Batch,
    placed_dimension: bool,
}

/// Run the verbs and fold them. `Err` only when nothing at all was applied.
fn commit(
    session: &mut EditSession,
    page: usize,
    moves: &Moves<'_>,
) -> Result<(Vec<String>, Report), EditError> {
    let commands = usize::from(!moves.objects.is_empty())
        + moves.leaves.len()
        + usize::from(!moves.widgets.is_empty())
        + usize::from(!moves.annots.is_empty());
    within_undo_depth(commands.max(moves.widgets.len()).max(moves.annots.len()))?;
    let mut said = moves.notes.to_vec();
    if !moves.objects.is_empty() {
        said.extend(session.move_objects_each(page, moves.objects)?);
    }
    let mut runs = run_each(session, page, moves);
    let pushed = usize::from(!moves.objects.is_empty())
        + runs.drawings.pushed
        + runs.widgets.pushed()
        + runs.annots.pushed();
    if pushed == 0 {
        let first = (runs.drawings.failure.take())
            .or_else(|| runs.widgets.failure.take())
            .or_else(|| runs.annots.failure.take());
        if let Some(e) = first {
            return Err(e);
        }
    }
    let kind = if !moves.objects.is_empty() || runs.drawings.pushed > 0 {
        CommandKind::MoveObject
    } else if runs.widgets.moved > 0 {
        CommandKind::MoveWidget
    } else {
        CommandKind::MoveAnnotation
    };
    let folded = session.coalesce_last(pushed, kind);
    for told in [&runs.drawings.told, &runs.widgets.told, &runs.annots.told] {
        said.extend(told.iter().cloned());
    }
    said.extend(words(&runs, moves, folded));
    let report = Report {
        objects: moves.objects.len(),
        parts: runs.parts,
        drawings: runs.drawings.pushed,
        widgets: runs.widgets.moved,
        annots: runs.annots.moved,
        folded,
    };
    Ok((said, report))
}

/// One command per drawing, one `move_widgets_each` and one
/// `move_annotations_each`, each kind tried whatever the others refuse.
fn run_each(session: &mut EditSession, page: usize, moves: &Moves<'_>) -> Runs {
    let mut parts = 0_usize;
    let drawings = each(session, moves.leaves, OnRefusal::Continue, |s, group| {
        let outcome = s.move_objects_each_in_form(page, group)?;
        parts += group.len();
        Ok::<_, EditError>(Some(outcome.disclosures))
    });
    let widgets = move_widgets(session, moves.widgets);
    let (annots, placed_dimension) = move_annots(session, moves.annots);
    Runs {
        drawings,
        parts,
        widgets,
        annots,
        placed_dimension,
    }
}

/// Every widget by one `move_widgets_each`.
fn move_widgets(session: &mut EditSession, widgets: &[(SelectedField, f64, f64)]) -> Batch {
    if widgets.is_empty() {
        return Batch::default();
    }
    let list: Vec<(&str, usize, f64, f64)> = widgets
        .iter()
        .map(|(f, dx, dy)| (f.field.as_str(), f.widget, *dx, *dy))
        .collect();
    match session.move_widgets_each(&list) {
        Ok(done) => {
            let left: usize = done.iter().map(|m| m.siblings_left_behind).sum();
            Batch {
                moved: done.len(),
                told: (left > 0)
                    .then(|| crate::text::forms::widget_siblings_unmoved(left))
                    .into_iter()
                    .collect(),
                failure: None,
            }
        }
        Err(e) => Batch::refused(e),
    }
}

/// Every unlocked markup, link and ce dimension by one
/// `move_annotations_each`; the engine re-places a ce dimension by the drag
/// rule. The flag is whether a ce dimension was among those moved.
///
/// A locked one is kept out of the batch, which is all-or-nothing and would
/// refuse the rest for it, and is named by the engine's own refusal.
fn move_annots(session: &mut EditSession, annots: &[(AnnotTarget, f64, f64)]) -> (Batch, bool) {
    let locked =
        annots
            .iter()
            .find(|(a, ..)| a.locked)
            .map(|(a, ..)| EditError::AnnotationLocked {
                id: a.id,
                subtype: a.subtype.clone(),
            });
    let list: Vec<(ObjId, f64, f64)> = annots
        .iter()
        .filter(|(a, ..)| !a.locked)
        .map(|(a, dx, dy)| (a.id, *dx, *dy))
        .collect();
    if list.is_empty() {
        return (locked.map_or_else(Batch::default, Batch::refused), false);
    }
    let (mut batch, dimension) = move_unlocked(session, &list);
    if batch.failure.is_none() {
        batch.failure = locked;
    }
    (batch, dimension)
}

/// [`move_annots`]'s one engine call.
fn move_unlocked(session: &mut EditSession, list: &[(ObjId, f64, f64)]) -> (Batch, bool) {
    match session.move_annotations_each(list) {
        Ok(done) => {
            let popup_left = done.iter().any(|m| {
                matches!(m, AnnotationBatchMove::Annotation(a) if a.popup_left_behind.is_some())
            });
            let dimension = done
                .iter()
                .any(|m| matches!(m, AnnotationBatchMove::Dimension { .. }));
            let told = popup_left
                .then(crate::text::markup::popup_left_behind)
                .into_iter()
                .collect();
            let batch = Batch {
                moved: done.len(),
                told,
                failure: None,
            };
            (batch, dimension)
        }
        Err(e) => (Batch::refused(e), false),
    }
}

/// What the status line adds after the verbs' own disclosures: each kind
/// that did not move, ce dimensions staying attached, an unfolded step.
fn words(runs: &Runs, moves: &Moves<'_>, folded: bool) -> Vec<String> {
    let mut said = Vec::new();
    if let Some(e) = &runs.drawings.failure {
        said.push(t::drawings_partly_moved(
            runs.drawings.pushed,
            moves.leaves.len(),
            &e.to_string(),
        ));
    }
    if let Some(e) = &runs.widgets.failure {
        said.push(t::fields_partly_moved(
            0,
            moves.widgets.len(),
            &e.to_string(),
        ));
    }
    if let Some(e) = &runs.annots.failure {
        said.push(t::annots_partly_moved(
            runs.annots.moved,
            moves.annots.len(),
            &e.to_string(),
        ));
    }
    if runs.placed_dimension {
        said.push(t::dimensions_stay_attached().to_owned());
    }
    if !folded {
        said.push(t::not_one_undo().to_owned());
    }
    said
}
