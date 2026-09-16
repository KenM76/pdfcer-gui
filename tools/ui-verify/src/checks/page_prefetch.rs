//! `pages_are_drawn_before_he_scrolls_to_them` — **the page he scrolls onto is
//! already drawn**, measured on a real document.
//!
//! # The operator's report, and why a unit test cannot answer it
//!
//! > *"on multipage documents I noticed with scanned pdf I have to wait for
//! > pages to load as a I scroll to them. As many pages as we can should be
//! > rendered and ready to be shown as I scroll. The ones on screen should
//! > always take precedence to be rendered first."*
//!
//! `render::prefetch`'s own tests pin the band's ORDER, which is arithmetic
//! over three integers. What they cannot see is whether anything ever calls
//! it. Render-ahead is reached down one arm of one `if` in a function that
//! runs only on a frame, against a live worker, a real cache and a wall clock,
//! and every way it can fail silently is on that side of the boundary: a
//! `settling` guard that never releases, a headroom test false on the first
//! frame, a candidate scan that finds everything resident because the visible
//! set is wider than the band. In each of those the unit tests stay green and
//! the operator keeps waiting.
//!
//! # The oracle is his sentence, not the mechanism's
//!
//! `strip-prefetch-requested` only says the band was asked for. What he asked
//! for is that a page is **already drawn when he arrives at it**, and the two
//! are different claims: a build that prefetched page 5 and evicted it before
//! he got there emits the first and fails the second.
//!
//! So this check scrolls, and asserts that **no page prefetched before the
//! scroll is asked for again as a visible page after it**. A page in both
//! streams was rendered, forgotten, and rendered again — the waiting he
//! reported, with the extra cost of having rendered it twice.
//!
//! That is also why the two trace names must stay distinct, and `page_cache`
//! records it from the other side: that check asserts no page appears twice in
//! the VISIBLE stream, and a prefetched page may legitimately be evicted and
//! re-offered, so folding the names together would make both oracles
//! unfalsifiable at once.
//!
//! # The three subsidiary assertions
//!
//! * **The band is non-empty.** If nothing was ever prefetched the main
//!   assertion is vacuously true, so that outcome is reported as SKIPPED. An
//!   instrument that can only return one answer is not an instrument.
//! * **`texels <= budget` on every disclosure line.** `prefetch_headroom` asks
//!   for room for one more page rather than for the cache to be under budget,
//!   precisely so render-ahead cannot fill past the ceiling, be trimmed back,
//!   and fill again for ever. A line over budget is that loop in progress.
//! * **Nearest-first, while the current page holds still.** The band's order
//!   must be the exact reverse of `StripRasters::retain`'s eviction order. Get
//!   it wrong and nothing fails — the two mechanisms grind against each other,
//!   rendering and evicting the same sheet, with nothing on screen to say so.
//!   It is the one property here whose defect is invisible from outside, which
//!   is why it is asserted from the running program and not only from the pure
//!   function.
//!
//! # It needs a continuous mode and a document with pages to spare
//!
//! Single-page mode keeps no strip at all, so the check runs in **Read**,
//! whose default page display is continuous. A band cannot exist on a one-page
//! fixture; the check says so rather than passing vacuously.

use std::collections::BTreeSet;

use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose default page display is continuous.
const MODE: &str = "read";

/// `strip-prefetch-requested page=N current=C` — render-ahead's own request.
const PREFETCH_EVENT: &str = "strip-prefetch-requested";

/// `strip-raster-requested page=N visible=M` — the strip asking for a page the
/// operator can SEE. Prefetches are deliberately not matched here.
const VISIBLE_EVENT: &str = "strip-raster-requested";

/// `strip-prefetch band=N texels=T budget=B` — the off-canvas disclosure.
const DISCLOSURE_EVENT: &str = "strip-prefetch";

/// How many wheel notches to send, once, forwards.
///
/// Deliberately modest. 40 notches draw about three pages, measured by
/// `page_cache`, and this check must not outrun the eight-page band: a scroll
/// landing past everything prefetched would pass the main assertion for the
/// wrong reason, because the pages it then re-requests were never prefetched.
/// A short scroll lands inside the band, where the claim has to hold.
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check waits for a strip to fill in around the page being read, so it \
             needs a document with pages to spare.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment and turns the wheel \
             over the canvas. Reported as SKIPPED rather than passed: a check that did not run \
             has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("page_prefetch.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env.push((
        driving::SHELL_DIAG_ENV.0.to_owned(),
        driving::SHELL_DIAG_ENV.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(50);
    let driver = Driver::new(session.window());

    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(40);

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
    let at = session.frame()?.declared_center(canvas);

    // No gesture here, and that is the point: his complaint is about what
    // pdfcer does while he is NOT scrolling. The strip starts one render per
    // frame by design, so the band fills a page at a time and a check that
    // looked immediately would read a band that had not finished as a band that
    // never started.
    driver.move_to(at)?;
    session.settle(120);

    let ahead = prefetched(&session)?;
    if ahead.is_empty() {
        return Err(Error::new(format!(
            "no `{PREFETCH_EVENT}` line after standing still on the opening page. Either this \
             document has too few pages for a band, the mode did not switch to a continuous \
             display, or render-ahead is never reached. Reported as SKIPPED rather than FAILED \
             because the first two are properties of the fixture and not of the program — but if \
             this run used a multi-page document in Read mode it is the THIRD, and \
             `render::settle::fill_strip` is where to look. Trace: {}",
            session.trace_path().display()
        )));
    }
    let ahead_pages: BTreeSet<usize> = ahead.iter().map(|(page, _)| *page).collect();
    let ahead_list: Vec<usize> = ahead_pages.iter().copied().collect();
    report.note(format!(
        "standing still drew {} page(s) ahead of him: {}",
        ahead_list.len(),
        list(&ahead_list)
    ));

    if let Some(over) = over_budget(&session)? {
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

    if let Some(wrong) = not_nearest_first(&ahead) {
        return Ok(Some(format!(
            "THE BAND WAS OFFERED OUT OF ORDER: {wrong}. `StripRasters::retain` evicts the entry \
             FURTHEST from the current page, so the prefetch order has to be the exact reverse of \
             it — nearest first. Offered any other way, render-ahead asks for the page eviction \
             is about to drop and the two alternate for as long as he keeps scrolling: render, \
             evict, render. Nothing fails, nothing looks wrong, and the machine works \
             permanently. See `render::prefetch::prefetch_ranking`. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("the band was offered nearest-page-first, which is the reverse of eviction");

    let before = visible_requests(&session)?.len();
    driver.scroll_at(at, -NOTCHES)?;
    session.settle(90);
    let arrived: Vec<usize> = visible_requests(&session)?
        .into_iter()
        .skip(before)
        .collect();
    report.note(format!(
        "scrolling {NOTCHES} notches asked for {} page(s) as visible: {}",
        arrived.len(),
        list(&arrived)
    ));

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
///
/// Scoped to one `current` on purpose. Scrolling changes which page is current
/// and the band is re-ranked around the new one, so a distance that drops
/// across that boundary is the feature working — comparing across it would
/// report a defect on every scroll.
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
