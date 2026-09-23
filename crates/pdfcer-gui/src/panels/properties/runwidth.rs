//! **Fit to width** — one text run's width, typed in points (`G038`).
//!
//! Draws at the Part rung when one line of one page text object is selected.
//! The engine's verb addresses a single show operator, so a line written in
//! several is shown greyed with the reason on hover, as is a run
//! `text_run_width_refusal` turns down. What only the plan can see (kerning,
//! missing metrics) comes back as a refusal on the status line.

use egui::Ui;
use pdfcer_core::vector::{VectorEditError, VectorObject, text_run_width_refusal};

use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::canvas::selection::SelectionLevel;
use crate::panels::objects::provider::TargetId;
use crate::text::panels::properties as t;

/// The width field's region.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.text.run-width";

/// What the section can address this frame.
enum Subject {
    /// One run of a page text object, with its current ink width.
    Run {
        object: usize,
        run: usize,
        width: f64,
    },
    /// A single-line text selection the verb cannot take, and why.
    Greyed(&'static str),
}

/// Draw the section; `true` when it drew.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    let page = doc.view.page_index;
    let Some(subject) = subject(doc, page) else {
        return false;
    };
    ui.label(t::run_width_heading());
    ui.horizontal(|ui| {
        ui.label(t::run_width_label());
        match subject {
            Subject::Greyed(why) => {
                let mut shown = 0.0_f64;
                let field = egui::DragValue::new(&mut shown).suffix(t::text_size_suffix());
                let response = ui.add_enabled(false, field).on_disabled_hover_text(why);
                crate::diag::ui_rect_visible(REGION, response.rect, ui.clip_rect());
            }
            Subject::Run { object, run, width } => {
                let id = ui
                    .id()
                    .with(("run-width", page, object, run, doc.edit_epoch));
                let mut typed = ui.data(|d| d.get_temp::<f64>(id)).unwrap_or(width);
                let response = ui
                    .add(
                        egui::DragValue::new(&mut typed)
                            .speed(0.5)
                            .range(0.01..=14_400.0)
                            .fixed_decimals(2)
                            .suffix(t::text_size_suffix()),
                    )
                    .on_hover_text(t::run_width_hint());
                ui.data_mut(|d| d.insert_temp(id, typed));
                crate::diag::ui_rect_visible(REGION, response.rect, ui.clip_rect());
                if (response.drag_stopped() || response.lost_focus())
                    && (typed - width).abs() > 1e-6
                {
                    actions.push(Action::TextStyle {
                        page,
                        runs: Vec::new(),
                        change: StyleChange::RunWidth {
                            object,
                            run,
                            width: typed,
                        },
                    });
                }
            }
        }
    });
    true
}

/// The run the Part-rung selection names, or `None` when this section has
/// nothing to say (no single selected line of a text object).
fn subject(doc: &OpenDoc, page: usize) -> Option<Subject> {
    if doc.selection.level() != SelectionLevel::Part {
        return None;
    }
    let entered = doc.selection.entered_object()?;
    if entered.page != page {
        return None;
    }
    let lines = doc.selection.selected_parts_on(page, entered.object);
    let provider = doc.page_objects()?;
    let VectorObject::Text(text) = provider.object_for(entered.object)? else {
        return None;
    };
    let [line] = lines.as_slice() else {
        return Some(Subject::Greyed(t::run_width_needs_one_run()));
    };
    let TargetId::Object(object) = entered.object else {
        return Some(Subject::Greyed(t::run_width_inside_form()));
    };
    let object = usize::try_from(object).ok()?;
    let runs = provider.text_line_runs_of(entered.object, *line)?;
    if runs.len() != 1 {
        return Some(Subject::Greyed(t::run_width_needs_one_run()));
    }
    let run = runs.start;
    if let Some(refusal) = text_run_width_refusal(text, run) {
        return Some(Subject::Greyed(match refusal {
            VectorEditError::TextRunHasNoWidth { .. } => t::run_width_no_baseline(),
            _ => t::run_width_unavailable(),
        }));
    }
    let bounds = text.runs.get(run)?.bounds;
    let width = bounds.max.x - bounds.min.x;
    Some(Subject::Run {
        object,
        run,
        width: if width.is_finite() { width } else { 0.0 },
    })
}
