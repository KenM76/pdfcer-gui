//! # `app::actions::movemixed` — content, form fields and annotations moved as one step
//!
//! Commits [`VectorAction::MoveMixed`](super::VectorAction::MoveMixed): what
//! Align and Distribute raise when the selection holds form fields or
//! annotations, parts of several placed drawings, or parts beside page
//! objects, or leaves something selected out. The engine moves several objects
//! in one command, the parts of one drawing in another, but a widget or an
//! annotation only one at a time, so this is one `move_objects_each` (when
//! there is content), one `move_objects_each_in_form` per drawing, one
//! `move_widget` per widget, one
//! `move_annotation` per markup or link and one `place_dimension` per ce
//! dimension, then `coalesce_last` over every command pushed. Filed as G189
//! (widgets) and G190 (annotations); batch verbs retire it.
//!
//! A ce dimension is placed by the rule a pointer drag uses
//! (`canvas::dimdrag::placed`), so it stays on what it measures: only its line
//! and value move, and its box may land short of where Align aimed it.
//!
//! ## A refusal does not roll the others back
//!
//! What was applied stays applied and is folded into one undo step (see
//! [`super::foldeach`]); the status line says how many moved and why the rest
//! did not. A move of more commands than one undo step folds is refused
//! before anything moves.

use pdfcer_core::edit::{CommandKind, EditError, EditSession};
use pdfcer_gui_base::annotkind::AnnotKind;
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
        Ok::<_, Refused>(said)
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

/// Why nothing at all moved: the engine's refusal, or the shell's.
enum Refused {
    Engine(EditError),
    Said(String),
}

impl From<EditError> for Refused {
    fn from(e: EditError) -> Self {
        Self::Engine(e)
    }
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Engine(e) => e.fmt(f),
            Self::Said(s) => f.write_str(s),
        }
    }
}

impl crate::app::unlock::Refusal for Refused {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Engine(e) if crate::app::unlock::Refusal::encrypted(e))
    }
}

/// Each kind's run, in the order they were applied.
struct Runs {
    drawings: Run<EditError>,
    parts: usize,
    widgets: Run<EditError>,
    annots: Run<Refused>,
    placed_dimension: bool,
}

/// Run the verbs and fold them. `Err` only when nothing at all was applied.
fn commit(
    session: &mut EditSession,
    page: usize,
    moves: &Moves<'_>,
) -> Result<(Vec<String>, Report), Refused> {
    within_undo_depth(
        usize::from(!moves.objects.is_empty())
            + moves.leaves.len()
            + moves.widgets.len()
            + moves.annots.len(),
    )?;
    let mut said = moves.notes.to_vec();
    if !moves.objects.is_empty() {
        said.extend(session.move_objects_each(page, moves.objects)?);
    }
    let mut runs = run_each(session, page, moves);
    let pushed = usize::from(!moves.objects.is_empty())
        + runs.drawings.pushed
        + runs.widgets.pushed
        + runs.annots.pushed;
    if pushed == 0 {
        let first = (runs.drawings.failure.take().map(Refused::Engine))
            .or_else(|| runs.widgets.failure.take().map(Refused::Engine))
            .or_else(|| runs.annots.failure.take());
        if let Some(e) = first {
            return Err(e);
        }
    }
    let kind = if !moves.objects.is_empty() || runs.drawings.pushed > 0 {
        CommandKind::MoveObject
    } else if runs.widgets.pushed > 0 {
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
        widgets: runs.widgets.pushed,
        annots: runs.annots.pushed,
        folded,
    };
    Ok((said, report))
}

/// One command per drawing, per widget and per annotation, each kind tried
/// to the end whatever the others refuse.
fn run_each(session: &mut EditSession, page: usize, moves: &Moves<'_>) -> Runs {
    let mut parts = 0_usize;
    let drawings = each(session, moves.leaves, OnRefusal::Continue, |s, group| {
        let outcome = s.move_objects_each_in_form(page, group)?;
        parts += group.len();
        Ok::<_, EditError>(Some(outcome.disclosures))
    });
    let widgets = each(
        session,
        moves.widgets,
        OnRefusal::Continue,
        |s, (field, dx, dy)| {
            let outcome = s.move_widget(&field.field, field.widget, *dx, *dy)?;
            let left = outcome.siblings_left_behind;
            Ok::<_, EditError>(Some(
                (left > 0)
                    .then(|| crate::text::forms::widget_siblings_unmoved(left))
                    .into_iter()
                    .collect(),
            ))
        },
    );
    let mut placed_dimension = false;
    let annots = each(
        session,
        moves.annots,
        OnRefusal::Continue,
        |s, (target, dx, dy)| {
            let popup_left = move_annot(s, target, *dx, *dy)?;
            placed_dimension |= target.kind == AnnotKind::CeDimension;
            Ok::<_, Refused>(Some(
                popup_left
                    .then(crate::text::markup::popup_left_behind)
                    .into_iter()
                    .collect(),
            ))
        },
    );
    Runs {
        drawings,
        parts,
        widgets,
        annots,
        placed_dimension,
    }
}

/// What the status line adds after the verbs' own disclosures: each kind
/// that moved only partly, ce dimensions staying attached, an unfolded step.
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
            runs.widgets.pushed,
            moves.widgets.len(),
            &e.to_string(),
        ));
    }
    if let Some(e) = &runs.annots.failure {
        said.push(t::annots_partly_moved(
            runs.annots.pushed,
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

/// Move one markup or link, or place one ce dimension. `Ok(true)` when a
/// markup's pop-up note stayed where it was.
fn move_annot(
    session: &mut EditSession,
    target: &AnnotTarget,
    dx: f64,
    dy: f64,
) -> Result<bool, Refused> {
    match target.kind {
        AnnotKind::Markup => Ok(session
            .move_annotation(target.id, dx, dy)?
            .popup_left_behind
            .is_some()),
        AnnotKind::CeDimension => {
            // `place_dimension` does not read the lock flag; `move_annotation`
            // does, and both kinds must agree.
            if target.locked {
                return Err(Refused::Engine(EditError::AnnotationLocked {
                    id: target.id,
                    subtype: target.subtype.clone(),
                }));
            }
            let model = session.dimension_model();
            let record = model
                .dimensions()
                .iter()
                .find(|r| r.annot == Some(target.id))
                .ok_or_else(|| Refused::Said(t::dimension_not_found().to_owned()))?;
            let (_, offset, text_along) = crate::canvas::dimdrag::placed(&record.kind, dx, dy)
                .ok_or_else(|| Refused::Said(t::dimension_not_placeable().to_owned()))?;
            session.place_dimension(record.id, offset, text_along)?;
            Ok(false)
        }
    }
}
