//! `resize_scales_a_shape` — **the eight grips commit**, driven end to end
//! against the operator's own drawing.
//!
//! # What this is for
//!
//! The resize grips were drawn at S4 and have been **cursored, hit-tested and
//! drag-consuming ever since, committing nothing** — the oldest unbuilt thing
//! in this project and `FEATURES.md`'s last Phase 1 ⛔. An operator aiming at
//! one got a resize cursor, a drag that felt like it was working, and no
//! change: `DEFECTS.md` D4a's shape exactly.
//!
//! They commit as of 2026-08-19, built out of `move_nodes` because `pdfcer-core`
//! still has no scale verb — see `crate` `canvas::resizing` for why *scaling a
//! path is moving every one of its nodes*, and for the four cases that are
//! still refused, in words.
//!
//! # ★ Why this cannot be a unit test
//!
//! Six links, and four of them are only observable in a running window:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a click selects an object and the outline draws grips | `canvas::handles` — the layout, not the hit |
//! | 2 | a press on a grip is routed to `DragKind::Resize` rather than to a marquee | `gesture::meaning` — the decision, not the routing |
//! | 3 | the drag's screen delta becomes scale factors about the right anchor | `canvas::resizing` — yes, and it is pure |
//! | 4 | the anchor converts screen → canvas → PDF against **this** frame's mapping | **nothing** |
//! | 5 | every node's new position reaches `move_nodes` as one command | **nothing** |
//! | 6 | the engine rewrites the content stream and the page redraws | `pdfcer-core` |
//!
//! Link 4 is the one that would fail silently and plausibly: a resize computed
//! against the wrong anchor still resizes, just about a different corner, so
//! the object moves *and* changes size. That looks like a slightly clumsy
//! gesture rather than a defect.
//!
//! # ★★ The oracle is `resize-commit`, and it carries the numbers
//!
//! `resize-commit grip=… sx=… sy=… ax=… ay=…`. A line saying only *"a resize
//! committed"* would be identical for a build that scaled about the centre,
//! mirrored an axis, or applied one factor to both — which is `DEFECTS.md`
//! D14's lesson (the freehand trail that authored two points) applied before it
//! bites: **a trace line must carry the number a wrong build would get wrong.**
//!
//! And the edit itself is asserted separately, through `move-nodes`, because a
//! commit that computed the right geometry and never reached the engine is the
//! defect this whole feature is a fix for.

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
///
/// ★★ **This was `move-nodes` until 2026-08-20**, and the change is the point
/// rather than a rename. A resize was built out of `move_nodes` — *scaling a
/// path IS moving every one of its nodes* — because `pdfcer-core` had no scale
/// verb at all. `Pass 113.0` shipped `transform_objects`, which wraps each
/// object in `q <cm> … Q` and therefore works on **text, pictures and several
/// objects at once**, none of which a node-move can express.
///
/// ★ The stale constant did not make this check pass wrongly; it made it FAIL
/// against a build where the resize had just got better — reporting *"the
/// action was raised and its apply arm never ran"* while the trace plainly
/// carried `transform-objects … transformed=1`. Worth knowing, because a check
/// that pins a MECHANISM rather than an OUTCOME goes red on the day the
/// mechanism improves, and the reflex when that happens is to doubt the code.
const APPLIED: &str = "transform-objects-applied";

/// How far to drag the grip, in screen pixels, on each axis.
///
/// Big enough that the factors are unambiguously greater than one — a two-pixel
/// drag would produce `sx = 1.002`, which a build that ignored the delta
/// entirely could also produce by rounding. Small enough to stay inside the
/// window on a modest client area.
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
            Err(skip) => report.skip(skip.to_string()),
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
             has a selectable SHAPE — not text and not a picture, both of which this feature \
             refuses by name. There is deliberately no default: a click on empty page is \
             symptom-identical to a broken hit test.",
        )
    })?;
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
    // ★ Computed from the SELECTION OUTLINE's own trace rect rather than from
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
    // ★ `declared_at(1.0, 1.0)` — the box's bottom-right corner, which is where
    // `handles::grip_rects` centres the south-east grip. Not the centre: the
    // centre of a selection box is `Grip::Move`, and a drag from there is a
    // MOVE, which would pass every assertion below for the wrong gesture.
    let frame = session.frame()?;
    let from = frame.declared_at(outline, 1.0, 1.0);
    // ★ Beyond the box, by fractions rather than by adding pixels to a
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
            None => format!(
                "★ THE GRIP DRAG COMMITTED NOTHING AND DECLINED NOTHING. That is the state \
                 this whole feature is a fix for: until 2026-08-19 every resize drag was \
                 consumed and thrown away, so a build that has reverted to it is silent on \
                 both channels — which is exactly what an operator reports as 'resize does not \
                 work'. Look at `canvas::interact`'s `GestureOutcome::Resize` arm. Trace: {}.",
                session.trace_path().display()
            ),
        }));
    };

    // --- 5: ★★ the numbers a wrong build would get wrong -------------------
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
