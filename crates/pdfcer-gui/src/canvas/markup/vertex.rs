//! # `canvas::markup::vertex` — polyline and polygon, and the gesture the
//! operator has to end
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/markup/vertex.md`.

use egui::{Pos2, Ui};
use pdfcer_core::page_tree::Page;

use super::{Geometry, MarkupKind};
use crate::app::actions::Action;
use crate::canvas::mapping::PageMapping;
use crate::canvas::tool;
use crate::viewer;

/// Where the in-progress vertex run lives between frames.
///
/// See §2. An `egui::Id` source string, never displayed.
// ui-text-exempt: an `egui::Id` source string, never displayed.
const VERTEX_MEMORY_KEY: &str = "pdfcer-markup-vertex-run";

/// **A run of clicked vertices, in progress.**
#[derive(Debug, Clone, PartialEq)]
pub struct VertexRun {
    /// The page the run is being drawn on — the staleness key, and the page the
    /// annotation is authored onto.
    pub page_index: usize,
    /// Which kind the run was begun with. Not a duplicate of
    /// `CanvasTool::Markup(kind)`: it is this state's record of the kind it was
    /// last *synchronised to*, and the difference between the two is exactly
    /// what [`load`] reacts to.
    pub kind: MarkupKind,
    /// The vertices, PDF user space, in click order. Never carries a polygon's
    /// closing vertex — see [`super::Geometry::Vertices`].
    pub vertices: Vec<(f64, f64)>,
}

impl VertexRun {
    /// A fresh run for `kind` on `page_index`.
    #[must_use]
    fn new(page_index: usize, kind: MarkupKind) -> Self {
        Self {
            page_index,
            kind,
            vertices: Vec::new(),
        }
    }

    /// Whether there is anything to abandon.
    #[must_use]
    pub fn in_progress(&self) -> bool {
        !self.vertices.is_empty()
    }
}

/// Read the run without creating one.
pub fn read(ctx: &egui::Context) -> Option<VertexRun> {
    ctx.data_mut(|d| d.get_temp::<VertexRun>(egui::Id::new(VERTEX_MEMORY_KEY)))
}

/// Write the run back.
fn store(ctx: &egui::Context, run: VertexRun) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(VERTEX_MEMORY_KEY), run));
}

/// Forget the run entirely.
fn clear(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<VertexRun>(egui::Id::new(VERTEX_MEMORY_KEY)));
}

/// Read the run, building one that already agrees with `kind` and `page_index`
/// if there is none — and discarding one that does not.
fn load(ctx: &egui::Context, page_index: usize, kind: MarkupKind) -> VertexRun {
    let Some(run) = read(ctx) else {
        return VertexRun::new(page_index, kind);
    };
    if run.kind != kind || run.page_index != page_index {
        return VertexRun::new(page_index, kind);
    }
    run
}

/// Convert one **canvas-space** click into a **PDF user-space** vertex.
#[must_use]
pub fn page_point(at: Pos2, page: &Page) -> Option<(f64, f64)> {
    let p = viewer::canvas_to_pdf_space(at, page)?;
    Some((f64::from(p.x), f64::from(p.y)))
}

/// **The one commit path**, reached by both endings.
pub(crate) fn commit(run: &mut VertexRun, actions: &mut Vec<Action>, pen: super::pen::Pen) -> bool {
    let geometry = Geometry::Vertices(run.vertices.clone());
    match super::action(run.kind, run.page_index, geometry, pen) {
        Ok(raised) => {
            let first = run.vertices.first().copied().unwrap_or_default();
            let last = run.vertices.last().copied().unwrap_or_default();
            super::trace_commit(
                run.kind,
                run.page_index,
                // The vertex COUNT and both ends, not a success flag: a run that
                // lost its first click, gained a duplicate closing point, or was
                // authored on the page the operator had paged to rather than the
                // one they drew on are the three things that can be wrong here,
                // and all three are visible on this line.
                &format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "vertices={} x0={:.2} y0={:.2} xn={:.2} yn={:.2}",
                    run.vertices.len(),
                    first.0,
                    first.1,
                    last.0,
                    last.1
                ),
            );
            actions.push(raised);
            run.vertices.clear();
            true
        }
        Err(reason) => {
            super::decline(run.kind, run.page_index, reason);
            false
        }
    }
}

/// **Take one click for an armed vertex tool** — a vertex, or the ending.
#[allow(
    clippy::too_many_arguments,
    reason = "a gesture entry point's inputs are eight independent facts about one frame — the pen, the armed kind, two pointer positions, the page, its geometry, the phase and the action queue. Grouping any subset into a struct would be grouping by arity rather than by meaning, and the resulting type would have no name that was true." // ui-text-exempt: lint justification, never displayed
)]
pub(crate) fn click(
    pen: super::pen::Pen,
    ctx: &egui::Context,
    kind: MarkupKind,
    page_index: usize,
    canvas_point: Pos2,
    double: bool,
    page: Option<&Page>,
    actions: &mut Vec<Action>,
) {
    let mut run = load(ctx, page_index, kind);
    if double {
        commit(&mut run, actions, pen);
        // Traced with *which* ending asked, which neither the engine's
        // `add-markup` line nor a screenshot can distinguish.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("markup-finish via=double-click kind={kind:?} page={page_index}")
        });
        store(ctx, run);
        return;
    }
    let Some(page) = page else {
        super::decline(kind, page_index, super::Refusal::NoPage);
        return;
    };
    let Some(point) = page_point(canvas_point, page) else {
        super::decline(kind, page_index, super::Refusal::DegeneratePage);
        return;
    };
    run.vertices.push(point);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // An armed vertex tool with two vertices and one with three are the same
        // screenshot at any threshold — the rubber-banded segments are hairlines
        // in the pen colour over a drawing that is already full of them. This
        // line is how a harness proves a click became a vertex, which is defect
        // 8's lesson applied to a gesture rather than to a grid.
        format!(
            "markup-vertex kind={kind:?} page={page_index} n={} x={:.2} y={:.2}",
            run.vertices.len(),
            point.0,
            point.1
        )
    });
    store(ctx, run);
}

/// **Is there a vertex run waiting to be committed?** — the application state
/// behind the `markup.finishable` condition.
#[must_use]
pub fn finishable(ctx: &egui::Context) -> bool {
    pending(ctx).is_some()
}

/// The run that both halves of the Finish control are about, or `None`.
///
/// One derivation behind [`finishable`] and [`finish`], for the reason
/// `measure::circular::pending` gives.
fn pending(ctx: &egui::Context) -> Option<VertexRun> {
    let armed = tool::selected(ctx)
        .markup_kind()
        .filter(|k| k.is_vertex())?;
    let run = read(ctx)?;
    if run.kind != armed {
        return None;
    }
    super::action(
        run.kind,
        run.page_index,
        Geometry::Vertices(run.vertices.clone()),
        // The DEFAULT pen, and only here. This call is a *predicate* — it
        // asks whether the run would commit at all, to decide whether to offer
        // Finish — and it throws the resulting action away. The pen changes no
        // refusal: every one of `action`'s guards is about geometry (finite
        // coordinates, enough vertices, some extent), and none of them reads a
        // colour or a width. Threading the live pen here would suggest the
        // answer depends on it.
        super::pen::Pen::default(),
    )
    .ok()
    .map(|_| run)
}

/// **The `markup.finish` command's whole effect**, reporting whether it did
/// anything.
pub fn finish(ctx: &egui::Context, actions: &mut Vec<Action>, pen: super::pen::Pen) -> bool {
    let Some(mut run) = pending(ctx) else {
        return false;
    };
    let (kind, page_index) = (run.kind, run.page_index);
    if !commit(&mut run, actions, pen) {
        return false;
    }
    store(ctx, run);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // The `add-markup` line the engine traces proves the edit landed; this
        // one proves which of the two endings asked for it, which a screenshot
        // cannot distinguish and neither can the engine.
        format!("markup-finish via=command kind={kind:?} page={page_index}")
    });
    true
}

/// **Abandon a vertex run in progress**, reporting whether there was one.
pub fn abandon(ctx: &egui::Context) -> bool {
    let Some(run) = read(ctx) else {
        return false;
    };
    if !run.in_progress() {
        return false;
    }
    clear(ctx);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        "canvas-escape outcome=AbandonedMarkupVertexRun".to_owned()
    });
    true
}

/// **Draw what the next click would add, and what Finish would commit.**
pub(in crate::canvas) fn preview(
    ui: &Ui,
    page: Option<&Page>,
    page_index: usize,
    kind: MarkupKind,
    map: &PageMapping,
    pointer: Option<Pos2>,
    pen: super::pen::Pen,
) {
    if !kind.is_vertex() {
        return;
    }
    let (Some(page), Some(run)) = (page, read(ui.ctx())) else {
        return;
    };
    if run.page_index != page_index || run.kind != kind || !run.in_progress() {
        return;
    }
    let painter = ui.painter();
    let stroke = egui::Stroke::new(super::pen_px(map, pen), super::pen_color(kind, pen));
    let screen: Vec<Pos2> = run
        .vertices
        .iter()
        .filter_map(|&(x, y)| to_screen(x, y, page, map))
        .collect();
    // A vertex the transform could not project is dropped rather than faked, so
    // the count below may be short — which is why the closing segment is taken
    // from `screen`'s own ends rather than from the run's.
    for pair in screen.windows(2) {
        if let [a, b] = pair {
            painter.line_segment([*a, *b], stroke);
        }
    }
    // Polygon AND Cloud close; PolyLine does not. The closing segment is the
    // one thing the preview must say that the click run does not, because
    // `/Polygon` closes back to `/Vertices[0]` by specification rather than by
    // anything the operator did — see §1.2. A cloud is a `/Polygon` with `/BE`
    // on it, so it closes by exactly the same clause.
    //
    // The preview draws the closing segment STRAIGHT for a cloud rather than
    // scalloped, and that is deliberate rather than unfinished: this is a
    // gesture overlay — the cursor — and R8b's rule is that a pre-commit
    // affordance may show what is being built while APPLIED content must render
    // exactly as saved content will. Approximating the border effect here would
    // be a second rendering path for something the engine bakes an appearance
    // for, and two paths drift. The scallop arrives when the annotation does.
    if matches!(kind, MarkupKind::Polygon | MarkupKind::Cloud)
        && let (Some(first), Some(last)) = (screen.first(), screen.last())
        && screen.len() > 2
    {
        painter.line_segment([*last, *first], stroke);
    }
    if let (Some(last), Some(at)) = (screen.last(), pointer) {
        painter.line_segment([*last, map.to_screen(at)], stroke);
    }
}

/// PDF user space → screen, both hops.
fn to_screen(x: f64, y: f64, page: &Page, map: &PageMapping) -> Option<Pos2> {
    #[allow(clippy::cast_possible_truncation)]
    let canvas = viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), page)?;
    Some(map.to_screen(canvas))
}

/// Plant a run in memory, for tests in sibling modules.
#[cfg(test)]
pub(crate) fn plant_run_for_test(ctx: &egui::Context, page_index: usize, kind: MarkupKind) {
    store(
        ctx,
        VertexRun {
            page_index,
            kind,
            vertices: vec![(10.0, 10.0), (90.0, 20.0), (50.0, 80.0)],
        },
    );
}

/// Plant a **two-vertex** run, which is a polyline and is not a polygon.
#[cfg(test)]
pub(crate) fn plant_short_run_for_test(ctx: &egui::Context, page_index: usize, kind: MarkupKind) {
    store(
        ctx,
        VertexRun {
            page_index,
            kind,
            vertices: vec![(10.0, 10.0), (90.0, 20.0)],
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::tool::CanvasTool;
    use pdfcer_core::object::{Dict, ObjId};
    use pdfcer_core::page_tree::Rect as PageRect;

    /// A minimal upright page, one unit per point.
    fn test_page() -> Page {
        Page {
            id: ObjId::new(1, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, 200.0, 300.0),
            crop_box: PageRect::from_corners(0.0, 0.0, 200.0, 300.0),
            crop_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            bleed_box: PageRect::from_corners(0.0, 0.0, 200.0, 300.0),
            bleed_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            trim_box: PageRect::from_corners(0.0, 0.0, 200.0, 300.0),
            trim_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            art_box: PageRect::from_corners(0.0, 0.0, 200.0, 300.0),
            art_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            rotate: 0,
            contents: Vec::new(),
            contents_unresolved: 0,
            resources_defaulted: false,
            contents_flattened: 0,
        }
    }

    /// Click `n` points in a line-ish run on `ctx`, returning the actions raised.
    fn run_clicks(ctx: &egui::Context, kind: MarkupKind, points: &[(f32, f32)]) -> Vec<Action> {
        let page = test_page();
        let mut actions = Vec::new();
        for &(x, y) in points {
            click(
                crate::canvas::markup::pen::Pen::default(),
                ctx,
                kind,
                0,
                Pos2::new(x, y),
                false,
                Some(&page),
                &mut actions,
            );
        }
        actions
    }

    // -----------------------------------------------------------------
    // The gesture
    // -----------------------------------------------------------------

    /// **Each click adds one vertex and authors nothing.**
    #[test]
    fn each_click_adds_a_vertex_and_authors_nothing() {
        let ctx = egui::Context::default();
        let actions = run_clicks(
            &ctx,
            MarkupKind::PolyLine,
            &[(10.0, 10.0), (40.0, 20.0), (60.0, 50.0)],
        );
        assert!(actions.is_empty(), "a vertex authors nothing on its own");
        let run = read(&ctx).expect("a run exists");
        assert_eq!(run.vertices.len(), 3);
        assert_eq!(run.kind, MarkupKind::PolyLine);
        // Canvas y is DOWN and PDF y is UP: the first click at canvas y=10 on a
        // 300-high page is PDF y=290. Asserted as a magnitude rather than "the
        // vertices are on the page", because a build that dropped the flip would
        // put them on the page too — upside down.
        assert!((run.vertices[0].0 - 10.0).abs() < 1e-3, "{run:?}");
        assert!((run.vertices[0].1 - 290.0).abs() < 1e-3, "{run:?}");
    }

    /// **`click, click, double-click` places three vertices and commits.**
    #[test]
    fn a_double_click_finishes_and_the_first_click_of_the_pair_still_counts() {
        let ctx = egui::Context::default();
        let page = test_page();
        let mut actions = run_clicks(
            &ctx,
            MarkupKind::PolyLine,
            &[(10.0, 10.0), (40.0, 20.0), (60.0, 50.0)],
        );
        click(
            crate::canvas::markup::pen::Pen::default(),
            &ctx,
            MarkupKind::PolyLine,
            0,
            Pos2::new(60.0, 50.0),
            true,
            Some(&page),
            &mut actions,
        );
        assert_eq!(actions.len(), 1, "the double-click commits exactly once");
        let Action::CommitMarkup { kind, geometry, .. } = &actions[0] else {
            panic!("a vertex ending must raise CommitMarkup: {actions:?}");
        };
        assert_eq!(*kind, MarkupKind::PolyLine);
        let Geometry::Vertices(v) = geometry else {
            panic!("a vertex kind must carry Vertices: {geometry:?}");
        };
        assert_eq!(v.len(), 3, "three clicks, three vertices");
        assert!(
            !read(&ctx).is_some_and(|r| r.in_progress()),
            "the run is emptied, so a second Finish cannot author it again"
        );
    }

    /// **The two endings author the same annotation from the same clicks.**
    #[test]
    fn the_double_click_and_the_command_author_the_same_annotation() {
        // Ending 1: the double-click, taken by the canvas.
        let by_click = egui::Context::default();
        let page = test_page();
        let mut click_actions = run_clicks(
            &by_click,
            MarkupKind::Polygon,
            &[(10.0, 10.0), (90.0, 20.0), (50.0, 80.0)],
        );
        click(
            crate::canvas::markup::pen::Pen::default(),
            &by_click,
            MarkupKind::Polygon,
            0,
            Pos2::new(50.0, 80.0),
            true,
            Some(&page),
            &mut click_actions,
        );

        // Ending 2: the ribbon command, through `egui::Memory`.
        let by_command = egui::Context::default();
        tool::select(&by_command, CanvasTool::Markup(MarkupKind::Polygon));
        let _ = run_clicks(
            &by_command,
            MarkupKind::Polygon,
            &[(10.0, 10.0), (90.0, 20.0), (50.0, 80.0)],
        );
        let mut command_actions = Vec::new();
        assert!(
            finish(
                &by_command,
                &mut command_actions,
                crate::canvas::markup::pen::Pen::default()
            ),
            "the command finishes"
        );

        assert_eq!(
            click_actions, command_actions,
            "the two endings must author the same annotation, on the same page, \
             from the same vertices"
        );
        assert_eq!(click_actions.len(), 1, "exactly one annotation per ending");
    }

    /// **A polygon needs one more click than a polyline before Finish lights.**
    #[test]
    fn finish_lights_after_two_clicks_for_a_polyline_and_three_for_a_polygon() {
        for (kind, needed) in [(MarkupKind::PolyLine, 2_usize), (MarkupKind::Polygon, 3)] {
            let ctx = egui::Context::default();
            tool::select(&ctx, CanvasTool::Markup(kind));
            assert!(!finishable(&ctx), "{kind:?}: nothing clicked yet");
            let points = [(10.0_f32, 10.0_f32), (90.0, 20.0), (50.0, 80.0)];
            for (n, &p) in points.iter().enumerate() {
                let _ = run_clicks(&ctx, kind, &[p]);
                assert_eq!(
                    finishable(&ctx),
                    n + 1 >= needed,
                    "{kind:?} after {} click(s)",
                    n + 1
                );
            }
        }
    }

    /// **Finish is offered only while the tool is still armed**, and asking the
    /// question does not manufacture a run.
    #[test]
    fn finish_needs_the_tool_armed_and_asking_creates_nothing() {
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Markup(MarkupKind::Polygon));
        assert!(!finishable(&ctx));
        assert!(read(&ctx).is_none(), "the question must not answer itself");

        plant_run_for_test(&ctx, 0, MarkupKind::Polygon);
        assert!(finishable(&ctx));

        tool::select(&ctx, CanvasTool::Select);
        assert!(
            !finishable(&ctx),
            "a run nothing is drawing must not keep offering Finish"
        );
        let mut actions = Vec::new();
        assert!(
            !finish(
                &ctx,
                &mut actions,
                crate::canvas::markup::pen::Pen::default()
            ),
            "…and the command refuses it too, by the same predicate"
        );
        assert!(actions.is_empty());

        // A different vertex kind armed is not this run's ending either.
        tool::select(&ctx, CanvasTool::Markup(MarkupKind::PolyLine));
        assert!(!finishable(&ctx));
    }

    /// **A change of kind or of page discards the run** — §2's two
    /// synchronisations, at the entry point that applies them.
    #[test]
    fn changing_kind_or_page_discards_the_run() {
        let ctx = egui::Context::default();
        let _ = run_clicks(&ctx, MarkupKind::PolyLine, &[(10.0, 10.0), (40.0, 20.0)]);
        assert_eq!(read(&ctx).map(|r| r.vertices.len()), Some(2));

        let switched = load(&ctx, 0, MarkupKind::Polygon);
        assert!(
            switched.vertices.is_empty(),
            "a polyline's vertices are not a polygon's"
        );

        let moved = load(&ctx, 1, MarkupKind::PolyLine);
        assert!(
            moved.vertices.is_empty(),
            "a run begun on sheet 1 means nothing on sheet 2"
        );
        assert_eq!(moved.page_index, 1);
    }

    /// **Escape abandons the run and reports that it took the key.**
    #[test]
    fn escape_abandons_a_run_and_says_whether_it_took_the_key() {
        let ctx = egui::Context::default();
        assert!(!abandon(&ctx), "nothing to abandon: the key is not ours");

        let _ = run_clicks(&ctx, MarkupKind::Polygon, &[(10.0, 10.0), (40.0, 20.0)]);
        assert!(abandon(&ctx));
        assert!(read(&ctx).is_none(), "the run is gone");
        assert!(!abandon(&ctx), "and it is not claimed twice");
    }

    /// A run that [`super::action`] refuses is kept rather than silently thrown
    /// away, so the operator can add the vertex that was missing.
    #[test]
    fn a_refused_run_survives_its_refusal() {
        let ctx = egui::Context::default();
        let page = test_page();
        let mut actions = run_clicks(&ctx, MarkupKind::Polygon, &[(10.0, 10.0), (40.0, 20.0)]);
        click(
            crate::canvas::markup::pen::Pen::default(),
            &ctx,
            MarkupKind::Polygon,
            0,
            Pos2::new(40.0, 20.0),
            true,
            Some(&page),
            &mut actions,
        );
        assert!(actions.is_empty(), "two vertices are not a polygon");
        assert_eq!(
            read(&ctx).map(|r| r.vertices.len()),
            Some(2),
            "the clicks survive, so a third can rescue them"
        );
    }

    /// With no page under it a click authors nothing and adds nothing, rather
    /// than pushing a vertex whose coordinates were never converted.
    #[test]
    fn a_click_with_no_page_adds_no_vertex() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        click(
            crate::canvas::markup::pen::Pen::default(),
            &ctx,
            MarkupKind::PolyLine,
            0,
            Pos2::new(10.0, 10.0),
            false,
            None,
            &mut actions,
        );
        assert!(actions.is_empty());
        assert!(read(&ctx).is_none_or(|r| !r.in_progress()));
    }
}
