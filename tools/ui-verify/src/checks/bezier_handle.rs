//! `bezier_handle_drag_changes_a_curve` — **a control point follows the
//! pointer and the engine rewrites the segment**, driven end to end.
//!
//! # What this is for
//!
//! `pdfcer`'s `gui` column ticked *"edit a Bézier handle"* `[x]`. Their sweep of
//! 2026-08-19 corrected it to `⬜ nothing` — one of six rows that were true of
//! the **old** in-repo shell and became false, untouched, when the column's
//! referent moved to this build.
//!
//! Nothing was blocking it. `EditSession::move_handle` has existed since Pass
//! 30.1 with a `Handle` enum, a planner, a `v`/`y` re-spelling path and a
//! disclosure contract. What was missing was a way to **see** a handle and a
//! way to **grab** one, and both are this shell's.
//!
//! # ★ Why this cannot be a unit test
//!
//! Five links, and three exist only in a running window:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a selected anchor on a curve draws two marks and a tether | `overlay` — nothing; it is pixels |
//! | 2 | a press within 8 px of a mark becomes `DragKind::Handle` and **not** a move | `handledrag::at` — the distance, not the routing |
//! | 3 | the handle outranks `Grip::Move`, which `grip_at` returns for the same press | ★ **nothing** |
//! | 4 | the pointer's canvas position converts to PDF user space against **this** frame's mapping | ★ **nothing** |
//! | 5 | `move_handle` rewrites one operator and the page redraws | `pdfcer-core` |
//!
//! Link 3 is the one that would fail silently and plausibly, and it is not
//! hypothetical: a handle sits **inside** the selection's bounding box, so
//! `handles::grip_at` answers `Grip::Move` for every press on one. Without the
//! priority rule in `gesture::meaning`, every attempt to drag a handle moves
//! the whole object instead — which looks like a clumsy gesture, not a defect,
//! and is exactly the shape of the bug that made the corner *anchors*
//! undraggable until the eight scale grips were confined to the Object rung.
//!
//! # The oracle
//!
//! `handle-commit node=… side=… to=[x y]`, **plus** `move-handle` from
//! `vector_edit`. Two lines because they answer different questions: the first
//! says the shell decided a handle drag happened and where it put the control
//! point, the second says the engine accepted it. A build that computed the
//! right position and never reached `EditSession` writes the first and not the
//! second, and from a chair that is indistinguishable from the handle not
//! moving at all.
//!
//! ★ It is **not** enough to assert that `move-handle` appeared. A drag that
//! was routed to `DragKind::Move` would write `canvas-move` and `move-objects`
//! — different lines — so the check would fail, correctly. But a drag routed to
//! the handle and given the *anchor's* position instead of the pointer's would
//! write both expected lines and change nothing visible. So `to=` is asserted
//! to differ from where the handle started, which is the number a wrong build
//! gets wrong.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// `canvas-anchors total=… selected=…`.
const ANCHORS_EVENT: &str = "canvas-anchors";
/// `canvas-handles n=…`.
const HANDLES_EVENT: &str = "canvas-handles";
/// `handle-commit node=… side=… to=[x y]` — the shell's own report.
const COMMIT_EVENT: &str = "handle-commit";
/// The label `vector_edit` traces when `move_handle` succeeded.
const APPLIED: &str = "move-handle";
/// The region the first SELECTED anchor publishes.
const SELECTED_ANCHOR: &str = "canvas.selected-anchor";

/// How far to drag the handle, in screen pixels, on each axis.
///
/// Forty: far enough that the committed position is unmistakably different
/// from where the handle started, and small enough to stay inside the window.
const DRAG_PX: f32 = 40.0;

/// See the module documentation.
pub struct BezierHandleDragChangesACurve;

impl Check for BezierHandleDragChangesACurve {
    fn name(&self) -> &'static str {
        "bezier_handle_drag_changes_a_curve"
    }

    fn defect(&self) -> &'static str {
        "a Bézier handle is drawn, takes the cursor and moves the whole object instead of the \
         control point — because a handle sits inside the selection box, where `grip_at` answers \
         `Grip::Move` for every press on one"
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
        Error::new(
            "no --pdf. This check needs a path with a CUBIC segment on page 1 — \
             `fixtures/polyline-nodes.pdf` is built for it.",
        )
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new("no --doc-point. Pass PAGE,X,Y in PDF user space naming a point on the path.")
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check descends two rungs, picks an anchor and \
             drags a handle. Reported as SKIPPED rather than passed.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("bezier_handle.trace.txt"));
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

    // --- 1: Edit ------------------------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: select the path and descend to the Part rung --------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    let at = frame.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(12);
    driver.double_click_at(at)?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.last(ANCHORS_EVENT).is_none() {
        return Err(Error::new(format!(
            "no `{ANCHORS_EVENT}` line after the descent, so the Part rung was never entered. \
             Aim --doc-point at the path. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 3: descend to the Node rung on each anchor until one has handles ---
    //
    // ★★ **A sweep over the anchors, not a guess at which one is curved.**
    //
    // Only anchors whose neighbouring segment is a cubic have handles at all —
    // `move_handle` refuses a straight one by name (`NoHandleHere`) and this
    // shell agrees by drawing nothing there. Which anchors those are is a fact
    // about the fixture's content stream, and hard-coding an index here would
    // couple the check to the generator in a way that breaks silently the day
    // a vertex is added: the check would descend onto a straight anchor, find
    // no handles, and SKIP while reporting nothing wrong.
    //
    // So it tries each published anchor in turn and stops at the first that
    // publishes a handle. The application answers the question; the harness
    // only asks it.
    let mut handle_rect = None;
    let mut anchor_rect = None;
    for n in 0..PUBLISHED_ANCHORS {
        // Re-read every iteration: descending re-lays the marks out, and a rect
        // read before the previous attempt names a place the anchor has left.
        // Third distinct cause of the same read-then-act bug in this suite.
        let trace = session.trace()?;
        let Some(rect) = driving::declared(&trace, ui_rect, anchor_region(n)) else {
            continue;
        };
        let frame = session.frame()?;
        driver.double_click_at(frame.declared_center(rect))?;
        session.settle(16);

        let trace = session.trace()?;
        let drawn = trace
            .last(HANDLES_EVENT)
            .and_then(|l| l.get_usize("n"))
            .unwrap_or(0);
        if drawn > 0
            && let Some(h) = driving::declared(&trace, ui_rect, handle_region(0))
        {
            handle_rect = Some(h);
            anchor_rect = driving::declared(&trace, ui_rect, SELECTED_ANCHOR);
            report.note(format!(
                "anchor {n} sits on a curve and drew {drawn} handle(s)"
            ));
            break;
        }
    }
    let (Some(handle), Some(anchor)) = (handle_rect, anchor_rect) else {
        return Err(Error::new(format!(
            "none of the {PUBLISHED_ANCHORS} published anchors drew a Bézier handle, so every \
             segment in the entered subpath is straight. That is an honest fact about the \
             fixture — `move_handle` refuses a straight segment by name and this shell draws no \
             handle there — so it is SKIPPED. Use `fixtures/polyline-nodes.pdf`, whose tail is \
             two cubics. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 4: drag the handle -------------------------------------------------
    //
    // ★ The handle mark, not the anchor. They are tens of pixels apart on this
    // fixture by construction — the control points are pulled sixty points off
    // the chord — precisely so a check that aimed at the wrong one fails
    // visibly rather than passing on a coincidence.
    let frame = session.frame()?;
    let from = frame.declared_center(handle);
    if from == frame.declared_center(anchor) {
        return Ok(Some(
            "the handle mark and the anchor mark are at the same screen point, so the handle is \
             drawn ON its anchor and the drag would be ambiguous. Either the control point \
             genuinely coincides with the on-curve point — a degenerate curve — or \
             `ObjectModelProvider::node_handles` is returning the anchor's own position, which \
             is the indexing error its doc comment warns about: the incoming handle is \
             `segments[k - 1].c2` and the outgoing is `segments[k].c1`."
                .to_owned(),
        ));
    }
    let commits_before = session.trace()?.events(COMMIT_EVENT).count();
    driver.drag(from, frame.offset_from(from, DRAG_PX, DRAG_PX))?;
    session.settle(30);

    // --- 5: the verdict -----------------------------------------------------
    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).nth(commits_before) else {
        let moved = trace.last("canvas-move").map(|l| l.raw.clone());
        return Ok(Some(match moved {
            Some(raw) => format!(
                "★★ THE DRAG ON A HANDLE MOVED SOMETHING ELSE: `{raw}`.\n\
                 This is the defect this check exists for. A handle sits INSIDE the selection's \
                 bounding box, so `handles::grip_at` answers `Grip::Move` for every press on \
                 one — and without the priority rule in `gesture::meaning`'s `edit_content` \
                 branch, every attempt to shape a curve moves the whole object instead. From a \
                 chair that is a clumsy gesture, not a bug. The rule is: the most specific thing \
                 under the pointer wins, and specificity is depth down the selection ladder."
            ),
            None => format!(
                "the drag on the handle raised nothing at all — no `{COMMIT_EVENT}` and no \
                 move. The press was consumed and thrown away, which means `DragKind::Handle` \
                 was chosen and `handledrag::drag` refused on the release: it needs the page, \
                 the mapping and a target provider, and `GestureOutcome::Handle` must be in \
                 `needs_targets` or the provider is `None`. That last one cost the resize \
                 gesture a whole driving session. Trace: {}.",
                session.trace_path().display()
            ),
        }));
    };
    report.note(format!("★ the handle drag committed: `{}`", commit.raw));

    // --- 6: and it reached the engine --------------------------------------
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the shell computed `{}` and no `{APPLIED}` line followed, so the action was raised \
             and `EditSession::move_handle` either never ran or refused. `NoHandleHere` is the \
             refusal to look for: it means the segment on that side is straight, and the shell \
             drew a handle where the engine says there is none — which would be \
             `node_handles` returning a control point for a `Line` segment. Trace: {}.",
            commit.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ the new control-point position reached the engine through `move_handle`");
    Ok(None)
}

/// How many anchor and handle regions the overlay publishes. Mirrors
/// `canvas::overlay::PUBLISHED_ANCHORS`.
const PUBLISHED_ANCHORS: usize = 6;

/// The region name for the `n`th drawn anchor mark.
fn anchor_region(n: usize) -> &'static str {
    const NAMES: [&str; PUBLISHED_ANCHORS] = [
        "canvas.anchor.0",
        "canvas.anchor.1",
        "canvas.anchor.2",
        "canvas.anchor.3",
        "canvas.anchor.4",
        "canvas.anchor.5",
    ];
    NAMES[n.min(PUBLISHED_ANCHORS - 1)]
}

/// The region name for the `n`th drawn handle mark.
fn handle_region(n: usize) -> &'static str {
    const NAMES: [&str; PUBLISHED_ANCHORS] = [
        "canvas.handle.0",
        "canvas.handle.1",
        "canvas.handle.2",
        "canvas.handle.3",
        "canvas.handle.4",
        "canvas.handle.5",
    ];
    NAMES[n.min(PUBLISHED_ANCHORS - 1)]
}
