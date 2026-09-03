//! `multi_node_move_moves_every_picked_anchor` — **Shift-picked anchors move
//! together**, driven against the operator's own drawing.
//!
//! # What this is for
//!
//! `pdfcer`'s `gui` column ticked *"multi-node select-and-move"* `[x]` for
//! months. Their own sweep of 2026-08-19 corrected it: *"objects move together;
//! nodes one at a time"*, one of six rows that were true of the **old** in-repo
//! shell and became false, without anyone touching them, when the column's
//! referent moved to this build.
//!
//! It was false in an unusually quiet way. The selection model had held a
//! multi-node set since the Node rung landed — `SelectionState::pick_within`
//! adds a Shift-clicked anchor as its own entry — and `canvas::moving::subject`
//! read `entered_object()`, which is the **first** entry. Four anchors picked,
//! one moved. Nothing failed; both halves' unit tests passed; the capability
//! was present in the data model and absent from every consumer.
//!
//! # ★★ And the operator could not see the anchors AT ALL
//!
//! `FEATURES.md` recorded, against `view.show_points`, that *"this build draws
//! no anchor mark at any rung"*. So the Node rung could be entered, a set could
//! be picked, and there was **no surface anywhere** that said which points were
//! in it. The marks and the multi-node move landed together on 2026-08-19
//! because they are one feature: a set the operator cannot see is not a set
//! they can choose.
//!
//! That is also what makes this check possible. An anchor's screen position is
//! a fact about the page's *decomposition* — a harness cannot compute it
//! without re-implementing the content walk — so before the marks existed there
//! was nothing to aim at, and this check could not have been written at all.
//!
//! # The oracle
//!
//! `move-nodes`, **plus** `canvas-move … action=MoveNodes` carrying more than
//! one entry. The second is the whole point: a build with the old bug raises
//! `MoveNode` (singular), reaches the engine, redraws, and looks like a working
//! drag — it just moves one of the four points the operator picked. A check
//! asserting only *"the move committed"* would pass on the defect.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::{Driver, Key};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// `canvas-anchors total=… selected=… unselected_drawn=…`.
const ANCHORS_EVENT: &str = "canvas-anchors";
/// `canvas-move page=… level=… action=…`.
const MOVE_EVENT: &str = "canvas-move";
/// The label `vector_edit` traces when `move_nodes` succeeded.
const APPLIED: &str = "move-nodes";

/// How far to drag the anchor set, in screen pixels, on each axis.
///
/// Comfortably past `PageDelta::is_travel`'s floor, and small enough to stay
/// inside the window on a modest client area.
const DRAG_PX: f32 = 25.0;

/// See the module documentation.
pub struct MultiNodeMoveMovesEveryPickedAnchor;

impl Check for MultiNodeMoveMovesEveryPickedAnchor {
    fn name(&self) -> &'static str {
        "multi_node_move_moves_every_picked_anchor"
    }

    fn defect(&self) -> &'static str {
        "Shift-picking several anchors highlights all of them and a drag moves ONE — a selection \
         the program shows and does not honour, which is worse than not offering it, because the \
         operator has no reason to check"
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
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a drawing with a path on page 1."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point on a SHAPE — the \
             Node rung only exists inside a path.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check descends two rungs by double-clicking, \
             Shift-picks an anchor and drags. Reported as SKIPPED rather than passed.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("multi_node.trace.txt"));
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

    // --- 2: click the shape, then descend twice -----------------------------
    //
    // ★ Two double-clicks at the SAME point, which is the ladder this shell
    // defines: a click selects the object, a double-click descends to the part,
    // a second descends to the node. Aiming the second descent somewhere else
    // would be a different gesture — `click_inside` ascends the moment a click
    // leaves the entered object — and the check would then be measuring the
    // ascent.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    let at = frame.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(12);
    driver.double_click_at(at)?;
    session.settle(20);

    // --- 3: the anchors must be on screen -----------------------------------
    let trace = session.trace()?;
    let Some(anchors) = trace.last(ANCHORS_EVENT) else {
        return Err(Error::new(format!(
            "no `{ANCHORS_EVENT}` line after the descent, so the Part rung was never entered. \
             That happens when the point named a text run or an image — neither has anchors — \
             or when the double-click was read as two single clicks. Both are facts about the \
             fixture and the harness rather than about the move, which is why this is SKIPPED. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let total: usize = anchors.get_usize("total").unwrap_or(0);
    report.note(format!(
        "the Part rung was entered on an object with {total} anchors: `{}`",
        anchors.raw
    ));
    if total < 2 {
        return Err(Error::new(format!(
            "the entered object has {total} anchor(s), so there is no second one to Shift-pick. \
             Aim --doc-point at a polyline or a rectangle."
        )));
    }

    // --- 4: click a MARKED anchor, then Shift-pick a second -----------------
    //
    // ★★ Both aims come from the application's own published anchor rects, and
    // the first version of this check had neither. It descended twice at the
    // same point and asserted the Node rung had been entered — which failed,
    // because the Node rung is reached by clicking **near an anchor**, and a
    // point on a shape is almost never on one. It then tried to find the second
    // anchor by sweeping outward in nine-pixel steps and hoping.
    //
    // Both are the same mistake: guessing at a coordinate the application knows
    // and the harness cannot compute. An anchor's screen position is a fact
    // about the page's *decomposition* — deriving it here would mean
    // re-implementing the content walk in the test tool, which is the one thing
    // a test tool must never do, because then it can be wrong in the same
    // direction as the thing it is testing.
    //
    // So the overlay publishes its first few drawn anchors and this aims at two
    // of them. That is also why the anchor MARKS had to exist before this check
    // could: before 2026-08-19 nothing drew them, so there was nothing to
    // publish and nothing to aim at.
    let first_rect = driving::declared(&trace, ui_rect, anchor_region(0)).ok_or_else(|| {
        Error::new(format!(
            "the Part rung was entered and no `{}` was published, so the harness has no anchor \
             to aim at. On an object with more anchors than the overlay's cap the unselected \
             marks are suppressed deliberately — aim --doc-point at a simpler shape. Trace: {}.",
            anchor_region(0),
            session.trace_path().display()
        ))
    })?;
    let frame = session.frame()?;
    let first = frame.declared_center(first_rect);
    // ★ A DOUBLE click, because the Node rung is descended into and not merely
    // clicked. `SelectionState::click_inside` picks an anchor only when
    // `self.level` is already `Node` — a plain click at the Part rung re-picks
    // the *part*. Driving it is what showed this: two double-clicks at the aim
    // point reached the Part rung, a plain click on a marked anchor selected
    // nothing, and the check reported "0 anchors selected" over a build whose
    // ladder was working exactly as `RIBBON_IA.md` §4 specifies.
    //
    // So the descent to Node happens ON the anchor, which is also the only way
    // it can: `descend` needs a hit that names one, and a double-click at the
    // original aim point almost never lands on an anchor.
    driver.double_click_at(first)?;
    session.settle(16);

    // ★★ The second anchor's rect is read AFTER the descent, and the first
    // version of this check read it before. Driving it is the only reason that
    // is known: descending to the Node rung enters a *subpath*, and the marks
    // are subpath-scoped, so the whole published set changes — the trace shows
    // `canvas.anchor.1` moving from x=393.9 to x=336.4 across the descent.
    //
    // The stale aim landed outside the entered object, which `click_inside`
    // correctly treats as an ascent, and the check ended up at the Object rung
    // with two OBJECTS selected. It reported "1 anchor selected" and blamed
    // `pick_within`, which had done nothing wrong.
    //
    // This is the read-then-act interval `driving::stable_rect`'s doc comment
    // is about, arriving through a third door: not a layout settling, not a
    // dock re-arranging, but **the act itself changing what is published**.
    let trace = session.trace()?;
    let selected_rect = driving::declared(&trace, ui_rect, ANCHOR_REGION);
    let second = (0..PUBLISHED_ANCHORS)
        .filter_map(|n| driving::declared(&trace, ui_rect, anchor_region(n)))
        // Not the one already selected: Shift-clicking it again REMOVES it,
        // which `pick_within` does on purpose and which would leave the count
        // at zero rather than at two.
        .find(|r| selected_rect.is_none_or(|s| (r.min.x - s.min.x).abs() > ANCHOR_APART_PX))
        .ok_or_else(|| {
            Error::new(format!(
                "the subpath has one anchor. Aim --doc-point at a polyline. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
    let frame = session.frame()?;
    driver.click_with_modifier(frame.declared_center(second), Key::Shift)?;
    session.settle(16);

    let trace = session.trace()?;
    let picked = trace
        .last(ANCHORS_EVENT)
        .and_then(|l| l.get_usize("selected"))
        .unwrap_or(0);
    report.note(format!("{picked} anchor(s) selected after the Shift-click"));
    if picked < 2 {
        return Ok(Some(format!(
            "★ TWO MARKED ANCHORS WERE CLICKED, THE SECOND WITH SHIFT, AND {picked} ENDED UP \
             SELECTED.\n\
             The multi-node MOVE is downstream of this: if the selection cannot hold two \
             anchors there is nothing for the plural verb to carry. \
             `SelectionState::pick_within` is the function — it adds a Shift-picked entry and \
             removes one that was already there, so a build that toggles rather than extends \
             ends up back at one. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 5: drag the set ----------------------------------------------------
    // ★ The drag starts from the anchor's CURRENT published rect, not from
    // `first` — the position that was read before the descent. Third time this
    // check has been bitten by the same interval, and the third distinct cause:
    // the marks are re-laid out when the rung changes, so a screen position
    // read two gestures ago names a place the anchor has left.
    //
    // The rule that falls out of it is worth more than the fix: **a harness may
    // hold a coordinate for exactly as long as it performs no act that could
    // move it.** Every act here moves them.
    let trace = session.trace()?;
    let from = driving::declared(&trace, ui_rect, ANCHOR_REGION)
        .map_or(first, |r| frame.declared_center(r));
    let moves_before = trace.events(MOVE_EVENT).count();
    let frame = session.frame()?;
    driver.drag(from, frame.offset_from(from, DRAG_PX, DRAG_PX))?;
    session.settle(30);

    // --- 6: the verdict -----------------------------------------------------
    let trace = session.trace()?;
    let Some(moved) = trace.events(MOVE_EVENT).nth(moves_before) else {
        return Ok(Some(format!(
            "{picked} anchors were selected and the drag raised no `{MOVE_EVENT}` at all, so the \
             gesture was consumed and thrown away. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the drag committed: `{}`", moved.raw));

    if !moved.raw.contains("MoveNodes") {
        return Ok(Some(format!(
            "★★ {picked} ANCHORS WERE SELECTED AND THE DRAG RAISED THE SINGULAR VERB: `{}`.\n\
             This is the defect exactly. A build with it raises `MoveNode`, reaches the engine, \
             redraws, and looks like a working drag — it moves one of the anchors the operator \
             picked and leaves the rest behind, which reads as a rendering fault rather than as \
             a missing feature. `canvas::moving::subject` must gather every selected anchor on \
             the entered object via `SelectionState::selected_nodes_on`, not read \
             `entered_object()`, which is the FIRST entry.",
            moved.raw
        )));
    }

    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the drag raised `{}` and no `{APPLIED}` line followed, so the action was raised and \
             its apply arm never ran. Trace: {}.",
            moved.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ every picked anchor reached the engine as ONE `move_nodes` command");
    Ok(None)
}

/// The region name for the `n`th drawn anchor mark.
///
/// Mirrors `canvas::overlay::anchor_region`. Two copies of a name is exactly
/// what `text::commands`' rule warns about, and it is unavoidable across a
/// process boundary: the harness reads the application's trace and cannot link
/// against it. The mitigation is that a rename breaks this check loudly on its
/// next run, which is what `ui-rect`'s `declared` returning `None` produces —
/// a SKIP naming the missing region, not a silent pass.
fn anchor_region(n: usize) -> &'static str {
    const NAMES: [&str; 6] = [
        "canvas.anchor.0",
        "canvas.anchor.1",
        "canvas.anchor.2",
        "canvas.anchor.3",
        "canvas.anchor.4",
        "canvas.anchor.5",
    ];
    NAMES[n.min(NAMES.len() - 1)]
}

/// How many anchor regions the overlay publishes. Mirrors
/// `canvas::overlay::PUBLISHED_ANCHORS`.
const PUBLISHED_ANCHORS: usize = 6;

/// How far apart two anchor rects must be, in logical points, to be different
/// anchors.
///
/// An anchor mark is six points wide, so two rects whose left edges are within
/// two points of each other are the same mark republished — which happens on
/// every frame the layout moves, because `ui-rect` is a change log.
const ANCHOR_APART_PX: f32 = 2.0;

/// The region the first SELECTED anchor publishes. Mirrors
/// `canvas::overlay::SELECTED_ANCHOR_REGION`.
const ANCHOR_REGION: &str = "canvas.selected-anchor";
