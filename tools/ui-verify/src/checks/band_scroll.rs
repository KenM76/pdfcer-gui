//! `band_scroll` — **a ribbon command past the fold is still reachable**, and
//! the harness proves it by walking the band to it.
//!
//! # ★★★ The defect this exists for was in the HARNESS, and it reported the
//! application as broken
//!
//! Until 2026-08-25 the ribbon's last resort at a narrow width was a
//! `⏷ N more` **dropdown**: one click and everything hidden appeared at once.
//! `driving::declared_or_in_overflow` was written against that, and one click
//! was the whole search.
//!
//! On the operator's instruction — *"do the scroll like Word"*, asked twice —
//! the dropdown became a `›` arrow that **shifts the band by exactly one
//! group** (`egui-shell`'s `ribbon::band`: `set_first(.., scrolled + 1)`). The
//! published region name stayed `ribbon.overflow`, deliberately, because it is
//! a cross-repo stability contract. So nothing renamed, nothing failed to
//! compile, no test went red — and the helper carried on clicking once.
//!
//! ★★★ **And the exact mechanism is one step subtler than that, which is why
//! it was measured rather than reasoned about.** The old order was: open every
//! **collapsed** group at the band's starting position; scroll **once**; then
//! look at the band **bare**. So the hole is not simply "two or more scrolls" —
//! it is *any command that needs a collapsed group opened at a stop past the
//! first*, which at 1,100 pt is About, Shortcuts and Properties after exactly
//! ONE scroll. The first version of this check asserted `scrolls >= 2`, ran,
//! and SKIPPED against the very build it was written for. Driving corrected the
//! diagnosis; the reasoning had been plausible and wrong.
//!
//! Either way the command was reported absent, in a confident sentence naming
//! it:
//!
//! > *"no `ribbon.item.file.about` region on the File tab or in its
//! > overflow."*
//!
//! Measured 2026-09-02: `about_reports_the_build`,
//! `shortcuts_reference_is_live` and `properties_metadata_round_trips` all
//! SKIPPED with that message at the harness's default 1,100 pt window, where
//! the File tab's **Document** and **pdfcer** groups are two and three stops
//! away. All three were worked around with `session.maximize()`, which is a
//! good thing for those three to do for their own reasons and does nothing at
//! all for the next check to meet this.
//!
//! ⇒ **A helper's prose and the mechanism it drives agreed when the prose was
//! written.** That is the shape this project keeps meeting, and the reason a
//! fix to it needs a check of its own rather than a corrected paragraph.
//!
//! # ★★ Why the assertion is more than "it was found"
//!
//! Because *"it was found"* passes on the broken build. Every other caller of
//! the helper asks a yes/no question, and on a correct application the answer
//! is `Some` whether the search took one stop or five. A check that asserted
//! only reachability would be green against the single-click implementation for
//! any command that happens to sit one stop past the fold — which is most of
//! them, and is exactly why the bug survived a full sweep.
//!
//! So this check reads `driving::BandSearch`, which the search fills in as it
//! goes, and requires a run that is **beyond the old search's reach**: either
//! two scroll stops, or one stop plus a collapsed group opened there. Measured
//! on 2026-09-03 it is the second of those — `1 scroll, found_in_popup` — and
//! the assertion is written as the disjunction so a ribbon re-layout that turns
//! it into the first does not silently disarm the check.
//!
//! ★ If a future ribbon change puts About one stop away, this check SKIPS
//! rather than passing, and says it needs recalibrating against whatever
//! command is then furthest right. A check that quietly loses its power is
//! worse than one that says so — see `crate::checks` rule 5.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | launch at the harness's default width, **no** `maximize()` | the File tab draws and `ribbon.overflow` is declared — i.e. the band really is scrolled short |
//! | B | confirm the subject is NOT on the band as it stands | `ribbon.item.file.about` absent; otherwise SKIP, the case cannot occur at this width |
//! | C | `driving::search_the_band` for it | found, by a walk the old one-click search could not have made |
//! | D | press it | `dialog:about` declared — the rect the search returned was live, not a fossil |

use crate::checks::driving::{
    OVERFLOW, SHELL_DIAG_ENV, declared, declared_names, frame_of, list, search_the_band,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The subject. Chosen because it is in the File tab's **last** group, so it is
/// the furthest thing on that tab from the band's start — and because it opens
/// a window whose appearance is unambiguous proof the rect was live.
const ITEM: &str = "ribbon.item.file.about";

/// The window pressing it opens.
const BODY: &str = "dialog:about";

/// The scroll count that on its own puts a run beyond the old search's reach.
/// One stop plus a collapsed group does too; see the assertion and the module
/// header.
const MIN_SCROLLS: usize = 2;

/// See the module documentation.
pub struct ACommandTwoScrollStopsAwayIsStillReachable;

impl Check for ACommandTwoScrollStopsAwayIsStillReachable {
    fn name(&self) -> &'static str {
        "a_command_two_scroll_stops_away_is_still_reachable"
    }

    fn defect(&self) -> &'static str {
        "the harness's ribbon search clicks the overflow arrow ONCE, which was the whole search \
         when the overflow was a dropdown and moves the band by a single group now that it is a \
         scroll arrow, and then looks at the band bare — so any command needing a collapsed group \
         opened past the first stop is reported as a lost command, which is what happened to \
         three checks in one sweep"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks ribbon controls. Reported as \
             SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // --- A: launch NARROW, and do not widen it -----------------------------
    //
    // ★★★ The one thing this check must not do is `session.maximize()`. Its
    // whole subject is the band at a width that cannot show every group, and a
    // maximised window is a window where the search has nothing to search.
    //
    // No `--pdf` either: About draws above the no-document guard, and every
    // ribbon group this walks past is on the File tab, which is present with
    // nothing loaded. Launching empty removes a whole class of fixture
    // dependence from a check about geometry.
    let mut spec = LaunchSpec::new(&exe, ctx.out("band-scroll.trace.txt"));
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
        "launched {} as pid {} at the harness's default width, deliberately NOT maximised",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    if declared(&trace, ui_rect, OVERFLOW).is_none() {
        return Err(Error::new(format!(
            "the File tab declares no `{OVERFLOW}` region, so this window is wide enough to show \
             every group on the band and there is nothing to scroll. That is a correct ribbon and \
             a check with no subject. Groups seen: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.group."))
        )));
    }
    report.note("the band is short of width: the right scroll arrow is on screen");

    // --- B: the subject is genuinely off the band --------------------------
    if declared(&trace, ui_rect, ITEM).is_some() {
        return Err(Error::new(format!(
            "`{ITEM}` is already on the band at this width, so the case under test — a command \
             the search has to WALK to — cannot occur in this run. Recalibrate this check against \
             whichever File-tab command is furthest right; the items on the band are: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.file."))
        )));
    }
    report.note(format!(
        "`{ITEM}` is not on the band as it stands, as required"
    ));

    // --- C: walk to it, and count the steps --------------------------------
    let (found, seen) = search_the_band(&session, &driver, ui_rect, ITEM)?;
    let Some(rect) = found else {
        return Ok(Some(format!(
            "the search walked the whole File band — {} scroll(s), {} rewind click(s), {} \
             collapsed group(s) opened — and never found `{ITEM}`. Either the command is genuinely \
             not registered in this build, or the walk stops early. Items it did see: {}.",
            seen.scrolls,
            seen.rewinds,
            seen.popups,
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.file."
            ))
        )));
    };
    report.note(format!(
        "found `{ITEM}` at {rect:?} after {} scroll(s), {} rewind click(s) and {} collapsed \
         group(s) opened",
        seen.scrolls, seen.rewinds, seen.popups
    ));

    // ★★★ THE assertion, and it is a PAIR. See the module header: the old
    // search looked inside collapsed groups at the band's starting position,
    // then scrolled once, then looked at the band **bare**. So a run only
    // distinguishes the fix from the bug if it needed either a second scroll,
    // or a collapsed group opened at a stop past the first.
    let beyond_the_old_search =
        seen.scrolls >= MIN_SCROLLS || (seen.scrolls >= 1 && seen.found_in_popup);
    if !beyond_the_old_search {
        return Err(Error::new(format!(
            "`{ITEM}` was reached in {} scroll stop(s){}, which the single-click search this \
             check exists to catch could have completed too. The application is fine; this check \
             has lost its power because the File tab's layout moved. Recalibrate it against \
             whichever command now needs either {MIN_SCROLLS} scrolls or a collapsed group opened \
             after scrolling, and read this module's header before softening the condition.",
            seen.scrolls,
            if seen.found_in_popup {
                " inside a collapsed group"
            } else {
                " on the band"
            }
        )));
    }
    report.note(format!(
        "that walk is beyond the old single-click search: {} scroll(s){}",
        seen.scrolls,
        if seen.found_in_popup {
            ", and the command was inside a collapsed group at that stop — the old code looked \
             at the band bare after its one click"
        } else {
            ", where the old code clicked exactly once"
        }
    ));

    // --- D: the rect was live ----------------------------------------------
    //
    // ★ Not decoration. `ui-rect` is a change log, so a search that returned a
    // FOSSIL — the rect a control had before it scrolled away — would satisfy
    // every assertion above and aim a click at empty band. Pressing it and
    // watching the window open is the only thing that distinguishes the two.
    driver.click_at(frame_of(&session, &session.trace()?, ui_rect, ITEM)?.declared_center(rect))?;
    session.settle(24);
    let trace = session.trace()?;
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "the search returned a rect for `{ITEM}` and clicking it declared no `{BODY}` region, \
             so the coordinate was a fossil rather than the control's live position — the click \
             landed on empty band. Regions declared this run: {}.",
            list(&declared_names(&trace, ui_rect, "dialog:"))
        )));
    }
    report.note("pressing the rect the search returned opened the About window: it was live");

    let shot = ctx.out("band-scroll.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!(
                "the window could not be captured ({e}); the trace assertions above still hold"
            ));
        }
    }
    Ok(None)
}
