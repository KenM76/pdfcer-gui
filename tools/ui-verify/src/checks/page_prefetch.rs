//! `pages_are_drawn_before_he_scrolls_to_them` — **the page he scrolls onto is
//! already drawn**, measured on a real document.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_prefetch.md`.

use std::collections::BTreeSet;

use super::dimdrive::launch_path;
use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The ribbon command for the mode whose default page display is continuous.
const MODE: &str = "mode.read"; // ui-text-exempt: a command id, never displayed

/// The document driven when no `--pdf` is given: few pages, but more than fit.
const FIXTURE: &str = "four-pages.pdf";
const FIXTURE_METHOD: &str = "It is checked in; restore it from git.";

/// `strip-prefetch-requested page=N current=C` — render-ahead's own request.
const PREFETCH_EVENT: &str = "strip-prefetch-requested";

/// `strip-raster-requested page=N visible=M` — the strip asking for a page the
/// operator can SEE. Prefetches are deliberately not matched here.
const VISIBLE_EVENT: &str = "strip-raster-requested";

/// `strip-visible pages=A,B current=C` — the strip's on-screen pages.
const SHOWN_EVENT: &str = "strip-visible";

/// `strip-prefetch band=N texels=T budget=B` — the off-canvas disclosure.
const DISCLOSURE_EVENT: &str = "strip-prefetch";

/// How many wheel notches to send, once, forwards.
const NOTCHES: i32 = 12;

/// See the module documentation.
pub struct PagesAreDrawnBeforeHeScrollsToThem;

impl Check for PagesAreDrawnBeforeHeScrollsToThem {
    fn name(&self) -> &'static str {
        "pages_are_drawn_before_he_scrolls_to_them"
    }

    fn defect(&self) -> &'static str {
        "the strip only ever asks for pages already on screen, so every sheet is rendered at the \
         moment he arrives at it and he waits — worst on a scanned document, where a page's \
         raster is an image the size of the page. Invisible to every other oracle: the picture \
         that eventually appears is correct, and the only symptom is a person waiting"
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
    let doc = ctx.out("page_prefetch.pdf");
    let source = match &ctx.pdf {
        Some(pdf) => pdf.clone(),
        None => driving::repo_fixture(FIXTURE, FIXTURE_METHOD)?,
    };
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    report.note(format!("document: {}", source.display()));
    let (session, pointer) = launch_path(ctx, report, doc, MODE, "page_prefetch", &[])?;
    let outcome = measure(ctx, report, &session, &pointer);
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

fn measure(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(40);
    let at = canvas_centre(ctx, session)?;

    // No gesture here, and that is the point: his complaint is about what
    // pdfcer does while he is NOT scrolling. The strip starts one render per
    // frame by design, so the band fills a page at a time and a check that
    // looked immediately would read a band that had not finished as a band that
    // never started.
    pointer.hover(session, at)?;
    session.settle(120);

    let ahead = prefetched(session)?;
    if let Some(failure) = band_fault(ctx, session, &ahead)? {
        return Ok(Some(failure));
    }
    let ahead_pages: BTreeSet<usize> = ahead.iter().map(|(page, _)| *page).collect();
    report.note(format!(
        "standing still drew {} page(s) ahead of him, nearest first: {}",
        ahead_pages.len(),
        list(&ahead_pages.iter().copied().collect::<Vec<_>>())
    ));
    scrolled(report, session, pointer, at, &ahead_pages)
}

/// The centre of the canvas, where the wheel is turned.
fn canvas_centre(ctx: &CheckContext, session: &Session) -> Result<WindowPoint> {
    let trace = session.trace()?;
    let line = trace.last(ctx.profile.vocab.canvas_event).ok_or_else(|| {
        Error::new(format!(
            "the trace carries no `{}` event, so there is nowhere to put the wheel.",
            ctx.profile.vocab.canvas_event
        ))
    })?;
    let canvas = line
        .get_rect(ctx.profile.vocab.canvas_rect_field)
        .ok_or_else(|| {
            Error::new(format!(
                "the `{}` event has no parsable `{}=` field.",
                ctx.profile.vocab.canvas_event, ctx.profile.vocab.canvas_rect_field
            ))
        })?;
    Ok(WindowPoint::centre_of(canvas))
}

/// The band standing still produced, judged: empty, over budget, or out of
/// order is a failure (or a skip, for an empty band on a `--pdf` document).
fn band_fault(
    ctx: &CheckContext,
    session: &Session,
    ahead: &[(usize, usize)],
) -> Result<Option<String>> {
    if ahead.is_empty() && ctx.pdf.is_none() {
        return Ok(Some(format!(
            "no `{PREFETCH_EVENT}` line after standing still on page 1 of {FIXTURE} in Read \
             mode, where the pages below it are off screen: render-ahead never ran. \
             `app::settle::fill_strip` is where to look. Trace: {}",
            session.trace_path().display()
        )));
    }
    if ahead.is_empty() {
        return Err(Error::new(format!(
            "no `{PREFETCH_EVENT}` line after standing still on the opening page. Either this \
             document has too few pages for a band, the mode did not switch to a continuous \
             display, or render-ahead is never reached. Reported as SKIPPED rather than FAILED \
             because the first two are properties of the fixture and not of the program — but if \
             this run used a multi-page document in Read mode it is the THIRD, and \
             `app::settle::fill_strip` is where to look. Trace: {}",
            session.trace_path().display()
        )));
    }
    if let Some(over) = over_budget(session)? {
        return Ok(Some(format!(
            "THE STRIP CACHE WENT OVER ITS BUDGET: {over}. `OpenDoc::prefetch_headroom` asks for \
             room for one MORE page rather than for the cache to be under budget, and the \
             difference is the whole of why render-ahead terminates. `retain` trims to AT OR \
             BELOW the budget, so a prefetch gated on `texels() < budget` fills past it, is \
             trimmed back under it, and fills again — re-rendering a page every few frames on an \
             idle window, for ever. A disclosure line over budget is that loop, in progress. \
             Trace: {}",
            session.trace_path().display()
        )));
    }

    Ok(not_nearest_first(ahead).map(|wrong| {
        format!(
            "THE BAND WAS OFFERED OUT OF ORDER: {wrong}. `StripRasters::retain` evicts the entry \
             FURTHEST from the current page, so the prefetch order has to be the exact reverse of \
             it — nearest first. Offered any other way, render-ahead asks for the page eviction \
             is about to drop and the two alternate for as long as he keeps scrolling: render, \
             evict, render. Nothing fails, nothing looks wrong, and the machine works \
             permanently. See `render::prefetch::prefetch_ranking`. Trace: {}",
            session.trace_path().display()
        )
    }))
}

/// Wheel forward onto the band; no page drawn ahead may be asked for again.
fn scrolled(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    at: WindowPoint,
    ahead_pages: &BTreeSet<usize>,
) -> Result<Option<String>> {
    let ahead_list: Vec<usize> = ahead_pages.iter().copied().collect();
    let before = visible_requests(session)?.len();
    pointer.wheel(session, at, -(NOTCHES as f32), false)?;
    session.settle(90);
    let arrived: Vec<usize> = visible_requests(session)?
        .into_iter()
        .skip(before)
        .collect();
    report.note(format!(
        "scrolling {NOTCHES} notches asked for {} page(s) as visible: {}",
        arrived.len(),
        list(&arrived)
    ));
    // Asking for nothing after a wheel that went nowhere would read as a band
    // that worked, so the scroll must have put a prefetched page on screen.
    let shown = on_screen(session)?;
    report.note(format!("on screen after the scroll: {}", list(&shown)));
    if !shown.iter().any(|page| ahead_pages.contains(page)) {
        return Err(Error::new(format!(
            "the scroll left pages {} on screen, none of them drawn ahead ({}), so nothing \
             about arriving on a prefetched page was measured. Trace: {}",
            list(&shown),
            list(&ahead_list),
            session.trace_path().display()
        )));
    }

    // A page prefetched before the scroll and asked for again as a visible page
    // after it was rendered, forgotten, and rendered again — which is exactly
    // the waiting he reported, at twice the cost.
    let wasted: Vec<usize> = arrived
        .iter()
        .copied()
        .filter(|page| ahead_pages.contains(page))
        .collect();
    if wasted.is_empty() {
        report.note(format!(
            "none of the {} page(s) drawn ahead of him had to be drawn again when he scrolled \
             onto them",
            ahead_list.len()
        ));
        return Ok(None);
    }

    Ok(Some(format!(
        "{} PAGE(S) WERE PREFETCHED AND THEN RE-RENDERED WHEN HE REACHED THEM: {}. Prefetched \
         before the scroll: {}. Asked for as visible after it: {}. Render-ahead did its work and \
         the cache threw it away before he arrived, so he waited anyway AND the page was rendered \
         twice. The two places that can do this are `StripRasters::retain` — which must evict \
         furthest-first, never visibility-first — and the render key, which must be identical for \
         a page fetched ahead and the same page once it is on screen. Trace: {}",
        wasted.len(),
        list(&wasted),
        list(&ahead_list),
        list(&arrived),
        session.trace_path().display()
    )))
}

/// Every page render-ahead asked for, with the page that was current when it
/// did, in order.
fn prefetched(session: &Session) -> Result<Vec<(usize, usize)>> {
    Ok(session
        .trace()?
        .events(PREFETCH_EVENT)
        .filter_map(|l| Some((l.get_usize("page")?, l.get_usize("current")?)))
        .collect())
}

/// The strip's on-screen pages at its last `strip-visible` line.
fn on_screen(session: &Session) -> Result<Vec<usize>> {
    let trace = session.trace()?;
    let pages = trace
        .last(SHOWN_EVENT)
        .and_then(|l| l.get("pages"))
        .ok_or_else(|| {
            Error::new(format!(
                "no `{SHOWN_EVENT} pages=` line, so where the scroll landed is unknown. Trace: {}",
                session.trace_path().display()
            ))
        })?;
    Ok(pages.split(',').filter_map(|p| p.parse().ok()).collect())
}

/// Every page the strip asked for because it was on screen, in order.
fn visible_requests(session: &Session) -> Result<Vec<usize>> {
    Ok(session
        .trace()?
        .events(VISIBLE_EVENT)
        .filter_map(|l| l.get_usize("page"))
        .collect())
}

/// The first disclosure line whose resident texels exceed the budget, if any.
fn over_budget(session: &Session) -> Result<Option<String>> {
    for line in session.trace()?.events(DISCLOSURE_EVENT) {
        let (Some(texels), Some(budget)) = (line.get_usize("texels"), line.get_usize("budget"))
        else {
            continue;
        };
        if texels > budget {
            return Ok(Some(format!(
                "band={} holding {texels} texels against a budget of {budget}",
                line.get("band").unwrap_or("?")
            )));
        }
    }
    Ok(None)
}

/// The first place the band's distance from the current page went BACKWARDS
/// while the current page held still, if any.
fn not_nearest_first(ahead: &[(usize, usize)]) -> Option<String> {
    for pair in ahead.windows(2) {
        let (page_a, current_a) = pair[0];
        let (page_b, current_b) = pair[1];
        if current_a != current_b {
            continue;
        }
        if page_b.abs_diff(current_a) < page_a.abs_diff(current_a) {
            return Some(format!(
                "with page {} being read, page {} was offered after page {} — further first",
                current_a + 1,
                page_b + 1,
                page_a + 1
            ));
        }
    }
    None
}

/// A page list for a message, one-based because that is what he sees.
fn list(pages: &[usize]) -> String {
    if pages.is_empty() {
        return "none".to_owned();
    }
    pages
        .iter()
        .map(|p| (p + 1).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    //! The two order predicates, which are pure over a trace's parsed numbers.
    //!
    //! What they cannot see is whether the trace carries those lines at all —
    //! that is the driven half, and the whole reason this file exists.

    use super::not_nearest_first;

    #[test]
    fn a_nearest_first_band_around_one_current_page_is_accepted() {
        let ahead = [(51, 50), (49, 50), (52, 50), (48, 50)];
        assert_eq!(not_nearest_first(&ahead), None);
    }

    #[test]
    fn a_furthest_first_band_is_named_with_both_pages() {
        let ahead = [(58, 50), (51, 50)];
        let complaint = not_nearest_first(&ahead).expect("the order went backwards");
        assert!(complaint.contains("52"), "the offending page, one-based");
        assert!(complaint.contains("59"), "and what preceded it");
    }

    #[test]
    fn a_drop_in_distance_across_a_scroll_is_not_a_defect() {
        // He scrolled: page 50 was current, then 60 was. The band re-ranks
        // around the new page, so the distance falls — and must not be read as
        // an ordering defect.
        let ahead = [(58, 50), (61, 60)];
        assert_eq!(not_nearest_first(&ahead), None);
    }
}
