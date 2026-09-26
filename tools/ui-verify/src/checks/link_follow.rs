//! **Clicking a `/Link`** — the two checks for a capability that did not exist
//! in this shell at all until 2026-09-01.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/link_follow.md`.

use std::path::PathBuf;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// `link-click page=… index=… kind=…` — the shell's record of a link press.
const CLICK_EVENT: &str = "link-click";

/// The canvas's per-frame line, carrying `page=` and `zoom=`.
const CANVAS: &str = "canvas";

/// `page-links page=… links=… unresolvable=… named=…` — emitted on a cache
/// **build**, never on a hit.
const RESOLVE_EVENT: &str = "page-links";

/// The status bar's edit-disclosure region.
const DISCLOSURE_REGION: &str = "status-group:edit-disclosure";

/// Four `/GoTo` links on page 1, targeting pages 2, 3, 4 and 2.
const GOTO_FIXTURE: &str = "fixtures/goto-actions.pdf";

/// Four non-navigating actions on page 1: `/URI`, `/JavaScript`, `/Launch`,
/// and a `/GoToR`.
const ACTION_FIXTURE: &str = "fixtures/non-navigation-links.pdf";

/// The centre of `goto-actions.pdf`'s **third** link — rect 36,620–200,650,
/// `/FitH`, targeting page 4.
const GOTO_LINK: (f64, f64) = (118.0, 635.0);

/// Where that link should land, 0-based.
const GOTO_TARGET_PAGE: usize = 3;

/// The centre of `non-navigation-links.pdf`'s **first** link — rect
/// 36,700–200,730, a `/URI`.
const ACTION_LINK: (f64, f64) = (118.0, 715.0);

/// How many Ctrl+wheel notches to zoom in before the non-navigation click.
const ZOOM_NOTCHES: usize = 4;

/// Both fixtures are US Letter.
const PAGE_PT: (f64, f64) = (612.0, 792.0);

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The `page=` the canvas last reported.
fn page_of(trace: &Trace) -> Option<usize> {
    trace.last(CANVAS)?.get_usize("page")
}

/// **The whole view state**, as the canvas last reported it: page, zoom and
/// scroll offset.
fn view_of(trace: &Trace) -> Option<(usize, String, String)> {
    let line = trace.last(CANVAS)?;
    Some((
        line.get_usize("page")?,
        line.get("zoom")?.to_owned(),
        line.get("off")?.to_owned(),
    ))
}

// ---------------------------------------------------------------------------
// 1 — a link that resolves
// ---------------------------------------------------------------------------

/// **Clicking a `/GoTo` link arrives at the page it names.**
pub struct ALinkGoesToThePageItNames;

impl Check for ALinkGoesToThePageItNames {
    fn name(&self) -> &'static str {
        "a_link_goes_to_the_page_it_names"
    }

    fn defect(&self) -> &'static str {
        "Clicking a link does nothing — a table of contents is a page of dead text — or it \
         navigates somewhere other than the page the link names"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_goto(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Launch on `fixture`, provoke a canvas rect, and return the session, the
/// driver and a mapping to aim with.
///
/// Everything before the subject, shared by both checks. `Err` is a SKIP.
fn set_up(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    trace_name: &str,
) -> Result<(Session, Driver, CanvasMapping)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the canvas. Reported as SKIPPED \
             rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let pdf = workspace_root().join(fixture);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the link fixture is not at {}. It is copied from the engine's \
             `fixtures/synthetic/links/`, where `tools/gen-link-fixtures.py` generates it.",
            pdf.display()
        )));
    }

    // The fixture is PINNED and `--pdf` is ignored, for the same reason
    // `checks::ocr` pins its own: this check's assertions name a specific link
    // at a specific rectangle targeting a specific page. Pointed at the
    // operator's drawing it would click empty paper and report a working
    // feature as broken — which is a false failure, and this project has
    // already spent an afternoon on one.
    let mut spec = LaunchSpec::new(&exe, ctx.out(trace_name));
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
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    let driver = Driver::new(session.window());

    // A layout probe, as `checks::delete_key` does and for its stated reason:
    // some builds trace their canvas rect only on a pointer event, and a
    // freshly opened document then gives the harness nothing to aim against.
    // Deliberately at the client-area centre, which on a Letter page at a
    // fitted zoom is the middle of the sheet and hits no link.
    let trace = if trace.last(ctx.profile.vocab.canvas_event).is_some() {
        trace
    } else {
        driver.click_at(session.frame()?.layout_probe_point())?;
        session.settle(10);
        session.trace()?
    };

    let mapping = CanvasMapping::from_trace(
        &trace,
        &ctx.profile.vocab,
        crate::coords::PageGeometry {
            width_pt: PAGE_PT.0,
            height_pt: PAGE_PT.1,
        },
        0,
    )?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}",
        mapping.image_rect, mapping.zoom
    ));
    Ok((session, driver, mapping))
}

/// Click a document point on page 0 and settle.
fn click_doc(
    session: &Session,
    driver: &Driver,
    mapping: &CanvasMapping,
    at: (f64, f64),
) -> Result<()> {
    let window = mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?;
    driver.click_at(session.frame()?.to_screen(window))?;
    session.settle(20);
    Ok(())
}

fn drive_goto(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, driver, mapping) = set_up(ctx, report, GOTO_FIXTURE, "link-goto.trace.txt")?;

    let before = page_of(&session.trace()?).unwrap_or(0);
    if before != 0 {
        return Err(Error::new(format!(
            "the document opened on page {before} rather than page 0, so this check's aim — a \
             link on the FIRST page — is at the wrong sheet."
        )));
    }
    click_doc(&session, &driver, &mapping, GOTO_LINK)?;

    let trace = session.trace()?;
    let Some(click) = trace.last(CLICK_EVENT) else {
        return Ok(Some(format!(
            "★★★ THE CLICK ON A LINK PRODUCED NOTHING. No `{CLICK_EVENT}` line followed a \
             press at the centre of a `/Link` whose rectangle the engine reports as \
             36,620–200,650 on this page.\n\n\
             That is the operator's own report — *\"does a clickable table of contents \
             work?\"* — reproduced. Either `canvas::links::under_pointer` found no link (check \
             `{RESOLVE_EVENT}`: {}) or the arm that consumes it never ran, which on this \
             ladder means something above it took the press. Trace: {}.",
            trace
                .last(RESOLVE_EVENT)
                .map_or_else(|| "not traced at all".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    report.note(format!("the click reached a link: `{}`", click.raw));

    if click.get("kind") != Some("page") {
        return Ok(Some(format!(
            "THE LINK WAS NOT RESOLVED TO A PAGE: `{}`. The engine's own `list-links` reports \
             this link as `dest=page target=4`, so a shell that classified it as anything else \
             is disagreeing with the reader it called. `kind=named` or `kind=unmapped` here \
             usually means the `DestinationReader` was built against a different document \
             revision than the page was resolved from.",
            click.raw
        )));
    }

    let after = page_of(&session.trace()?).unwrap_or(before);
    if after != GOTO_TARGET_PAGE {
        return Ok(Some(format!(
            "★★ THE LINK WENT TO THE WRONG PAGE: {before} → {after}, where {GOTO_TARGET_PAGE} \
             was named (page 4, 1-based — the engine's `list-links` prints `target=4`).\n\n\
             {}\n\nTrace: {}.",
            if after == 0 {
                "★★★ It landed on page 0, which is the signature failure: a destination that \
                 could not be resolved, defaulted to index 0, and navigated anyway. That is \
                 the exact case this fixture is built to catch — the engine's note asks for \
                 fixtures whose links never target page 1 precisely because a defaulted 0 \
                 would otherwise look correct."
            } else {
                "An off-by-one between the engine's 0-based `page_index` and the 1-based page \
                 numbers this program shows is the first thing to check."
            },
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the view moved from page {before} to page {after} — the page the link names, and \
         the furthest of the four targets in this fixture, so it cannot have been reached by \
         a defaulted index"
    ));
    Ok(None)
}

// ---------------------------------------------------------------------------
// 2 — a link that cannot be followed
// ---------------------------------------------------------------------------

/// **A `/URI` link is disclosed, not performed, and not silently ignored.**
///
/// The falsifying half of the pair. See the module header on why this is the
/// check that matters and why its assertion is a conjunction.
pub struct ALinkItCannotFollowSaysSo;

impl Check for ALinkItCannotFollowSaysSo {
    fn name(&self) -> &'static str {
        "a_link_it_cannot_follow_says_so_instead_of_jumping"
    }

    fn defect(&self) -> &'static str {
        "A link pdfcer cannot perform — a URI, a script, another file, a deleted target page — \
         either navigates somewhere it does not go, or does nothing at all and says nothing \
         about why"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_action(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive_action(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    let (session, driver, mapping) = set_up(ctx, report, ACTION_FIXTURE, "link-action.trace.txt")?;

    // **ZOOM IN BEFORE CLICKING**, and this is the assertion's teeth rather
    // than a convenience. See [`view_of`] for the falsification that failed
    // without it: the fixture opens on page 0, and the plausible wrong
    // implementation resolves an unresolvable destination to a defaulted page 0
    // — so "the page did not change" is true of the broken build too.
    //
    // Zoomed in, the view is nowhere near where any defaulted navigation would
    // put it, and there is no realistic catch-all that leaves all three numbers
    // alone.
    //
    // Ctrl+wheel over the link's own position, so the link stays under the
    // pointer as the zoom rises — `Driver::scroll_at_held`'s documented reason
    // for existing.
    let at = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(0, ACTION_LINK.0, ACTION_LINK.1))?);
    driver.scroll_at_held(at, &[crate::input::Key::Ctrl.vk()], 1, ZOOM_NOTCHES)?;
    session.settle(20);

    let Some(before) = view_of(&session.trace()?) else {
        return Err(Error::new(
            "the canvas reported no view state after the zoom, so this check has nothing to compare against.",
        ));
    };
    report.note(format!(
        "zoomed in first — page {}, zoom {}, offset {}. \
         ★ Without this the check CANNOT FAIL: the fixture opens on page 0 and a \
         wrongly-defaulted jump to page 0 moves nothing",
        before.0, before.1, before.2
    ));

    // Re-read the mapping: the zoom changed, so every earlier screen coordinate
    // is stale. `D:/dev/rag/egui/` carries that as a standing finding — harness
    // coordinates go stale the moment the layout moves.
    let mapping = CanvasMapping::from_trace(
        &session.trace()?,
        &ctx.profile.vocab,
        crate::coords::PageGeometry {
            width_pt: PAGE_PT.0,
            height_pt: PAGE_PT.1,
        },
        0,
    )?;
    click_doc(&session, &driver, &mapping, ACTION_LINK)?;

    let trace = session.trace()?;
    let Some(click) = trace.last(CLICK_EVENT) else {
        return Ok(Some(format!(
            "THE CLICK ON A `/URI` LINK PRODUCED NOTHING. No `{CLICK_EVENT}` line followed. A \
             link this program cannot perform still has to be RECOGNISED — a click that falls \
             through to text selection leaves the operator with no way to learn that the box \
             they pressed was a link at all. `{RESOLVE_EVENT}`: {}.",
            trace
                .last(RESOLVE_EVENT)
                .map_or_else(|| "not traced at all".to_owned(), |l| l.raw.clone())
        )));
    };
    if click.get("kind") != Some("action") {
        return Ok(Some(format!(
            "THE `/URI` WAS CLASSIFIED AS `{}`: `{}`. The engine's `list-links` reports this \
             one as `dest=action action=URI`. A `/URI` reaching this shell as anything else is \
             a destination the reader resolved and the shell then re-interpreted.",
            click.get("kind").unwrap_or("nothing"),
            click.raw
        )));
    }
    report.note(format!(
        "the `/URI` was recognised as an action: `{}`",
        click.raw
    ));

    // HALF ONE: an ABSENCE — and it is the WHOLE view, not just the page.
    let after = view_of(&session.trace()?).unwrap_or_else(|| before.clone());
    if after != before {
        return Ok(Some(format!(
            "★★★ A `/URI` LINK NAVIGATED. The view was page {} at zoom {} offset {} \
             before the click, and is page {} at zoom {} offset {} after it — on a \
             link that names no page at all.\n\n\
             This is the failure the whole five-variant enum exists to prevent, and \
             it has no symptom an operator would report: the view moves, something is \
             shown, and they conclude the document's links are wrong. \
             `canvas::links::follow` must reach its `NonNavigation` arm; a catch-all \
             feeding every variant to `destination::actions_for` produces exactly \
             this. Trace: {}.",
            before.0,
            before.1,
            before.2,
            after.0,
            after.1,
            after.2,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the view did not move at all — still page {}, zoom {}, offset {}",
        after.0, after.1, after.2
    ));

    // HALF TWO: a PRESENCE. It must have said why.
    //
    // Both halves, because either alone passes against a wrong build: a shell
    // that ignored the click entirely satisfies the absence, and one that
    // navigated AND explained itself satisfies the presence.
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE_REGION).is_none() {
        return Ok(Some(format!(
            "★★ THE LINK DID NOTHING AND SAID NOTHING. The click was recognised as a \
             non-navigation action and the view correctly did not move — and no \
             `{DISCLOSURE_REGION}` region was declared on any frame afterwards, so no sentence \
             reached the operator.\n\n\
             From where they are sitting that is identical to the link being dead, which is \
             the state this whole feature was built to remove. R9 says an unavailable \
             capability renders nothing; it does not say an unavailable capability explains \
             nothing when it is asked for directly. Regions the status bar did declare: {}.",
            driving::list(&driving::declared_names(
                &session.trace()?,
                ui_rect,
                "status-"
            ))
        )));
    }
    report.note(
        "★★ and it said so — the status bar's disclosure row was drawn. The pair is the \
         point: a shell that ignored the click would satisfy the 'did not navigate' half on \
         its own",
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The `/GoTo` target is not page 0 and not page 1.**
    #[test]
    fn the_goto_target_cannot_be_reached_by_a_defaulted_index() {
        // NOT a `const` block, even though clippy suggests one: a
        // const-evaluated `assert!` cannot format, and the message is most of
        // this test's value — a bare "assertion failed" sends a reader to the
        // line rather than to the reasoning.
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(
                GOTO_TARGET_PAGE > 1,
                "a target of {GOTO_TARGET_PAGE} is reachable by a defaulted 0 or an \
                 off-by-one, so a green result would establish nothing"
            );
        }
    }

    /// The two aim points are inside the rectangles the engine reports.
    #[test]
    fn both_aim_points_are_inside_the_rectangles_the_engine_reports() {
        // goto-actions.pdf, index 2: rect=36,620,200,650
        assert!((36.0..=200.0).contains(&GOTO_LINK.0), "{GOTO_LINK:?}");
        assert!((620.0..=650.0).contains(&GOTO_LINK.1), "{GOTO_LINK:?}");
        // non-navigation-links.pdf, index 0: rect=36,700,200,730
        assert!((36.0..=200.0).contains(&ACTION_LINK.0), "{ACTION_LINK:?}");
        assert!((700.0..=730.0).contains(&ACTION_LINK.1), "{ACTION_LINK:?}");
        // …and both are inside the page, or the mapping would refuse them.
        for p in [GOTO_LINK, ACTION_LINK] {
            assert!(p.0 > 0.0 && p.0 < PAGE_PT.0, "{p:?}");
            assert!(p.1 > 0.0 && p.1 < PAGE_PT.1, "{p:?}");
        }
    }

    /// The two aim points are far enough apart to be different links.
    #[test]
    fn the_two_aim_points_are_not_the_same_row() {
        assert!((GOTO_LINK.1 - ACTION_LINK.1).abs() > 30.0);
    }

    /// Every trace event this check reads is spelled once.
    #[test]
    fn the_event_names_are_distinct() {
        let all = [CLICK_EVENT, CANVAS, RESOLVE_EVENT];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
