//! `dragging_a_markup_moves_it` — **draw a shape, drag it, and it is somewhere
//! else.**
//!
//! # What this is for
//!
//! `FEATURES.md` carried this under the Format contextual tab:
//!
//! > *"In `pdfcer-gui` a placed markup can be selected and deleted but not moved
//! > or resized yet."*
//!
//! The move half landed on 2026-08-28, the day `pdfcer-core` shipped
//! `move_annotation`. This is the check that keeps it.
//!
//! ## ★★★ Why the failure it guards is worse than "the drag does nothing"
//!
//! Before this, an annotation drag was **consumed and discarded**.
//! `canvas::interact` forks on *"is an annotation selected?"* and the only
//! module on that branch answered for ce dimensions — so a stamp took the
//! branch, got `None`, and the content branch that does move things was
//! unreachable by construction. The operator pressed inside a shape, dragged it
//! across the sheet, released, and it was where it started, with no message
//! anywhere.
//!
//! ⇒ A fork whose branches can **both** answer *"not mine"* is worse than a
//! missing feature: the gesture is eaten. This check exists at the fork.
//!
//! ## ★★★ What this check CANNOT see, stated first because it is the point
//!
//! A move has two halves and **only one of them shows up in a render**:
//! `/Rect` moves the painted result for free, while `/L`, `/Vertices`,
//! `/InkList` and `/QuadPoints` hold **absolute page coordinates** and are what
//! any *other* tool regenerates an appearance from. Write only the first and
//! the annotation looks right here, right in a screenshot, right in a pixel
//! check — and is rebuilt **in its old place** by the next viewer that rebuilds
//! it.
//!
//! ★★ **So a pixel oracle is the wrong instrument for this feature**, and that
//! is not a limitation of this check but a fact about the format. Every
//! screenshot this project can take reads the appearance stream. The trace line
//! is the only place the second half is visible from here, through
//! `keys=` — the count of geometry keys the engine found and moved.
//!
//! `keys=0` is **correct** for a Stamp, a Text note or a Link, whose `/Rect`
//! *is* their geometry. It is asserted as *reported*, never as non-zero, for
//! exactly that reason — and the shape drawn here is a `/Square`, which the
//! engine does not list among the geometry-key subtypes either. What this check
//! establishes is that the chain reaches the engine's own two-halves
//! implementation; the engine's tests establish that both halves are written.
//!
//! ## The chain, and every link that can break silently
//!
//! | # | link | owner | a unit test can see it? |
//! |---|---|---|---|
//! | 1 | `markup.rectangle` arms the tool | shell | yes |
//! | 2 | a drag authors a `/Square` | `canvas::markup` | yes |
//! | 3 | a click on it **selects** it | `canvas::selection::annot` | yes |
//! | 4 | a second drag reaches `annotdrag` rather than `dimdrag` | `canvas::interact` | **no** — the fork is wiring |
//! | 5 | the action reaches `move_annotation` | `app::actions::annots` | yes |
//!
//! ★ Link 4 is the one this check is for, and it is the one that was broken.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review mode, then arm the rectangle tool — both through the harness seam
/// rather than through ribbon clicks.
///
/// ★ `mode.review` because markup is authored there, and driving from a named
/// mode makes the run reproducible rather than dependent on whatever mode the
/// last session left behind.
const INVOKE: &str = "mode.review,markup.rectangle";
/// The line the canvas writes when a shape is authored.
const COMMIT_EVENT: &str = "markup-commit";
/// The line the apply arm writes when the engine has authored it.
const APPLY_EVENT: &str = "add-markup";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// The line the drag writes when it decides to commit a move.
const DRAG_EVENT: &str = "annot-drag";
/// The line the apply arm writes when the engine has moved it.
///
/// ★ `-applied`, per the convention this project adopted after making the
/// same-name mistake twice: `vector_edit` writes its own `move-annotation …`
/// line for the identical edit, and `.last()` on the bare name reads that one.
const MOVED_EVENT: &str = "move-annotation-applied";
/// The page's own region, so a failure can say whether a sheet was even drawn.
///
/// ★★ `page`, not `canvas`. `canvas` is the name of a **trace event** in the
/// profile's vocabulary — the line carrying the view's rect and zoom — and it
/// is not a `ui-rect` region at all. Asking `declared()` for it answers `None`
/// on a perfectly healthy build, which is a check reporting the program broken
/// because the harness looked in the wrong dictionary. The regions this
/// application publishes for the sheet are `canvas-viewport`, `central-panel`
/// and `page`; the last is the one that means *a document is on screen*.
const PAGE_REGION: &str = "page";

/// Where the shape is drawn, as fractions of the page.
///
/// ★ Well inside the sheet and away from the title block on a real drawing, so
/// the click that selects it in step 3 cannot land on page content instead —
/// and away from the edges, so the move in step 4 has somewhere to go.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));
/// Where the move drag goes, as fractions of the page.
///
/// ★ A displacement in **both** axes, deliberately. A move that only travels in
/// x would pass on a build that dropped `dy` — and `dy` is the one with a sign
/// convention to get wrong, because PDF user space increases **upward** while
/// every screen coordinate here increases downward.
const MOVE_TO: (f64, f64) = (0.62, 0.62);

/// See the module documentation.
pub struct DraggingAMarkupMovesIt;

impl Check for DraggingAMarkupMovesIt {
    fn name(&self) -> &'static str {
        "dragging_a_markup_moves_it"
    }

    fn defect(&self) -> &'static str {
        "a placed markup can be selected, restyled and deleted and cannot be MOVED — the drag is \
         consumed by the annotation branch of the canvas fork and discarded there, so an \
         operator drags a stamp across the sheet, lets go, and it is where it started with no \
         message anywhere"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape with a drag, clicks it, \
             and drags it again. Every one of those is a real pointer gesture.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("markup-move.trace.txt"));
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

    // --- 1 & 2: draw a rectangle -------------------------------------------
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    let from = aim(ctx, &session, page, corner(SHAPE.0))?;
    let to = aim(ctx, &session, page, corner(SHAPE.1))?;
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).last() else {
        return Ok(Some(format!(
            "THE RECTANGLE TOOL AUTHORED NOTHING: a drag across the page produced no \
             `{COMMIT_EVENT}` line, so `markup.rectangle` did not arm or the drag was not seen \
             as one. This is the step BEFORE the one under test — there is no annotation to \
             move. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if trace.events(APPLY_EVENT).count() == 0 {
        return Ok(Some(format!(
            "THE ENGINE NEVER AUTHORED THE SHAPE: the canvas decided to — `{}` — and no \
             `{APPLY_EVENT}` line followed, so the apply arm did not run or `add_markup` \
             refused. A refused `vector_edit` traces `add-markup-refused`; look for that first. \
             Trace: {}.",
            commit.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★ a rectangle was authored: `{}`", commit.raw));

    // --- 3: click it, and confirm it is SELECTED ----------------------------
    //
    // ★ The centre of the shape just drawn, not a fixed point: the shape is
    // placed in page fractions and the click has to land inside it whatever the
    // sheet size. A fixed screen point would work on one fixture.
    let centre = corner((
        f64::midpoint(SHAPE.0.0, SHAPE.1.0),
        f64::midpoint(SHAPE.0.1, SHAPE.1.1),
    ));
    let centre_screen = aim(ctx, &session, page, centre)?;
    // ★★★ PUT THE TOOL DOWN FIRST, and `sys::vk::V`'s own doc comment is the
    // reason, written for exactly this situation:
    //
    // > With a measure or markup tool armed, a click on the page is a PICK
    // > rather than a selection, so any check that needs to select something it
    // > just authored has to disarm first.
    //
    // The first run of this check did not, and reported "THE SHAPE COULD NOT BE
    // SELECTED" about a build whose selection works perfectly - the click drew
    // a second rectangle instead. A harness that leaves a mode armed is
    // measuring a different program from the one an operator uses, and the
    // failure it produces names the wrong subject.
    //
    // ★★ **The POINTER first, the chord as the fallback — 2026-08-28.**
    //
    // `V` has worked in this check since it was written, and it is kept for
    // that reason. What changed is that it is no longer *first*:
    // `driving::arm_select_from_ribbon` records six runs of
    // `the_line_weight_switch_reaches_the_resize` in which `V` arrived **zero**
    // times with a dock panel that check had raised, and — the part that
    // matters — arrived silently, producing no line anywhere, so the check
    // blamed the panel. This check raises no panel, which is why `V` works
    // here; a click works whether or not that stays true.
    //
    // ★ The fallback stays rather than being deleted, because a route with
    // years of green behind it is better than a SKIP when the ribbon route is
    // unavailable. `scale_switch` refuses the same fallback, and its own note
    // says why: there, the chord is measured not to work.
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        report.note(
            "the ribbon route to the select tool was unavailable, so the pen was put down with \
             the `V` chord instead — the route this step used until 2026-08-28",
        );
        driver.press(crate::sys::vk::V)?;
        session.settle(12);
    }
    driver.click_at(centre_screen)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(selected) = trace.events(SELECT_EVENT).last() else {
        return Ok(Some(format!(
            "THE SHAPE COULD NOT BE SELECTED: a click at its centre produced no \
             `{SELECT_EVENT}` line. Selecting is the step before the one under test, and it has \
             worked since 2026-08-18, so this says the click missed — the shape is drawn in page \
             fractions and this aimed at the middle of them. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the shape was selected: `{}`", selected.raw));
    if selected.get("locked") == Some("true") {
        return Err(Error::new(
            "the annotation reports itself LOCKED (§12.5.3 bit 8), and a locked annotation is \
             deliberately not draggable — the ghost is withheld and no move is raised. SKIPPED \
             rather than failed: that is correct behaviour, and it means this fixture cannot \
             exercise the move.",
        ));
    }

    // --- 4 & 5: drag it, and confirm the engine moved it --------------------
    let landing = aim(ctx, &session, page, corner(MOVE_TO))?;
    driver.drag(centre_screen, landing)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(dragged) = trace.events(DRAG_EVENT).last() else {
        return Ok(Some(format!(
            "★★★ THE DRAG ON A SELECTED MARKUP RAISED NOTHING: no `{DRAG_EVENT}` line.\n\
             **This is the exact state the feature shipped in for ten days.** \
             `canvas::interact` forks on `selection.annot().is_some()`, and if `annotdrag` is \
             not reached on that branch the gesture is CONSUMED — the content branch below is \
             unreachable behind an annotation selection, so nothing moves and nothing declines. \
             Check the fork, then `annotdrag::eligible`, which withholds for a ce dimension and \
             for a locked annotation. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the drag decided to move it: `{}`", dragged.raw));

    let Some(moved) = trace.events(MOVED_EVENT).last() else {
        return Ok(Some(format!(
            "★★ THE MOVE WAS RAISED AND NOTHING REACHED THE DOCUMENT: `{}` and no \
             `{MOVED_EVENT}` line.\n\
             The action was raised and its apply arm never ran, or `move_annotation` refused. It \
             refuses a **widget** and a **ce dimension** by name through \
             `AnnotationMoveWrongVerb` — neither can arrive here, so a refusal would itself be \
             the finding. A refused `vector_edit` traces `move-annotation-refused`. Trace: {}.",
            dragged.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★★★ the engine moved it: `{}`", moved.raw));

    // --- the oracle: it moved, in both axes ---------------------------------
    let dx: f64 = moved.get("dx").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let dy: f64 = moved.get("dy").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    if dx.abs() < 1.0 || dy.abs() < 1.0 {
        return Ok(Some(format!(
            "★ THE MOVE TRAVELLED IN ONLY ONE AXIS: `{}` reports dx={dx:.3} dy={dy:.3}, and the \
             drag was diagonal by construction.\n\
             A zero on one axis means the canvas→page conversion dropped it. `dy` is the one to \
             suspect: PDF user space increases **upward** (§8.3.2.3) and every screen coordinate \
             in this harness increases downward, so a sign or a term lost in \
             `moving::page_delta` shows up here and nowhere else. Trace: {}.",
            moved.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ moved dx={dx:.2} pt, dy={dy:.2} pt — both axes travelled, geometry keys reported \
         as {}",
        moved.get("keys").unwrap_or("?")
    ));

    // ★ `keys=` is REPORTED and never asserted. A `/Square`'s geometry is its
    // `/Rect`, so zero is the correct answer here — and asserting non-zero
    // would fail on precisely the subtypes the engine says have none. What the
    // number is for is the reader of a failed run: a build that started
    // reporting keys on a Square would be a change worth noticing.
    Ok(None)
}
