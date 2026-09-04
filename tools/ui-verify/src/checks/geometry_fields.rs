//! `geometry_fields_resize_a_shape` — **the typed X/Y/W/H route**, driven end
//! to end against the operator's own drawing.
//!
//! # What this is for
//!
//! `FEATURES.md`'s Phase 1 remainder listed *"Editable geometry — X/Y/W/H in
//! the Properties panel, typed rather than dragged"* for the life of this
//! project. It landed on 2026-08-19, the same day as the grips and out of the
//! same machinery, and this check is what stops it from being the grips' story
//! all over again: **drawn, cursored, and committing nothing.**
//!
//! # ★ Why this is not covered by `resize_scales_a_shape`
//!
//! Because the two routes share only their *last* link. The grip check proves
//! that `resizing::action`'s output reaches `move_nodes`; this one proves that a
//! **panel** can reach `resizing::action` at all, and the four links in between
//! are entirely different code:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | the Properties panel draws a geometry section when content is selected | `properties::geometry` — the plan, not the draw |
//! | 2 | the section's bounds come from the object's **anchors** | unit-tested, on synthetic points |
//! | 3 | the draft survives a frame and is not re-seeded under the operator | ★ **nothing** — it is a per-frame `sync` and a stale stamp is invisible to a unit test |
//! | 4 | Apply turns the draft into a command instead of staying greyed | ★ **nothing** |
//! | 5 | the command reaches the engine | shared with the grip check |
//!
//! Link 3 is the one that would fail silently and plausibly. `sync` runs
//! **every frame**, and if its stamp were wrong in either direction the symptom
//! is not a crash: too eager and the operator's typing is wiped between the
//! keystroke and the button press, so Apply is permanently greyed and the
//! feature reads as *"the fields do nothing"*; too lazy and the fields describe
//! an object that has since changed. Both are states a running window shows in
//! two seconds and no unit test can construct, because the thing under test is
//! *the sequence of frames*, not the function.
//!
//! # ★★ Why the harness SCRUBS the field instead of typing into it
//!
//! An `egui::DragValue` takes a number two ways: click-then-type, or drag to
//! scrub. Typing needs a double-click to enter text mode, a select-all, a
//! keystroke per digit and a commit — six OS-level events whose failure modes
//! (a double-click misread as two clicks, an IME, a stuck modifier) are the
//! *harness's* and would be reported as the application's.
//!
//! Scrubbing is one drag, and it is **arithmetically checkable**: the field
//! moves by `pixels × SPEED`, and `SPEED` is a named constant in the panel
//! precisely so this check can assert the number rather than assert that
//! something changed. `CONTINUE.md` §7's rule is the reason to prefer it — a
//! harness assertion is a claim about the program *and* about the harness, so
//! the route with fewer harness-owned failure modes is the honest one.
//!
//! # The oracle
//!
//! `move-nodes`, the same line the grip check ends on, **plus** a `resize-scale`
//! whose `sx` exceeds 1. The second is what distinguishes this from a check that
//! could pass on a build where Apply raised a *move* and no scale at all — which
//! is exactly what a `plan()` with its width comparison inverted would do, and
//! it would look like a working button.
//!
//! ★★ `resize-scale` was **added for this check, after its first run failed
//! wrongly.** The oracle was `resize-commit`, which the gesture route writes and
//! the typed route does not, so the check reported *"Apply committed nothing"*
//! over a trace that showed the object's bounds going from 317.87 to 358.00 on
//! the very next frame. The feature worked and the instrument was lying. The
//! fix was not to widen the check — it was to move the *fact both routes share*
//! into the one function both routes call, which is `resizing::action`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// The geometry section's own region — its presence is the precondition.
const SECTION_REGION: &str = "properties.geometry";
/// The Width spinner.
const WIDTH_REGION: &str = "properties.geometry.width";
/// The Apply button.
const APPLY_REGION: &str = "properties.geometry.apply";

/// How many scroll notches to spend looking for Apply below the fold.
///
/// Six. The button sits directly under the four fields, so one or two notches
/// is the realistic case; six is enough for a panel slot squeezed by other
/// panels above it and small enough that a check which will never find it fails
/// quickly rather than scrolling for a minute.
const SCROLL_ATTEMPTS: usize = 6;
/// `resize-scale sx=… sy=… ax=… ay=…` — raised by `resizing::action` itself,
/// so it is the line BOTH routes emit.
///
/// ★ Deliberately not `resize-commit`, which is the *gesture's* line and
/// carries the grip that was dragged. The first driven run of this check
/// asserted on that one, failed with "Apply committed nothing", and was wrong:
/// the trace in the same file showed the object's bounds going from 317.87 to
/// 358.00 on the frame after the press. The feature worked; the oracle named a
/// line only the other route writes.
const COMMIT_EVENT: &str = "resize-scale";
/// `resize-declined reason=…`.
const DECLINED_EVENT: &str = "resize-declined";
/// The label `vector_edit` traces when the edit reached the engine.
///
/// ★★ `move-nodes` until 2026-08-20. The typed route shares `resizing::action`
/// with the grips, so it moved to `transform_objects` with them — which is the
/// whole reason the two routes share that function, and is what this check's
/// own header claims. See `resize.rs`'s note on the same constant for why a
/// check that pins a MECHANISM goes red on the day the mechanism improves.
const APPLIED: &str = "transform-objects-applied";

/// How far to scrub the Width field, in screen pixels.
///
/// At the panel's `SPEED` of 0.5 points per pixel this is **+40 points** — far
/// beyond the tenth-of-a-point tolerance `plan` uses to decide the operator
/// typed something, and far enough that `sx` is unambiguously greater than 1 on
/// any object bigger than a few points. A ten-pixel scrub would be five points,
/// which on a large shape rounds to `sx = 1.004` and could also be produced by a
/// build that ignored the draft and re-seeded from slightly stale bounds.
const SCRUB_PX: f32 = 80.0;

/// See the module documentation.
pub struct GeometryFieldsResizeAShape;

impl Check for GeometryFieldsResizeAShape {
    fn name(&self) -> &'static str {
        "geometry_fields_resize_a_shape"
    }

    fn defect(&self) -> &'static str {
        "the Properties panel's X/Y/W/H fields accept a number and Apply commits nothing — the \
         typed route to a resize looking available and being inert, which is the same defect \
         shape the eight grips had for the whole life of the project"
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
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. This check needs a drawing with a selectable shape on page 1.")
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point where the fixture \
             has a selectable SHAPE. There is deliberately no default: a click on empty page \
             is symptom-identical to a broken hit test.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, clicks page \
             content, scrubs a spinner and presses a button. Reported as SKIPPED rather than \
             passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
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
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("geometry_fields.trace.txt"));
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
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit, the one mode whose canvas selects content ----------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: select the shape -----------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    driver.click_at(frame.to_screen(window_point))?;
    session.settle(20);

    // --- 3: the section must be on screen ----------------------------------
    //
    // ★ A SKIP, not a FAIL, and the distinction is the whole reason this branch
    // has its own paragraph. The section draws only in an arrangement where the
    // Properties panel is mounted, and which panels are mounted is the
    // operator's saved dock layout — a property of the profile the harness
    // launched with, not of the feature. Failing here would report "editable
    // geometry is broken" for a run whose only fault was a dock arrangement,
    // and `CONTINUE.md` §7's rule is that a harness must not blame the program
    // for a condition the harness set up.
    let trace = session.trace()?;
    let Some(_section) = driving::declared(&trace, ui_rect, SECTION_REGION) else {
        return Err(Error::new(format!(
            "no `{SECTION_REGION}` region after selecting, so either the Properties panel is \
             not mounted in this profile's `{MODE}` arrangement, or the click selected \
             something the section declines by design — an annotation, several objects at \
             once, or a text run. Both are honest states rather than defects, which is why \
             this is SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note("the geometry section drew, so a single path object is selected");

    // --- 4: scrub Width to the right ---------------------------------------
    let width = driving::declared(&trace, ui_rect, WIDTH_REGION).ok_or_else(|| {
        Error::new(format!(
            "the section drew and published no `{WIDTH_REGION}`. That is a shell defect rather \
             than a fixture one, but it is reported as a SKIP because the harness cannot tell \
             the two apart from here."
        ))
    })?;
    let frame = session.frame()?;
    let from = frame.declared_at(width, 0.7, 0.5);
    // Fractions rather than added pixels — `coords`' rule is that a coordinate
    // is produced by a conversion and never assembled. The fraction is computed
    // from the spinner's own width so the travel is the same number of screen
    // pixels whatever the panel's width happens to be.
    let w = (width.max.x - width.min.x).max(1.0);
    let to = frame.declared_at(width, 0.7 + SCRUB_PX / w, 0.5);
    driver.drag(from, to)?;
    session.settle(20);

    // --- 5: press Apply ----------------------------------------------------
    //
    // ★ The rect is re-read AFTER the scrub. Apply is `add_enabled`, and an
    // enabled button and a disabled one are the same size — but the section
    // above it is not: a wider number is a wider spinner row under
    // `horizontal`, and a panel narrow enough to wrap moves everything below.
    // Reading the rect before the scrub would be the read-then-act interval
    // `driving::stable_rect`'s doc comment describes, in its cheapest form.
    // ★★★ SCROLL TO IT FIRST — added 2026-08-26, and it is the fix for a
    // failure this check reported as a dead button.
    //
    // The Properties panel is a `ScrollArea`, and in an ordinary dock layout its
    // slot is shorter than its content. Apply sits directly under the fields and
    // was **14 points below the panel's viewport**:
    //
    //     properties.geometry        [[786.0 591.7] - [1100.0 762.0]]
    //     properties.geometry.apply  [[786.0 776.7] - [ 835.0 804.7]]
    //
    // The check read the declared rect, clicked its centre, hit empty canvas,
    // and reported *"APPLY COMMITTED NOTHING AND DECLINED NOTHING"* — which
    // reads as a defect in the application and was filed as one. The button was
    // never broken; it was never pressed.
    //
    // Two changes closed it. The application now publishes these regions with
    // `ui_rect_visible`, so a control nobody can see is not offered as a target
    // at all — an absent region is a far better answer than a present one that
    // cannot be clicked. And this loop does what the operator would do: scrolls
    // the panel until Apply is on screen.
    //
    // ★ Scrolling at the WIDTH FIELD's centre rather than at the section's,
    // because the section rect shrinks as its content scrolls out of view and a
    // point derived from it walks. The width field is what the scrub just
    // used, so it is known-good and known-inside the panel.
    let mut apply = None;
    for attempt in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, APPLY_REGION) {
            apply = Some(rect);
            if attempt > 0 {
                report.note(format!(
                    "Apply was below the panel's fold; {attempt} scroll notch(es) brought it \
                     into view. That is what an operator does, and it is not a defect — the \
                     Properties panel is a scroll area and its slot is shorter than its content"
                ));
            }
            break;
        }
        let Some(field) = driving::declared(&trace, ui_rect, WIDTH_REGION) else {
            return Err(Error::new(format!(
                "the Width field stopped being visible while scrolling for Apply, so there is \
                 nothing left to aim at. Trace: {}.",
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(field), -1)?;
        session.settle(12);
    }
    // ★ Evidence before the verdict, on the path that gives up. A layout
    // question has exactly one oracle — a rendered screenshot — and "the button
    // is not on screen" is a layout question. Without this, the next reader has
    // six scroll notches and a coordinate and no way to see what the panel
    // actually looked like.
    if apply.is_none() {
        let shot = ctx.out("geometry_fields.no-apply.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
            report.note(
                "the window is saved beside the trace: look at the Properties panel and see whether Apply is below its fold, off the window, or absent altogether — those are three different problems and the coordinates alone cannot tell them apart",
            );
        }
    }
    let apply = apply.ok_or_else(|| {
        Error::new(format!(
            "no `{APPLY_REGION}` region after scrubbing the Width field and scrolling the \
             Properties panel {SCROLL_ATTEMPTS} times. The button is published with \
             `ui_rect_visible`, so an absent region means it is not on screen — the panel's \
             slot may be too short for even the scrolled position to reveal it. SKIPPED rather \
             than failed: a button that was never pressed proves nothing about pressing it. \
             Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    let commits_before = trace.events(COMMIT_EVENT).count();
    let frame = session.frame()?;
    driver.click_at(frame.declared_center(apply))?;
    session.settle(30);

    // --- 6: the verdict ----------------------------------------------------
    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).nth(commits_before) else {
        let declined = trace
            .events(DECLINED_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Some(match declined {
            Some(reason) => format!(
                "Apply was pressed and the resize was DECLINED: reason={reason}. The typed \
                 route reaches the same six refusals as the grips, deliberately, so this is a \
                 real answer rather than a silence — but `NotAPath` here means the click aimed \
                 at text or a picture and the section should not have drawn at all, which IS a \
                 defect. Trace: {}.",
                session.trace_path().display()
            ),
            None => format!(
                "★ THE WIDTH FIELD WAS SCRUBBED BY {SCRUB_PX:.0} PIXELS AND APPLY COMMITTED \
                 NOTHING AND DECLINED NOTHING.\n\
                 Three candidates, in the order worth checking. (1) **Apply stayed greyed** — \
                 `differs_from` compares the draft to the bounds it was seeded from, so a \
                 `sync` whose stamp is recomputed too eagerly wipes the scrub before the button \
                 is read, and the operator sees fields that accept a number and a button that \
                 never lights. (2) **The scrub did not land** — the drag missed the spinner, \
                 which the trace shows as an unchanged Width row rect. (3) **`plan` returned an \
                 empty plan** — its width comparison inverted. Only the first is a defect the \
                 operator would report as 'the fields do nothing', and it is the likeliest. \
                 Trace: {}.",
                session.trace_path().display()
            ),
        }));
    };

    let sx: f64 = commit.get("sx").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    report.note(format!("★ Apply committed a scale: `{}`", commit.raw));
    if sx <= 1.0 {
        return Ok(Some(format!(
            "★ THE WIDTH WAS SCRUBBED UPWARD AND THE SHAPE DID NOT GET WIDER: sx={sx:.4}.\n\
             A factor at or below 1 from a rightward scrub means either the scrub ran the wrong \
             way (an `egui` DragValue increases to the right, always) or `plan` divided the old \
             width by the new one. The second is the interesting failure: it produces a \
             perfectly working button that shrinks what the operator widened, and no unit test \
             in `properties::geometry` would catch it unless it asserted the DIRECTION, which \
             `resizing_it_pivots_on_the_stated_corner` does."
        )));
    }

    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "Apply computed `{}` and no `{APPLIED}` line followed, so the action was raised and \
             its apply arm never ran. Nothing reached the document, which from a chair is \
             indistinguishable from the button doing nothing. Trace: {}.",
            commit.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ the typed width reached the engine through `transform_objects`");
    Ok(None)
}
