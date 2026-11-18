//! # `canvas::pathextend` — Continue path: clicks that draw on from an end node
//!
//! Armed by `format.path_continue` with one end node selected. While armed,
//! each canvas click adds a straight segment's end; a double-click, Enter or
//! Escape commits the run as one `VectorAction::PathEnd`, Backspace takes back
//! the last click, and Escape with nothing clicked disarms. The run is egui
//! memory, never content: the rubber-band [`preview`] is the cursor, and what
//! commits renders as the saved file will.

use egui::{Pos2, Ui};
use pdfcer_core::page_tree::Page;

use crate::app::actions::{Action, VectorAction};
use crate::canvas::mapping::PageMapping;
use crate::viewer;
use pdfcer_gui_base::subactions::{NodeHost, PathEdit};

// ui-text-exempt: an `egui::Id` source string, never displayed.
const RUN_KEY: &str = "pdfcer-path-extend-run";

/// **A Continue path run in progress.** Points are PDF page space.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendRun {
    /// The page the path is on.
    pub page: usize,
    /// The path.
    pub host: NodeHost,
    /// The object-scoped end node drawn from.
    pub node: usize,
    /// Where that node is.
    pub start: (f64, f64),
    /// The clicked points, in order.
    pub points: Vec<(f64, f64)>,
}

/// The armed run, if any.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<ExtendRun> {
    ctx.data_mut(|d| d.get_temp::<ExtendRun>(egui::Id::new(RUN_KEY)))
}

/// Whether a run is armed, so a press means a click and selects nothing.
#[must_use]
pub fn active(ctx: &egui::Context) -> bool {
    read(ctx).is_some()
}

fn store(ctx: &egui::Context, run: ExtendRun) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(RUN_KEY), run));
}

fn clear(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<ExtendRun>(egui::Id::new(RUN_KEY)));
}

/// Arm a run from node `node` of `host`, which sits at `start`.
pub fn arm(ctx: &egui::Context, page: usize, host: NodeHost, node: usize, start: (f64, f64)) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "path-extend-armed page={page} node={node} x={:.2} y={:.2}",
            start.0, start.1
        )
    });
    store(
        ctx,
        ExtendRun {
            page,
            host,
            node,
            start,
            points: Vec::new(),
        },
    );
}

/// **Take a canvas click for an armed run.** Reports whether the click was
/// the run's; a click on another page is left to the canvas.
pub fn click(
    ctx: &egui::Context,
    page_index: usize,
    at: Pos2,
    double: bool,
    page: Option<&Page>,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(mut run) = read(ctx).filter(|r| r.page == page_index) else {
        return false;
    };
    if double {
        finish(ctx, actions, "double-click"); // ui-text-exempt: a trace value
        return true;
    }
    let Some(point) = page.and_then(|page| viewer::canvas_to_pdf_space(at, page)) else {
        return true;
    };
    let point = (f64::from(point.x), f64::from(point.y));
    run.points.push(point);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "path-extend-point page={page_index} n={} x={:.2} y={:.2}",
            run.points.len(),
            point.0,
            point.1
        )
    });
    store(ctx, run);
    true
}

/// Commit the run as one edit and disarm. With nothing clicked the edit
/// declines in words.
fn finish(ctx: &egui::Context, actions: &mut Vec<Action>, via: &'static str) {
    let Some(run) = read(ctx) else {
        return;
    };
    clear(ctx);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "path-extend-finish via={via} page={} node={} points={}",
            run.page,
            run.node,
            run.points.len()
        )
    });
    actions.push(Action::Vector(VectorAction::PathEnd {
        page: run.page,
        host: run.host,
        node: run.node,
        edit: PathEdit::Extend(run.points),
    }));
}

/// **Enter finishes the run; Backspace takes back its last click.** Each key
/// is consumed only while a run is armed and no text field is composing.
pub fn keys(ctx: &egui::Context, actions: &mut Vec<Action>) {
    if crate::canvas::textedit::composing(ctx) || !active(ctx) {
        return;
    }
    let consume = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
    if consume(egui::Key::Enter) {
        finish(ctx, actions, "enter"); // ui-text-exempt: a trace value
    } else if consume(egui::Key::Backspace)
        && let Some(mut run) = read(ctx)
    {
        run.points.pop();
        let left = run.points.len();
        store(ctx, run);
        crate::diag::trace(|| format!("path-extend-key key=backspace points={left}"));
    }
}

/// **Escape on an armed run**: commit what was clicked, as Enter would, or
/// disarm when nothing was. Reports whether there was a run to take the key.
pub fn escape(ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
    let Some(run) = read(ctx) else {
        return false;
    };
    if run.points.is_empty() {
        clear(ctx);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=AbandonedPathExtendRun".to_owned()
        });
    } else {
        finish(ctx, actions, "escape"); // ui-text-exempt: a trace value
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=FinishedPathExtendRun".to_owned()
        });
    }
    true
}

/// **Draw the segments clicked so far and the one the next click would add.**
pub(in crate::canvas) fn preview(
    ui: &Ui,
    page: Option<&Page>,
    page_index: usize,
    map: &PageMapping,
    pointer: Option<Pos2>,
) {
    let (Some(page), Some(run)) = (page, read(ui.ctx())) else {
        return;
    };
    if run.page != page_index {
        return;
    }
    let stroke = egui::Stroke::new(1.5, ui.visuals().selection.stroke.color);
    let screen: Vec<Pos2> = std::iter::once(run.start)
        .chain(run.points.iter().copied())
        .filter_map(|(x, y)| crate::canvas::markup::vertex::to_screen(x, y, page, map))
        .collect();
    let painter = ui.painter();
    for pair in screen.windows(2) {
        if let [a, b] = pair {
            painter.line_segment([*a, *b], stroke);
        }
    }
    for p in screen.iter().skip(1) {
        painter.circle_stroke(*p, 3.0, stroke);
    }
    if let (Some(last), Some(at)) = (screen.last(), pointer) {
        painter.line_segment([*last, map.to_screen(at)], stroke);
    }
}
