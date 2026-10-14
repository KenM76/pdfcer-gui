//! # `app::actions::movemixed` — content, form fields and annotations moved as one step
//!
//! Commits [`VectorAction::MoveMixed`](super::VectorAction::MoveMixed): what
//! Align and Distribute raise when the selection holds form fields or
//! annotations. The engine moves several objects in one command but a widget
//! or an annotation only one at a time, so this is one `move_objects_each`
//! (when there is content), one `move_widget` per widget, one
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
//! The only public way to take a command back is `EditSession::undo`, which
//! leaves it on the Redo stack, where Redo would re-apply a partial move the
//! operator never saw. So what was applied stays applied, is still folded into
//! one undo step, and the status line says how many moved and why the rest
//! did not.

use pdfcer_core::edit::{CommandKind, EditError, EditSession};
use pdfcer_gui_base::annotkind::AnnotKind;
use pdfcer_gui_base::annotselection::AnnotTarget;
use pdfcer_gui_base::docidentity::SelectedField;

use crate::app::state::OpenDoc;
use crate::text::panels::align as t;

/// What one Align or Distribute moves, each with its page-space delta.
pub(super) struct Moves<'a> {
    pub objects: &'a [(usize, f64, f64)],
    pub in_form: bool,
    pub widgets: &'a [(SelectedField, f64, f64)],
    pub annots: &'a [(AnnotTarget, f64, f64)],
}

/// Apply one [`VectorAction::MoveMixed`](super::VectorAction::MoveMixed).
pub(super) fn apply(doc: &mut OpenDoc, page: usize, moves: &Moves<'_>, gesture: &'static str) {
    let n = moves.objects.len() + moves.widgets.len() + moves.annots.len();
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
            "move-mixed-applied gesture={gesture} page={page} objects={}/{} widgets={}/{} \
             annots={}/{} one_step={} moves={}",
            report.objects,
            moves.objects.len(),
            report.widgets,
            moves.widgets.len(),
            report.annots,
            moves.annots.len(),
            report.folded,
            asked.join(";")
        )
    });
}

/// What [`commit`] managed.
#[derive(Default)]
struct Report {
    objects: usize,
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

/// Run the verbs and fold them. `Err` only when nothing at all was applied.
fn commit(
    session: &mut EditSession,
    page: usize,
    moves: &Moves<'_>,
) -> Result<(Vec<String>, Report), Refused> {
    let mut said = Vec::new();
    let mut pushed = 0_usize;
    if !moves.objects.is_empty() {
        if moves.in_form {
            said.extend(
                session
                    .move_objects_each_in_form(page, moves.objects)?
                    .disclosures,
            );
        } else {
            said.extend(session.move_objects_each(page, moves.objects)?);
        }
        pushed += 1;
    }
    let mut first_refusal: Option<Refused> = None;
    let mut widget_refusal: Option<String> = None;
    let mut moved_widgets = 0_usize;
    for (field, dx, dy) in moves.widgets {
        match session.move_widget(&field.field, field.widget, *dx, *dy) {
            Ok(outcome) => {
                moved_widgets += 1;
                pushed += 1;
                if outcome.siblings_left_behind > 0 {
                    said.push(crate::text::forms::widget_siblings_unmoved(
                        outcome.siblings_left_behind,
                    ));
                }
            }
            Err(e) => {
                widget_refusal.get_or_insert_with(|| e.to_string());
                first_refusal.get_or_insert(Refused::Engine(e));
            }
        }
    }
    let mut annot_refusal: Option<String> = None;
    let mut moved_annots = 0_usize;
    let mut placed_dimension = false;
    for (target, dx, dy) in moves.annots {
        match move_annot(session, target, *dx, *dy) {
            Ok(popup_left) => {
                moved_annots += 1;
                pushed += 1;
                placed_dimension |= target.kind == AnnotKind::CeDimension;
                if popup_left {
                    said.push(crate::text::markup::popup_left_behind());
                }
            }
            Err(e) => {
                annot_refusal.get_or_insert_with(|| e.to_string());
                first_refusal.get_or_insert(e);
            }
        }
    }
    if pushed == 0
        && let Some(e) = first_refusal
    {
        return Err(e);
    }
    let kind = if !moves.objects.is_empty() {
        CommandKind::MoveObject
    } else if moved_widgets > 0 {
        CommandKind::MoveWidget
    } else {
        CommandKind::MoveAnnotation
    };
    let folded = session.coalesce_last(pushed, kind);
    if let Some(reason) = widget_refusal {
        said.push(t::fields_partly_moved(
            moved_widgets,
            moves.widgets.len(),
            &reason,
        ));
    }
    if let Some(reason) = annot_refusal {
        said.push(t::annots_partly_moved(
            moved_annots,
            moves.annots.len(),
            &reason,
        ));
    }
    if placed_dimension {
        said.push(t::dimensions_stay_attached().to_owned());
    }
    if !folded {
        said.push(t::not_one_undo().to_owned());
    }
    Ok((
        said,
        Report {
            objects: moves.objects.len(),
            widgets: moved_widgets,
            annots: moved_annots,
            folded,
        },
    ))
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
