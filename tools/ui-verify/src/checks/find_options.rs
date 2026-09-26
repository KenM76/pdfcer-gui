//! Two driven checks over the find bar's two preferences — `OPERATOR_REQUESTS.md`
//! **O179** and **O180**, both reported by the operator on 2026-09-12 and both
//! fixed the same day.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/find_options.md`.

use std::path::{Path, PathBuf};

use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::sys::vk;
use crate::trace::Trace;

/// How many frames to let the window settle before the first keystroke.
const SETTLE_FRAMES: u32 = 48;

// ===========================================================================
// O179 — the Zoom tick governs the POSITION as well as the zoom
// ===========================================================================

/// **With the find bar's *Zoom* option off, going to a hit must not move the
/// page on the canvas.**
pub struct ZoomOffHoldsTheViewOnAFindJump;

impl Check for ZoomOffHoldsTheViewOnAFindJump {
    fn name(&self) -> &'static str {
        "zoom_off_holds_the_view_on_a_find_jump"
    }

    fn defect(&self) -> &'static str {
        "with the find bar's Zoom option off, going to a hit still moves the page on the canvas"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess_zoom(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess_zoom(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let plan = Plan::new(ctx)?;

    // Restore the sandbox on every path out, including the failures. A check
    // that leaves `find_zoom_on_jump = false` behind has changed the starting
    // state of every check that runs after it, and a suite that shares state
    // measures the order it ran in.
    let _restore = RestorePrefs(plan.userdata.clone());

    // --- the control: the option ON, where the reveal MUST be armed ---------
    write_prefs(&plan.userdata, "find_zoom_on_jump = true\n")?;
    let on = plan.gesture(ctx, report, "zoom_on")?;
    let armed = on.events("find-reveal").any(|l| l.get("frac").is_some())
        || on.events("find-reveal-solved").next().is_some();
    if !armed {
        return Err(Error::new(
            "the control launch, with the Zoom option ON, produced no armed `find-reveal` line — \
             so either this fixture has no hit for the needle, or the reveal mechanism did not \
             run at all. Reported as SKIPPED rather than as a pass: the assertion this check \
             makes is an ABSENCE, and an absence is satisfied by every build in which the \
             mechanism never fired. Point the check at a document containing the letter `e`.",
        ));
    }
    report.note("control: with Zoom ON the reveal was armed, so the fixture and the gesture work");

    // --- the subject: the option OFF ---------------------------------------
    write_prefs(&plan.userdata, "find_zoom_on_jump = false\n")?;
    let off = plan.gesture(ctx, report, "zoom_off")?;

    let declined = off
        .events("find-reveal")
        .any(|l| l.get("declined") == Some("zoom-off"));
    if !declined {
        let seen: Vec<&str> = off
            .events("find-reveal")
            .map(|l| l.raw.as_str())
            .take(4)
            .collect();
        return Ok(Some(format!(
            "with `find_zoom_on_jump = false` the application never traced `find-reveal \
             declined=zoom-off`, so `reveal_current` did not consult the preference. The \
             `find-reveal` lines it did trace: {}. This is the O179 defect exactly — the flag was \
             read when re-fitting the zoom, and the CENTRING is a separate mechanism.",
            if seen.is_empty() {
                "none".to_owned()
            } else {
                seen.join(" | ")
            }
        )));
    }
    report.note("the guard was reached and declined to arm the reveal");

    // The independent half. See the struct doc: a guard that records its own
    // decision is not evidence about what the frame settled on.
    if let Some(solved) = off.events("find-reveal-solved").last() {
        return Ok(Some(format!(
            "the guard traced `declined=zoom-off`, and the canvas then SPENT a reveal anyway: \
             `{}`. The preference is being read and overruled, which is worse than not being \
             read — every unit test of the guard would still pass. Somewhere between \
             `reveal_current` and the canvas a second site is arming `OpenDoc::find_reveal`.",
            solved.raw
        )));
    }
    report.note("and no reveal was spent by the canvas in that run");

    Ok(None)
}

// ===========================================================================
// O180 — a blank at either end of the query
// ===========================================================================

/// **A trailing space must not change what a search finds, and the bar must say
/// so.**
pub struct ATrailingBlankDoesNotChangeWhatASearchFinds;

impl Check for ATrailingBlankDoesNotChangeWhatASearchFinds {
    fn name(&self) -> &'static str {
        "a_trailing_blank_does_not_change_what_a_search_finds"
    }

    fn defect(&self) -> &'static str {
        "a space pasted onto the end of a search term makes the search find nothing, and nothing \
         discloses it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess_blank(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess_blank(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let mut plan = Plan::new(ctx)?;
    let _restore = RestorePrefs(plan.userdata.clone());

    // Both launches leave the preference at its default, which is ON. The
    // variable under test is the QUERY, not the setting — the setting's own
    // "off" branch is covered by `find::query`'s unit tests, and driving it
    // would mean asserting that a search finds NOTHING, which is the assertion
    // every broken build satisfies.
    write_prefs(&plan.userdata, "")?;

    // --- the control: the bare needle --------------------------------------
    let clean = plan.gesture(ctx, report, "blank_control")?;
    let Some(baseline) = last_search(&clean) else {
        return Err(Error::new(
            "the control launch traced no `find` line, so no search ran and the comparison below \
             would be between two absences. Reported as SKIPPED.",
        ));
    };
    if baseline.hits == 0 {
        return Err(Error::new(
            "the control launch searched and found 0 hits, so there is nothing for a trailing \
             space to break. Whether this document contains the letter `e` is the fixture's \
             business — point the check at a text document. Reported as SKIPPED rather than \
             passed: a comparison of 0 against 0 is satisfied by a completely broken search.",
        ));
    }
    report.note(format!(
        "control: the bare needle found {} hit(s)",
        baseline.hits
    ));

    // --- the subject: the needle with a space after it ---------------------
    plan.trailing_space = true;
    let padded = plan.gesture(ctx, report, "blank_subject")?;

    let Some(with_blank) = last_search(&padded) else {
        return Ok(Some(
            "typing a space after the needle produced no `find` line at all, so pressing Enter \
             did not run a search. The trimmed query is not empty, so this is not the \
             empty-query branch — which would have traced `find-declined reason=empty-query`."
                .to_owned(),
        ));
    };
    if with_blank.hits != baseline.hits {
        return Ok(Some(format!(
            "the bare needle found {} hit(s) and the same needle followed by one space found {}. \
             That is O180 exactly: an invisible character decided the answer. \
             `find::query::for_search` should have trimmed it before `EditSession::search_text` \
             ever saw it. The line: `{}`",
            baseline.hits, with_blank.hits, with_blank.raw
        )));
    }
    report.note(format!(
        "the needle with a trailing space found the same {} hit(s)",
        with_blank.hits
    ));

    // And it searched for the TRIMMED string, not merely for something that
    // happened to give the same count. Without this, an application that passed
    // the raw query to a core that trimmed it internally would be
    // indistinguishable — and that is a different contract, owned by a crate
    // this project does not control.
    if with_blank.needle != baseline.needle {
        return Ok(Some(format!(
            "the hit counts agree but the application searched for {:?} when the space was typed \
             and {:?} when it was not. The count is a coincidence; the needle is the fact. The \
             trace field is written from the query AFTER `for_search`, so a difference here means \
             the untrimmed string reached the engine.",
            with_blank.needle, baseline.needle
        )));
    }
    report.note(format!(
        "and it searched for {:?}, the trimmed needle",
        with_blank.needle
    ));

    // --- and the bar disclosed it ------------------------------------------
    let Some(bar) = padded
        .events("find-bar")
        .filter(|l| l.get("edge_blanks").is_some())
        .last()
    else {
        return Ok(Some(
            "no `find-bar` line carried an `edge_blanks` field, so the disclosure's own condition \
             was never evaluated. Either the field was renamed or the bar stopped tracing; either \
             way this check can no longer tell a disclosed trim from a silent one."
                .to_owned(),
        ));
    };
    if bar.get("edge_blanks") != Some("true") {
        return Ok(Some(format!(
            "a space was typed after the needle and the bar reported `edge_blanks=false`, so the \
             disclosure row was not drawn. Trimming SILENTLY is the same defect wearing the other \
             coat — the operator typed one thing, pdfcer searched for another, and no surface \
             said so. The line: `{}`",
            bar.raw
        )));
    }
    report.note("the bar noticed the blank, so the disclosure row was drawn");

    Ok(None)
}

// ===========================================================================
// The shared machinery
// ===========================================================================

/// Everything both checks need in order to drive one launch.
struct Plan {
    exe: PathBuf,
    pdf: PathBuf,
    /// Where the sandboxed `preferences.txt` lives — beside the binary, which
    /// is what the application reads when it is run out of a scratch copy.
    userdata: PathBuf,
    /// Whether to press the space bar after the needle. O180's subject launch
    /// sets it; nothing else does.
    trailing_space: bool,
}

impl Plan {
    fn new(ctx: &CheckContext) -> Result<Self> {
        let exe = ctx.resolve_exe().ok_or_else(|| {
            Error::new(format!(
                "no binary to drive. Pass --exe, or build the profile's default at {}.",
                ctx.profile.default_exe
            ))
        })?;
        let pdf = ctx.pdf.clone().ok_or_else(|| {
            Error::new(
                "no --pdf. Find is gated on a document having pages, so with nothing open the \
                 chord correctly does nothing and this check would be measuring the gate rather \
                 than the feature.",
            )
        })?;
        // Checked before launching. A run that cannot type should not leave a
        // window on the operator's desktop to find that out.
        if !ctx.allow_input {
            return Err(Error::new(
                "input is disabled (--no-input), and this check is entirely input: it presses \
                 Ctrl+F, types a needle and presses Enter. Reported as SKIPPED rather than \
                 passed — a check that did not run has learned nothing.",
            ));
        }
        let userdata = exe
            .parent()
            .ok_or_else(|| Error::new("the binary has no parent directory to write userdata into"))?
            .join("userdata");
        Ok(Self {
            exe,
            pdf,
            userdata,
            trailing_space: false,
        })
    }

    /// Launch, open Find, type the needle, press Enter, and hand back the trace.
    fn gesture(&self, ctx: &CheckContext, report: &mut CheckReport, tag: &str) -> Result<Trace> {
        let mut spec =
            LaunchSpec::new(&self.exe, ctx.out(&format!("find_options.{tag}.trace.txt")));
        spec.pdf = Some(self.pdf.clone());
        spec.env.push((
            ctx.profile.diag_env.0.to_owned(),
            ctx.profile.diag_env.1.to_owned(),
        ));
        spec.allow_stale = ctx.allow_stale;
        spec.source_root = ctx.source_root.clone();

        let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
        report.note(format!("[{tag}] launched as pid {}", session.pid()));
        report.artifact(session.trace_path().to_path_buf());
        session.settle(SETTLE_FRAMES);

        let driver = Driver::new(session.window());

        // The control chord, before the feature. Without it this check cannot
        // tell "the preference is ignored" from "nothing was ever typed at the
        // window", and it will confidently report the first — which
        // `find_bar`'s own first run did, against a build in which Ctrl+F
        // works. `Ctrl+2` is bound to `mode.review` and is a chord the
        // application's key table could always spell.
        driver.press_chord(&[vk::CONTROL], vk::DIGIT_2)?;
        session.settle(12);
        let probe = session.trace()?;
        if !probe
            .events("chord-command")
            .any(|l| l.get("id") == Some("mode.review"))
        {
            return Err(Error::new(format!(
                "[{tag}] the control chord Ctrl+2 (`mode.review`) produced no `chord-command` \
                 line, so no keystroke reached the application and nothing below would mean \
                 anything. Reported as SKIPPED rather than as a Find failure: a check that types \
                 into nothing must never name a feature as the culprit."
            )));
        }

        driver.press_chord(&[vk::CONTROL], vk::F)?;
        // The field takes focus on the frame the bar opens, so the needle
        // cannot be typed in the same breath.
        session.settle(12);
        driver.press(vk::E)?;
        session.settle(6);
        if self.trailing_space {
            // The whole subject of O180, delivered as one keystroke, and a
            // space rather than a tab for the reason `vk::SPACE` records.
            driver.press(vk::SPACE)?;
            session.settle(6);
        }
        driver.press(vk::ENTER)?;
        session.settle(24);

        let trace = session.trace()?;
        if !trace.started(ctx.profile.vocab.start_event) {
            return Err(Error::new(format!(
                "[{tag}] the trace has no `{}` line, so the diagnostic switch did not reach the \
                 process. Captured stderr is at {}.",
                ctx.profile.vocab.start_event,
                session.trace_path().display()
            )));
        }
        Ok(trace)
    }
}

/// What the last search in a trace was, and what it found.
struct Search {
    /// The needle as the application traced it — **after** trimming, because
    /// the field is written from the prepared query rather than from the box.
    needle: String,
    hits: usize,
    /// The whole line, so a failure can quote the evidence rather than a
    /// reconstruction of it.
    raw: String,
}

fn last_search(trace: &Trace) -> Option<Search> {
    trace.events("find").last().map(|l| Search {
        needle: l.get("needle").unwrap_or("?").to_owned(),
        hits: l.get_usize("hits").unwrap_or(0),
        raw: l.raw.clone(),
    })
}

/// Write the sandbox's preference file.
fn write_prefs(userdata: &Path, body: &str) -> Result<()> {
    crate::sandbox::write_prefs(userdata, body).map_err(|e| {
        Error::new(format!(
            "could not write the preferences in {}: {e}. Reported as SKIPPED — a preference that \
             could not be written means the check never began.",
            userdata.display()
        ))
    })
}

/// Put the sandbox back to the bare seed when the check ends, however it ends.
struct RestorePrefs(PathBuf);

impl Drop for RestorePrefs {
    fn drop(&mut self) {
        if let Err(e) = crate::sandbox::reset_prefs(&self.0) {
            eprintln!(
                "ui-verify: WARNING — could not reset {} ({e}). A later check may be measuring a \
                 find preference this one left behind.",
                self.0.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The needle is the letter `e`.
    #[test]
    fn the_needle_is_the_letter_e() {
        assert_eq!(u32::from(vk::E), u32::from(b'E'));
    }

    /// The space bar is `VK_SPACE`.
    #[test]
    fn the_blank_is_the_space_bar() {
        assert_eq!(vk::SPACE, 0x20, "VK_SPACE");
    }

    /// A `Plan` starts without the trailing space.
    #[test]
    fn the_trailing_space_is_off_until_a_check_asks_for_it() {
        // Constructed by hand rather than through `Plan::new`, which needs a
        // `CheckContext` and a real binary.
        let plan = Plan {
            exe: PathBuf::from("x"),
            pdf: PathBuf::from("y"),
            userdata: PathBuf::from("z"),
            trailing_space: false,
        };
        assert!(!plan.trailing_space);
    }
}
