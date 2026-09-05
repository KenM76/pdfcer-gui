//! `a_corner_can_be_added_and_taken_away` — the operator's *"I also can't edit
//! or delete nodes of a markup shape once it is drawn"*, for the one shape
//! where the engine can do it.
//!
//! ## ⬜ THIS CHECK HAS NOT BEEN RUN
//!
//! **Written 2026-09-05 and NOT DRIVEN by the session that wrote it.** Another
//! track owned the pointer for the whole of that session — it was driving the
//! canvas-input work — and two driven runs on one machine corrupt each other:
//! `Driver::raise_and_confirm` brings *its* window to the front, and a check
//! that loses the foreground mid-gesture reports the application as broken.
//! So this is a check that has been written, compiled and registered, and has
//! never seen a running binary. **Its passing is unproven and its failure
//! messages are untested prose.** Run it before quoting it.
//!
//! ## What it is for
//!
//! `pdfcer-core` has had three vertex verbs since `Pass 107.0` and this shell
//! called exactly one of them:
//!
//! | verb | `edit.rs` | called before 2026-09-05 |
//! |---|---|---|
//! | `move_dimension_vertex` | 37892 | yes — `app/actions/dimensions.rs` |
//! | `insert_dimension_vertex` | 37927 | **no** |
//! | `remove_dimension_vertex` | 37955 | **no** |
//!
//! So a corner could be dragged and the number of corners could not change.
//! `canvas::dimdrag::count_edit` now reaches both, on a Ctrl-drag and a
//! Ctrl+Shift-drag from a corner handle **with the Points tool armed**, and
//! this check is the only instrument that can say whether that arrives.
//!
//! ## ★★★ Why every link here needs a running window
//!
//! `canvas::dimdrag::tests` already asserts the arithmetic, the preflight and
//! the tool gate without a window — and all of it would pass on a build where
//! the operator can do none of this. Six things stand between those tests and
//! the operator's hand, and not one is observable in-process:
//!
//! | # | link | why a unit test cannot see it |
//! |---|---|---|
//! | 1 | the Points tool **arms in Review** | `Capabilities::for_mode` reads the real manifest, and the mode is entered by clicking a segment |
//! | 2 | `A` reaches `view.tool_node` in Review | a chord is filtered by **tab** visibility, and the ribbon item is `shown_when("mode.edit_content")` — so the chord is the ONLY route and nothing in-process exercises it |
//! | 3 | Ctrl survives the OS → winit → egui path during a drag | `press_held`'s own note: a modifier that goes down and up inside one frame's event batch is applied and undone before the event that was meant to carry it |
//! | 4 | the press on the handle classifies as `DragKind::DimensionVertex` | `canvas::pressing` resolves it from a screen position through two coordinate spaces |
//! | 5 | `count_edit` is reached with a `session` that holds the record | the sidecar is read from the real `EditSession`, not a fixture struct |
//! | 6 | the engine accepts it and the annotation is regenerated | `insert-dimension-vertex` is the funnel's own line, and only a real edit writes it |
//!
//! ★★ Link 2 is the one most likely to be the reason this check fails first,
//! and it is worth reading before diagnosing anything else. The ribbon and rail
//! items for `view.tool_node` both carry `shown_when("mode.edit_content")` in
//! `shell::manifest`, which another track owned on the day this was written.
//! **In Review the tool is reachable by the `A` chord and by nothing else.** A
//! failure at the arming step is therefore a manifest gap rather than a canvas
//! one, and the message at that step says so.
//!
//! ## ★★ What it deliberately does NOT assert
//!
//! **That the drawing changed shape on screen.** A screenshot is captured as an
//! artifact for a human, and the assertions read the trace, for
//! `measure_perimeter`'s stated reason: asserting on accent pixels needs the
//! theme's colour and the polyline's route through the canvas transform, which
//! is a second derivation of the thing under test.
//!
//! **The refusal path** — a closed triangle that cannot lose a corner. That is
//! asserted without a window in `canvas::dimdrag::tests`, against the engine's
//! real predicate, and driving it would need a second traced shape with three
//! corners for a fact the engine owns. What is worth driving is the path that
//! crosses six process boundaries, which is the one below.
//!
//! ## The shape it traces, and why four corners
//!
//! A closed square. Four is one more than the minimum for a ring, which is what
//! makes **both** halves of this check legal on the same shape: the insert
//! takes it to five and the remove takes it back to four, so a single traced
//! shape exercises the two verbs in sequence and the corner count is a running
//! number a reader can check by eye — `4 → 5 → 4`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode this is driven in, and the choice is the point of the check.
///
/// Review is the mode a ce dimension is authored in and the mode whose
/// `edit_content` is false, so it is the one where the Points tool's gate
/// changed on 2026-09-05. Driving this in Edit would exercise the same code
/// with the interesting half of the gate short-circuited.
const MODE: &str = "review";
/// The Measure tab, which Review is shown.
const TAB: &str = "ribbon.tab.measure";
/// The ribbon item that arms the Perimeter tool.
const ITEM: &str = "ribbon.item.measure.perimeter";
/// `measure-tool tool=…` — the canvas reporting what armed.
const ARM_EVENT: &str = "measure-tool";
/// The `Debug` spelling of `CanvasTool::Measure(MeasureKind::Perimeter)`.
const ARM_VALUE: &str = "Measure(Perimeter)";
/// `measure-perimeter-vertex n=… length_pt=…` — one line per vertex taken.
const VERTEX_EVENT: &str = "measure-perimeter-vertex";
/// `add-dimension …` — the engine accepted the traced shape.
const COMMIT_EVENT: &str = "add-dimension";
/// The prefix each corner handle is published under, suffixed with its index.
const VERTEX_REGION: &str = "canvas.dimension-vertex";
/// `command-declined id=view.tool_node reason=…` — the chord was refused.
///
/// ★★★ **The Points tool's arming is asserted as the ABSENCE of this line, and
/// that is forced by the instrument rather than chosen.** `canvas::tool::select`
/// writes no trace at all — it is a two-line memory write, and the four tools
/// that DO trace (`text-tool`, `markup-tool`, `measure-tool`,
/// `text-edit-tool`) each do it from their own toggle rather than from
/// `select`. So there is no positive line saying *the Points tool is now armed*.
///
/// What there is, is `app::dispatch::navigate`'s decline, written on exactly
/// the path this check is about: before 2026-09-05, pressing `A` in Review
/// produced this line and armed nothing.
///
/// ⇒ The absence is a weak signal on its own and it is **not carrying the
/// assertion alone**. The positive proof is downstream and total:
/// `canvas::dimdrag::intent` returns `Move` unless
/// `canvas::tool::active(ctx).is_node()`, so a `dimension-vertex-insert` line
/// **cannot** be produced by a build where the tool did not arm. Step 5 is
/// therefore the real evidence, and this step exists to make a failure legible
/// — without it a build with the old gate would fail at step 5 with a message
/// about modifiers.
///
/// ★ Reported rather than worked around: a one-line trace in
/// `canvas::tool::select` would make this a positive assertion and would serve
/// every future tool check. The session that wrote this owned only that file's
/// Node capability arm.
const TOOL_DECLINED: &str = "command-declined";
/// The command the chord raises.
const NODE_COMMAND: &str = "view.tool_node";
/// `dimension-vertex-insert id=… index=… corners=… x=… y=…` — the SHELL's own
/// report of the gesture.
const SHELL_INSERT: &str = "dimension-vertex-insert";
/// `dimension-vertex-remove …` — its twin.
const SHELL_REMOVE: &str = "dimension-vertex-remove";
/// `insert-dimension-vertex page=0 n=1 epoch=… disclosures=…` — the **funnel's**
/// line, which is the engine's acknowledgement.
///
/// ★★ Distinct from [`SHELL_INSERT`] deliberately, and the distinction is the
/// whole reason both are asserted: one says the gesture was understood and the
/// other says the document changed. A check that read only the first could not
/// tell a shell that never asked from an engine that refused — which is
/// `measure_perimeter`'s own note about `dimension-vertex` versus
/// `move-dimension-vertex`, and the reason `check-trace-names.py` exists.
const ENGINE_INSERT: &str = "insert-dimension-vertex";
/// `remove-dimension-vertex …` — its twin.
const ENGINE_REMOVE: &str = "remove-dimension-vertex";

/// The four corners of the traced square, as fractions of the page box.
const CORNERS: [(f64, f64); 4] = [(0.30, 0.30), (0.60, 0.30), (0.60, 0.60), (0.30, 0.60)];

/// See the module documentation.
pub struct ACornerCanBeAddedAndTakenAway;

impl Check for ACornerCanBeAddedAndTakenAway {
    fn name(&self) -> &'static str {
        "a_corner_can_be_added_and_taken_away"
    }

    fn defect(&self) -> &'static str {
        "a shape the operator has drawn cannot gain or lose a corner — the Points tool will not \
         arm in the mode the shape was drawn in, or the modifier never reaches the drag, or the \
         gesture reaches no engine verb. His report, verbatim: \"I also can't edit or delete \
         nodes of a markup shape once it is drawn.\" Two of the three verbs `pdfcer-core` \
         provides for this were called from nowhere in this shell until 2026-09-05"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        // ★ Stated in the report as well as in this file's header, because a
        // sweep summary is what gets read and a module header is not.
        report.note(
            "⬜ NOT DRIVEN BY ITS AUTHOR — written 2026-09-05 while another track held the \
             pointer. Its failure messages have never been seen.",
        );
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Drag from `from` to `to` with `modifiers` held down for the whole gesture.
///
/// ★★★ **The modifier is held ACROSS the press, the walk and the release, and
/// that is not politeness.** `Driver::press_held`'s own note records the
/// finding: a modifier that goes down and up inside one frame's event batch can
/// be applied and undone before the event it was meant to carry is dispatched,
/// because modifier state reaches egui through winit's `ModifiersChanged`. A
/// harness that pressed Ctrl just before the button would produce a plain drag
/// and report *"the corner was moved, not added"* about a perfectly working
/// build.
///
/// It also matches what `canvas::dimdrag::intent` actually does: it reads the
/// modifiers **live on every frame**, so a Ctrl released half way through the
/// walk turns the gesture back into a move — deliberately, and visibly, because
/// the preview follows. Holding it throughout is the only way to drive the
/// gesture the operator's hand makes.
///
/// Written here rather than on `Driver` because `Driver::drag_via` already
/// takes a single `Option<Key>` modifier and this needs two; widening that
/// signature is a change to a shared instrument, and the session that wrote
/// this check did not own `tools/ui-verify/src/input.rs`.
fn drag_holding(
    driver: &Driver,
    modifiers: &[u16],
    from: ScreenPoint,
    to: ScreenPoint,
) -> Result<()> {
    crate::sys::with_modifiers(modifiers, || driver.drag(from, to))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, a \
             ribbon control, five points on the page, presses a chord and performs two \
             modifier-held drags. Reported as SKIPPED rather than passed: a check that did not \
             run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. The harness needs the page box to turn this \
                 check's four corner fractions into points. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("dimension_corner_count.trace.txt"));
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

    // --- 1: Review, the Measure tab, the Perimeter tool -------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, TAB) else {
        return Ok(Some(format!(
            "the `{MODE}` mode declares no `{TAB}` region, so no measure tool can be reached and \
             there is no shape to put corners on."
        )));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)? else {
        return Ok(Some(format!(
            "the Measure tab declares no `{ITEM}`. Items declared: {}.",
            list(&crate::checks::driving::declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.measure."
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
            "the Perimeter item was clicked and no `{ARM_EVENT} tool={ARM_VALUE}` followed, so \
             there is no way to draw the shape this check is about."
        )));
    }

    // --- 2: trace a closed square and commit it ---------------------------
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
    for screen in &aimed {
        driver.click_at(*screen)?;
        session.settle(12);
    }
    let taken = session.trace()?.events(VERTEX_EVENT).count();
    if taken != CORNERS.len() {
        return Ok(Some(format!(
            "{taken} of {} corners were taken, so the shape this check reshapes was never drawn. \
             This is `measure_perimeter`'s subject, not this check's — run that one first.",
            CORNERS.len()
        )));
    }
    driver.click_at(aimed[0])?;
    session.settle(30);
    let trace = session.trace()?;
    if trace.events(COMMIT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "the ring did not close and commit, so there is no ce dimension to put a corner on. \
             Again `measure_perimeter`'s subject. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("a four-corner closed shape was drawn and reached the engine");

    // --- 3: select it, so its corner handles are published ----------------
    //
    // ★ The click goes to the MIDPOINT OF AN EDGE, not to the middle of the
    // square. `dimdrag::annot_shapes` hit-tests the drawn INK rather than the
    // `/Rect`, deliberately — a perimeter traced round a building whose box was
    // clickable would make the drawing underneath unselectable — so the middle
    // of the square is a hole. `measure_perimeter` learned this the hard way
    // and its note carries the argument.
    driver.click_at(frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        f64::midpoint(CORNERS[0].0, CORNERS[1].0) * page.width_pt,
        f64::midpoint(CORNERS[0].1, CORNERS[1].1) * page.height_pt,
    ))?))?;
    session.settle(18);
    let trace = session.trace()?;
    let Some(handle_one) = declared(&trace, ui_rect, &format!("{VERTEX_REGION}.1")) else {
        return Ok(Some(format!(
            "the selected shape published no `{VERTEX_REGION}.1`, so there is no corner to aim \
             at. Either the click did not select it or the handles are not drawn. Regions seen: \
             {}.",
            list(&crate::checks::driving::declared_names(
                &trace,
                ui_rect,
                VERTEX_REGION
            ))
        )));
    };

    // --- 4: ★★★ arm the Points tool — IN REVIEW ---------------------------
    //
    // The gate this whole piece of work turns on. Before 2026-09-05
    // `retire_forbidden`'s Node arm was `caps.edit_content`, which Review does
    // not have, so the tool retired the instant it armed and the chord answered
    // with a decline sentence.
    //
    // ★ The `A` chord and not a ribbon press, and that is forced rather than
    // chosen: `view.tool_node`'s ribbon and rail items both carry
    // `shown_when("mode.edit_content")`, so in Review there is no control to
    // click. A chord is filtered by TAB visibility and View is in every mode,
    // so the key gets through. See this module's header.
    driver.press(vk::A)?;
    session.settle(16);
    let trace = session.trace()?;
    if let Some(declined) = trace
        .events(TOOL_DECLINED)
        .find(|l| l.get("id") == Some(NODE_COMMAND))
    {
        return Ok(Some(format!(
            "★ THE POINTS TOOL WILL NOT ARM IN {}: the `A` chord was declined — `{}`. Two \
             candidates, in the order to check them. (1) `app::dispatch::navigate`'s \
             `view.tool_node` arm, which must read `edit_content || author_measure`. (2) \
             `canvas::tool::retire_forbidden`'s Node arm, which must use the IDENTICAL \
             predicate — this tool's second subject is a ce dimension's corners, and \
             reshaping one is a MEASURE edit, which is the capability Review has. A \
             disagreement between the two shows as a tool that arms and is retired on the \
             next frame: a flicker with no sentence attached. Trace: {}.",
            MODE.to_uppercase(),
            declined.raw,
            session.trace_path().display()
        )));
    }
    report.note(
        "the `A` chord was not declined in Review — see this module's `TOOL_DECLINED` \
         note for why that is a weak signal on its own, and why step 5 is the real evidence",
    );

    // --- 5: ★★★ Ctrl-drag corner 1 — a corner is ADDED --------------------
    let frame = session.frame()?;
    let from = frame.declared_center(handle_one);
    // A document point rather than "the handle plus n pixels", for
    // `measure_perimeter`'s reason: it survives a zoom change between runs and
    // it states where the corner is going in the units the assertion is about.
    // Outside the square, so the new shape cannot coincide with the old one by
    // arithmetic accident.
    let out = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        0.80 * page.width_pt,
        0.42 * page.height_pt,
    ))?);
    drag_holding(&driver, &[vk::CONTROL], from, out)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(shell) = trace.last(SHELL_INSERT) else {
        return Ok(Some(format!(
            "★ CTRL-DRAGGING A CORNER ADDED NOTHING: no `{SHELL_INSERT}` line. The Points tool \
             is armed and the handle was published at {handle_one:?}, so the press reached the \
             canvas. Check, in order: (1) did Ctrl survive the drag — `canvas::dimdrag::intent` \
             reads `modifiers.command` live, and `Driver::press_held`'s note records modifier \
             state being applied and undone inside one frame's batch; (2) did the press classify \
             as `DragKind::DimensionVertex` (`canvas::pressing`); (3) does the trace carry \
             `{SHELL_REMOVE}` instead, which would mean Shift was stuck down. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let corners_after_insert = shell.get("corners").and_then(|v| v.parse::<usize>().ok());
    if corners_after_insert != Some(CORNERS.len() + 1) {
        return Ok(Some(format!(
            "the add-a-corner gesture reported `corners={corners_after_insert:?}`, not {}. The \
             count is the one number a wrong build gets wrong invisibly — an insert on the wrong \
             segment and a correct one both move the shape. Line: `{}`.",
            CORNERS.len() + 1,
            shell.raw
        )));
    }
    if trace.events(ENGINE_INSERT).count() == 0 {
        return Ok(Some(format!(
            "★ the shell asked for the corner and the DOCUMENT did not change: `{SHELL_INSERT}` \
             is present and `{ENGINE_INSERT}` is not. So `DimensionAction::InsertVertex` was \
             raised and `EditSession::insert_dimension_vertex` refused it or was never reached — \
             `app/actions/dimensions.rs`' arm and `apply::vector_edit`'s funnel are the two \
             places to look. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ a corner was ADDED and reached the engine: `{}`",
        shell.raw
    ));

    let shot = ctx.out("dimension_corner_added.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }

    // --- 6: ★★★ Ctrl+Shift-drag the new corner — it is TAKEN AWAY ---------
    //
    // The new corner is index 2 (it was inserted after index 1), and its handle
    // is re-published from the sidecar every frame, so it is aimed at by its
    // region rather than by remembering where the pointer was dropped.
    let trace = session.trace()?;
    let Some(handle_new) = declared(&trace, ui_rect, &format!("{VERTEX_REGION}.2")) else {
        return Ok(Some(format!(
            "the shape gained a corner and published no `{VERTEX_REGION}.2`, so the handle list \
             did not grow with the shape. `canvas::dimdrag::vertices` reads the sidecar record, \
             so this means the record and the drawn handles disagree — which would leave the \
             operator with a corner they can see and cannot grab. Regions seen: {}.",
            list(&crate::checks::driving::declared_names(
                &trace,
                ui_rect,
                VERTEX_REGION
            ))
        )));
    };
    let frame = session.frame()?;
    let from = frame.declared_center(handle_new);
    // Somewhere else on the page. A removal ignores the drop point — it has no
    // destination — and this check asserts that by dropping it somewhere the
    // shape has no business reaching.
    let anywhere = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        0.15 * page.width_pt,
        0.85 * page.height_pt,
    ))?);
    drag_holding(&driver, &[vk::CONTROL, vk::SHIFT], from, anywhere)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(shell) = trace.last(SHELL_REMOVE) else {
        return Ok(Some(format!(
            "★ CTRL+SHIFT-DRAGGING A CORNER REMOVED NOTHING: no `{SHELL_REMOVE}` line, which is \
             the literal half of the operator's report — \"I also can't edit or delete nodes\". \
             The add half worked in this same run, so the tool is armed and the handle is \
             grabbable; what differs is one modifier. `canvas::dimdrag::intent` maps \
             `(command, shift)` to `Remove`, and `canvas::interact` applies \
             `constrain::reposition` when Shift is down — if the trace carries `{SHELL_INSERT}` \
             here instead, Shift never arrived. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let corners_after_remove = shell.get("corners").and_then(|v| v.parse::<usize>().ok());
    if corners_after_remove != Some(CORNERS.len()) {
        return Ok(Some(format!(
            "the remove-a-corner gesture reported `corners={corners_after_remove:?}`, not {}. \
             The shape went 4 → 5 → this, so anything else means the wrong corner went or the \
             count is being derived from something other than the shape. Line: `{}`.",
            CORNERS.len(),
            shell.raw
        )));
    }
    if trace.events(ENGINE_REMOVE).count() == 0 {
        return Ok(Some(format!(
            "★ the shell asked to take the corner away and the DOCUMENT did not change: \
             `{SHELL_REMOVE}` is present and `{ENGINE_REMOVE}` is not. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ a corner was REMOVED and reached the engine: `{}`",
        shell.raw
    ));

    // ★★ …and the operator was TOLD, which is the other half of his report.
    //
    // Both verbs re-measure, so both owe the disclosure `MoveVertex` owes plus
    // the corner count. `disclosures=` is the funnel's own field and a zero
    // there means the edit landed and said nothing — the silence that produced
    // the report in the first place.
    let Some(funnel) = trace.last(ENGINE_REMOVE) else {
        return Err(Error::new(
            "the removal's funnel line vanished between two reads of the same trace file",
        ));
    };
    if funnel.get("disclosures") == Some("0") {
        return Ok(Some(format!(
            "★ the corner was removed and NOTHING WAS SAID: `{}` carries `disclosures=0`. The \
             shape changed and so did the number it prints, and the operator cannot recover the \
             old value — the geometry it was measured from is gone. \
             `crate::text::measure::vertex_removed` is the sentence and \
             `app/actions/dimensions.rs`' arm is what returns it.",
            funnel.raw
        )));
    }
    report.note(format!(
        "the removal disclosed off-canvas: `{}`",
        funnel.raw
    ));

    let shot = ctx.out("dimension_corner_removed.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    Ok(None)
}
