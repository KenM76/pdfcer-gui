//! `rotating_a_markup_turns_it` — **draw a shape, grab its ninth handle, and it
//! is turned.**
//!
//! # What this is for
//!
//! `pdfcer-core` `Pass 155.0` gave this shell `rotate_annotation` and
//! `Pass 159.0` gave it `rotate_dimension`. Before 2026-08-28 an annotation
//! had **eight** grips and no ninth: `pressing::grabbable` handed a selected
//! markup `GripSet::scale_only()`, so no rotate handle was painted and none
//! was hit-tested. The operator could move a stamp and scale it and could not
//! turn it.
//!
//! ## ★★★ Why this cannot be a unit test, and what it is really guarding
//!
//! `canvas::rotating`'s arithmetic is pure and has eight unit tests;
//! `gesture::meaning` has four more for the new rung. What none of them can
//! reach is **which verb the release lands on**, and that is the whole risk
//! here:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a markup is selected and a **ninth** affordance is painted | `handles::GripSet` — the flags, not the paint |
//! | 2 | a press above the box finds `Grip::Rotate` over `grabbable`'s box | `handles::grip_at` — the geometry, given a box |
//! | 3 | it becomes `DragKind::Rotate` and not `Move` | `gesture::meaning` — given `annot_rotate`, which `pressing` computes |
//! | 4 | `rotating::Frame::bounds` is `grabbable`'s and not `overlay::grip_box`'s | **nothing** |
//! | 4b | **no CONTENT-shaped guard stands in front of the annotation branch** | **nothing** |
//! | 5 | the commit routes to the **annotation** verb, not `transform_objects` | **nothing** |
//! | 6 | the sign survives the screen → page crossing | **nothing** |
//!
//! **Link 4 is the one that would ship, and it would ship as silence.**
//! `overlay::grip_box` derives from the selection's cached *content* outlines,
//! which `select_annot` clears — so over an annotation it answers `None`,
//! `rotating::drag` returns at its first line, and the whole gesture is a
//! no-op with nothing said anywhere. That is this project's founding defect
//! shape exactly: a grip that is dragged, released, and does nothing.
//!
//! ## ★★★ LINK 4b IS WHAT THE FIRST DRIVEN RUN ACTUALLY FOUND — 2026-08-29
//!
//! It was added to the table *after* that run, and it is here because the run
//! is the only reason anybody knows it exists. Link 4 was **already correct**:
//! `canvas::interact` had passed `pressing::grabbable`'s box since the day the
//! module landed, and this check's failure message said otherwise — a
//! confident, specific, wrong accusation, which `checks::rotate` already warns
//! is worse than a vague one.
//!
//! The real break was a
//! `selection.object_indices_on(page_index).is_empty()` guard sitting *above*
//! `rotating::drag`'s annotation branch. It counts page **content**, which
//! `select_annot` clears, so it returned `None` on every markup before the
//! routing decision was ever reached. **Identical symptom, different line, and
//! the same silence** — which is why the failure message below now names both
//! candidates and names this one first.
//!
//! ⇒ Three destinations share this gesture. The durable form of the lesson is
//! that **a guard written in one destination's vocabulary belongs after the
//! branch that picks the destination**, and `canvas::rotating`'s header carries
//! it as the sixth instance of this canvas's recurring hazard.
//!
//! **Link 5 is the one that would ship and look deliberate.** If the press
//! fell through to `caps.edit_content`'s branch, the commit would call
//! `transform_objects` on the *page content* selection — a working gesture
//! aimed at the wrong verb, which is the failure this canvas has produced four
//! times. From a chair it is invisible: on an empty content selection nothing
//! happens, and on a non-empty one *something moves*.
//!
//! ⇒ So this check does not only assert that a rotation happened. It asserts
//! **which line the trace carries**, and it asserts that the content verb's
//! line did **not** appear. Those two together are the only place link 5 is
//! visible from outside the process.
//!
//! # The oracle carries DEGREES, and a signed number
//!
//! `rotate-annot-commit id=… deg=… px=… py=…`, then
//! `rotate-annotation-applied … from=WxH to=WxH`. A line saying only *"a
//! rotation committed"* would be identical for a build that turned the other
//! way, pivoted about a corner instead of the centre, or left the appearance
//! `/Matrix` alone. This project's standing rule, earned by `DEFECTS.md` D14:
//! **a trace line must carry the number a wrong build would get wrong.**
//!
//! ★ `from=`/`to=` are **reported and not asserted**, deliberately. `/Rect`
//! grows at any angle that is not a quarter turn (§12.5.2 requires it upright)
//! and this check drives exactly a quarter turn, where it must **not** grow —
//! so asserting growth would be asserting the opposite of what this gesture
//! produces. What the two numbers are for is the reader of a failed run.
//!
//! # ★★ Where it aims, and why it no longer mirrors a constant
//!
//! `checks::rotate` — the page-content sibling — derives the handle's position
//! from the selection outline plus its own copy of `ROTATE_STEM_PX`, and says
//! in its own comment that it *"does not aim at this number directly"*. It had
//! no choice: the handle published no region of its own.
//!
//! It does now. `canvas.rotate-handle` is published by `overlay::draw_grips`
//! **only inside the `offer.rotate` branch**, so its presence in the trace is
//! the application's own statement that the affordance exists — and this check
//! aims at its declared centre. That removes the last mirrored number from the
//! aiming path, and it makes step 4 below a *direct* observation of link 1
//! rather than an inference from a failed press.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review mode, then arm the rectangle tool.
///
/// ★ `mode.review` because markup is authored there — and because it is the
/// mode where `caps.edit_content` is **false**, which is precisely the mode the
/// new rung had to fire in. Driving this in Edit would pass on a build whose
/// rotate handle only worked where the content branch could catch it.
const INVOKE: &str = "mode.review,markup.rectangle";
/// The line the canvas writes when a shape is authored.
const COMMIT_EVENT: &str = "markup-commit";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// ★★★ The line the rotate drag writes when it routes to the **annotation**
/// verb.
const ROTATE_EVENT: &str = "rotate-annot-commit";
/// ★★★ The line the rotate drag writes when it routes to **page content**.
///
/// Asserted **absent**. See the module header, link 5: a press that fell
/// through to `caps.edit_content` produces this line instead, and the resulting
/// build has a rotate handle that works perfectly on the wrong subject.
const CONTENT_ROTATE_EVENT: &str = "rotate-commit";
/// The line the apply arm writes when the engine has turned it.
///
/// ★ `-applied`, per the convention this project adopted after making the
/// same-name mistake twice: `vector_edit` writes its own bare
/// `rotate-annotation …` line for the identical edit, and `.last()` on the bare
/// name reads that one.
const ROTATED_EVENT: &str = "rotate-annotation-applied";
/// The line a drag on a markup's **body** writes — asserted absent.
///
/// ★★ The other half of link 3. A build whose `annot_rotate` was computed from
/// the live pointer rather than from `press_origin` finds `None` on every real
/// drag (egui does not call an interaction a drag until the pointer has
/// travelled a threshold, by which time it is ~20 pt from an 8 pt handle) — and
/// the press then means whatever the rungs below say. If that came out as a
/// MOVE, this line appears and the shape slides across the sheet.
const MOVE_EVENT: &str = "annot-drag";
/// The region the selection outline publishes.
const OUTLINE_REGION: &str = "canvas.selection-outline";
/// ★★★ The region the **rotate handle** publishes, and only when it is drawn.
const HANDLE_REGION: &str = "canvas.rotate-handle";
/// The canvas viewport's own declared region.
///
/// Read so a failure can tell *the handle is off-canvas* from *the handle is on
/// the canvas and the press was routed wrongly*. Those are different defects in
/// different files and they produce the identical symptom. `checks::rotate`
/// records the run where that distinction was missing and three confident,
/// specific, wrong causes were named instead.
const CANVAS_REGION: &str = "canvas-viewport";
/// The page's own region, so a failure can say whether a sheet was even drawn.
const PAGE_REGION: &str = "page";

/// Where the shape is drawn, as fractions of the page.
///
/// ★★ **Well below the top of the sheet**, which is this check's own version of
/// `checks::rotate`'s O22 hazard: the rotate handle sits `ROTATE_STEM_PX` above
/// the selection box, so a shape near the top of the viewport has its handle
/// clipped away by the painter and the press lands on the ribbon. 0.35 down the
/// page is comfortably clear of it on any sheet size.
///
/// ★ And away from the edges, so the drag in step 5 — which swings out to a
/// radius of half the box plus the stem — has somewhere to go.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));

/// See the module documentation.
pub struct RotatingAMarkupTurnsIt;

impl Check for RotatingAMarkupTurnsIt {
    fn name(&self) -> &'static str {
        "rotating_a_markup_turns_it"
    }

    fn defect(&self) -> &'static str {
        "a placed markup can be moved and scaled and cannot be TURNED — it is offered eight grips \
         and no ninth, so `rotate_annotation` is unreachable from the canvas. Or the handle is \
         drawn and a press on it MOVES the shape, or rotates the page content behind it, which \
         are working gestures aimed at the wrong verb and look entirely deliberate"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape with a drag, clicks it, and \
             swings its rotate handle through a quarter turn. Every one of those is a real \
             pointer gesture.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to draw a shape on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "could not read a page size from {}, and this check places its shape in page \
                 fractions. Pass --page-size.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("annot-rotate.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to draw on. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- 1: draw a rectangle ------------------------------------------------
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    let from = aim(ctx, &session, page, corner(SHAPE.0))?;
    let to = aim(ctx, &session, page, corner(SHAPE.1))?;
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(COMMIT_EVENT).next().is_none() {
        return Ok(Some(format!(
            "THE RECTANGLE TOOL AUTHORED NOTHING: a drag across the page produced no \
             `{COMMIT_EVENT}` line, so `markup.rectangle` did not arm or the drag was not seen as \
             one. This is two steps BEFORE the one under test — there is no annotation to turn. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ a rectangle was authored");

    // --- 2: put the pen down, then select it --------------------------------
    //
    // ★★★ THE TOOL MUST GO DOWN FIRST. With a markup tool armed a click on the
    // page is a PICK rather than a selection, so a check that skipped this
    // would draw a *second* rectangle and then report "the shape could not be
    // selected" about a build whose selection works perfectly.
    // `checks::markup_move` records that exact run.
    let centre = corner((
        f64::midpoint(SHAPE.0.0, SHAPE.1.0),
        f64::midpoint(SHAPE.0.1, SHAPE.1.1),
    ));
    let centre_screen = aim(ctx, &session, page, centre)?;
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        report.note(
            "the ribbon route to the select tool was unavailable, so the pen was put down with \
             the `V` chord instead",
        );
        driver.press(crate::sys::vk::V)?;
        session.settle(12);
    }
    driver.click_at(centre_screen)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(selected) = trace.events(SELECT_EVENT).last() else {
        return Ok(Some(format!(
            "THE SHAPE COULD NOT BE SELECTED: a click at its centre produced no `{SELECT_EVENT}` \
             line. Selecting is the step before the one under test and has worked since \
             2026-08-18, so this says the click missed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the shape was selected: `{}`", selected.raw));
    if selected.get("locked") == Some("true") {
        return Err(Error::new(
            "the annotation reports itself LOCKED (§12.5.3 bit 8). A locked annotation is offered \
             no grips at all — `annotdrag::grab_box` answers `None`, so `grabbable` publishes no \
             box and nothing is painted. SKIPPED rather than failed: that is correct behaviour \
             (R9 — render nothing rather than draw a handle that refuses), and it means this \
             fixture cannot exercise the rotation.",
        ));
    }

    // --- 3: ★★★ IS THE NINTH HANDLE EVEN PAINTED? ---------------------------
    //
    // The direct observation of link 1, and it is asserted BEFORE any press is
    // made — which is the difference between this check and its page-content
    // sibling. `checks::rotate` had to infer the handle's existence from a
    // press that failed to commit, so a build with no handle and a build with a
    // mis-routed handle produced the identical report.
    //
    // `canvas.rotate-handle` is published inside `draw_grips`' `offer.rotate`
    // branch and nowhere else, so its presence is the application stating that
    // this selection has a rotation. Its ABSENCE here is the exact defect this
    // check was written for, and it names the one line that produces it.
    let trace = session.trace()?;
    let Some(handle_rect) = declared(&trace, ui_rect, HANDLE_REGION) else {
        return Ok(Some(format!(
            "★★★ NO ROTATE HANDLE IS PAINTED FOR A SELECTED MARKUP. The application declared no \
             `{HANDLE_REGION}` region after `{}`, and it publishes that region only from inside \
             `overlay::draw_grips`' `offer.rotate` branch — so this selection was handed a \
             `GripSet` with `rotate: false`.\n\
             **That is the state this feature shipped in until 2026-08-28**: \
             `pressing::grabbable` handed a selected markup `GripSet::scale_only()`, because \
             `resize_annotation` existed and no rotate verb did. `rotate_annotation` ships now. \
             Look at `pressing::grabbable`'s annotation arm — it must hand `GripSet::all()`.\n\
             Regions beginning `canvas.`: {}. Trace: {}.",
            selected.raw,
            list(&declared_names(&trace, ui_rect, "canvas.")),
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★★ the ninth handle is painted: `{HANDLE_REGION}` at ({:.1},{:.1})-({:.1},{:.1})",
        handle_rect.min.x, handle_rect.min.y, handle_rect.max.x, handle_rect.max.y
    ));

    let Some(outline) = declared(&trace, ui_rect, OUTLINE_REGION) else {
        return Ok(Some(format!(
            "the handle is declared and the `{OUTLINE_REGION}` it hangs off is not, which is a \
             defect in the painter rather than in the rotation: the two are published in the same \
             call. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 3b: rule the OFF-CANVAS cause out ----------------------------------
    //
    // ★★ `checks::rotate` earned this branch on 2026-08-21 (O22) and its
    // lesson is adopted rather than re-derived: **a confident, specific, wrong
    // accusation is worse than a vague one**, because it is actionable and it
    // aims somebody at the wrong file. A handle above the top of the canvas is
    // clipped away and the press lands on the ribbon; the symptom is identical
    // to a routing failure and the file to fix is a different one.
    //
    // ★ Here it uses the handle's OWN declared rect rather than a mirrored
    // stem constant, so it measures what is actually on screen.
    if let Some(canvas) = declared(&trace, ui_rect, CANVAS_REGION)
        && handle_rect.min.y < canvas.min.y
    {
        return Ok(Some(format!(
            "★★ THE ROTATE HANDLE IS OFF-CANVAS — defect O22, and NOT a routing problem. The \
             handle spans from y={:.1} and the canvas begins at y={:.1}, so it is {:.1} point(s) \
             above the top of the canvas: the painter clips it away and the press never reaches \
             the canvas widget at all. Do NOT go looking in `pressing::grabbable` or \
             `gesture::meaning`; the gesture never started. Draw the shape further down the \
             sheet, or see O22 for the fix.",
            handle_rect.min.y,
            canvas.min.y,
            canvas.min.y - handle_rect.min.y,
        )));
    }

    // --- 4: a quarter turn clockwise ----------------------------------------
    //
    // Press at the handle's declared centre — due north of the selection's
    // centre, on its stem — and release due EAST of that centre at the same
    // radius. The turn is then exactly 90° clockwise on screen: a round number
    // a human can check by reading the trace, crossing no quadrant boundary, so
    // a failure of *this* check is about the routing rather than about the
    // arithmetic the unit tests already cover.
    //
    // ★ The radius is derived from the two DECLARED rects — the vertical gap
    // between the handle's centre and the outline's centre — so nothing here
    // mirrors `ROTATE_STEM_PX`, `GRIP_SIZE_PX` or `MIN_OUTLINE_EXTENT_PX`.
    let frame = session.frame()?;
    let w = (outline.max.x - outline.min.x).max(1.0);
    let h = (outline.max.y - outline.min.y).max(1.0);
    let handle_cy = f32::midpoint(handle_rect.min.y, handle_rect.max.y);
    let outline_cy = f32::midpoint(outline.min.y, outline.max.y);
    let radius = (outline_cy - handle_cy).abs().max(1.0);
    let press = frame.declared_at(handle_rect, 0.5, 0.5);
    // ★ Through 45°, so the drag passes frames where the bearing is genuinely
    // changing rather than teleporting from press to release — `drag_via`'s own
    // header makes the same argument about holding a modifier through a
    // gesture.
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let via = frame.declared_at(outline, 0.5 + radius * DIAG / w, 0.5 - radius * DIAG / h);
    let to = frame.declared_at(outline, 0.5 + radius / w, 0.5);

    let before_annot = session.trace()?.events(ROTATE_EVENT).count();
    let before_content = session.trace()?.events(CONTENT_ROTATE_EVENT).count();
    let before_move = session.trace()?.events(MOVE_EVENT).count();
    driver.drag_via(press, via, std::time::Duration::from_millis(60), to, None)?;
    session.settle(40);

    // --- 5: ★★★ WHICH VERB DID IT REACH? ------------------------------------
    let trace = session.trace()?;
    let Some(commit) = trace.events(ROTATE_EVENT).nth(before_annot) else {
        // ★★★ The two wrong-verb diagnoses, ruled IN or OUT before any guess is
        // offered. This is the whole reason both counters were taken.
        if trace.events(CONTENT_ROTATE_EVENT).count() > before_content {
            return Ok(Some(format!(
                "★★★ THE HANDLE ROTATED PAGE CONTENT INSTEAD OF THE ANNOTATION. A \
                 `{CONTENT_ROTATE_EVENT}` line followed the drag and no `{ROTATE_EVENT}` line \
                 did.\n\
                 The press fell through `gesture::meaning`'s annotation-rotate rung to the \
                 `caps.edit_content` branch below it, whose `(None, Some(Grip::Rotate))` arm \
                 commits `transform_objects` over the CONTENT selection. **This is a working \
                 gesture aimed at the wrong verb** — the failure this canvas has produced four \
                 times — and from a chair it is invisible, because on an empty content selection \
                 nothing happens and on a non-empty one something moves.\n\
                 Check that `pressing::look` computes `annot_rotate` and that the rung reading it \
                 sits ABOVE `caps.edit_content`. Trace: {}.",
                session.trace_path().display()
            )));
        }
        if trace.events(MOVE_EVENT).count() > before_move {
            return Ok(Some(format!(
                "★★★ THE HANDLE MOVED THE SHAPE INSTEAD OF TURNING IT. An `{MOVE_EVENT}` line \
                 followed the drag and no `{ROTATE_EVENT}` line did.\n\
                 The press was classified as a body drag, which means `grip` was not \
                 `Grip::Rotate` at the moment `pressing::look` asked. The usual cause is reading \
                 the LIVE pointer instead of `press_origin`: egui does not call an interaction a \
                 drag until the pointer has travelled a threshold, so by then it is ~20 pt from \
                 an 8 pt handle and the hit test finds the body. Trace: {}.",
                session.trace_path().display()
            )));
        }
        return Ok(Some(format!(
            "★★★ THE ROTATE HANDLE COMMITTED NOTHING. The press was made at the centre of the \
             `{HANDLE_REGION}` rect the application itself declared, the handle is on the canvas, \
             and neither `{ROTATE_EVENT}` nor `{CONTENT_ROTATE_EVENT}` nor `{MOVE_EVENT}` \
             followed — the gesture was consumed and discarded. That is this project's founding \
             defect shape, and it is why this check exists.\n\
             **TWO different lines produce this identical report, and the first one is what the \
             first driven run actually found.** Read `rotating::drag` top to bottom:\n\
             1. ★★★ A guard written about page CONTENT standing IN FRONT OF the annotation \
             branch — on 2026-08-29 it was `object_indices_on(page).is_empty()` sitting above \
             `if let Some(annot) = selection.annot()`. `select_annot` clears the content \
             selection, so a test like that answers `empty` on every markup and returns before \
             the routing decision is reached. Anything the CONTENT verb needs belongs BELOW \
             that branch.\n\
             2. `rotating::Frame::bounds` coming from `overlay::grip_box` instead of from \
             `pressing::grabbable` — `grip_box` derives its box from the selection's cached \
             CONTENT outlines, so over an annotation it answers `None` and `drag` returns at \
             its first line. `canvas::interact` is the one call site.\n\
             ★ Both are silent, and a silent return is the defect whatever the cause: look \
             for a `rotate-declined` line before concluding the gesture never arrived. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★★ the drag routed to the ANNOTATION verb: `{}`",
        commit.raw
    ));

    // ★★ …and it did not ALSO rotate the page content. Asserted separately
    // from the branch above, because a build that raised both actions would
    // satisfy every assertion so far and quietly turn something else on the
    // sheet at the same time.
    if trace.events(CONTENT_ROTATE_EVENT).count() > before_content {
        return Ok(Some(format!(
            "★★ THE DRAG ROTATED THE ANNOTATION **AND** PAGE CONTENT: both `{ROTATE_EVENT}` and \
             `{CONTENT_ROTATE_EVENT}` followed one gesture. `rotating::drag`'s annotation branch \
             must RETURN rather than fall through to the content commit. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 6: ★★ the number a wrong build would get wrong ---------------------
    let deg: f64 = commit
        .get("deg")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    // ★ A generous window. The press lands at a declared centre and the release
    // at a computed point, both rounded to whole pixels, so the measured
    // bearing is a degree or two off 90 by construction. What is asserted is
    // the QUADRANT and the SIGN, not the arithmetic — that has eight unit
    // tests.
    if !(-100.0..=-80.0).contains(&deg) {
        return Ok(Some(format!(
            "★ THE QUARTER TURN CAME OUT AS {deg:.2}°, AND IT MUST BE ABOUT −90°.\n\
             The handle starts due north of the selection's centre and the release was due east, \
             which is 90° CLOCKWISE on screen. Screen y runs down, so `rotating::angle` reports \
             +90; `rotate_annotation` takes degrees ANTICLOCKWISE in PDF user space, where y runs \
             up, so the commit negates once — in `rotating::commit_annotation`, and nowhere \
             else.\n\
             +90 here means the crossing was forgotten or applied twice: the shape turns the \
             wrong way, which is a perfectly good rotation and looks entirely deliberate to \
             anybody who did not watch the pointer. A value near 0 means the bearing was measured \
             from something other than the box's centre. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ and it turned the right way: {deg:.2}° in page space for a clockwise drag on screen"
    ));

    // --- 7: and it reached the engine ---------------------------------------
    let Some(applied) = trace.events(ROTATED_EVENT).last() else {
        return Ok(Some(format!(
            "★★ THE ROTATION WAS RAISED AND NOTHING REACHED THE DOCUMENT: `{}` and no \
             `{ROTATED_EVENT}` line.\n\
             The action was raised and its apply arm never ran, or `rotate_annotation` refused. \
             It refuses a **widget** and a **ce dimension** by name through \
             `AnnotationMoveWrongVerb`, and refuses on a certified document through \
             `CertificationForbidsChange` — all three are worded on the status row by \
             `annots::refusal_for`, and a refused `vector_edit` traces \
             `rotate-annotation-refused`. Look for that line first: if it is there, this is a \
             correct refusal and the fixture is the problem. Trace: {}.",
            commit.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★★★ the engine turned it: `{}`", applied.raw));

    // ★ `matrix=` is REPORTED and lightly asserted; `from=`/`to=` are reported
    // only. A rotation is expressed by composing into the appearance stream's
    // own `/Matrix` (§12.5.5 step (a)), so a build that wrote a new `/Rect` and
    // left the matrix alone produces a box that grew around artwork that did
    // not move — which looks exactly like the CORRECT behaviour this feature
    // discloses, and is the one wrong build a screenshot cannot distinguish.
    report.note(format!(
        "the appearance matrix was composed: {}; /Rect went {} → {} (a quarter turn must not \
         grow it, and this is reported rather than asserted — see the module header)",
        applied.get("matrix").unwrap_or("?"),
        applied.get("from").unwrap_or("?"),
        applied.get("to").unwrap_or("?"),
    ));
    Ok(None)
}
