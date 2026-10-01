//! `checks::reaching` — **putting a control where the pointer can hit it.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/reaching.md`.

use crate::error::Result;
use crate::geom::LRect;
use crate::input::Click;
use crate::launch::Session;

use super::driving::declared;
/// **Bring a docked panel to the front of its tab stack.**
pub fn raise_dock_tab(
    session: &Session,
    driver: &impl Click,
    ui_rect: &str,
    panel_command_id: &str,
) -> Result<bool> {
    let trace = session.trace()?;

    // Route 1 — the dock tab. What this function was built for, and what
    // it still uses whenever a tab strip is drawn at all.
    let region = format!("dock.tab.{panel_command_id}");
    // One condition, not two: `declared` also hands back fossil
    // `ui-rect-gone` entries, and a zero-sized rect is exactly how one of
    // those presents. A tab entry that is not substantial is not a tab we
    // declined to press — it is not a tab.
    if let Some(tab) = declared(&trace, ui_rect, &region)
        && tab.is_substantial()
    {
        driver.click_rect(session, tab)?;
        session.settle(20);
        return Ok(true);
    }

    // Route 2 — already there. Probed BEFORE the rail is pressed, and
    // that order is the entire safety argument for route 3.
    //
    // `dock.body.<id>` is published only while the panel is the drawn,
    // active tab of a visible side, which is precisely what a caller
    // means by "raised". Pressing anything in that state would be a
    // toggle acting on an open panel, and the dock closes those.
    if declared(&trace, ui_rect, &format!("dock.body.{panel_command_id}")).is_some() {
        return Ok(true);
    }

    // Route 3 — the left rail.
    //
    // Published through `ui_rect_visible`, so a declared rect is a
    // REACHABLE rect rather than merely a laid-out one, and marked
    // `RailFold::Never` for the panel entries — unlike the dock tab
    // it cannot be folded into the overflow chevron, and unlike the dock
    // tab it is published in every mode.
    let rail = format!("rail.tabs.{panel_command_id}");
    let Some(entry) = declared(&trace, ui_rect, &rail) else {
        return Ok(false);
    };
    if !entry.is_substantial() {
        return Ok(false);
    }
    let mark = trace.mark();
    driver.click_rect(session, entry)?;
    session.settle(24);

    // Verify rather than assume, and anchor the question at `mark`.
    //
    // The body probe above should have caught the on-screen case, but the
    // dock owns the predicate — `DockState::is_on_screen` asks about the
    // side as well as about the active tab — and the two can disagree. If
    // they did, the press just CLOSED the panel the caller asked to read.
    //
    // The anchor is not decoration. The trace is a cumulative log, so a
    // `panel-closed` emitted by the check's own setup minutes earlier
    // would answer this question with a yes and send the pointer back to
    // the rail for no reason. `Trace::mark` was taken before the press.
    //
    // There is no loop here and there cannot be one: a second disagreement
    // leaves the panel closed and the caller's own precondition reports
    // it, which is the right outcome for a state this helper cannot reach.
    let after = session.trace()?;
    let reclosed = after
        .last_after("panel-closed", mark)
        .is_some_and(|l| l.get("id") == Some(panel_command_id));
    if reclosed {
        driver.click_rect(session, entry)?;
        session.settle(24);
    }
    Ok(true)
}

/// **Scroll a panel until a control it declares is WHOLLY inside the panel's
/// body, and hand back the control's fresh rectangle.**
pub fn bring_into_body(
    session: &Session,
    driver: &impl Click,
    ui_rect: &str,
    body: &str,
    wanted: &str,
    attempts: usize,
    report: &mut crate::report::CheckReport,
) -> Result<Option<LRect>> {
    let mut last = None;
    for attempt in 0..=attempts {
        let trace = session.trace()?;
        let Some(rect) = declared(&trace, ui_rect, wanted) else {
            return Ok(None);
        };
        last = Some(rect);
        let Some(body_rect) = declared(&trace, ui_rect, body) else {
            // No body to measure against: the caller's own precondition owns
            // that question, so hand back what was declared rather than
            // inventing a verdict here.
            return Ok(Some(rect));
        };
        if body_rect.contains_rect(rect) {
            if attempt > 0 {
                report.note(format!(
                    "`{wanted}` was declared below `{body}`'s fold; {attempt} scroll notch(es) \
                     brought the whole control inside it"
                ));
            }
            return Ok(Some(rect));
        }
        if attempt == attempts {
            report.note(format!(
                "★ `{wanted}` is declared at {rect:?} and `{body}` is {body_rect:?}, so the \
                 control is NOT wholly inside its panel after {attempts} scroll notch(es). A \
                 click at its centre would land outside the panel."
            ));
            break;
        }
        // THE WHEEL GOES IN THE LOWER PART OF THE BODY, NOT ITS CENTRE.
        //
        // A dock body is not all scroll area. The Bookmarks panel draws its
        // authoring row, a hint and a separator ABOVE its `ScrollArea`, and on
        // the run this was written for the body's centre landed on that fixed
        // furniture: six notches, nothing moved, and the check reported the
        // control unreachable. Three quarters of the way down is inside the
        // scrolling region on every panel in this shell — the fixed furniture
        // is always at the top, because a control after an unbounded
        // `ScrollArea` is the defect `panels::bookmarks` records at length.
        let h = body_rect.max.y - body_rect.min.y;
        let lower = LRect::new(
            crate::geom::Pt {
                x: body_rect.min.x,
                y: body_rect.min.y + h * 0.7,
            },
            crate::geom::Pt {
                x: body_rect.max.x,
                y: body_rect.min.y + h * 0.8,
            },
        );
        driver.scroll_rect(session, lower, -1)?;
        session.settle(12);
    }
    Ok(last)
}

/// **Scroll a pane until `wanted` is on screen, and answer where it is.**
pub fn scroll_to(
    session: &crate::launch::Session,
    driver: &crate::input::Driver,
    ui_rect: &str,
    anchor: &str,
    wanted: &str,
    attempts: usize,
    report: &mut crate::report::CheckReport,
) -> crate::error::Result<Option<crate::geom::LRect>> {
    for attempt in 0..attempts {
        let trace = session.trace()?;
        if let Some(rect) = declared(&trace, ui_rect, wanted) {
            if attempt > 0 {
                report.note(format!(
                    "`{wanted}` was below the panel's fold; {attempt} scroll notch(es) brought \
                     it into view"
                ));
            }
            return Ok(Some(rect));
        }
        let Some(at) = declared(&trace, ui_rect, anchor) else {
            return Err(crate::error::Error::new(format!(
                "`{anchor}` stopped being visible while scrolling for `{wanted}`, so there is \
                 nothing left to aim the wheel at. Trace: {}.",
                session.trace_path().display()
            )));
        };
        let point = session.frame()?.declared_center(at);
        driver.scroll_at(point, -1)?;
        session.settle(12);
        // Instrumentation, kept rather than removed. When this loop fails the
        // question is always the same — *did the wheel move anything?* — and a
        // note answering it is the difference between "the controls are
        // missing" and "the wheel landed somewhere that does not scroll".
        // Three wrong anchors were diagnosed by reading exactly this.
        if let Some(after) = declared(&session.trace()?, ui_rect, anchor) {
            report.note(format!(
                "scroll {attempt}: wheel at ({}, {}), `{anchor}` now {:?}",
                point.x(),
                point.y(),
                after
            ));
        }
    }
    Ok(None)
}

/// **Open a panel's collapsed footer** — the header published as region
/// `footer` by `panels::footer::show` — so the controls inside it publish
/// their regions. The footer reports its state as `panel-footer id=… open=…`;
/// an open footer is left alone, since the header is a toggle. Returns whether
/// the footer is open afterwards.
pub fn open_footer(
    session: &Session,
    driver: &impl Click,
    ui_rect: &str,
    footer: &str,
) -> Result<bool> {
    session.settle(4);
    let trace = session.trace()?;
    let open = |t: &crate::trace::Trace| {
        t.events("panel-footer")
            .filter(|l| l.get("id") == Some(footer))
            .last()
            .map(|l| l.get("open") == Some("true"))
    };
    match open(&trace) {
        Some(true) => return Ok(true),
        None => return Ok(false),
        Some(false) => {}
    }
    let Some(header) = declared(&trace, ui_rect, footer) else {
        return Ok(false);
    };
    driver.click_rect(session, header)?;
    session.settle(20);
    Ok(open(&session.trace()?) == Some(true))
}
