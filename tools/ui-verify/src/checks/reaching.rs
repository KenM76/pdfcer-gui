//! `checks::reaching` — **putting a control where the pointer can hit it.**
//!
//! # Why this is its own file
//!
//! Split out of [`super::driving`] on 2026-09-05 under **R2** (no `.rs` file
//! over 1,500 lines), when the first full driven sweep added two more helpers
//! of this shape and took that file to 1,534.
//!
//! The seam is a real subject rather than an arbitrary cut, and the tell is
//! that all three functions here answer the **same question** —
//!
//! > *the application says this control is at that rectangle. Can the pointer
//! > actually reach it there, and if not, what has to happen first?*
//!
//! — where the rest of `driving` answers *"where is it"*, *"what frame is it
//! in"*, and *"press it"*. This project has spent real sessions on the gap
//! between those two questions, and every incident is written up on the
//! function that closed it:
//!
//! | function | the failure it exists to prevent |
//! |---|---|
//! | [`scroll_to`] | a control below a pane's fold, never declared, reported as missing |
//! | [`raise_dock_tab`] | a docked pane that is **not in front publishes nothing**, which is indistinguishable from a panel with nothing to say |
//! | [`bring_into_body`] | a control declared through the ungated `diag::ui_rect`, laid out **past the bottom of its panel**, clicked at a centre that is outside it |
//!
//! ★★ The three are stated as one family because getting the wrong one produces
//! the same class of report every time: **a confident, articulate failure about
//! a mechanism the trace can rule out.** [`bring_into_body`]'s header carries
//! the worked example — a Paste button nineteen points inside its panel, and a
//! failure message naming `paste_outline_item`, which had never been asked.
//!
//! ★ They are re-exported from [`super::driving`], so `driving::scroll_to` and
//! the rest keep working at every existing call site. The file is what R2
//! limits; the module path a check reads is a separate question and moving it
//! would have been churn with no reader served.

use crate::error::Result;
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::Session;

use super::driving::declared;
/// **Bring a docked panel to the front of its tab stack.**
///
/// # ★★★ Why this is a funnel and not a paragraph — 2026-09-05
///
/// `RESUME.md` records the finding in its own words: *"A docked pane that is
/// not in front publishes nothing, which is indistinguishable from a panel with
/// nothing to say."* The dock draws only the **active** tab's body, and
/// `dock.tab.<id>` is published for every tab whether or not it is active — so
/// a check that reads a panel's output and does not raise it first is reading
/// silence and reporting it as a defect. That has already cost this project one
/// misdiagnosis (a Properties pane behind another tab, reported as a panel that
/// failed to describe the selection) and it was fixed **at one call site**.
///
/// It was fixed at one call site twice more the same day — `bookmark_add`
/// carries its own copy, with a comment saying it *"made the first version of
/// this check pass with the defect planted back in"*. Three checks in the same
/// family did not have it. So it lives here now, and a new check that reads a
/// panel gets the correct behaviour by calling one function rather than by
/// having read someone else's comment.
///
/// # What it does
///
/// Three routes, tried in order, and the order is the safety argument.
///
/// | # | region | what it does |
/// |---|---|---|
/// | 1 | `dock.tab.<id>` | clicks the tab. The original behaviour. |
/// | 2 | `dock.body.<id>` | the panel is already drawn and active — presses NOTHING and returns `true`. |
/// | 3 | `rail.tabs.<id>` | presses the left-rail entry, then checks it did not close the panel. |
///
/// # ★★★ Why route 3 exists — the 2026-09-05 suppression, found 2026-09-12
///
/// Ken asked that day for *"no tabs in the left side bar when the left rail
/// is visible"*, and [`egui_shell::dock::Dock::tabs_suppressed`] delivers it:
/// when a rail was drawn and can raise **every** panel in the stack, the dock
/// draws no tab strip at all. In this application all three of its conditions
/// hold in every mode, so **`dock.tab.*` has not been published since.**
///
/// This function went on returning `Ok(false)` — correctly, by its own
/// contract — and three of its five callers **discard the bool**. They then
/// failed later with messages naming bookmark mechanisms that were never at
/// fault. Three checks skipped for a week and the suite stayed green, because
/// a SKIP is not red.
///
/// ⇒ The rule earned, and it generalises past this file: **a helper that can
/// decline must not hand back a `no` a caller is free to ignore.** The
/// durable form of that is for the helper to stop declining while a route
/// exists, which is what route 3 is. Fixing the three checks instead would
/// have left the fourth and fifth callers — and every future one — holding
/// the same loaded bool.
///
/// # ★★ Why route 2 must be tried BEFORE route 3
///
/// The rail entry dispatches the panel command, and `view.panel_*` is a
/// **toggle** (`app::panels::toggle_panel`, operator decision 2026-08-14):
/// pressing it while the panel is on screen **closes** it. A fallback that
/// pressed the rail unconditionally would therefore close the panel its
/// caller asked to read, and the caller would report an empty panel — the
/// exact class of confident wrong answer this whole family of helpers exists
/// to prevent, reintroduced by its own repair.
///
/// `dock.body.<id>` is published only while the panel is the drawn, active
/// tab of a visible side, so it answers the toggle's question directly.
///
/// # What `false` still means
///
/// The contract is unchanged and it is still not an error: **this panel has
/// no route on screen in this mode.** A caller's own precondition, which
/// knows what it needs, is the right place to judge that. What changed is
/// that `false` no longer *also* means "the dock happens to be drawing no
/// tabs today", which is what made it uninformative.
///
/// # Errors
///
/// If the trace cannot be read, the window frame cannot be resolved, or the
/// pointer cannot be driven.
pub fn raise_dock_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    panel_command_id: &str,
) -> Result<bool> {
    let trace = session.trace()?;

    // Route 1 — the dock tab. What this function was built for, and what
    // it still uses whenever a tab strip is drawn at all.
    let region = format!("dock.tab.{panel_command_id}");
    // ★ One condition, not two: `declared` also hands back fossil
    // `ui-rect-gone` entries, and a zero-sized rect is exactly how one of
    // those presents. A tab entry that is not substantial is not a tab we
    // declined to press — it is not a tab.
    if let Some(tab) = declared(&trace, ui_rect, &region)
        && tab.is_substantial()
    {
        driver.click_at(session.frame()?.declared_center(tab))?;
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
    // `RailFold::Never` for the five panel entries — unlike the dock tab
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
    driver.click_at(session.frame()?.declared_center(entry))?;
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
        driver.click_at(session.frame()?.declared_center(entry))?;
        session.settle(24);
    }
    Ok(true)
}

/// **Scroll a panel until a control it declares is WHOLLY inside the panel's
/// body, and hand back the control's fresh rectangle.**
///
/// # ★★★ Why [`scroll_to`] is not enough, and the incident — 2026-09-05
///
/// [`scroll_to`] stops as soon as the region is **declared**. That is the right
/// test for a control published through `diag::ui_rect_visible`, which is
/// silent while the control is off screen. It is the wrong test for one
/// published through the plain `diag::ui_rect`, which reports the rectangle
/// egui laid the widget out at whether or not any of it is on screen.
///
/// `a_bookmark_subtree_can_be_copied_and_pasted` failed on exactly that. The
/// Bookmarks panel's body was `[[52 181.7] - [274 766.0]]`; its **Paste**
/// button — which only exists once something has been copied, so it appears at
/// the very bottom of a list that has just grown — was declared at
/// `[[52 761.0] - [158.8 785.0]]`. Nineteen points of it were inside the panel
/// and the rest, including its centre, was not. The harness clicked the centre,
/// hit whatever lay under the dock, and reported:
///
/// > *the Paste button was pressed and no `bookmark-paste-applied` line
/// > followed, so the action was raised and never applied — or
/// > `paste_outline_item` refused*
///
/// which names two application mechanisms and is about neither. **This is the
/// project's recorded worst outcome — a confident, articulate failure about the
/// wrong subject** — and it is the third recurrence of the same root: *a rect
/// proves layout, not visibility.*
///
/// ⇒ The durable fix is here rather than in the check, for the same reason
/// [`raise_dock_tab`] is: the next check to read a control near the bottom of a
/// scrolling panel should get the right behaviour from a function call.
///
/// ★ It is **not** a substitute for the application publishing the gated form.
/// That remains the better fix and it is product code: `panels::bookmarks::clip`
/// calls `crate::diag::ui_rect`, where `panels::layers` and the rotation row
/// call `ui_rect_visible`. Reported, not changed, from here.
///
/// # What it does
///
/// Reads `body` and `wanted`. While `body` does not wholly contain `wanted`, it
/// rolls the wheel one notch down over the body's centre and re-reads. Returns
/// the rectangle once it fits, or `None` if `wanted` is never declared, or the
/// last rectangle seen once `attempts` are spent — with a note saying so, so a
/// caller that then fails is failing with the reason on the page.
///
/// # Errors
///
/// If the trace cannot be read, the frame cannot be resolved, or the wheel
/// cannot be driven.
pub fn bring_into_body(
    session: &Session,
    driver: &Driver,
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
        // ★★★ THE WHEEL GOES IN THE LOWER PART OF THE BODY, NOT ITS CENTRE.
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
        let point = session.frame()?.declared_center(lower);
        driver.scroll_at(point, -1)?;
        session.settle(12);
    }
    Ok(last)
}

/// **Scroll a pane until `wanted` is on screen, and answer where it is.**
///
/// # ★★★ Why this is a helper and not two copies of a loop
///
/// It was two copies for about ten minutes, and the second copy is what forced
/// the extraction: the field-scoped controls sit below the fold of the
/// Properties slot, and the widget-scoped controls sit below *those*. A check
/// that scrolled once found the first and reported the second missing — which
/// is the failure this function's existence prevents, and it is worth naming
/// because the message it produced was confident and wrong (*"the section is
/// not being called"*, about a section that was in the same trace).
///
/// ★★★ **It scrolls at the DOCK PANE, not at the content**, and that took three
/// wrong anchors to arrive at.
///
/// A wheel event has to land inside the scroll area, so the anchor's centre has
/// to be **on screen**. Three candidates were tried and each failed differently:
///
/// | anchor | why it failed |
/// |---|---|
/// | `properties.widget_edit` (a section's `min_rect`) | published ungated, so it exists even when the section is entirely off screen. The wheel went outside the window |
/// | `properties.form_field` published as `max_rect` | that is the space the `Ui` was ALLOWED, not the space it took — it named a rect over the **Objects** panel, and six notches scrolled the object list |
/// | `properties.form_field` published as `min_rect` | correct about where the section is, and the section is 741 pt tall in a 180 pt slot, so its **centre is below the window** |
///
/// ⇒ The generalisation: **content rects are not scroll anchors.** Any region
/// belonging to scrolled content can have its centre outside the viewport, by
/// definition, because that is what scrolling means. What is always visible is
/// the **pane**, and `egui_shell::dock` publishes it as
/// `dock.body.<panel command id>`.
///
/// `D:/dev/rag/egui/` carries the family this belongs to: harness coordinates
/// go stale when a layout changes, and a wheel aimed at a remembered position
/// scrolls whatever is there now.
///
/// Returns `None` when the region never appears — the caller decides whether
/// that is a failure or a skip, because only the caller knows what it means.
///
/// # ★★ `attempts` is the caller's, and it is not a tuning knob
///
/// It is *how far the caller is willing to say it looked*, and it belongs in
/// the caller because the failure message does. A properties pane is a few
/// notches deep; a settings dialog with seven collapsed groups is more. A
/// constant here would make every caller's "I looked and it was not there"
/// mean a different distance without saying so.
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
        // ★ Instrumentation, kept rather than removed. When this loop fails the
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
