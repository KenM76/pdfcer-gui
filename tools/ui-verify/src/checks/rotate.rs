//! `rotate_handle_turns_a_selection` — **the ninth grip**, driven end to end.
//!
//! # What this is for
//!
//! `ui-conventions/handles.md` H2 says the standard set is *"eight resize
//! grips, a body, and a rotation handle offset outside the top edge"*, and it
//! quotes the operator's own report as the failure mode:
//!
//! > *"unfortunately there was no way to reposition, resize, or rotate it on
//! > the screen. Can I please please please have that too?"*
//!
//! Reposition and resize landed on 2026-08-20. **Rotate is the third word in
//! that sentence** and had no affordance at all until `Pass 113.0` gave the
//! shell a verb.
//!
//! # ★★ Why this cannot be a unit test
//!
//! `canvas::rotating`'s arithmetic is pure and has eight of them. What they
//! cannot reach is the chain:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a click selects an object and the outline draws **nine** affordances | `canvas::handles` — the layout, not the hit |
//! | 2 | a press above the top edge finds `Grip::Rotate` rather than empty page | `handles::grip_at` — the geometry, not the routing |
//! | 3 | it becomes `DragKind::Rotate` and **not** `DragKind::Move` | `gesture::meaning` — and this is the link that would fail silently |
//! | 4 | the bearing is measured from the selection's centre in screen space | `canvas::rotating` — yes, and it is pure |
//! | 5 | the sign survives the screen → page crossing | **nothing** |
//! | 6 | every selected index reaches `transform_objects` as one command | **nothing** |
//!
//! **Link 3 is the one that would ship.** `Grip::is_resize` used to be
//! `self != Grip::Move`, and the wildcard arm below it read `(None, Some(_)) =>
//! DragKind::Move`. Either left alone would have produced a *working gesture
//! aimed at the wrong verb* — a handle that moved the selection, or one that
//! resized it about a corner. Both look deliberate. Nothing in the workspace
//! asks about them.
//!
//! Link 5 is the one that would look like a feature: an object that turns the
//! wrong way is not obviously a defect to anybody who did not watch the
//! pointer.
//!
//! # The oracle carries DEGREES, and a signed number
//!
//! `rotate-commit deg=… px=… py=… objects=… constrained=…`. A line saying only
//! *"a rotation committed"* would be identical for a build that turned the
//! other way, pivoted about a corner instead of the centre, or snapped when it
//! should not have. This project's standing rule, stated by `resize.rs` and
//! earned by `DEFECTS.md` D14: **a trace line must carry the number a wrong
//! build would get wrong.**
//!
//! # What it drives, and why that shape
//!
//! A quarter turn clockwise: press on the handle, then release **due east of
//! the selection's centre**. Two properties make that the right gesture to
//! drive rather than a small nudge:
//!
//! * the expected answer is a round number a human can check by reading the
//!   trace, and
//! * it crosses no quadrant boundary, so a build with a broken wrap still
//!   produces the right answer here — which means a failure of *this* check is
//!   about the routing rather than about the arithmetic the unit tests already
//!   cover.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// `rotate-commit deg=… px=… py=… objects=… constrained=…`.
const COMMIT_EVENT: &str = "rotate-commit";
/// The label `vector_edit` traces when the edit reached the engine.
const APPLIED: &str = "transform-objects-applied";
/// The region the selection outline publishes.
const OUTLINE_REGION: &str = "canvas.selection-outline";

/// The canvas viewport's own declared region.
///
/// Read so the check can tell *the handle is outside the canvas* from *the
/// handle is on the canvas and the press was routed wrongly*. Those are
/// different defects in different files and they produce the identical
/// symptom — no `rotate-commit` line.
const CANVAS_REGION: &str = "canvas-viewport";

/// Half a grip's edge, in points — mirrors `canvas::handles::GRIP_SIZE_PX / 2`.
///
/// The handle is a square CENTRED on the stem's end, so its topmost pixel is
/// half a grip above that centre. Using the centre alone would let a handle
/// that is half off-canvas read as reachable.
const HALF_GRIP_PT: f32 = 4.0;

/// How far above the selection box the handle sits, in points.
///
/// ★ It mirrors `canvas::handles::ROTATE_STEM_PX` and is **not** imported from
/// it — this harness drives a built binary and must not compile against the
/// application's internals, or it would agree with a build by construction
/// rather than by observation.
///
/// The check does not aim at this number directly. It is used only to know how
/// far outside the published outline to look, and the press is then made at the
/// point the application itself declared — see `driving::declared_at`.
const STEM_PT: f32 = 20.0;

/// See the module documentation.
pub struct RotateHandleTurnsASelection;

impl Check for RotateHandleTurnsASelection {
    fn name(&self) -> &'static str {
        "rotate_handle_turns_a_selection"
    }

    fn defect(&self) -> &'static str {
        "there is no way to rotate anything on the canvas — the third word of the operator's \
         \"reposition, resize, or rotate\", with no affordance at all. Or the handle is drawn and \
         a press on it MOVES or RESIZES the selection instead, which is a working gesture aimed \
         at the wrong verb and looks entirely deliberate"
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
        Error::new("no --pdf. This check needs a drawing with selectable content.")
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point with selectable \
             content on it. There is deliberately no default: a click on empty page is \
             symptom-identical to a broken hit test.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, clicks page \
             content and drags the rotate handle. Reported as SKIPPED rather than passed.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("rotate.trace.txt"));
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

    // --- 2: select something -----------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    driver.click_at(frame.to_screen(window_point))?;
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
            "the click at (page {}, {:.1}, {:.1}) selected nothing, so there is no outline and \
             no handle. A fact about the fixture and the point, not about the build — aim at \
             content. SKIPPED for exactly that reason.",
            target.page + 1,
            target.x,
            target.y
        )));
    }
    report.note("the click selected something, so the outline and its handles are drawn");

    // --- 3: find the handle, from the application's own declaration ---------
    //
    // ★ The outline's rect, and the handle derived from it — never a guess.
    // `handles.md` H8: where a handle sits is the end of a
    // document→canvas→screen conversion and is a fact only the application
    // knows. A harness that guessed would land inside the object, start a MOVE,
    // and then pass while exercising the wrong gesture entirely.
    let trace = session.trace()?;
    let outline = driving::declared(&trace, ui_rect, OUTLINE_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{OUTLINE_REGION}` region after selecting, so the \
             harness does not know where the handle is. It refuses to guess."
        ))
    })?;
    let frame = session.frame()?;
    let w = (outline.max.x - outline.min.x).max(1.0);
    let h = (outline.max.y - outline.min.y).max(1.0);

    // ★★ EVERY POINT BELOW IS BUILT WITH `declared_at`, AND THAT IS THE
    // DISCIPLINE RATHER THAN A CONVENIENCE.
    //
    // `ScreenPoint`'s only constructor in this crate is `WindowFrame::to_screen`
    // — deliberately, so that a screen coordinate can only be *produced by a
    // conversion* and never assembled from two integers. This check would have
    // been the first place to break that, because it needs a point at an
    // arbitrary bearing from the selection's centre.
    //
    // It does not need to. `declared_at` takes fractions of the declared box and
    // does not clamp them — precisely so a check can aim outside a control's
    // edge — so the whole gesture is expressible in the box's own coordinates:
    //
    //   the handle   0.5 across, `-STEM/h` up      (due north, on the stem)
    //   the release  `0.5 + r/w` across, 0.5 down  (due east, same radius)
    //
    // where `r` is the radius in points — half the box's height plus the stem —
    // converted to a fraction of whichever axis it is applied to. The asymmetry
    // between `r/w` and `r/h` is the whole reason both extents are read: a
    // fraction is not a distance until you say of what.
    let radius_pt = 0.5 * h + STEM_PT;
    let handle = frame.declared_at(outline, 0.5, -STEM_PT / h);

    // --- 3b: IS THE HANDLE EVEN ON THE CANVAS? -----------------------------
    //
    // ★★★ Added 2026-08-21 after this check failed and named three causes,
    // ALL THREE WRONG. `OPERATOR_REQUESTS.md` O22.
    //
    // The rotate handle sits `ROTATE_STEM_PX` ABOVE the selection box. Select
    // something near the top of the viewport and the handle's whole square
    // lands outside the canvas: the painter clips it away, so it is never
    // drawn, and the press lands on whatever occupies that strip of the
    // window — the ribbon. Measured on `SW41177.pdf` at `--doc-point
    // 0,1211,1021`: canvas top y=143.0, outline top y=150.2, handle centre
    // y=130.2. Nine pixels above the canvas.
    //
    // Without this branch the check reports `THE ROTATE HANDLE COMMITTED
    // NOTHING` and lists `Grip::is_resize`, `gesture::meaning` and
    // `needs_targets` — three real hazards, none of which is what happened,
    // and all three inside the application. A reader would go looking in the
    // routing for a defect that is in the LAYOUT.
    //
    // ★ This is the same failure mode the two checks written this evening
    // both committed: **a confident, specific, wrong accusation is worse than
    // a vague one**, because it is actionable and it aims somebody at the
    // wrong file. A check that can rule a cause OUT should.
    if let Some(canvas) = driving::declared(&trace, ui_rect, CANVAS_REGION) {
        let handle_top = outline.min.y - STEM_PT - HALF_GRIP_PT;
        if handle_top < canvas.min.y {
            return Ok(Some(format!(
                "★★ THE ROTATE HANDLE IS OFF-CANVAS — defect O22, and NOT a routing \
                 problem. The selection's top edge is at y={:.1}, the handle therefore \
                 spans from y={:.1}, and the canvas begins at y={:.1}. The handle is {:.1} \
                 point(s) above the top of the canvas, so it is clipped away by the painter \
                 (the operator sees eight grips and no ninth) and the press never reaches \
                 the canvas widget at all. \
                 ★ ANY selection whose top is within {:.0} pt of the top of the view has \
                 this, whatever kind of object it is — it is not about text. Do NOT go \
                 looking in `Grip::is_resize` or `gesture::meaning`; the gesture never \
                 started. See O22 for the fix, which is a pasteboard rather than moving \
                 the handle.",
                outline.min.y,
                handle_top,
                canvas.min.y,
                canvas.min.y - handle_top,
                STEM_PT + HALF_GRIP_PT,
            )));
        }
    }

    // --- 4: a quarter turn clockwise ---------------------------------------
    //
    // Release due EAST of the centre, at the same distance the handle sits
    // above it. The handle starts due north, so the turn is exactly 90°
    // clockwise on screen — a round number a human can check by reading the
    // trace, crossing no quadrant boundary.
    //
    // The mid-point is at 45°, so the drag passes through frames where the
    // bearing is genuinely changing rather than teleporting from press to
    // release. `drag_via`'s own header makes the same argument for holding a
    // modifier throughout a gesture.
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let via = frame.declared_at(
        outline,
        0.5 + radius_pt * DIAG / w,
        0.5 - radius_pt * DIAG / h,
    );
    let to = frame.declared_at(outline, 0.5 + radius_pt / w, 0.5);
    let before = session.trace()?.events(COMMIT_EVENT).count();
    driver.drag_via(handle, via, std::time::Duration::from_millis(60), to, None)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).nth(before) else {
        return Ok(Some(format!(
            "★ THE ROTATE HANDLE COMMITTED NOTHING. The press was made at the point the \
             application itself declared, above the selection outline it published, and no \
             `{COMMIT_EVENT}` line followed.\n\
             Three links can produce this and they are worth checking in order: `Grip::is_resize` \
             must answer FALSE for `Rotate` (if it answers true the press became a RESIZE), \
             `gesture::meaning` must match `Grip::Rotate` by name before the `Some(_)` wildcard \
             (if not the press became a MOVE), and `needs_targets` must include \
             `GestureOutcome::Rotate` (without it the commit has no decomposition and declines). \
             All three produce a working gesture aimed at the wrong verb, which looks \
             deliberate. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 5: ★★ the number a wrong build would get wrong ---------------------
    let deg: f64 = commit
        .get("deg")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    report.note(format!("★ the rotation committed: `{}`", commit.raw));
    // ★ A generous window. The press lands at the handle's declared centre and
    // the release at a computed point, both rounded to whole pixels, so the
    // measured bearing is a degree or two off 90 by construction. What is being
    // asserted is the QUADRANT and the SIGN, not the arithmetic — that has eight
    // unit tests.
    //
    // ★★ NEGATIVE, and that is the assertion that catches link 5. Screen y is
    // down, so the drag is clockwise and `rotating::angle` reports +90; PDF user
    // space is y-up and `Matrix::rotate` turns anticlockwise in it, so the
    // committed angle must be −90. A build that forgot the crossing, or applied
    // it twice, lands on +90 — a perfectly good rotation, the wrong way round,
    // which nobody who did not watch the pointer would call a defect.
    if !(-100.0..=-80.0).contains(&deg) {
        return Ok(Some(format!(
            "★ THE QUARTER TURN CAME OUT AS {deg:.2}°, AND IT MUST BE ABOUT −90°.\n\
             The handle starts due north of the selection's centre and the release was due \
             east, which is 90° CLOCKWISE on screen. Screen y runs down, so `rotating::angle` \
             reports +90; PDF user space is y-up and `Matrix::rotate` turns anticlockwise in it, \
             so the commit negates once — in `rotating::drag`, and nowhere else.\n\
             +90 here means the crossing was forgotten or applied twice: the object turns the \
             wrong way, which is a perfectly good rotation and looks entirely deliberate. A \
             value near 0 means the bearing was measured from something other than the centre. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ and it turned the right way: {deg:.2}° in page space for a clockwise drag on screen"
    ));

    // --- 6: and it reached the engine --------------------------------------
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the rotation computed `{}` and no `{APPLIED}` line followed, so the action was \
             raised and its apply arm never ran. Nothing reached the document, which from a \
             chair is indistinguishable from the handle doing nothing. Trace: {}.",
            commit.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ the rotation reached the engine through `transform_objects`");
    Ok(None)
}
