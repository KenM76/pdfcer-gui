//! `measure_perimeter_traces_and_closes` — the perimeter tool arms, takes
//! vertices, closes on the first one, and the dimension reaches the engine.
//!
//! # Why this check exists at all, given `measure_linear` already passes
//!
//! Because the perimeter tool is the first gesture on this tab with **no fixed
//! arity**, and every link that differs from the linear tool is a link no
//! existing check crosses:
//!
//! | # | link | new here? |
//! |---|---|---|
//! | 1 | the ribbon item arms `Measure(Perimeter)` | new command, new variant |
//! | 2 | a click becomes a **vertex** rather than one of three fixed picks | new |
//! | 3 | the running total accumulates across an unbounded run | new |
//! | 4 | a click on the **first vertex** closes the ring | new — no other tool has this ending |
//! | 5 | closing **commits**, and the engine accepts it | the same `add_dimension` the others reach |
//!
//! Links 2–4 are the ones that cannot be unit-tested: `PerimeterPick` can be
//! driven from a test without a window and is, but *whether a click on the page
//! reaches it* and *whether the ring test converts screen pixels to the right
//! vertex* are properties of call sites and of a coordinate conversion, and
//! both are only observable in a running process.
//!
//! # ★ The assertion that matters most is the CLOSING one
//!
//! `closes_the_ring` compares the click against the first vertex **in canvas
//! space**, which means it crosses the page→canvas bridge — the conversion
//! `canvas::mapping`'s header calls *the classic silent defect*, because the
//! canvas is Y-down from the page's top-left with `/Rotate` applied and every
//! point pdfcer publishes is Y-up from the un-rotated CropBox.
//!
//! Get it wrong and the ring never closes: the operator clicks the first corner
//! of a footprint they have just traced and gets a fifth vertex on top of it.
//! No unit test can see that, because the arithmetic is individually correct on
//! both sides and it is the *caller* that mixes two spaces of the same type.
//!
//! # What this check does NOT assert, and where that is recorded
//!
//! **That the number is scaled.** The value goes through `Group` by
//! construction — it is a `DimensionKind`, so `format_measurement` handles it
//! with the same code path every other dimension uses, and the engine pins that
//! with its own tests. Asserting it here would need the fixture to carry a
//! calibrated group, which is a second gesture (Set scale) inside a check about
//! a first one. The tool-panel running total is formatted through the same
//! function and is the surface where a scale defect would show.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode the tool is offered in. Measuring reads the page and authors a ce
/// dimension, which Review permits — the same mode `measure_linear` drives in,
/// deliberately, so a mode-gating change breaks both together.
const MODE: &str = "review";
/// ★ The **Length** tool's ribbon item — the same gesture that never closes.
///
/// Checked at the end of this run rather than in a second check, because the
/// property worth asserting about it is a NEGATIVE one relative to Perimeter —
/// *clicking the first vertex adds a vertex instead of closing* — and a
/// negative is only meaningful beside the positive it differs from. Two checks
/// would let the pair drift: Perimeter's could stop closing and Length's would
/// still pass.
const LENGTH_ITEM: &str = "ribbon.item.measure.length";
/// The `Debug` spelling of `CanvasTool::Measure(MeasureKind::PathLength)`.
const LENGTH_ARM: &str = "Measure(PathLength)";
/// The Measure tab.
const TAB: &str = "ribbon.tab.measure";
/// The ribbon item that arms the tool.
const ITEM: &str = "ribbon.item.measure.perimeter";
/// `measure-tool tool=…` — the canvas reporting what armed.
const ARM_EVENT: &str = "measure-tool";
/// The `Debug` spelling of `CanvasTool::Measure(MeasureKind::Perimeter)`.
const ARM_VALUE: &str = "Measure(Perimeter)";
/// `measure-perimeter-vertex n=… length_pt=…` — one line per vertex taken.
const VERTEX_EVENT: &str = "measure-perimeter-vertex";
/// `measure-finish via=… kind=perimeter page=…` — the gesture ended.
const FINISH_EVENT: &str = "measure-finish";
/// `add-dimension …` — the engine accepted it and the document changed.
const COMMIT_EVENT: &str = "add-dimension";
/// The prefix each vertex handle is published under, suffixed with its index.
const VERTEX_REGION: &str = "canvas.dimension-vertex";
/// `move-dimension-vertex …` — the engine accepted a reshaped corner.
const VERTEX_COMMIT: &str = "move-dimension-vertex";
/// `dimension-vertex id=… index=… dx=… dy=… snap=…` — the SHELL's own report of
/// the same drag, which is where the snap answer rides. Distinct from
/// [`VERTEX_COMMIT`], which is the engine's acknowledgement: one says what was
/// asked for and the other says it was accepted, and a check that conflated
/// them could not tell a shell that never asked from an engine that refused.
const SHELL_VERTEX: &str = "dimension-vertex";

/// The four corners, as fractions of the page box.
///
/// A rectangle rather than an irregular shape, because the total is then
/// arithmetic a reader can check by hand from the page size printed in the
/// report — and a check whose expected value cannot be verified by eye is a
/// check that can be wrong in the same direction as the code.
const CORNERS: [(f64, f64); 4] = [(0.30, 0.30), (0.60, 0.30), (0.60, 0.60), (0.30, 0.60)];

/// See the module documentation.
pub struct MeasurePerimeterTracesAndCloses;

impl Check for MeasurePerimeterTracesAndCloses {
    fn name(&self) -> &'static str {
        "measure_perimeter_traces_and_closes"
    }

    fn defect(&self) -> &'static str {
        "the Perimeter tool is on the ribbon and arms nothing, clicks on the page do not become \
         vertices, the running total does not accumulate, or clicking the first vertex fails to \
         close the ring — which leaves an operator who has just traced a footprint unable to \
         finish it, and is invisible to every unit test because the ring test crosses the \
         page-to-canvas bridge and both sides of it are individually correct"
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
             ribbon control and five points on the page. Reported as SKIPPED rather than passed: \
             a check that did not run has learned nothing.",
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
                "cannot read a page size from {}. The harness needs the page box to turn this check's four corner fractions into points, and the page height to flip PDF y                  (up) into window y (down). Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("measure_perimeter.trace.txt"));
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

    // --- 1: the mode, then the tab, then the tool -------------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, TAB) else {
        return Ok(Some(format!(
            "the `{MODE}` mode declares no `{TAB}` region, so the Measure tab is not offered and \
             no tool on it can be reached."
        )));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)? else {
        return Ok(Some(format!(
            "the Measure tab declares no `{ITEM}`, on the band or in the overflow. Items \
             declared: {}.",
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
            "the Perimeter item was clicked and no `{ARM_EVENT} tool={ARM_VALUE}` followed — the \
             ribbon control is drawn and reachable and the canvas tool did not arm. Either \
             `shell::commands::measure_for_command` does not map `measure.perimeter`, or \
             `canvas::tool::arm_measure` was not reached."
        )));
    }
    report.note("Measure ▸ Perimeter armed the tool");

    // --- 2: four vertices --------------------------------------------------
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
    let mut aimed = Vec::with_capacity(CORNERS.len());
    for (fx, fy) in CORNERS {
        let point = DocPoint::new(0, fx * page.width_pt, fy * page.height_pt);
        aimed.push(frame.to_screen(mapping.doc_to_window(point)?));
    }

    let mut lengths: Vec<f64> = Vec::new();
    for (n, screen) in aimed.iter().enumerate() {
        driver.click_at(*screen)?;
        session.settle(12);
        let trace = session.trace()?;
        let taken = trace.events(VERTEX_EVENT).count();
        if taken != n + 1 {
            return Ok(Some(format!(
                "click {} of {} produced {taken} `{VERTEX_EVENT}` line(s) in total, not {}. A \
                 click that becomes no vertex means `canvas::measure::click`'s Perimeter arm was \
                 not reached — the gesture machine swallowed the press, the click landed outside \
                 the page rect, or the point resolution declined it.",
                n + 1,
                CORNERS.len(),
                n + 1
            )));
        }
        let line = trace.events(VERTEX_EVENT).last().expect("just counted one");
        let Some(length) = line.get("length_pt").and_then(|v| v.parse::<f64>().ok()) else {
            return Err(Error::new(format!(
                "the `{VERTEX_EVENT}` line carries no readable `length_pt=`: `{}`",
                line.raw
            )));
        };
        lengths.push(length);
    }
    // ★★ THE PREVIEW, AND ONLY A SCREENSHOT CAN SAY IT DREW.
    //
    // Captured after the last vertex, with the pointer still on the page.
    // Everything asserted above reads the trace, and on 2026-08-20 all of it
    // passed on a build whose preview drew NOTHING - `gesture_in_progress` had
    // not learned the perimeter's pick, so `measure::preview` returned before
    // painting a single segment. The operator reported it the same day: *"both
    // these tools need a preview just like the measure tool has."*
    //
    // A picture rather than an assertion, deliberately. Asserting on accent
    // pixels would need the theme's colour and the polyline's exact route
    // through the canvas transform, which is a second derivation of the thing
    // under test. The artifact is what a human looks at, and its absence from
    // the report is what says nobody has.
    let shot = ctx.out("measure_perimeter_preview.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    report.note(format!(
        "four vertices taken; running total after each: {}",
        lengths
            .iter()
            .map(|v| format!("{v:.1}"))
            .collect::<Vec<_>>()
            .join(" → ")
    ));

    // ★ The total must RISE with every vertex after the first. A total that
    // stayed still would mean the vertex was recorded and the length function
    // is not summing it; a total that fell would mean the vertex list is being
    // replaced rather than appended. Both look identical on screen — the
    // polyline is drawn from the same list either way — which is why this is
    // asserted on the number rather than on the picture.
    if lengths.first() != Some(&0.0) {
        return Ok(Some(format!(
            "the FIRST vertex reported a running total of {:.2} pt, not 0. One point is no \
             segments, so anything but zero means `length_points` is measuring something that is \
             not there.",
            lengths[0]
        )));
    }
    for pair in lengths.windows(2) {
        if pair[1] <= pair[0] {
            return Ok(Some(format!(
                "the running total did not rise: {:.2} → {:.2} pt. Every vertex after the first \
                 adds a segment, so a total that stalls means the vertex reached the pick and the \
                 sum did not, and a total that falls means the list is being replaced rather than \
                 appended. Totals: {lengths:?}.",
                pair[0], pair[1]
            )));
        }
    }

    // --- 3: ★★ click the first vertex again — the ring closes and commits --
    let before_commits = session.trace()?.events(COMMIT_EVENT).count();
    driver.click_at(aimed[0])?;
    session.settle(30);
    let trace = session.trace()?;

    let Some(finish) = trace.last(FINISH_EVENT) else {
        let taken = trace.events(VERTEX_EVENT).count();
        return Ok(Some(format!(
            "clicking the first vertex again did NOT close the ring: no `{FINISH_EVENT}` line, \
             and the vertex count is now {taken} (it was {}). ★ This is the defect this check \
             exists for. `perimeter::closes_the_ring` compares the click against the first vertex \
             in CANVAS space, which crosses the page-to-canvas bridge — Y-down from the page's \
             top-left with /Rotate applied, against Y-up from the un-rotated CropBox. Get it \
             wrong and the operator clicks the corner they started at and gets a fifth vertex on \
             top of it. Trace: {}.",
            CORNERS.len(),
            session.trace_path().display()
        )));
    };
    if finish.get("via") != Some("close-ring") {
        return Ok(Some(format!(
            "the gesture finished, but not by closing: `{}`. A click on the first vertex must \
             close the ring — `via=double-click` here would mean the click was read as the second \
             of a pair, and `via=command` cannot happen without a ribbon press.",
            finish.raw
        )));
    }

    let commits = trace.events(COMMIT_EVENT).count();
    if commits <= before_commits {
        return Ok(Some(format!(
            "the ring closed and no `{COMMIT_EVENT}` line followed, so `Action::CommitDimension` \
             was raised and the engine did not accept it — or it was never raised. \
             `perimeter::commit` refuses below three vertices, and four were taken. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let committed = trace.last(COMMIT_EVENT).expect("just counted one");
    report.note(format!(
        "★ the ring closed and the dimension reached the engine: `{}`",
        committed.raw
    ));

    // ★ And the pick is EMPTIED. A second Finish must not author the same shape
    // again from a set the operator believes they have spent — the same rule
    // `circular::commit` states, and the one place a three-ending tool could
    // most easily get it wrong.
    let after = session.trace()?.events(VERTEX_EVENT).count();
    if after != CORNERS.len() {
        return Ok(Some(format!(
            "after the commit the tool has traced {after} vertices in total, not {}. The commit \
             must EMPTY the pick, so a stray extra line here means the closing click also landed \
             as a vertex.",
            CORNERS.len()
        )));
    }
    // --- 4: ★★ the LENGTH tool is the same gesture and does NOT close ------
    //
    // The operator, 2026-08-20: *"add a length tool that works like the
    // perimeter tool without needing to close the profile."*
    //
    // Asserted here rather than in a check of its own, because what is worth
    // proving about Length is a NEGATIVE relative to Perimeter — clicking the
    // first vertex adds a vertex instead of closing — and a negative is only
    // meaningful beside the positive it differs from. Two separate checks would
    // let the pair drift apart: Perimeter's could stop closing and Length's
    // would go on passing, which is the two tools silently becoming one.
    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, LENGTH_ITEM)? else {
        return Ok(Some(format!(
            "the Measure tab declares no `{LENGTH_ITEM}`. Perimeter is there and Length is not, so the open-path half of the gesture is unreachable — which is the operator's ask verbatim."
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    if !session
        .trace()?
        .events(ARM_EVENT)
        .any(|l| l.get("tool") == Some(LENGTH_ARM))
    {
        return Ok(Some(format!(
            "the Length item was clicked and no `{ARM_EVENT} tool={LENGTH_ARM}` followed."
        )));
    }

    let before = session.trace()?.events(VERTEX_EVENT).count();
    // ★ The finish count BEFORE this section, and the reason it is needed is a
    // harness defect this check produced on its first run: the Perimeter half
    // above ends with a `close-ring` finish, so asking for the LAST finish line
    // in the whole trace reported the Length tool as having closed when it had
    // taken all five clicks correctly. A confident, specific, wrong accusation
    // — the same shape as reading a refusal as an absence. Anything asserted
    // about a second gesture in one process has to be scoped to lines that
    // arrived after the first one ended.
    let finishes_before = session.trace()?.events(FINISH_EVENT).count();
    for screen in &aimed {
        driver.click_at(*screen)?;
        session.settle(10);
    }
    // …and now the click that WOULD close a perimeter.
    driver.click_at(aimed[0])?;
    session.settle(20);
    let trace = session.trace()?;
    let taken = trace.events(VERTEX_EVENT).count() - before;
    if taken != CORNERS.len() + 1 {
        return Ok(Some(format!(
            "★ the Length tool took {taken} vertices from {} clicks. The last click landed on the first vertex, and for THIS tool that is an ordinary vertex — a path that returns to where it started is a perfectly ordinary path. Fewer means it closed the ring like Perimeter does, which is the one behaviour the operator asked it not to have.",
            CORNERS.len() + 1
        )));
    }
    if let Some(finish) = trace
        .events(FINISH_EVENT)
        .skip(finishes_before)
        .find(|l| l.get("via") == Some("close-ring"))
    {
        return Ok(Some(format!(
            "★ the Length tool CLOSED: `{}`. `perimeter::click` gates the ring on the armed kind being Perimeter, so that gate is gone or inverted.",
            finish.raw
        )));
    }
    report.note(format!(
        "★ the Length tool took all {taken} clicks as vertices, so the first-vertex click did not close it"
    ));
    // --- 5: ★★ SELECT THE SHAPE AND DRAG A CORNER --------------------------
    //
    // The rest of the operator's ask: *"I want to be able to edit the endpoints
    // of the lines to adjust the shape."*
    //
    // Two links no unit test can cross:
    //
    // 1. **the handles are grabbable WHERE THEY ARE DRAWN.** The hit test and
    //    the painter each convert page → canvas → screen independently. If they
    //    disagree, the operator sees a handle and cannot grab it — "visible
    //    control, silently inert" in its purest form, and the failure this
    //    project keeps meeting.
    // 2. **the release RE-MEASURES.** This is the first ce-dimension verb that
    //    deliberately changes what a dimension measures, so a drag that moved
    //    the shape and left the number alone would be the worst outcome
    //    available: a drawing whose caption disagrees with its own geometry.
    // ★ PUT THE TOOL DOWN FIRST. With a measure tool armed a click on the page
    // is a PICK, not a selection — `gesture::press_kind`'s highest rung — so
    // the shape would never become selected and the handles would never be
    // asked for. `V` is the select tool's chord.
    //
    // This cost one UNVERIFIED run to notice, and the check reported it as
    // UNVERIFIED rather than as "the handles are not drawn", which is the
    // distinction that made it a two-minute fix instead of an investigation
    // into the painter.
    //
    // ★★ **The POINTER first, the chord as the fallback — 2026-08-28.**
    //
    // `driving::arm_select_from_ribbon`'s doc carries the measurements: with a
    // dock panel raised, `V` was observed arriving zero times in six runs, and
    // it failed *silently* — no line anywhere — so the check that met it
    // reported the wrong subject. This check has been passing on `V` and raises
    // no panel of its own, so the chord stays reachable rather than being
    // deleted; what changes is the order. A click does not depend on keyboard
    // focus and it has an oracle of its own.
    //
    // ★ Keeping the fallback is deliberate and it is the difference from
    // `scale_switch`, which refuses one: a check whose subject sits inside a
    // panel it raised must SKIP rather than retry with a primitive measured not
    // to work there. Here the pen merely has to go down, and a route that has
    // worked all along is better than a skip.
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        report.note(
            "the ribbon route to the select tool was unavailable, so the pen was put down with \
             the `V` chord instead — the route this step used until 2026-08-28",
        );
        driver.press(vk::V)?;
        session.settle(10);
    }
    // ★★ ON THE INK, not in the middle of the ring.
    //
    // This aimed at the centre of the traced square until 2026-08-20 and the
    // run reported UNVERIFIED — correctly. A click there now MISSES, because a
    // dimension is hit-tested on its drawn segments rather than on its bounding
    // box, so the empty middle of a perimeter belongs to whatever is behind it.
    // That is the operator's own report, working.
    //
    // So the click goes to the midpoint of the first edge — between corners 0
    // and 1 — which is ink. A check that kept aiming at the hole would have
    // reported the selection as broken while it was behaving correctly.
    driver.click_at(frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        f64::midpoint(CORNERS[0].0, CORNERS[1].0) * page.width_pt,
        f64::midpoint(CORNERS[0].1, CORNERS[1].1) * page.height_pt,
    ))?))?;
    session.settle(18);
    let trace = session.trace()?;
    let Some(handle) = declared(&trace, ui_rect, &format!("{VERTEX_REGION}.0")) else {
        report.note(format!(
            "clicking the finished shape declared no `{VERTEX_REGION}.0`. Either the click did not select the dimension, or the handles are not drawn. The corner drag is UNVERIFIED. Regions seen: {}.",
            list(&crate::checks::driving::declared_names(
                &trace,
                ui_rect,
                VERTEX_REGION
            ))
        ));
        return Ok(None);
    };
    report.note("the selected perimeter published its vertex handles");

    let frame = session.frame()?;
    let from = frame.declared_center(handle);
    // ★ The destination is a DOCUMENT point, converted the same way the four
    // corners were, rather than "the handle plus ninety pixels". Two reasons:
    // it survives a zoom change between runs, and it states where the corner is
    // going in the units the assertion is about. A pixel offset would be a
    // magic number whose only meaning is "far enough".
    //
    // Well outside the traced square, so the new perimeter cannot coincide with
    // the old one by arithmetic accident.
    let to = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        0,
        0.12 * page.width_pt,
        0.12 * page.height_pt,
    ))?);
    let before = session.trace()?.events(VERTEX_COMMIT).count();
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(VERTEX_COMMIT).count() <= before {
        return Ok(Some(format!(
            "★ a corner handle was published at {handle:?}, the drag started on it, and no `{VERTEX_COMMIT}` line followed. The handle is drawn and inert — either `dimdrag::vertex_at` and the painter disagree about where it is, or the press did not classify as `DragKind::DimensionVertex`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the corner drag reached the engine: `{}`",
        trace.last(VERTEX_COMMIT).expect("just counted one").raw
    ));

    // ★★ …and the shell ASKED whether the corner should snap.
    //
    // `ui-conventions/drag-moves.md` D6, and the gap the 2026-08-20 sweep
    // named: *"a vertex drag does not snap, while the tool that placed that
    // vertex does — so you can pick a corner onto geometry and then be unable
    // to put it back."*
    //
    // What is asserted is that the `snap=` FIELD EXISTS, not that a candidate
    // was found. The destination above is a document point chosen for being far
    // from the traced square, so whether anything is near it is a fact about
    // the fixture; asserting a hit would make this check pass or fail on which
    // drawing it was pointed at. A build that never wired the query reports no
    // field at all, and that is a fact about the BUILD — which is the
    // distinction this whole harness exists to keep.
    let Some(line) = trace.last(SHELL_VERTEX) else {
        return Ok(Some(format!(
            "the corner drag committed and the shell traced no `{SHELL_VERTEX}` line, so there is nothing to read the snap answer off. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let Some(snap) = line.get("snap") else {
        return Ok(Some(format!(
            "★ THE CORNER DRAG NEVER ASKED WHETHER TO SNAP: `{}` carries no `snap=` field. \
             The tool that PLACED this vertex snaps to endpoints, midpoints and \
             intersections; a drag that does not is a corner an operator can put onto a line \
             and then never put back. Look at `canvas::interact`'s \
             `GestureOutcome::DimensionVertex` arm — and at `needs_targets`, which is where \
             `Resize` and `Handle` both failed the same way before it: without the \
             decomposition the query has nothing to ask, so the drag works perfectly and \
             silently never snaps. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "the corner drag asked the snap query and got `snap={snap}` (a miss is expected here \
         — the destination is deliberately far from the traced square)"
    ));
    Ok(None)
}
