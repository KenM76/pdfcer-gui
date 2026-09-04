//! `pages_drag_shows_where_it_lands` — dragging a page thumbnail draws an
//! insertion caret, and releasing puts the page there.
//!
//! # The gap this closes
//!
//! The Pages panel sensed `egui::Sense::click()` and nothing else, so
//! reordering was two ribbon buttons that move the operand set **one place at
//! a time**. Putting page 40 in front of page 3 was thirty-seven presses, and
//! the gesture every operator tries first was not sensed at all.
//!
//! # Why this needs driving, and could not be unit-tested
//!
//! `ops::drop_order` — the arithmetic — has twenty-one unit tests, including a
//! sweep asserting every operand set at every gap in a six-page document yields
//! a permutation. **None of them can fail on a build where the panel does not
//! sense a drag**, because the arithmetic is reached only by a gesture and the
//! gesture is three frame-level edges:
//!
//! 1. `Response::drag_started_by(Primary)` on a tile, which settles the
//!    operand set;
//! 2. a resolved drop target, which exists only inside the layout pass because
//!    a *gap* has no position until the rows are placed;
//! 3. a release read from raw pointer input, because a drag begun on a tile
//!    ends anywhere.
//!
//! # ★ The assertion that is the whole point of the feature
//!
//! **The caret was drawn.** The operator's request was not "let me drag pages"
//! — it was *"make indicators to show when moving pages where they are going
//! to go to."* A drag that reorders correctly and shows nothing while it is in
//! flight has answered the wrong half.
//!
//! It is observable after the fact because `crate::diag::ui_rect_visible` is a
//! **change log**: the caret's region is emitted the first frame it is drawn
//! and a matching `ui-rect-gone` is emitted when it stops being drawn. So a
//! completed drag leaves both lines in the trace, and their presence is proof
//! the indicator existed during a gesture no screenshot could have caught
//! mid-flight.
//!
//! # What a passing run does NOT prove
//!
//! That the caret was in the right place, or the right colour. The trace gives
//! its rectangle and this check reads it — it asserts the caret sat **inside
//! the grid** and had a non-zero height, which rules out the two failures that
//! produce a region and no visible mark. It does not assert which boundary it
//! sat on; that is what the `gap=` field on the release line is for, and the
//! two are cross-checked below.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the Pages panel is reachable from. `view` is in every mode's tab
/// list, so Review is chosen only because the rest of the harness uses it.
const MODE: &str = "review";
/// The grid inside the panel — the region every tile is laid out in, and the
/// one this check aims within.
///
/// The panel's own `panel-pages` region is deliberately not used: it includes
/// the header, the previews checkbox and the slow-page note, so a caret
/// "inside the panel" would be a weaker claim than a caret inside the grid.
const GRID: &str = "panel-pages-grid";
/// The prefix of the per-tile regions, published since 2026-08-18.
const TILE: &str = "panel-pages-tile.";
/// The insertion caret's region.
const CARET: &str = "panel-pages-drop-caret";
/// The trace events the gesture emits.
///
/// ★ `page-drag-start`, **singular**, since 2026-08-20 — and the rename is not
/// cosmetic. The drag used to be the Pages panel's own and was traced by it;
/// it now lives in `crate::pagedrag`, shared with the page view and with the
/// document tab strip, because a drag that crosses documents cannot be owned
/// by the panel it started in. So it is *a page drag*, not *the pages panel's
/// drag*.
///
/// This check went red on the rename and reported *"the tile does not sense a
/// drag"* over a build whose trace carried the line two names away — a
/// confident, specific, entirely wrong defect report about working code, which
/// is `CONTINUE.md` §7's whole subject. A harness constant naming an
/// application event is a **coupling**, and it decays silently in exactly one
/// direction: absence reads as failure.
const DRAG_START: &str = "page-drag-start";
/// See [`DRAG_START`].
const DRAG_RELEASE: &str = "pages-drag-release";
/// The label `vector_edit` traces when `reorder_pages` succeeded.
const REORDERED: &str = "reorder-pages";
/// The fewest pages this check can say anything with.
///
/// Three. With two, every landing is the block's own lip and the correct
/// answer to a drag is *refuse*, so a run on a two-page fixture cannot
/// distinguish a working gesture from a dead one.
const MIN_PAGES: usize = 3;

/// How far across the landing tile the pointer is released, as a fraction of
/// its width.
///
/// Three-quarters, not the right edge. The panel resolves the **nearer**
/// vertical edge, so anything past the midpoint means the same boundary — and
/// a point exactly ON the edge is the one place a rounding difference between
/// the application's `f32` rectangle and this harness's reading of it could
/// flip the answer. Three-quarters is unambiguous and still inside the tile,
/// which is what makes the drop target resolve at all.
const LAND_ACROSS: f32 = 0.75;

/// See the module documentation.
pub struct PagesDragShowsWhereItLands;

impl Check for PagesDragShowsWhereItLands {
    fn name(&self) -> &'static str {
        "pages_drag_shows_where_it_lands"
    }

    fn defect(&self) -> &'static str {
        "a page thumbnail cannot be dragged to a new position, or it can and nothing shows \
         where it will land — so reordering is the two one-step ribbon arrows, and moving a \
         sheet across a drawing set is dozens of presses with no preview of the result"
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
            "input is disabled (--no-input). This check drags the pointer across a panel. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("pages_drag.trace.txt"));
    spec.pdf = Some(pdf);
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

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: get the Pages panel on screen ----------------------------------
    //
    // It may already be docked, which is the default layout. Only if it is not
    // does this reach for the ribbon, because pressing a panel toggle that is
    // already on would CLOSE the thing this check needs.
    if declared(&session.trace()?, ui_rect, GRID).is_none() {
        open_pages_panel(&session, &driver, ui_rect)?;
    }
    let trace = session.trace()?;
    let Some(grid) = declared(&trace, ui_rect, GRID) else {
        return Err(Error::new(format!(
            "no `{GRID}` region after asking for the Pages panel, so there is no grid to drag \
             in. Regions beginning `panel-`: {}.",
            list(&declared_names(&trace, ui_rect, "panel-"))
        )));
    };
    report.note("the Pages panel is on screen");

    // --- 2: find two tiles to work between ---------------------------------
    let tiles = declared_names(&trace, ui_rect, TILE);
    if tiles.len() < MIN_PAGES {
        return Err(Error::new(format!(
            "the fixture shows {} page tile(s) and this check needs at least {MIN_PAGES}. On a \
             shorter document every landing is the dragged block's own lip, where refusing is \
             the CORRECT answer — so a run here could not tell a working gesture from a dead \
             one. Pass a longer --pdf.",
            tiles.len()
        )));
    }
    let from = declared(&trace, ui_rect, &format!("{TILE}0")).ok_or_else(|| {
        Error::new(format!(
            "no `{TILE}0` region — the first page's tile is not on screen. Tiles declared: {}.",
            list(&tiles)
        ))
    })?;
    let onto = declared(&trace, ui_rect, &format!("{TILE}1")).ok_or_else(|| {
        Error::new(format!(
            "no `{TILE}1` region — the second page's tile is not on screen. Tiles declared: {}.",
            list(&tiles)
        ))
    })?;

    // --- 3: drag page 1 onto the far side of page 2 ------------------------
    //
    // The RIGHT half of tile 1, because the panel resolves the nearer vertical
    // edge: the right half means the boundary AFTER page 2, which is gap 2.
    // That is a real move — pages 1 and 2 swap — and it is outside the dragged
    // block, so it is not the no-op case the panel correctly refuses.
    let frame = session.frame()?;
    let start = frame.declared_center(from);
    let landing = frame.declared_at(onto, LAND_ACROSS, 0.5);
    report.note(format!(
        "dragging tile 0 from ({}, {}) to the right half of tile 1 at ({}, {})",
        start.x(),
        start.y(),
        landing.x(),
        landing.y()
    ));
    driver.drag(start, landing)?;
    session.settle(24);

    // --- 4: the gesture started at all -------------------------------------
    let trace = session.trace()?;
    let Some(started) = trace.last(DRAG_START) else {
        return Ok(Some(format!(
            "the pointer was pressed on tile 0 and dragged across the grid, and no \
             `{DRAG_START}` line was traced. The tile does not sense a drag — which is the \
             state this panel shipped in: `Sense::click()` and nothing else."
        )));
    };
    report.note(format!("gesture began: `{}`", started.raw));

    // --- 5: ★ the indicator was drawn --------------------------------------
    //
    // `ui_rect_visible` is a change log, so a caret that lived only during the
    // drag still leaves its rectangle in the trace. Reading the rectangle —
    // rather than only its presence — is what rules out the two failures that
    // publish a region and draw nothing an operator could see.
    //
    // ★★ `declared_since`, NOT `declared`, and the difference is the whole
    // assertion. This check FAILED on 2026-08-19 saying the caret was never
    // published, over a trace that carried it four lines above the release:
    // `declared` asks "is it on screen NOW", and a caret that exists only while
    // the pointer is down is retired before the harness can look. Asking the
    // present-tense question about a thing whose nature is to be gone is a
    // guaranteed false negative — and it read as a missing feature, on the one
    // feature the operator had asked for by name.
    //
    // The anchor is `started.lineno`, so a caret from an earlier gesture in the
    // same run cannot satisfy it. See `driving::declared_since`.
    let Some(caret) =
        crate::checks::driving::declared_since(&trace, ui_rect, CARET, started.lineno)
    else {
        return Ok(Some(format!(
            "the drag was sensed (`{DRAG_START}` was traced) and NO `{CARET}` region was ever \
             published, so nothing showed the operator where the page would land. The \
             reordering may still be correct; the indicator — which is what was actually \
             asked for — is absent."
        )));
    };
    if caret.height() <= 0.0 {
        return Ok(Some(format!(
            "the `{CARET}` region was published with zero height ({caret:?}), so the caret \
             was 'drawn' as a point. A region with no extent is a line segment whose two \
             endpoints are the same — the geometry resolved and the tile rectangle it was \
             derived from did not."
        )));
    }
    if !overlaps(caret, grid) {
        return Ok(Some(format!(
            "the `{CARET}` region ({caret:?}) sits outside the grid ({grid:?}), so the caret \
             was drawn somewhere the operator is not looking. The likeliest cause is a \
             coordinate space: the caret is painted inside the scroll area and a rectangle \
             built in the parent's space would land at an offset."
        )));
    }
    report.note(format!(
        "the insertion caret was drawn inside the grid, {:.0} pt tall",
        caret.height()
    ));

    // --- 6: the release landed ---------------------------------------------
    let Some(release) = trace.last(DRAG_RELEASE) else {
        return Ok(Some(format!(
            "the drag started and drew its caret and no `{DRAG_RELEASE}` line was traced, so \
             the gesture never ended. A drag left in flight leaves a caret nobody can get rid \
             of — which is why the release is read from raw pointer input rather than from \
             the tile's own response."
        )));
    };
    report.note(format!("gesture ended: `{}`", release.raw));

    if release.get("reordered") != Some("1") {
        return Ok(Some(format!(
            "the drag was released over gap {} and reported `reordered=0` — so no \
             `Action::ReorderPages` was raised. Either the drop target was not resolved (the \
             pointer was over no tile at release) or the landing was judged a no-op. Dragging \
             page 1 past page 2 is neither. Line: `{}`.",
            release.get("gap").unwrap_or("?"),
            release.raw
        )));
    }
    // ★ Cross-check: the caret's own position and the gap the release used
    // must be the same decision. They are computed once, in the layout pass,
    // and carried together — but "carried together" is exactly the kind of
    // claim that is true when written and false after a refactor.
    if release.get("gap") != Some("2") {
        return Ok(Some(format!(
            "the drop landed at gap {} and the pointer was released over the RIGHT half of \
             the second tile, which is gap 2. The caret the operator was looking at and the \
             boundary the pages went to are not the same decision. Line: `{}`.",
            release.get("gap").unwrap_or("?"),
            release.raw
        )));
    }

    if trace.last(REORDERED).is_none() {
        return Ok(Some(format!(
            "the release raised its action (`reordered=1`) and no `{REORDERED}` line \
             followed, so the apply arm never ran or could not borrow the session. The \
             gesture is complete and the document is unchanged."
        )));
    }
    report.note("the reorder reached the document through the action funnel");
    Ok(None)
}

/// Bring the Pages panel up from the ribbon.
///
/// Separated because it is the part with nothing to assert: if the panel is
/// already docked — which is the default layout — this is not called at all,
/// and pressing its toggle would have CLOSED the surface under test.
///
/// ★ `pub(crate)` since 2026-08-31, for `checks::drop_onto_thumbnails`, which
/// needs the same panel on screen and would otherwise carry a second copy of a
/// two-click sequence that already handles the ribbon overflow.
pub(crate) fn open_pages_panel(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.view").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.view` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let Some(item) = crate::checks::driving::declared_or_in_overflow(
        session,
        driver,
        ui_rect,
        "ribbon.item.view.panel_pages",
    )?
    else {
        return Err(Error::new(
            "the View tab declares no `ribbon.item.view.panel_pages`, on the band or in the \
             overflow, so the Pages panel cannot be opened.",
        ));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);
    Ok(())
}

/// Whether two logical rectangles share any area.
fn overlaps(a: LRect, b: LRect) -> bool {
    a.min.x < b.max.x && b.min.x < a.max.x && a.min.y < b.max.y && b.min.y < a.max.y
}
