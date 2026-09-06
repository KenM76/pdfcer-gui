//! `a_markup_shapes_nodes_can_be_edited` — the operator's *"I also can't edit
//! or delete nodes of a markup shape once it is drawn"*, for the half that is
//! a **markup shape** rather than a ce dimension.
//!
//! ## ★★★ Why this check and not the sixteen unit tests beside the feature
//!
//! `canvas::annotnodes::tests` asserts the arithmetic, the subtype table and
//! the engine's own floor, against a real `EditSession`, and **every one of
//! them would pass on a build where the operator can do none of this.** This
//! project once had eight green tests while the feature performed 1 of 14
//! steps; R1 is the rule that came out of it, and this file is what discharges
//! it here.
//!
//! Seven links stand between `annotnodes::resolved` and a hand on a mouse, and
//! not one is observable in process:
//!
//! | # | link | why a unit test cannot see it |
//! |---|---|---|
//! | 1 | the **Polygon markup tool** arms in Review | `Capabilities::for_mode` reads the real manifest, and the mode is entered by clicking a segment |
//! | 2 | four clicks become a four-vertex `/Polygon` in the file | `markup::vertex` accumulates in `egui::Memory` and commits on a double-click |
//! | 3 | a click **selects** the shape it just drew | `selection::annot` hit-tests a `/Rect` resolved through two coordinate spaces |
//! | 4 | `canvas.markup-node.N` is **painted where the hit test looks** | the anchor's position is the end of a page → canvas → screen conversion; only the running application knows it |
//! | 5 | the press classifies as `DragKind::MarkupVertex` and **outranks** the body and the eight resize grips | `gesture::press_kind`'s precedence, over flags `canvas::pressing` resolves from a screen position |
//! | 6 | `Ctrl`+`Shift` survives the OS → winit → egui path **for the whole drag** | `Driver::press_held`'s note: a modifier applied and undone inside one frame's event batch |
//! | 7 | the engine accepts it, the `/AP` is re-baked, and **the pixels change** | only a real edit followed by a real render can show this |
//!
//! ## ★★ What it asserts that a trace alone cannot: the PIXELS
//!
//! Step 7 is the one this check was written for. `move-annotation-vertex` in
//! the trace says the verb ran; it does not say the **drawing changed**. A
//! reshape rewrites three things — the `/Vertices` array, the `/Rect`, and the
//! baked `/AP` stream — and the engine's own note is that a shell writing only
//! some of them *"looks right in your canvas, right in a screenshot, right in
//! pdfcer, and is reconstructed in the old place by the next viewer"*. The
//! inverse failure is just as reachable here: an edit that lands in the model
//! and never reaches the raster leaves the operator dragging a node and
//! watching nothing move.
//!
//! So the shape is dragged **out of its own bounding box**, into paper that was
//! blank before, and the ink there is counted before and after.
//!
//! ★★★ **The selection is CLEARED before the "after" capture, and that is the
//! whole honesty of the pixel assertion.** A selected shape draws its outline
//! and its node anchors, which are ink, at exactly the place the drag ended —
//! so a build that moved nothing but drew an anchor at the pointer would put
//! ink in the sampled box and pass. Deselecting first leaves only what
//! `pdfcer-render` drew from the annotation's own appearance stream.
//!
//! ## The fixture is PINNED and `--pdf` is ignored
//!
//! `fixtures/four-pages.pdf`, and the reason is the pixel assertion above: it
//! needs a region of page that is **blank before the drag**, and "blank" is not
//! a property a sweep's `--pdf` can promise. On the operator's own CAD sheet
//! the destination box would already be full of ink and the assertion could
//! neither pass nor fail honestly. The check says so in its notes when a
//! `--pdf` was supplied and thrown away, because a sweep that silently ignored
//! a flag is indistinguishable from one that honoured it.
//!
//! ## The shape, and why a square
//!
//! Four nodes. A `/Polygon`'s floor is **three**, so four is one above it —
//! which is what lets a single traced shape exercise the whole sequence and
//! reach the boundary:
//!
//! ```text
//! 4 nodes  → drag node 1                → 4 nodes, moved   (and the pixels change)
//! 4 nodes  → Ctrl+Shift-drag node 1     → 3 nodes          (a node is DELETED)
//! 3 nodes  → Ctrl+Shift-drag node 1     → REFUSED, and the refusal is SHOWN
//! ```
//!
//! The third step is the operator's actual complaint in its purest form: a
//! gesture that cannot be honoured. It passes only if the status bar's `⊗` slot
//! is on screen — `status-group:decline` is published as a `ui-rect` on the
//! frame it draws, and its **absence is the failure**, because a refusal
//! nobody is told about is the founding defect of this project.

use std::path::PathBuf;

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The pinned fixture. See the module header for why `--pdf` is ignored.
const FIXTURE: &str = "fixtures/four-pages.pdf";
/// Review — the mode markup is authored in, and the mode whose `edit_content`
/// is **false**.
///
/// ★ Driving this in Edit would exercise the same code with the interesting
/// half of every gate short-circuited: `press_kind`'s markup-node rung is
/// gated on `author_markup` precisely so it fires where `edit_content` does
/// not, and Edit has both.
const MODE: &str = "review";
/// The Markup tab, which Review is shown.
const TAB: &str = "ribbon.tab.markup";
/// The ribbon item that arms the Polygon tool.
const ITEM: &str = "ribbon.item.markup.polygon";
/// `markup-tool tool=…` — the canvas reporting what armed.
const ARM_EVENT: &str = "markup-tool";
/// The `Debug` spelling of `CanvasTool::Markup(MarkupKind::Polygon)`.
const ARM_VALUE: &str = "Markup(Polygon)";
/// `markup-vertex kind=… page=… n=… x=… y=…` — one line per click while the
/// shape is being **drawn**.
///
/// ★★ Note how close this is to the node-editing lines below, and that the
/// closeness is exactly why they are spelled `markup-node-*`. This event
/// belongs to `canvas::markup::vertex` and has since polygons became
/// authorable; a node-move line under the same first token would make
/// `Trace::last` return whichever came later, and a check asserting a node
/// MOVED would read a line about a node being PLACED.
const DRAW_EVENT: &str = "markup-vertex";
/// `add-markup page=… n=…` — the funnel's line, i.e. the engine accepted the
/// shape and it is now in the document.
const COMMIT_EVENT: &str = "add-markup";
/// The prefix each node anchor is published under, suffixed with its index.
const NODE_REGION: &str = "canvas.markup-node";
/// `markup-node-move id=… index=… nodes=… x=… y=… snap=…` — the SHELL's report
/// that the gesture was understood.
const SHELL_MOVE: &str = "markup-node-move";
/// `markup-node-remove …` — its twin for a deletion.
const SHELL_REMOVE: &str = "markup-node-remove";
/// `markup-node-declined id=… index=… intent=… reason=…` — the preflight said
/// no on the frame the button came up.
const SHELL_DECLINED: &str = "markup-node-declined";
/// `move-annotation-vertex page=0 n=1 epoch=… disclosures=…` — the **funnel's**
/// line, which is the engine's acknowledgement that the document changed.
///
/// ★★ Distinct from [`SHELL_MOVE`] deliberately, and asserting both is the
/// point: one says the gesture was understood, the other says the document
/// changed. A check that read only the first could not tell a shell that never
/// asked from an engine that refused.
const ENGINE_MOVE: &str = "move-annotation-vertex";
/// `remove-annotation-vertex …` — its twin.
const ENGINE_REMOVE: &str = "remove-annotation-vertex";
/// `move-annotation-vertex-applied …` — this shell's own line beside the
/// funnel's, carrying the before→after node count and the `/Rect` pair.
const APPLIED_SUFFIX: &str = "-applied";
/// The `⊗` slot in the status bar. `app::status::decline` draws into it and
/// publishes it as a `ui-rect` on the frame it draws.
const DECLINE_REGION: &str = "status-group:decline";
/// `command-declined id=view.tool_node reason=…` — the `A` chord was refused.
const TOOL_DECLINED: &str = "command-declined";
/// The command the `A` chord raises.
const NODE_COMMAND: &str = "view.tool_node";

/// The four corners of the traced square, as fractions of the page box.
const CORNERS: [(f64, f64); 4] = [(0.25, 0.25), (0.55, 0.25), (0.55, 0.55), (0.25, 0.55)];
/// Where node 1 is dragged to — **outside** the square, so the shape after the
/// drag cannot coincide with the shape before it by arithmetic accident, and so
/// the sampled box is paper the polygon has never covered.
///
/// ★★ **Upper right, and the first run is why it is not lower right.** The
/// first draft aimed at `(0.80, 0.15)` and the "before" box came back holding
/// 423 ink pixels of 1,406 — because `four-pages.pdf`'s page 1 carries a
/// coloured **title block** in exactly that corner. The assertion still passed,
/// on a delta of 28 pixels against a floor of 423, which is a measurement one
/// stray antialiased edge could have made either way. ⇒ **Read the capture
/// before believing a pixel assertion**; the artifact was sitting in
/// `evidence/` saying so.
const DESTINATION: (f64, f64) = (0.85, 0.75);
/// **The smallest ink change this check will call a change.**
///
/// ★ Four pixels, and the reasoning is `InkReport::is_text`'s: one or two
/// pixels either way is antialiasing on an edge that did not move, while a
/// 2 pt stroke crossing a box contributes a run. A strict `>` on a raw count
/// would let noise decide the verdict, and this project's standing rule is
/// that when a measurement runs out you read something else rather than
/// widening a tolerance — so the fix here is a floor with a stated reason plus
/// a SECOND, opposite measurement below, not a looser comparison.
const INK_DELTA_FLOOR: usize = 4;
/// Half-width of the box sampled for ink, as a fraction of the page's width.
///
/// ★ A fraction and not a constant in points, and the first run is why: 22 pt
/// on this fixture at fit-page zoom is an **8 x 9 pixel** window, which is too
/// few pixels for `ink_run_into` to say anything with. Small enough that the
/// square's original edges are nowhere near it — the nearest is a quarter of
/// the page away — and large enough to contain the corner two edges now meet
/// at, even after a snap has nudged it.
const SAMPLE_HALF_FRACTION: f64 = 0.04;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// See the module documentation.
pub struct AMarkupShapesNodesCanBeEdited;

impl Check for AMarkupShapesNodesCanBeEdited {
    fn name(&self) -> &'static str {
        "a_markup_shapes_nodes_can_be_edited"
    }

    fn defect(&self) -> &'static str {
        "a shape the operator has drawn as a comment shows no node anchors, or shows them and \
         a drag on one moves the whole shape, or moves the node in the model and never repaints, \
         or deletes a node and cannot say why when it will not delete another. His report, \
         verbatim: \"I also can't edit or delete nodes of a markup shape once it is drawn.\" The \
         engine's four verbs for this landed on 2026-09-05 and this shell called none of them"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Drag from `from` to `to` with `modifiers` held for the WHOLE gesture.
///
/// ★★★ **Held across the press, the walk and the release, and that is not
/// politeness.** `Driver::press_held`'s own note records the finding: a
/// modifier that goes down and up inside one frame's event batch can be applied
/// and undone before the event it was meant to carry is dispatched, because
/// modifier state reaches egui through winit's `ModifiersChanged`. A harness
/// that pressed Ctrl just before the button would produce a plain drag and
/// report *"the node was moved, not deleted"* about a perfectly working build.
///
/// It also matches what `canvas::dimdrag::intent` actually does: it reads the
/// modifiers **live on every frame**, so a Ctrl released half way through turns
/// the gesture back into a move and the preview says so on that very frame.
/// Holding it throughout is the only way to drive the gesture the operator's
/// hand makes.
fn drag_holding(
    driver: &Driver,
    modifiers: &[u16],
    from: ScreenPoint,
    to: ScreenPoint,
) -> Result<()> {
    crate::sys::with_modifiers(modifiers, || driver.drag(from, to))
}

/// The document-space box sampled for ink around a point.
fn sample_box(page: PageGeometry, at: (f64, f64)) -> (DocPoint, DocPoint) {
    let (x, y) = (at.0 * page.width_pt, at.1 * page.height_pt);
    let half = SAMPLE_HALF_FRACTION * page.width_pt;
    (
        DocPoint::new(0, x - half, y - half),
        DocPoint::new(0, x + half, y + half),
    )
}

/// Capture the window once, and count the ink in **both** boxes this check
/// watches — where the node is going, and where it came from.
///
/// ★★★ **Two boxes from ONE capture, and the pair is the assertion.** A single
/// "ink arrived at the destination" reading is satisfied by anything that puts
/// dark pixels there, including a build that drew a stray anchor. A single "ink
/// left the origin" reading is satisfied by a build that simply stopped drawing
/// the shape. Requiring **both, in opposite directions, in the same frame** is
/// what makes the pair describe a node that MOVED rather than one that appeared
/// or vanished.
fn ink_pair(
    session: &Session,
    mapping: &CanvasMapping,
    page: PageGeometry,
    origin: (f64, f64),
    out: &std::path::Path,
) -> Result<(crate::pixels::InkReport, crate::pixels::InkReport, PathBuf)> {
    let image = crate::capture::window_to_png(session, out)?;
    let frame = session.frame()?;
    let measure = |at: (f64, f64)| -> Result<crate::pixels::InkReport> {
        let (lo, hi) = sample_box(page, at);
        let a = mapping.doc_to_window(lo)?;
        let b = mapping.doc_to_window(hi)?;
        let rect = crate::geom::LRect::new(
            crate::geom::Pt::new(a.x().min(b.x()), a.y().min(b.y())),
            crate::geom::Pt::new(a.x().max(b.x()), a.y().max(b.y())),
        );
        Ok(crate::pixels::ink_run_into(
            &image,
            frame.logical_to_capture_pixels(rect),
        ))
    };
    let there = measure(DESTINATION)?;
    let here = measure(origin)?;
    Ok((there, here, out.to_path_buf()))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, a \
             ribbon control, five points on the page, presses a chord and performs three drags, \
             two of them with modifiers held. Reported as SKIPPED rather than passed: a check \
             that did not run has learned nothing.",
        ));
    }
    let pdf = fixture_path();
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the pinned fixture is missing: {}",
            pdf.display()
        )));
    }
    if let Some(supplied) = ctx.pdf.as_ref()
        && supplied.file_name() != pdf.file_name()
    {
        report.note(format!(
            "--pdf {} was IGNORED; this check pins {FIXTURE} because its pixel assertion needs \
             page that is blank before the drag, which no sweep fixture can promise",
            supplied.display()
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = crate::fixture::page_geometry(&pdf).ok_or_else(|| {
        Error::new(format!(
            "cannot read a page size from {}. The harness needs the page box to turn this \
             check's corner fractions into points.",
            pdf.display()
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("markup_node_edit.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let driver = Driver::new(session.window());

    // --- 1: Review, the Markup tab, the Polygon tool ----------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, TAB) else {
        return Ok(Some(format!(
            "the `{MODE}` mode declares no `{TAB}` region, so no markup tool can be reached and \
             there is no shape to put nodes on."
        )));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)? else {
        return Ok(Some(format!(
            "the Markup tab declares no `{ITEM}`. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.markup."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    let trace = session.trace()?;
    if !trace
        .events(ARM_EVENT)
        .any(|l| l.get("tool") == Some(ARM_VALUE))
    {
        return Ok(Some(format!(
            "the Polygon item was clicked and no `{ARM_EVENT} tool={ARM_VALUE}` followed, so \
             there is no way to draw the shape this check is about."
        )));
    }

    // --- 2: draw a four-node polygon and commit it ------------------------
    let canvas_page = trace
        .last("canvas")
        .and_then(|l| l.get("page"))
        .and_then(|v| v.parse::<usize>().ok());
    if canvas_page != Some(0) {
        return Err(Error::new(
            "the canvas is not showing page 1, so the page geometry this check computed does not \
             describe what is on screen. Aiming anyway would produce a confidently-wrong click.",
        ));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let frame = session.frame()?;
    let mut aimed: Vec<ScreenPoint> = Vec::with_capacity(CORNERS.len());
    for (fx, fy) in CORNERS {
        aimed.push(frame.to_screen(mapping.doc_to_window(DocPoint::new(
            0,
            fx * page.width_pt,
            fy * page.height_pt,
        ))?));
    }

    // ★★★ **THREE single clicks and then a DOUBLE on the fourth corner — the
    // ending CONSUMES a click, and the first run of this check got it wrong.**
    //
    // `canvas::markup::vertex::click` states it in its own header: *"`click,
    // click, double-click` places THREE vertices and commits: A, B, then the
    // first click of the pair as C, then the second as the ending."* Four
    // single clicks followed by a double therefore places **five** vertices,
    // and this check reported `nodes=5->5` against a build whose move was
    // perfect — a harness defect wearing an application defect's message,
    // which is this project's commonest failure and the reason *ask what the
    // check SAMPLED before asking what is broken* is a standing rule here.
    //
    // ⇒ The shell's design is deliberate and worth carrying to any other
    // vertex tool driven from this harness: the ending is not a separate act,
    // so an operator never has to click somewhere they did not want a vertex
    // in order to say they have finished.
    for screen in &aimed[..CORNERS.len() - 1] {
        driver.click_at(*screen)?;
        session.settle(12);
    }
    // The double-click ending — `markup::vertex`'s own, and the one an operator
    // uses. `markup.finish` on the ribbon is the other and is not what a hand
    // reaches for.
    driver.double_click_at(aimed[CORNERS.len() - 1])?;
    session.settle(30);
    let taken = session.trace()?.events(DRAW_EVENT).count();
    if taken != CORNERS.len() {
        return Ok(Some(format!(
            "{taken} of {} clicks became polygon vertices (`{DRAW_EVENT}`), so the shape this \
             check reshapes was never drawn. That is `canvas::markup::vertex`'s subject, not \
             this check's — and remember the double-click's FIRST press is one of them.",
            CORNERS.len()
        )));
    }
    let trace = session.trace()?;
    if trace.events(COMMIT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "the polygon did not commit — no `{COMMIT_EVENT}`. So there is no markup shape in \
             the document to put nodes on. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("a four-node polygon was drawn and reached the engine");

    // ★★★ THE "BEFORE" CAPTURE — the shape is on the page, drawn by
    // `pdfcer-render` from its own appearance stream, and **nothing is
    // selected**, so there is no outline and no anchor anywhere in either box.
    //
    // Taken here rather than before the polygon existed, and the reason is the
    // second half of the pair: the ORIGIN box must contain the corner that is
    // about to move, or "ink left the origin" measures nothing.
    let (before_there, before_here, before_shot) = ink_pair(
        &session,
        &mapping,
        page,
        CORNERS[1],
        &ctx.out("markup_node_before.png"),
    )?;
    report.artifact(before_shot);
    report.note(format!(
        "before the drag: destination {} · origin {}",
        before_there.summary(),
        before_here.summary()
    ));
    // ★★ The baseline is asserted rather than assumed, because a check whose
    // baseline is wrong cannot report anything honestly. If the corner is not
    // there to begin with, "ink left the origin" is vacuous — and this is
    // exactly the class of thing that goes unnoticed when a fixture changes.
    if before_here.ink < INK_DELTA_FLOOR {
        return Err(Error::new(format!(
            "the polygon's node 1 is not visible at its own corner before the drag ({}), so \
             neither half of the pixel assertion below can mean anything. The shape committed, \
             so this is the harness aiming at the wrong place or the page not having rendered \
             yet — not an application defect. Capture: {}.",
            before_here.summary(),
            ctx.out("markup_node_before.png").display()
        )));
    }

    // --- 3: select it, so its node anchors are published ------------------
    //
    // ★ Press `V` FIRST. With the Polygon tool still armed a click on the page
    // is another vertex, not a selection — `sys::vk::V`'s own doc comment says
    // so, and the first run of the ce-dimension check ignored it and then
    // reported "the shape could not be selected" about a build whose selection
    // is fine.
    driver.press(vk::V)?;
    session.settle(12);
    // The midpoint of the top edge: on the shape's own ink, and clear of every
    // node anchor so the click cannot be read as one.
    driver.click_at(frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        f64::midpoint(CORNERS[0].0, CORNERS[1].0) * page.width_pt,
        CORNERS[0].1 * page.height_pt,
    ))?))?;
    session.settle(18);
    let trace = session.trace()?;
    let Some(node_one) = declared(&trace, ui_rect, &format!("{NODE_REGION}.1")) else {
        return Ok(Some(format!(
            "★ THE SHAPE HAS NO NODE ANCHORS: nothing published `{NODE_REGION}.1`. Either the \
             click did not select the polygon, or `canvas::annotnodes::geometry` answered `None` \
             for it, or the painter's anchor loop was never reached. Regions seen: {}. That is \
             the operator's report in its first form — a shape whose nodes cannot be found at \
             all. Trace: {}.",
            list(&declared_names(&trace, ui_rect, NODE_REGION)),
            session.trace_path().display()
        )));
    };

    // --- 4: ★★★ drag node 1 — it MOVES, and the pixels move with it -------
    let frame = session.frame()?;
    let from = frame.declared_center(node_one);
    let out = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        DESTINATION.0 * page.width_pt,
        DESTINATION.1 * page.height_pt,
    ))?);
    driver.drag(from, out)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(shell) = trace.last(SHELL_MOVE) else {
        return Ok(Some(format!(
            "★ DRAGGING A NODE DID NOTHING: no `{SHELL_MOVE}` line. The anchor was published at \
             {node_one:?}, so the press landed on it. Check, in order: (1) did the press \
             classify as `DragKind::MarkupVertex` — `canvas::pressing` resolves `markup_node` \
             and `gesture::press_kind`'s rung is gated on `author_markup`; (2) does the trace \
             carry `annot-drag` instead, which would mean the node rung lost to the body rung \
             and the operator moved the whole shape; (3) does it carry `resize-annotation`, \
             which would mean it lost to a corner grip. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if shell.get("index") != Some("1") {
        return Ok(Some(format!(
            "the drag grabbed node `{:?}`, not node 1. The anchor aimed at was \
             `{NODE_REGION}.1`, so either the painter's index and the hit test's disagree — they \
             read the same list in the same frame, so that would mean one of them is caching — \
             or the tie-break in `annotnodes::node_at` claimed a coincident node. Line: `{}`.",
            shell.get("index"),
            shell.raw
        )));
    }
    if trace.events(ENGINE_MOVE).count() == 0 {
        return Ok(Some(format!(
            "★ the shell asked to move the node and the DOCUMENT did not change: `{SHELL_MOVE}` \
             is present and `{ENGINE_MOVE}` is not. So `AnnotAction::MoveNode` was raised and \
             `EditSession::reshape_annotation` refused it or was never reached — \
             `app/actions/annots.rs`'s router and `reshape` are the two places to look. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    // ★★ The shell's own applied line carries the node count and the `/Rect`
    // pair, which is what says the reshape rewrote BOTH halves rather than only
    // the geometry array.
    let applied = trace.last(&format!("{ENGINE_MOVE}{APPLIED_SUFFIX}"));
    if let Some(line) = applied
        && line.get("nodes") != Some("4->4")
    {
        return Ok(Some(format!(
            "a MOVE changed the node count: `{}`. A move must leave `nodes=4->4` — anything else \
             means the gesture reached an insert or a remove verb.",
            line.raw
        )));
    }
    report.note(format!(
        "★ a node was MOVED and reached the engine: `{}`",
        shell.raw
    ));

    // ★★★ …AND THE PIXELS. Deselect first, so what is counted is the
    // annotation's own appearance and not its selection outline or its anchors.
    //
    // The click goes to a corner of the sheet the polygon has never occupied
    // and the destination box does not contain, so it can neither re-select the
    // shape nor add ink to what is about to be measured.
    driver.click_at(frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        0.06 * page.width_pt,
        0.92 * page.height_pt,
    ))?))?;
    session.settle(40);
    let (after_there, after_here, after_shot) = ink_pair(
        &session,
        &mapping,
        page,
        CORNERS[1],
        &ctx.out("markup_node_after.png"),
    )?;
    report.artifact(after_shot.clone());
    let arrived = after_there.ink.saturating_sub(before_there.ink);
    let left = before_here.ink.saturating_sub(after_here.ink);
    if arrived < INK_DELTA_FLOOR || left < INK_DELTA_FLOOR {
        return Ok(Some(format!(
            "★★★ THE NODE MOVED IN THE MODEL AND NOT ON THE PAGE. `{ENGINE_MOVE}` is in the \
             trace, so the engine accepted the reshape — and with the selection cleared the \
             DESTINATION box gained {arrived} ink pixels and the ORIGIN box lost {left}, against \
             a floor of {INK_DELTA_FLOOR} for each. So what the operator looks at does not carry \
             the edit. Three candidates, in the order to check them: the `/AP` stream was not \
             re-baked; the page raster was not invalidated (`vector_edit`'s `page_epochs` bump); \
             or the `/Rect` was not moved and §12.5.5 is still placing the old artwork at the \
             old rectangle. **This is precisely the failure no trace assertion can see** — the \
             engine's own note is that a shell writing some of the three looks right in every \
             renderer. Destination {} → {}; origin {} → {}. Captures: {} and {}.",
            before_there.summary(),
            after_there.summary(),
            before_here.summary(),
            after_here.summary(),
            ctx.out("markup_node_before.png").display(),
            after_shot.display()
        )));
    }
    report.note(format!(
        "★ and the DRAWING changed, in both directions: the destination gained {arrived} ink \
         pixels and the origin lost {left}"
    ));

    // --- 5: arm the Points tool, and DELETE a node ------------------------
    //
    // ★ The `A` chord and not a ribbon press: `view.tool_node`'s ribbon and
    // rail items both carry `shown_when("mode.edit_content")`, so in Review
    // there is no control to click. A chord is filtered by TAB visibility and
    // View is in every mode, so the key gets through.
    //
    // Re-select the shape first — the deselecting click above cleared it.
    driver.click_at(frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        f64::midpoint(CORNERS[2].0, CORNERS[3].0) * page.width_pt,
        CORNERS[2].1 * page.height_pt,
    ))?))?;
    session.settle(18);
    driver.press(vk::A)?;
    session.settle(16);
    let trace = session.trace()?;
    if let Some(declined) = trace
        .events(TOOL_DECLINED)
        .find(|l| l.get("id") == Some(NODE_COMMAND))
    {
        return Ok(Some(format!(
            "★ THE POINTS TOOL WILL NOT ARM IN {}: the `A` chord was declined — `{}`. Adding and \
             removing a node is gated on it, so this is where the delete half dies. Two places, \
             and they must agree: `app::dispatch::navigate`'s `view.tool_node` arm and \
             `canvas::tool::arm::retire_forbidden`'s Node arm. Both read `edit_content || \
             author_measure` today, and `canvas::annotnodes::tests::\
             the_points_tool_arms_wherever_a_markup_shape_can_be_authored` is the unit tripwire \
             for that coupling. Trace: {}.",
            MODE.to_uppercase(),
            declined.raw,
            session.trace_path().display()
        )));
    }

    let trace = session.trace()?;
    let Some(node_one) = declared(&trace, ui_rect, &format!("{NODE_REGION}.1")) else {
        return Ok(Some(format!(
            "the shape lost its anchors between the move and the delete. Regions seen: {}.",
            list(&declared_names(&trace, ui_rect, NODE_REGION))
        )));
    };
    let frame = session.frame()?;
    // Somewhere else on the page. A removal ignores the drop point — it has no
    // destination — and this asserts that by dropping it where the shape has no
    // business reaching.
    let anywhere = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        0.12 * page.width_pt,
        0.80 * page.height_pt,
    ))?);
    drag_holding(
        &driver,
        &[vk::CONTROL, vk::SHIFT],
        frame.declared_center(node_one),
        anywhere,
    )?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(shell) = trace.last(SHELL_REMOVE) else {
        return Ok(Some(format!(
            "★ CTRL+SHIFT-DRAGGING A NODE DELETED NOTHING: no `{SHELL_REMOVE}` line, which is \
             the literal half of the operator's report — \"I also can't edit or delete nodes\". \
             The move half worked in this same run, so the anchor is grabbable and the press \
             classifies; what differs is two modifiers. `canvas::dimdrag::intent` maps \
             `(command, shift)` to `Remove` and requires the Points tool armed. If the trace \
             carries `{SHELL_MOVE}` here instead, the modifiers never arrived — see \
             `drag_holding`'s note. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if shell.get("nodes") != Some("3") {
        return Ok(Some(format!(
            "the delete gesture reported `nodes={:?}`, not 3. The count is the one number a \
             wrong build gets wrong invisibly — a remove that took the neighbour and a correct \
             one both change the shape. Line: `{}`.",
            shell.get("nodes"),
            shell.raw
        )));
    }
    if trace.events(ENGINE_REMOVE).count() == 0 {
        return Ok(Some(format!(
            "★ the shell asked to delete the node and the DOCUMENT did not change: \
             `{SHELL_REMOVE}` is present and `{ENGINE_REMOVE}` is not. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ a node was DELETED and reached the engine: `{}`",
        shell.raw
    ));

    // --- 6: ★★★ take it below the floor — the refusal must be SHOWN -------
    //
    // A `/Polygon` keeps three. The shape now has three, so this gesture cannot
    // be honoured — and the whole subject of the operator's report is what
    // happens next. A build that drops the gesture silently passes every
    // assertion above and is the defect.
    let trace = session.trace()?;
    let Some(node_one) = declared(&trace, ui_rect, &format!("{NODE_REGION}.1")) else {
        return Ok(Some(format!(
            "after the delete the shape published no `{NODE_REGION}.1`, so the anchor list did \
             not shrink with the shape — `canvas::annotnodes::nodes` re-reads the annotation \
             every frame, so this would mean the painter and the document disagree. Regions \
             seen: {}.",
            list(&declared_names(&trace, ui_rect, NODE_REGION))
        )));
    };
    let frame = session.frame()?;
    let before_declines = trace.events(SHELL_DECLINED).count();
    drag_holding(
        &driver,
        &[vk::CONTROL, vk::SHIFT],
        frame.declared_center(node_one),
        anywhere,
    )?;
    session.settle(40);

    let trace = session.trace()?;
    if trace.events(SHELL_DECLINED).count() <= before_declines {
        return Ok(Some(format!(
            "★★★ A NODE WAS DELETED FROM A THREE-NODE POLYGON, OR THE GESTURE VANISHED. No new \
             `{SHELL_DECLINED}` line. A `/Polygon`'s floor is three and \
             `EditSession::reshape_annotation_preview` refuses below it by name, so one of two \
             things happened: the preflight was not asked (check \
             `canvas::annotnodes::resolved`), or it was asked, refused, and the release raised \
             nothing — which is the silence this whole feature answers. Trace: {}.",
            session.trace_path().display()
        )));
    }
    // ★★★ …and it is ON SCREEN. The trace line says the shell knows; this says
    // the OPERATOR was told. They are different claims and only the second is
    // the operator's complaint.
    let trace = session.trace()?;
    if declared(&trace, ui_rect, DECLINE_REGION).is_none() {
        return Ok(Some(format!(
            "★★★ THE REFUSAL WAS SILENT. `{SHELL_DECLINED}` is in the trace, so the shell knew \
             the gesture could not be honoured — and `{DECLINE_REGION}` was never published, so \
             the status bar drew nothing and the operator watched a node refuse to go with no \
             explanation anywhere. That is the founding defect shape of this project and the \
             exact thing his report describes. Look at `app::actions::apply`'s \
             `AnnotAction::DeclineNodeEdit` arm, `decline::record_markup_node_refused`, and \
             `Declined::MarkupNodeRefused`'s `line()`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ and the refusal was SHOWN: the status bar published its decline slot");

    let shot = ctx.out("markup_node_refused.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    Ok(None)
}
