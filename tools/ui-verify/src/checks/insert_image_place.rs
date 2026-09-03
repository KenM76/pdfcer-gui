//! `the_insert_window_steps_aside_so_you_can_point` — **press the button, the
//! window goes, click the page, the window comes back with your numbers in
//! it.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O66**, 2026-08-31:
//!
//! > *"anything we are inserting like this should have an option in its
//! > dialogue box to place it with the mouse instead of by positional
//! > co-ordinates."*
//!
//! ## ★★★ Why this needs a DRIVEN check and not only unit tests
//!
//! Every piece of this arm is unit-tested and every piece passes in isolation:
//! `canvas::placing`'s arm/cancel/result cycle, `dialogs::placing`'s derived
//! `hidden`, `text::placing`'s sentences. What no unit test can see is the
//! **join** — that pressing a real button in a real window makes that window
//! stop being drawn, that a real click on the canvas is routed to the placement
//! arm rather than to the marquee underneath it, and that the window comes
//! back.
//!
//! That is the shape this project has shipped broken before: every part tested,
//! the join untested, the join wrong. Eight green unit tests once sat under a
//! feature that did one of its fourteen steps.
//!
//! ## ★★ The oracle is the REGION'S ABSENCE, and that is deliberate
//!
//! `dialogs::insert_image` publishes `insert-image.place` on every frame it
//! draws. While a placement is pending it draws nothing at all — its `show`
//! returns *still open* before the window is built — so the region stops being
//! declared.
//!
//! ⇒ *"The window stepped aside"* is observable as that region going, and *"it
//! came back"* as the region returning. Neither is an inference from silence:
//! step A confirms the region was there first, so an absence at step B cannot
//! be confused with a dialog that never opened. That confirmation is the
//! precaution this suite's own rule asks for — before asserting on an absent
//! line, ask what else happened.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | open the dialog with an image | `insert-image.place` declared |
//! | B | press *Place it on the page…* | `place-armed kind=Image`, and the region goes |
//! | C | click the page | `place-result kind=Image llx=… lly=…` |
//! | D | the window is back | `insert-image.place` declared again |
//!
//! ★ Step C asserts the **result**, not an inserted image, because the button
//! places nothing: it fills the numbers in and the operator still presses
//! Insert. Asserting an insert here would assert a different feature, and would
//! pass against a build that bypassed the dialog entirely.
//!
//! ## ★★ What this check deliberately does NOT drive: Escape
//!
//! The other half of O66 is that Escape abandons a placement and brings the
//! window back. It is not driven here, and the reason is not laziness:
//!
//! - `scale_switch`'s header records, with six runs of evidence, that **a
//!   keystroke is not a reliable harness primitive** in this shell. A step that
//!   fails half the time would make this check's real subject unreportable.
//! - The property is already unfalsifiable by construction. `hidden` is
//!   *derived* from the pending record, so whatever clears that record — the
//!   Escape claimant, a mode change, the document closing — un-hides the
//!   window. There is no per-route flag to forget, and `dialogs::placing`'s own
//!   test cancels through a module `PlaceHandoff` never calls and watches the
//!   window return.
//!
//! ⇒ A driven Escape would re-test a mechanism that cannot fail per route, at
//! the cost of a flaky step. Step D is the load-bearing observation and it *is*
//! driven.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, frame_of, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then the insert-image command.
const INVOKE: &str = "mode.edit,edit.insert_image";
/// The seam that answers the native picker without a human at the keyboard.
const IMAGE_PATH_ENV: &str = "PDFCER_DIAG_IMAGE_PATH"; // ui-text-exempt: an environment variable name
/// The button this check presses — `dialogs::insert_image::REGION_PLACE`.
const PLACE_REGION: &str = "insert-image.place"; // ui-text-exempt: trace region name, never displayed
/// The line `canvas::placing::arm` writes.
const ARMED_EVENT: &str = "place-armed"; // ui-text-exempt: trace event name, never displayed
/// The line `canvas::placing::finish` writes.
const RESULT_EVENT: &str = "place-result"; // ui-text-exempt: trace event name, never displayed
/// Every region the insert window publishes shares this prefix.
const REGION_PREFIX: &str = "insert-image"; // ui-text-exempt: trace region prefix, never displayed
/// Where on the page the click lands, as a fraction of the sheet.
///
/// ★ Away from the edges and away from the centre. A default placement already
/// sits near the middle, so a click there could pass against a build that
/// ignored the pointer completely.
const AT: (f64, f64) = (0.31, 0.62);
/// How far the recorded placement may sit from the point that was clicked.
///
/// ★ Generous against the real error and tight against the real defect. The
/// measured agreement is under half a point; one screen pixel at the fit zoom
/// this fixture opens at is about 3 pt, so a few points absorbs the click
/// quantisation. The mirror this check was written after is about 300 pt out,
/// and a centre-defaulting build is of the same order.
const TOLERANCE_PT: f64 = 8.0;

pub struct TheInsertWindowStepsAside;

impl Check for TheInsertWindowStepsAside {
    fn name(&self) -> &'static str {
        "the_insert_window_steps_aside_so_you_can_point"
    }

    fn defect(&self) -> &'static str {
        "a picture can only be positioned by typing four numbers — or the Place button is drawn \
         and arms nothing, or arms a placement whose window never returns, which leaves the \
         operator with no dialog and no route back to one"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is a button press and a click on the \
             page; both are real pointer gestures.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to point at."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "could not read a page size from {}, and this check aims in page fractions. \
                 Pass --page-size.",
                pdf.display()
            ))
        })?,
    };

    // The image the picker will "choose".
    //
    // ★ Its own file rather than sharing `insert_image`'s, because both checks
    // write into one output directory and a shared path would make each run
    // depend on whether the other had run first. That order-dependence is a
    // hazard this suite has already been bitten by once.
    let image = ctx.out("insert-image-place-fixture.png");
    let png = crate::png::encode_rgb(8, 8, &[0x40_u8; 8 * 8 * 3]).ok_or_else(|| {
        Error::new(
            "the harness's own PNG encoder refused a fixture it was handed the right number of \
             bytes for. Nothing about the application has been tested; this is the check's own \
             precondition.",
        )
    })?;
    std::fs::write(&image, &png).map_err(|e| {
        Error::new(format!(
            "cannot write the fixture image to {}: {e}",
            image.display()
        ))
    })?;

    // --- launch ------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("insert-image-place.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((IMAGE_PATH_ENV.to_owned(), image.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with the insert window open on an 8x8 fixture",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    // --- A: the window is open and offering the button ---------------------
    let trace = session.trace()?;
    if declared(&trace, ui_rect, PLACE_REGION).is_none() {
        return Err(Error::new(format!(
            "`{PLACE_REGION}` is not declared, so either the insert window did not open or this \
             build does not offer the button. SKIPPED rather than failed: an absence here is \
             indistinguishable from a picker that was never answered, and this check has not \
             reached its own subject. Regions beginning `{REGION_PREFIX}`: {}.",
            list(&declared_names(&trace, ui_rect, REGION_PREFIX))
        )));
    }
    report.note("★ the insert window is open and offers `Place it on the page…`");

    // --- B: press it, and the window steps aside ---------------------------
    //
    // ★★★ **`frame_of`, never `session.frame()`.** The insert window is a real
    // OS viewport — its regions are tagged `viewport="0BC0"` and its rects
    // start at `x=0` because they are relative to ITS client origin, not the
    // application's. Converting them against the main window aims hundreds of
    // points away, at numbers that look perfectly ordinary.
    //
    // ⇒ This check was written with `session.frame()` and failed on its first
    // run reporting "THE BUTTON ARMED NOTHING" — a confident, precise and
    // entirely wrong diagnosis of the subject, produced by a click that never
    // touched the button. `D:/dev/rag/egui/` carries three instances of the
    // same shape; this is the fourth.
    //
    // ★ The frame is resolved from a trace read now rather than the one above:
    // a stale coordinate is the other harness hazard this project has written
    // up twice, and re-reading is free.
    let trace = session.trace()?;
    let button = declared(&trace, ui_rect, PLACE_REGION)
        .ok_or_else(|| Error::new("the button was retired between two frames."))?;
    let frame = frame_of(&session, &trace, ui_rect, PLACE_REGION)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(ARMED_EVENT).count() == 0 {
        return Ok(Some(format!(
            "★★★ THE BUTTON ARMED NOTHING: it was pressed and no `{ARMED_EVENT}` line followed. \
             The press reached a control that records a request nobody drains — look at \
             `app::frame`'s placement round trip and at `DialogsState::take_place_request`. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    if declared(&trace, ui_rect, PLACE_REGION).is_some() {
        return Ok(Some(format!(
            "★★ THE WINDOW DID NOT STEP ASIDE: `{ARMED_EVENT}` was traced and `{PLACE_REGION}` is \
             still declared, so the dialog is sitting over the page the operator has just been \
             asked to point at. `dialogs::insert_image::show` must consult \
             `PlaceHandoff::hidden` and return BEFORE the window is built. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★ the window stepped aside — its regions stopped being declared");

    // --- C: click the page, and the placement lands ------------------------
    let at = DocPoint::new(0, AT.0 * page.width_pt, AT.1 * page.height_pt);
    driver.click_at(aim(ctx, &session, page, at)?)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(result) = trace.last(RESULT_EVENT) else {
        return Ok(Some(format!(
            "★★★ THE CLICK PLACED NOTHING: no `{RESULT_EVENT}` line after a click on the page \
             with a placement armed. Something else claimed the press — the marquee, a text \
             sweep, a selection — which means the placement rung is not above them in \
             `canvas::gesture`, or `canvas::clicking`'s arm is never reached. Trace: {}.",
            session.trace_path().display()
        )));
    };
    // ★★★ **WHERE it landed, not merely that it landed.**
    //
    // The first version of this check asserted the line's existence and
    // nothing else, and it would have passed the build it was written
    // against — which recorded every placement MIRRORED in y, because
    // `canvas::placing::click` wrote a canvas-space point into a
    // `page_tree::Rect` without the flip that PDF user space needs. Click
    // near the top of the sheet, get the picture near the bottom.
    //
    // ⇒ A presence assertion on a coordinate-producing feature is a check
    // that cannot fail in the way the feature actually fails. This one
    // compares against the point the harness aimed at, in the space the
    // application publishes.
    let (Some(llx), Some(lly)) = (result.get_f32("llx"), result.get_f32("lly")) else {
        return Ok(Some(format!(
            "`{RESULT_EVENT}` was traced without both coordinates, so this check cannot say where the placement went: `{}`. Trace: {}.",
            result
                .field_names()
                .iter()
                .map(|f| (*f).to_owned())
                .collect::<Vec<_>>()
                .join(" "),
            session.trace_path().display()
        )));
    };
    let (want_x, want_y) = (at.x, at.y);
    let off = (f64::from(llx) - want_x).hypot(f64::from(lly) - want_y);
    if off > TOLERANCE_PT {
        // ★ The mirror is named explicitly, because it is the failure this
        // check has actually seen and its signature is unmistakable: the x
        // agrees to a fraction of a point and the y is the page height minus
        // the one aimed at.
        let mirrored = (f64::from(lly) - (page.height_pt - want_y)).abs() < TOLERANCE_PT;
        return Ok(Some(format!(
            "★★★ THE PLACEMENT LANDED IN THE WRONG PLACE: the click was at PDF ({want_x:.1}, {want_y:.1}) and `{RESULT_EVENT}` reports ({llx:.1}, {lly:.1}) — {off:.1} pt away.{} Trace: {}.",
            if mirrored {
                format!(
                    " ★★ AND IT IS THE Y MIRROR: {:.1} − {want_y:.1} = {lly:.1}, so a canvas point was written into a page rect without the flip into PDF user space. `canvas::placing::click` must convert through `markup::band::endpoints`, as every sibling arm in `canvas::clicking` does.",
                    page.height_pt
                )
            } else {
                String::new()
            },
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the placement landed at PDF ({llx:.1}, {lly:.1}) — {off:.1} pt from the point that was clicked, in the right space and the right half of the page"
    ));

    // --- D: …and the window is back ----------------------------------------
    if declared(&trace, ui_rect, PLACE_REGION).is_none() {
        return Ok(Some(format!(
            "★★★ THE WINDOW DID NOT COME BACK: the placement landed and `{PLACE_REGION}` is not \
             declared again. This is the stranding the design exists to make unrepresentable — \
             no dialog, and no route to one. `hidden` must be DERIVED from \
             `canvas::placing::pending`, so that clearing the record IS the un-hide; a stored \
             flag has to be cleared on every exit route and one of them will be missed. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★★ …and the window came back, with the placement in its fields");
    Ok(None)
}
