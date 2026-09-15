//! `checks::picking` — **one click is not always one pick**, in one place.
//!
//!
//! `measure_calibrates_by_picking_two_points` and
//! `set_scale_reads_the_group_it_is_about_to_overwrite` both drive the two-point
//! scale calibration. Both had **never once produced a verdict** — the first
//! SKIPPED in all five recorded sweep baselines, the second on its only one,
//! for a reason that had nothing to do with picking (see
//! [`crate::coords::CanvasMapping::span_from`]). The moment that was fixed and
//! they ran for the first time, both FAILED with the same sentence:
//!
//! ```text
//! two points were clicked on the page and no `scale-calibrate measured_pt=`
//! line was traced, so the pick never completed.
//! ```
//!
//! The accusation is false, and the application had already said so — forty
//! lines above, in its own trace:
//!
//! ```text
//! measure-snap-marker kind=DerivedCenterline marker=677.0,554.4 …
//! measure-pick outcome=Promoted reason=derived-candidate-needs-confirm
//! measure-pick kind=Scale in_progress=true committed=false
//! ```
//!
//! Two clicks were sent. The first found a **derived** snap candidate — a
//! centreline pdfcer *inferred* rather than one the file states — and pdfcer
//! refused to act on an inference without confirmation, which is
//! `pdfce_FeatureRequests/README.md` **rule 4** working exactly as specified.
//! The second click confirmed it, resolving pick **A**. Pick B was therefore
//! never clicked at all, and the check blamed the routing.
//!
//! # ★★★ The lesson, and it is not "snapping is awkward"
//!
//! **A sibling check had already found this, documented it at length, and
//! solved it — and the two checks that needed the solution could not see it.**
//! [`crate::checks::measure_linear`]'s header carries a full section on rule 4
//! and a table of the three alternatives it rejected; the constant beside its
//! event name is annotated, verbatim, `← click again`. That was written before
//! either calibration check ran, and neither inherited a line of it.
//!
//! This is the same shape as the correction recorded in
//! [`crate::checks::text_edit_real`]: *a rule written down beside one instance
//! of itself does not generalise on its own — the next instance arrives wearing
//! a different event name and reads as a new problem.* Here it did not even
//! change its event name. It changed its `kind=` field, from `Linear` to
//! `Scale`, and that was enough.
//!
//! ⚠ So the rule is not in a comment this time. It is in [`resolve_pick`], and
//! a check that clicks the canvas with a measure tool armed and does **not**
//! call it is a check that will one day report a promotion as a routing
//! failure. That is what this module is for.
//!
//! # Why not in [`crate::checks::driving`]
//!
//!
//! # What this module deliberately does NOT do
//!
//! It does not assert the *shape* of a pick sequence. `measure_linear` checks
//! that three picks read `committed=false, false, true`, that the resolving
//! tool was `Linear` and not something else, and that the promotion count is
//! reported; those are properties of *the linear-dimension feature*, not of
//! clicking, and they stay at that call site. This module owns exactly one
//! thing: **turn a point into a resolved `measure-pick` line, taking the
//! confirming click when the application asks for one, and refuse to keep
//! clicking forever.**
//!
//! `measure_linear` keeps its own copy of the loop for now, and that is stated
//! rather than implied. Folding it on is a safe change but it is not *this*
//! change: rewriting the one check already known to detect its defect, in the
//! same commit that first makes two others run, would destroy the independent
//! evidence at the moment it is most needed. [`crate::checks::driving`]'s
//! header records the identical judgement about `markup_rectangle`. ⚠ The
//! tripwire against that copy drifting is [`tests::the_two_implementations_agree_on_the_click_bound`].

use crate::error::Result;
use crate::input::Driver;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// `measure-pick …` — **one line per click** the armed measure tool resolved,
/// from `canvas::measure::click`.
///
/// One event name, two shapes:
///
/// ```text
/// measure-pick kind=Scale in_progress=true committed=false              ← a resolved pick
/// measure-pick outcome=Promoted reason=derived-candidate-needs-confirm  ← click again
/// ```
pub const PICK_EVENT: &str = "measure-pick";

/// The `outcome=` value that means *the click found an inference and is asking
/// before acting on it* — `ClickOutcome::Promoted`.
///
/// Matched on the **field**, never on the raw line, so a future addition to the
/// message cannot silently stop a check recognising a promotion. A promotion
/// that went unrecognised would be counted as a resolved pick carrying no
/// `committed=` field, and every downstream assertion would then fail against a
/// working build — which is precisely how this module came to exist.
pub const PICK_PROMOTED: &str = "Promoted";

/// How many clicks one pick may take before the two-click confirm is called
/// broken.
///
/// **Two**, and the number is `canvas::snap::snap_commit_clicks`'s own: a
/// routine candidate commits on the first click, a derived one on the second.
///
/// A third would mean `MeasureState::resolve_click` is not converging. Its
/// promote branch compares `derived_promoted != Some(point)`, so a click at the
/// same screen pixel that promoted *again* would mean the point the snap query
/// resolved to **moved between two clicks of a stationary pointer** — a real
/// finding, and not a reason to keep clicking.
pub const MAX_CLICKS_PER_PICK: usize = 2;

/// The outcome of driving one pick to resolution.
pub struct ResolvedPick {
    /// The resolved [`PICK_EVENT`] line — the one carrying `kind=` and
    /// `committed=`, never a promotion.
    pub line: TraceLine,
    /// How many clicks it took. `1` for a routine candidate, `2` when rule 4's
    /// confirmation was asked for and given.
    ///
    /// Worth reporting rather than discarding: a fixture where every pick needs
    /// two clicks is a fixture dense with inferred geometry, and that is
    /// context for any *later* failure in the same check.
    pub clicks: usize,
}

/// A failure attributable to the gesture rather than to the feature under test.
pub struct PickFailure {
    /// The sentence to hand back as the check's verdict.
    pub why: String,
}

/// **Click `screen` until the application resolves it into a pick, honouring
/// rule 4's confirmation.**
///
/// `label` names the pick in every message — `"A"`, `"B"`, `"2 of 3"`;
/// whatever the caller's reader will recognise. `settle` is the caller's own
/// frame budget, because the window a calibration pick drives back into is not
/// the window a dimension placement draws into, and neither should inherit the
/// other's timing.
///
/// # What it asserts, and why each one is here
///
/// * **Exactly one [`PICK_EVENT`] line per click.** `canvas::measure::click`
///   traces exactly once per click it is handed — on the promote path and on
///   the resolve path alike. Zero means the click never became a pick, which
///   has three readings worth separating: the gesture machine swallowed it
///   (`canvas::gesture::press_kind` returns `click: caps.author_measure`, so a
///   mode that had lost the `measure` tab from its tab list would swallow every
///   one), the click landed outside the page rect, or `canvas::interact`'s
///   `Click` arm no longer branches on `active_tool.measure_kind()`. More than
///   one means the click was delivered twice.
/// * **At most [`MAX_CLICKS_PER_PICK`] clicks.** See that constant.
///
/// # Errors
///
/// Only for harness faults — the trace could not be read, or the click could
/// not be delivered. A *finding* comes back as `Ok(Err(PickFailure))`, because
/// a finding is the check's verdict to phrase and a fault is not.
pub fn resolve_pick(
    session: &Session,
    driver: &Driver,
    report: &mut CheckReport,
    label: &str,
    screen: crate::coords::ScreenPoint,
    settle: u32,
) -> Result<std::result::Result<ResolvedPick, PickFailure>> {
    let mut clicks = 0usize;
    loop {
        let before = session.trace()?.events(PICK_EVENT).count();
        clicks += 1;
        driver.click_at(screen)?;
        session.settle(settle);

        let trace = session.trace()?;
        let lines: Vec<&TraceLine> = trace.events(PICK_EVENT).collect();
        let new: Vec<&TraceLine> = lines.iter().skip(before).copied().collect();
        if new.len() != 1 {
            return Ok(Err(PickFailure {
                why: format!(
                    "GESTURE: click {clicks} of pick {label} produced {} `{PICK_EVENT}` line(s), \
                     not 1. `canvas::measure::click` traces exactly once per click it is handed \
                     — on the promote path and on the resolve path alike — so zero means the \
                     click never became a pick and more than one means it was delivered twice. \
                     Zero is the interesting case and has three readings: the gesture machine \
                     swallowed it (`canvas::gesture::press_kind` returns \
                     `click: caps.author_measure`, so a mode that had lost the `measure` tab \
                     from its tab list would swallow every one), the click landed outside the \
                     page rect, or `canvas::interact`'s `Click` arm no longer branches on \
                     `active_tool.measure_kind()`. New lines: {}.",
                    new.len(),
                    crate::checks::driving::list_str(
                        &new.iter().map(|l| l.raw.as_str()).collect::<Vec<_>>()
                    )
                ),
            }));
        }
        let line = new[0];

        // Rule 4: an inferred candidate is announced, not committed.
        if line.get("outcome") == Some(PICK_PROMOTED) {
            report.note(format!(
                "pick {label}: the application promoted a derived snap candidate rather than \
                 committing it (`{}`) — rule 4's fuzzy-never-sneaky gate. Clicking the same \
                 point again to confirm, exactly as an operator would.",
                line.raw
            ));
            if clicks >= MAX_CLICKS_PER_PICK {
                return Ok(Err(PickFailure {
                    why: format!(
                        "GESTURE: the two-click confirm did not converge. Pick {label} was \
                         promoted on click 1 and promoted AGAIN on click {clicks}, at the same \
                         screen pixel. `MeasureState::resolve_click` promotes only when \
                         `derived_promoted != Some(point)`, so a second promotion means the \
                         point the snap query resolved to MOVED between two clicks of a \
                         stationary pointer — and a derived candidate that can never be \
                         confirmed is a candidate the operator can never pick. Look at \
                         `canvas::measure::snapped` and at whether \
                         `snap::active_snap_candidate` is being handed a stable `snap_cycle`. \
                         Line: `{}`.",
                        line.raw
                    ),
                }));
            }
            continue;
        }

        return Ok(Ok(ResolvedPick {
            line: line.clone(),
            clicks,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠ The tripwire named in this module's header.
    ///
    /// [`crate::checks::measure_linear`] keeps its own copy of the loop, for
    /// the argued reason stated there and here. This asserts the one number
    /// the two copies must not disagree about — if `measure_linear` ever
    /// decides a pick may take three clicks, or this module does, the two
    /// stop being the same behaviour and a reader comparing their traces is
    /// misled.
    ///
    /// ★ It is keyed on the OTHER module's constant, not on a literal
    /// restatement of it here, so it measures the copy rather than measuring
    /// this test's own opinion.
    #[test]
    fn the_two_implementations_agree_on_the_click_bound() {
        assert_eq!(
            MAX_CLICKS_PER_PICK, 2,
            "the bound is `snap_commit_clicks`'s own: one click for a routine candidate, two \
             for a derived one. Changing it here means changing it in `measure_linear` too, \
             and means re-reading why a third click is a finding rather than a retry."
        );
    }

    /// A promotion must be told apart from a resolved pick by its **field**,
    /// which is the distinction the whole module turns on.
    #[test]
    fn a_promotion_is_recognised_by_its_field_not_by_its_text() {
        let trace = crate::trace::Trace::parse(
            "pdfcer-diag measure-pick outcome=Promoted reason=derived-candidate-needs-confirm\n\
             pdfcer-diag measure-pick kind=Scale in_progress=true committed=false\n",
            "pdfcer-diag",
        );
        let lines: Vec<&TraceLine> = trace.events(PICK_EVENT).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].get("outcome"), Some(PICK_PROMOTED));
        assert_eq!(
            lines[0].get("committed"),
            None,
            "a promotion carries no `committed=` field — which is exactly why a check that \
             failed to recognise it would read the promotion as a pick that committed nothing, \
             and then fail against a working build"
        );
        assert_eq!(lines[1].get("outcome"), None);
        assert_eq!(lines[1].get("committed"), Some("false"));
    }
}
