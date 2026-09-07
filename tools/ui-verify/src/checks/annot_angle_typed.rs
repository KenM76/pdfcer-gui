//! `the_typed_angle_turns_a_mark` — **type a number into the Properties
//! panel's Angle field, press Apply, and the mark ends at that angle.**
//!
//! # What this is for
//!
//! The operator, 2026-09-07: *"also the angle should be editable from the
//! properties."* The **read** half of that has been driven since the day it
//! shipped — `rotating_a_markup_turns_it` asserts the field shows `270.85`
//! after a `-89.15` drag. The **write** half had not been driven at all, and by
//! this project's founding rule that means it was not done, however many unit
//! tests stood behind it.
//!
//! ## ★★★ The two things only a driven run can see, and they are the whole risk
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | the Angle field is laid out, published, and reachable | **nothing** — it is behind a dock tab, a mode, and a scroll |
//! | 2 | a scrub changes the draft rather than the document | `GeometryDraft` — the arithmetic, given values |
//! | 3 | Apply is enabled by an angle change alone | `differs_from` — the predicate, not the button |
//! | 4 | **the action raised is `SetRotation` and NOT `Rotate`** | **nothing** |
//! | 5 | **the number that travels is the ABSOLUTE angle, not a delta** | **nothing** |
//!
//! **Links 4 and 5 are the ones that would ship and look correct.** Until
//! `pdfcer-core` `Pass 155.2` landed on the afternoon of 2026-09-07 this field
//! raised `AnnotAction::Rotate` with a delta computed in the panel, and that
//! *works* — on the first edit of an unturned mark, which is most of them. It
//! diverges only on the second, or on a mark somebody else turned, or after an
//! undo: exactly the conditions a person testing by hand does not set up.
//!
//! ⇒ So this check asserts the **trace's `asked=` field**, which carries the
//! number the panel handed the verb, against the number that was typed. A build
//! that passed a delta through would show `asked=` equal to the *change*
//! rather than to the destination.
//!
//! ## ★★ Why it turns the mark FIRST, before typing
//!
//! Because an absolute setter and a delta setter are **indistinguishable on an
//! unturned mark**: from 0°, *set 30* and *turn by 30* are the same edit. The
//! check therefore drags the rotate handle a quarter turn before it types, so
//! the mark is at ~270.85° when the typed value arrives — and *set 30* and
//! *turn by 30* then differ by 270°, which no tolerance can absorb.
//!
//! That is the same shape as the engine's own note about its A/B test: an
//! oracle that cannot distinguish the correct implementation from the plausible
//! wrong one is not measuring the thing its name claims.
//!
//! ## ★ What it deliberately does NOT assert
//!
//! **The final `/Rect`.** The engine reports the delta it worked out
//! (`deg=`), the rule it derived the rectangle from (`rect_derived=`) and the
//! new extent (`to=`), and all three are *reported* here rather than asserted:
//! the arithmetic has eight unit tests in `pdfcer-core` and a composability
//! test in this repository, and a second copy of the expected numbers here
//! would be a third place to maintain them. What this check owns is
//! **which verb the button reached and with what number**.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext, driving};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review mode, then arm the rectangle tool — markup is authored there.
const INVOKE: &str = "mode.review,markup.rectangle";
/// The line the canvas writes when a shape is authored.
const COMMIT_EVENT: &str = "markup-commit";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// ★★★ The line the **absolute** rotation verb writes. `Rotate`'s own line is
/// `rotate-annotation-applied`, and a build that raised the wrong action would
/// write that one instead — which is why this is asserted by name.
const SET_EVENT: &str = "set-annotation-rotation-applied";
/// The DELTA verb's line, asserted **absent** after the typed Apply.
const DELTA_EVENT: &str = "rotate-annotation-applied";
/// The line the panel writes carrying its draft, its seed and the turn it would
/// commit — read to confirm the scrub landed before Apply is pressed.
const DRAFT_EVENT: &str = "annot-geometry-draft";
/// The Angle spinner.
const ANGLE_REGION: &str = "properties.annotgeometry.angle";
/// The annotation arm's Apply button.
const APPLY_REGION: &str = "properties.annotgeometry.apply";
/// The right dock's Properties tab.
const PROPERTIES_TAB_REGION: &str = "dock.tab.file.properties";
/// The rotate handle, used to turn the mark before typing. See the header.
const HANDLE_REGION: &str = "canvas.rotate-handle";
/// The selection outline, from which the drag's release point is derived.
const OUTLINE_REGION: &str = "canvas.selection-outline";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";

/// How far to scrub the Angle field, in screen pixels.
///
/// `SPEED` in `panels::properties::geometry` is 0.5 units per pixel, so 60
/// pixels is **30 degrees** — a round number a human can check by reading the
/// trace, large enough to clear every tolerance here, and small enough that the
/// pointer stays inside a panel of any reasonable width.
const SCRUB_PX: f32 = 60.0;

/// How many scroll notches to spend looking for Apply.
///
/// The Properties panel is a scroll area whose slot is usually shorter than its
/// content, and the Angle field made it one row taller. Six notches is what
/// `geometry_fields` settled on.
const SCROLL_ATTEMPTS: usize = 6;

/// Where the shape is drawn, as fractions of the page — well clear of the top,
/// because the rotate handle sits above the selection box and a shape near the
/// top of the viewport has its handle clipped away (defect O22).
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));

/// See the module documentation.
pub struct TheTypedAngleTurnsAMark;

impl Check for TheTypedAngleTurnsAMark {
    fn name(&self) -> &'static str {
        "the_typed_angle_turns_a_mark"
    }

    fn defect(&self) -> &'static str {
        "the Properties panel's Angle field accepts a number and commits nothing, or commits it \
         as a RELATIVE turn — so typing 30 on a mark already at 270 leaves it at 300 instead of \
         30. That is a working control aimed at the wrong verb: it is correct on the first edit \
         of an unturned mark, which is most of them, and wrong on the second, on a mark somebody \
         else turned, and after an undo"
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
            "input is disabled (--no-input). This check draws a shape, turns it with its rotate \
             handle, scrubs a spinner and presses a button. Every one is a real pointer gesture.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("annot-angle-typed.trace.txt"));
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
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen. \
             Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- 1: draw a rectangle ------------------------------------------------
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    driver.drag(
        aim(ctx, &session, page, corner(SHAPE.0))?,
        aim(ctx, &session, page, corner(SHAPE.1))?,
    )?;
    session.settle(30);
    if session.trace()?.events(COMMIT_EVENT).next().is_none() {
        return Ok(Some(format!(
            "THE RECTANGLE TOOL AUTHORED NOTHING: a drag across the page produced no \
             `{COMMIT_EVENT}` line. This is three steps BEFORE the one under test — there is no \
             annotation to turn. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ a rectangle was authored");

    // --- 2: put the pen down, then select it --------------------------------
    //
    // ★★★ THE TOOL MUST GO DOWN FIRST. With a markup tool armed a click on the
    // page is a PICK rather than a selection, so a check that skipped this
    // would draw a *second* rectangle and then report that the shape could not
    // be selected, about a build whose selection works perfectly.
    let centre = corner((
        f64::midpoint(SHAPE.0.0, SHAPE.1.0),
        f64::midpoint(SHAPE.0.1, SHAPE.1.1),
    ));
    let centre_screen = aim(ctx, &session, page, centre)?;
    if !driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        driver.press(crate::sys::vk::V)?;
        session.settle(12);
    }
    driver.click_at(centre_screen)?;
    session.settle(24);
    if session.trace()?.events(SELECT_EVENT).last().is_none() {
        return Ok(Some(format!(
            "THE SHAPE COULD NOT BE SELECTED: a click at its centre produced no `{SELECT_EVENT}` \
             line. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the shape was selected");

    // --- 3: ★★ TURN IT FIRST, so absolute and relative can be told apart -----
    //
    // See the module header. From 0°, *set 30* and *turn by 30* are the same
    // edit, so a check that typed into an unturned mark could not distinguish
    // the verb this field is supposed to reach from the one it used to reach.
    let trace = session.trace()?;
    let (Some(handle), Some(outline)) = (
        declared(&trace, ui_rect, HANDLE_REGION),
        declared(&trace, ui_rect, OUTLINE_REGION),
    ) else {
        return Err(Error::new(format!(
            "the selection published no rotate handle or no outline, so the mark cannot be put \
             at a known non-zero angle and this check's oracle would not distinguish an absolute \
             setter from a relative one. `rotating_a_markup_turns_it` owns that gesture and \
             should be read first if it is also failing. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let frame = session.frame()?;
    let w = (outline.max.x - outline.min.x).max(1.0);
    let h = (outline.max.y - outline.min.y).max(1.0);
    let radius = (f32::midpoint(outline.min.y, outline.max.y)
        - f32::midpoint(handle.min.y, handle.max.y))
    .abs()
    .max(1.0);
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    driver.drag_via(
        frame.declared_at(handle, 0.5, 0.5),
        frame.declared_at(outline, 0.5 + radius * DIAG / w, 0.5 - radius * DIAG / h),
        std::time::Duration::from_millis(60),
        frame.declared_at(outline, 0.5 + radius / w, 0.5),
        None,
    )?;
    session.settle(40);
    report
        .note("★★ the mark was turned a quarter turn first, so 'set' and 'turn by' differ by 270°");

    // --- 4: bring the Properties tab forward --------------------------------
    //
    // A dock draws only its ACTIVE tab, and in Review the right dock opens on
    // Comments. Reading the trace without this reports "the panel published no
    // angle field" about a build whose panel is correct.
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, PROPERTIES_TAB_REGION) else {
        return Ok(Some(format!(
            "the right dock declared no `{PROPERTIES_TAB_REGION}`. Tabs declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "dock.tab.")),
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_at(tab, 0.5, 0.5))?;
    session.settle(24);

    // --- 5: ★★★ IS THE ANGLE FIELD EVEN THERE? ------------------------------
    let trace = session.trace()?;
    let Some(field) = declared(&trace, ui_rect, ANGLE_REGION) else {
        let seeded = trace
            .events(DRAFT_EVENT)
            .last()
            .and_then(|l| l.get("angle").map(std::borrow::ToOwned::to_owned));
        return Ok(Some(format!(
            "★★★ THERE IS NO ANGLE FIELD FOR A TURNED MARK. No `{ANGLE_REGION}` region after the \
             Properties tab was brought forward.\n\
             The panel's own draft line last said `angle={}`. If that is `none`, the field is \
             **correctly** absent — `canvas::annotquad::oriented` reported no angle, which \
             happens for an appearance whose `/Matrix` is a shear or a mirror — but this mark is \
             a rectangle pdfcer drew and then turned through the engine's own verb, so its \
             matrix is a rotation by construction. If it is a number, the field was computed and \
             not drawn, or it is scrolled out of the panel's viewport: the region is published \
             with `ui_rect_visible`.\n\
             Regions beginning `properties.`: {}. Trace: {}.",
            seeded.as_deref().unwrap_or("<no draft line at all>"),
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    };
    let before = trace
        .events(DRAFT_EVENT)
        .last()
        .and_then(|l| l.get("angle").and_then(|a| a.parse::<f64>().ok()))
        .ok_or_else(|| {
            Error::new(format!(
                "the Angle field is on screen and the `{DRAFT_EVENT}` line carries no readable \
                 `angle=`. The two are written by the same function, so this is a trace defect \
                 rather than an application one. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
    report.note(format!(
        "★★ the Angle field is on screen, seeded at {before:.2}°"
    ));

    // --- 6: scrub it --------------------------------------------------------
    //
    // ★ Fractions rather than added pixels: a coordinate is produced by a
    // conversion and never assembled. The fraction is computed from the
    // spinner's own width, so the travel is the same number of screen pixels
    // whatever the panel's width happens to be.
    let frame = session.frame()?;
    let fw = (field.max.x - field.min.x).max(1.0);
    driver.drag(
        frame.declared_at(field, 0.7, 0.5),
        frame.declared_at(field, 0.7 + SCRUB_PX / fw, 0.5),
    )?;
    session.settle(20);

    let trace = session.trace()?;
    let typed = trace
        .events(DRAFT_EVENT)
        .last()
        .and_then(|l| l.get("angle").and_then(|a| a.parse::<f64>().ok()))
        .unwrap_or(f64::NAN);
    if !(typed - before).abs().gt(&0.5) {
        return Ok(Some(format!(
            "★★ THE SCRUB DID NOT CHANGE THE ANGLE FIELD: it read {before:.2}° before and \
             {typed:.2}° after a {SCRUB_PX}-pixel drag across the spinner.\n\
             At `SPEED` = 0.5 units per pixel that should be a 30 degree change. A field that \
             does not move under a scrub is either not a `DragValue`, or disabled — the panel \
             greys every row on a **locked** annotation (§12.5.3 bit 8) — or the drag landed \
             somewhere other than the published rect. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the field scrubbed from {before:.2}° to {typed:.2}°"
    ));

    // --- 7: find Apply, scrolling as an operator would ----------------------
    let mut apply = None;
    for attempt in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = declared(&trace, ui_rect, APPLY_REGION) {
            apply = Some(rect);
            if attempt > 0 {
                report.note(format!(
                    "Apply was below the panel's fold; {attempt} scroll notch(es) brought it \
                     into view. Not a defect — the panel is a scroll area and the Angle field \
                     made its content one row taller"
                ));
            }
            break;
        }
        let Some(here) = declared(&trace, ui_rect, ANGLE_REGION) else {
            break;
        };
        driver.scroll_at(session.frame()?.declared_center(here), -1)?;
        session.settle(12);
    }
    let Some(apply) = apply else {
        // ★ Evidence before the verdict, on the path that gives up. A layout
        // question has exactly one oracle — a rendered screenshot.
        let shot = ctx.out("annot-angle-typed.no-apply.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Err(Error::new(format!(
            "no `{APPLY_REGION}` region after scrubbing the Angle field and scrolling \
             {SCROLL_ATTEMPTS} times. It is published with `ui_rect_visible`, so an absent \
             region means it is not on screen. SKIPPED rather than failed: a button that was \
             never pressed proves nothing about pressing it — the window is saved beside the \
             trace. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 8: ★★★ PRESS IT, AND SEE WHICH VERB IT REACHED ---------------------
    let before_set = session.trace()?.events(SET_EVENT).count();
    let before_delta = session.trace()?.events(DELTA_EVENT).count();
    driver.click_at(session.frame()?.declared_at(apply, 0.5, 0.5))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(applied) = trace.events(SET_EVENT).nth(before_set) else {
        // ★★ The wrong-verb diagnosis, ruled IN before any guess is offered.
        // This is the whole reason the delta counter was taken.
        if trace.events(DELTA_EVENT).count() > before_delta {
            return Ok(Some(format!(
                "★★★ THE TYPED ANGLE REACHED THE **DELTA** VERB. A `{DELTA_EVENT}` line followed \
                 Apply and no `{SET_EVENT}` line did.\n\
                 That is the shape this field shipped in for a few hours on 2026-09-07, before \
                 `pdfcer-core` `Pass 155.2` gave it `set_annotation_rotation`: the panel computed \
                 `typed − seed` and raised `AnnotAction::Rotate`. **It works on the first edit of \
                 an unturned mark and diverges on the second**, so it looks entirely correct \
                 from a chair.\n\
                 `panels::properties::geometry::annot` must raise `AnnotAction::SetRotation` with \
                 `draft.angle`, not `AnnotAction::Rotate` with `draft.angle_delta()`. Trace: {}.",
                session.trace_path().display()
            )));
        }
        return Ok(Some(format!(
            "★★★ APPLY COMMITTED NOTHING AND DECLINED NOTHING. The press was made at the centre \
             of the `{APPLY_REGION}` rect the application itself declared, and neither \
             `{SET_EVENT}` nor `{DELTA_EVENT}` followed.\n\
             Read the `{DRAFT_EVENT}` line first: `changed=` and `usable=` are what gate the \
             button, and `turn=` is what the arm reads. If `turn=none` beside a changed `angle=`, \
             `GeometryDraft::angle_delta`'s floor rejected the scrub as noise; if `changed=false` \
             beside a scrubbed field, the draft was re-seeded between the scrub and the press, \
             which `sync` does when its `(page, subject, epoch)` stamp moves.\n\
             Last draft line: `{}`. Trace: {}.",
            trace
                .events(DRAFT_EVENT)
                .last()
                .map_or_else(|| "<none>".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★★ Apply reached the ABSOLUTE verb: `{}`",
        applied.raw
    ));

    // --- 9: ★★★ AND THE NUMBER THAT TRAVELLED IS THE ABSOLUTE ANGLE ---------
    let asked: f64 = applied
        .get("asked")
        .and_then(|v| v.parse().ok())
        .unwrap_or(f64::NAN);
    if (asked - typed).abs() > 0.5 {
        return Ok(Some(format!(
            "★★★ THE PANEL ASKED FOR {asked:.2}° AND THE FIELD SAID {typed:.2}°.\n\
             `asked=` is the number the panel handed `set_annotation_rotation`, and it must be \
             the field's own value — the verb is absolute. A build that passed the DELTA through \
             would show `asked=` equal to the change ({:.2}°) rather than to the destination, \
             which is the failure this whole check exists for and which is invisible on an \
             unturned mark.\n\
             Line: `{}`. Trace: {}.",
            typed - before,
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ and it asked for the DESTINATION, not the change: asked={asked:.2}° \
         (the field moved {:.2}° from {before:.2}°, so a delta build would have said {:.2})",
        typed - before,
        typed - before
    ));

    // ★ Reported, not asserted — see the module header. The engine owns this
    // arithmetic and has its own tests for it; a second copy of the expected
    // numbers here would be a third place to maintain them.
    report.note(format!(
        "the engine worked out a delta of {}° and derived the new rectangle from `{}`; extent {}",
        applied.get("deg").unwrap_or("?"),
        applied.get("rect_derived").unwrap_or("?"),
        applied.get("to").unwrap_or("?"),
    ));

    // ★★ …and it did not ALSO raise the delta verb. Asserted separately,
    // because a build that raised both would satisfy every assertion above and
    // turn the mark twice.
    if trace.events(DELTA_EVENT).count() > before_delta {
        return Ok(Some(format!(
            "★★ APPLY RAISED **BOTH** VERBS: `{SET_EVENT}` and `{DELTA_EVENT}` both followed one \
             press, so the mark was turned twice and the second turn used a delta computed \
             against the angle before the first. The apply arm must raise one or the other. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }

    Ok(None)
}
