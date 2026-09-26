//! `resize_scales_a_shape` — **the eight grips commit**, driven end to end
//! against the operator's own drawing.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/resize.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// `resize-commit grip=… sx=… sy=… ax=… ay=…` — the shell's own report.
const COMMIT_EVENT: &str = "resize-commit";
/// `resize-declined reason=…` — the six worded refusals.
const DECLINED_EVENT: &str = "resize-declined";
/// The trace label `vector_edit` traces when the edit reached the engine.
const APPLIED: &str = "transform-objects-applied";

/// How far to drag the grip, in screen pixels, on each axis.
const DRAG_PX: f32 = 60.0;

/// See the module documentation.
pub struct ResizeScalesAShape;

impl Check for ResizeScalesAShape {
    fn name(&self) -> &'static str {
        "resize_scales_a_shape"
    }

    fn defect(&self) -> &'static str {
        "the eight resize grips change the cursor, consume the drag and commit nothing — a \
         control that looks available and is inert, which the operator experiences as a resize \
         that silently does not work"
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
    // PINNED: `--pdf` and `--doc-point` are read and IGNORED here.
    //
    //
    // ⇒ `fixture::grip_gesture_target` holds the point and the reason. A
    // check whose subject cannot exist under an arbitrary aim must not be
    // steerable into a place where its subject does not exist.
    let (pdf, target) = crate::fixture::grip_gesture_target();
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the grip-gesture fixture is not at {}. Every check that drags a selection grip \
             needs it; `fixture::grip_gesture_target` says which point on it is the measurable \
             one, and what it cost to find out.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} at page 0, 300, 500 — the \
         one place on that sheet where a grip drag can be measured",
        pdf.display()
    ));
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, clicks page \
             content and drags a grip. Reported as SKIPPED rather than passed.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("resize.trace.txt"));
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
    let at = frame.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(16);

    let trace = session.trace()?;
    let selected = trace
        .last(vocab.click_event)
        .and_then(|l| l.get_usize(vocab.click_selection_field))
        .or_else(|| {
            trace
                .last(vocab.canvas_event)
                .and_then(|l| l.get_usize(vocab.canvas_selection_field))
        });
    if selected == Some(0) {
        return Err(Error::new(format!(
            "the click at (page {}, {:.1}, {:.1}) selected nothing, so there are no grips to \
             drag. That is a fact about the fixture and the point, not about the resize — aim \
             at a shape. Reported as SKIPPED rather than FAILED for exactly that reason.",
            target.page + 1,
            target.x,
            target.y
        )));
    }
    report.note("the click selected a shape, so the outline and its grips are drawn");

    // --- 3: find the south-east grip ---------------------------------------
    //
    // Computed from the SELECTION OUTLINE's own trace rect rather than from
    // the click point. A grip is at a corner of the selection, and the
    // selection's extent is a fact only the application knows — a harness that
    // guessed "a few pixels down and right of where I clicked" would be aiming
    // at the object's interior on any shape bigger than a grip, which is a
    // MOVE drag and would pass this check for the wrong reason.
    let trace = session.trace()?;
    let outline = driving::declared(&trace, ui_rect, OUTLINE_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{OUTLINE_REGION}` region after selecting, so the \
             harness does not know where the grips are. It refuses to guess: a guessed grip \
             position lands inside the object, which is a MOVE drag, and this check would then \
             pass while measuring the wrong gesture."
        ))
    })?;
    // `declared_at(1.0, 1.0)` — the box's bottom-right corner, which is where
    // `handles::grip_rects` centres the south-east grip. Not the centre: the
    // centre of a selection box is `Grip::Move`, and a drag from there is a
    // MOVE, which would pass every assertion below for the wrong gesture.
    let frame = session.frame()?;
    let from = frame.declared_at(outline, 1.0, 1.0);
    // Beyond the box, by fractions rather than by adding pixels to a
    // `ScreenPoint`: `coords`' own rule is that a coordinate is produced by a
    // conversion and never assembled, and `declared_at` does not clamp its
    // fractions precisely so a check can aim past a control's edge. The
    // fraction is computed from the box's own width so the travel is the same
    // number of screen pixels whatever size the selection is.
    let w = (outline.max.x - outline.min.x).max(1.0);
    let h = (outline.max.y - outline.min.y).max(1.0);
    let to = frame.declared_at(outline, 1.0 + DRAG_PX / w, 1.0 + DRAG_PX / h);

    // --- 4: drag it --------------------------------------------------------
    let commits_before = session.trace()?.events(COMMIT_EVENT).count();
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    let commit = trace.events(COMMIT_EVENT).nth(commits_before);
    let Some(commit) = commit else {
        let declined = trace
            .events(DECLINED_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Some(match declined {
            Some(reason) => format!(
                "the grip drag was DECLINED: reason={reason}. `NotAPath` means the point aimed \
                 at text or a picture, which this feature refuses by name — aim at a shape. \
                 `ManyObjects` means the click selected more than one. Both are honest \
                 refusals and neither is what this check is for; anything else is a defect. \
                 Trace: {}.",
                session.trace_path().display()
            ),
            //
            // A run against `a1-titleblock.pdf` at `0,300,500` failed here and
            // cost most of an hour, because the message below describes a
            // **regression in the application** and the actual cause was the
            // **aim**: that click selects the sheet's own border rectangle —
            // the properties panel reported `2383.94 x 1683.78`, exactly A1 —
            // so the south-east grip sits on the corner of the rendered page
            // and a drag aimed deliberately BEYOND it lands off the canvas.
            //
            // ⇒ None of that is visible in *"committed nothing and declined
            // nothing"*, and the sentence after it is a confident, specific
            // accusation naming `canvas::interact`. **A check that cannot say
            // what it aimed at cannot be believed about what it found.**
            //
            // The verdict is unchanged — a page-sized selection is still a
            // legitimate thing to fail on, and weakening the assertion would
            // be the wrong repair. What changes is whether the reader spends
            // the next hour in the right file.
            None => format!(
                "aimed at a selection {:.0} x {:.0} screen px.{}\n\
                 ★ THE GRIP DRAG COMMITTED NOTHING AND DECLINED NOTHING. That is the state \
                 this whole feature is a fix for: until 2026-08-19 every resize drag was \
                 consumed and thrown away, so a build that has reverted to it is silent on \
                 both channels — which is exactly what an operator reports as 'resize does not \
                 work'. Look at `canvas::interact`'s `GestureOutcome::Resize` arm. Trace: {}.",
                w,
                h,
                // Compared against the CANVAS region rather than the window:
                // the sheet is fitted into the canvas, so "the selection is
                // nearly the canvas" is what "the selection is the whole page"
                // looks like from here. A window-relative test would also count
                // the dock and the ribbon and never fire.
                if driving::declared(&trace, ui_rect, "canvas-viewport")
                    .is_some_and(|v| w >= v.width() * 0.9 && h >= v.height() * 0.9)
                {
                    " ⚠ THAT IS ESSENTIALLY THE WHOLE PAGE — the click almost certainly \
                     selected the sheet's own border rather than a shape on it, so the grip \
                     is at the page's own edge. Until 2026-09-08 that edge sat under egui's \
                     FLOATING scroll bar — invisible, and first in line for the press — so \
                     the drag scrolled the view instead (`canvas::present::scroll_style`). \
                     A page-sized selection is now the regression test for that, so do NOT \
                     re-aim: read `canvas-gesture` in the trace. `started=0` with `origin=1` \
                     means something other than the canvas took the press."
                } else {
                    ""
                },
                session.trace_path().display()
            ),
        }));
    };

    // --- 5: the numbers a wrong build would get wrong -------------------
    // Parsed from the field rather than read through a typed accessor, because
    // the trace is text and `TraceLine` offers `usize` and `Rect` only. A
    // missing or unparsable field answers 0.0, which fails the assertion below
    // — the safe direction: a check that could not read the number must not
    // report that the number was right.
    let sx: f64 = commit.get("sx").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let sy: f64 = commit.get("sy").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    report.note(format!("★ the resize committed: `{}`", commit.raw));
    if sx <= 1.0 || sy <= 1.0 {
        return Ok(Some(format!(
            "★ THE SOUTH-EAST GRIP WAS DRAGGED DOWN AND RIGHT AND THE SHAPE DID NOT GROW: \
             sx={sx:.4}, sy={sy:.4}.\n\
             Both factors must exceed 1. A value below 1 on the y axis means the SCREEN-Y SIGN \
             is inverted — screen y is down, so a south grip dragged downward grows the box — \
             and that is the one error here that produces a perfectly plausible resize in the \
             wrong direction. `canvas::resizing::factors` owns the rule and has a unit test \
             per grip; this is the link that proves the sign survives the real event stream."
        )));
    }

    // --- 6: and it reached the engine --------------------------------------
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the resize computed `{}` and no `{APPLIED}` line followed, so the action was \
             raised and its apply arm never ran — or ran and could not borrow the session. \
             Nothing reached the document, which from a chair is indistinguishable from the \
             grips doing nothing at all. Trace: {}.",
            commit.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ the scale reached the engine through `transform_objects`");
    Ok(None)
}

/// The region the selection outline publishes.
const OUTLINE_REGION: &str = "canvas.selection-outline";
