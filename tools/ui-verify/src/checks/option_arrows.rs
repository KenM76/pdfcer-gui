//! `the_option_arrows_are_greyed_only_at_the_ends_of_the_list` — the Move-up
//! and Move-down buttons beside a drop-down's choices are dead at the ends of
//! the list and live everywhere else.
//!
//! # The defect this is written against
//!
//! An `/Opt` editor can draw all three rows' arrows live and then filter the
//! press — `if clicked() && n > 0`. The top row's Move-up button then looks
//! pressable, reports a click, and has its result discarded. The operator
//! presses it and the list does not move.
//!
//! # Why a driven run, and why the trace had to gain a field first
//!
//! **A correctly disabled control and a live one whose result is thrown away
//! are the same observation.** A harness that presses the button and then reads
//! the list sees *"nothing moved"* in both cases, so pressing it proves nothing
//! at all — which is why this check presses nothing.
//!
//! What separates them is `egui::Response::enabled()`, whose value is decided
//! by whether the widget was allocated inside a disabled `Ui`. A widget merely
//! painted grey, or given a weaker `Sense`, reports itself **enabled**. So
//! `enabled=false` is evidence about the mechanism rather than about the
//! appearance, and this check exists because `pdfcer_gui::diag::ui_control` now
//! publishes it.
//!
//! # What a blanket disable cannot produce
//!
//! Six assertions across three rows, not two:
//!
//! | row | up | down |
//! |---|---|---|
//! | 0 | **disabled** | enabled |
//! | 1 | enabled | enabled |
//! | 2 | enabled | **disabled** |
//!
//! Asserting only the two disabled ones would pass on a build that disabled
//! every arrow — a shipped section where nothing reorders, which is worse than
//! the defect it replaced. The four `enabled` rows are the half that names what
//! the wrong mechanism cannot do.
//!
//! Three rows is also the fewest that can distinguish *"the ends"* from *"the
//! first and the last are special-cased by index"*: with two rows every arrow
//! is at an end.
//!
//! # No pointer, so this runs beside somebody working
//!
//! The document arrives on argv, Edit mode and the Properties panel arrive
//! through `PDFCER_DIAG_INVOKE`, and the field selection arrives through
//! `PDFCER_DIAG_SELECT_FIELD`. That last seam exists because
//! `doc.selected_field` — the Properties pane's only input — had no writer but
//! a click, which made every properties surface in the shell unreachable by R1
//! on any day the operator was at his machine.
//!
//! # Why row 2's own rectangle is not asserted
//!
//! Because it cannot be brought into view by asking for a taller window.
//! `PDFCER_DIAG_VIEWPORT`'s height is clamped to the monitor with no error and
//! no report, so 2000, 2200 and 2600 all produced the same window and the same
//! clip rect on this machine — measured, not assumed. The third row sits about
//! fifteen logical points below the panel's scroll viewport at every one of
//! them, and a check asserting its rect would fail identically on a correct
//! build and on a genuinely misplaced control.
//!
//! What is asserted instead is that the **section** is declared and that at
//! least one row's arrows are, which separates *"the rows draw and the operator
//! scrolls to the third"* from *"this section shipped unreachable"* — the
//! defect class `D:/dev/rag/egui/` records the shell shipping before, with
//! every gate green.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The event `pdfcer_gui::diag::ui_control` writes.
const CONTROL: &str = "ui-control";
/// The `/Opt` section's own region, and the prefix its controls share.
const SECTION: &str = "properties.choice_opts";
/// Nine fields, ten widgets, every widget carrying an appearance stream.
const FIXTURE: &str = "all-field-kinds.pdf";
/// Why no other document substitutes for it.
const METHOD: &str = "The check needs a choice field carrying EXACTLY three plain /Opt strings: \
                      three rows are the fewest that can tell `n > 0 && n < last` from a special \
                      case on the first and last index, and a fourth would add no assertion. \
                      `ComboOne` in this fixture carries three by construction — see \
                      `fixtures/all-field-kinds.PROVENANCE.py`.";
/// The field the seam selects.
const FIELD: &str = "ComboOne";
/// Edit mode, because selection is the Edit reading of a click; then the panel.
const INVOKE: &str = "mode.edit,file.properties";
/// Off-screen, so no focus is taken, and under the monitor's work area.
///
/// Asking for more height is silently ignored: see the module header.
const VIEWPORT: &str = "-4000,-4000,1200,1350";

/// What every arrow must report, as `(region suffix, enabled)`.
///
/// The table in the module header, in the order a reader checks it.
const ARROWS: [(&str, bool); 6] = [
    ("row0.up", false),
    ("row0.down", true),
    ("row1.up", true),
    ("row1.down", true),
    ("row2.up", true),
    ("row2.down", false),
];

/// See the module documentation.
pub struct TheOptionArrowsAreGreyedOnlyAtTheEndsOfTheList;

impl Check for TheOptionArrowsAreGreyedOnlyAtTheEndsOfTheList {
    fn name(&self) -> &'static str {
        "the_option_arrows_are_greyed_only_at_the_ends_of_the_list"
    }

    fn defect(&self) -> &'static str {
        "the Move-up button on the first choice in a drop-down's list, and the Move-down button \
         on the last, take a press and do nothing with it — so the operator cannot tell a \
         control that declines from one that is broken"
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
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("option-arrows.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_SELECT_FIELD".to_owned(), FIELD.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with the seam selecting {FIELD} — no input is sent",
        exe.display(),
        session.pid()
    ));
    // Three frames' worth of dependency: the mode, then the panel the mode
    // permits, then the census the seam resolves a name against.
    session.settle(60);
    let trace = session.trace()?;

    // --- 1: the seam found the field ---------------------------------------
    //
    // Asked first because every assertion below is vacuous without it, and
    // because the seam discloses its own miss: a misspelt name and a broken
    // selection mechanism produce the same empty panel and have opposite fixes.
    let Some(seam) = trace.last("form-field-seam") else {
        return Err(Error::new(format!(
            "the shell traced no `form-field-seam` line, so `PDFCER_DIAG_SELECT_FIELD` was never \
             read. Either this build predates the seam, or the run never reached Edit mode — the \
             seam is asked only on the frames `canvas::forms` treats as authoring. Reported as \
             SKIP rather than FAIL: nothing about the arrows has been measured. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("seam: `{}`", seam.raw));
    if seam.get("found") != Some("true") {
        return Err(Error::new(format!(
            "the seam found no field named `{FIELD}` in the placement census: `{}`. The census \
             excludes any widget with no appearance stream, so this is a fact about the fixture \
             rather than about the arrows. {METHOD}",
            seam.raw
        )));
    }

    // --- 2: the section is on screen at all --------------------------------
    if declared(&trace, ui_rect, SECTION).is_none() {
        return Ok(Some(format!(
            "★★ THE /Opt SECTION IS NOT DRAWN. The field was selected and the Properties panel \
             declared no `{SECTION}` region. Regions under that prefix: {}. An empty list means \
             `properties::fieldedit::section`'s choice branch never called \
             `choiceopts::section`; the branch is keyed on the field's subtype, so check that \
             first. Trace: {}.",
            list(&declared_names(&trace, ui_rect, SECTION)),
            session.trace_path().display()
        )));
    }

    // --- 3: the instrument published, and what it published ----------------
    //
    // A missing `ui-control` line is its own failure rather than a default,
    // because absence is exactly what a call site that publishes the rect and
    // forgets the state produces — and that omission looks identical to a
    // control with only one state.
    let mut wrong = Vec::new();
    let mut missing = Vec::new();
    for (suffix, want) in ARROWS {
        let name = format!("{SECTION}.{suffix}");
        match state_of(&trace, &name) {
            None => missing.push(name),
            Some(got) if got != want => {
                wrong.push(format!("{suffix}: enabled={got}, expected enabled={want}"));
            }
            Some(_) => {}
        }
    }
    if !missing.is_empty() {
        return Ok(Some(format!(
            "★★ NO `{CONTROL}` LINE for {}. The region's rectangle may still be published — the \
             two are separate lines — and that is the half-published pair `diag::ui_control` \
             takes a `&Response` to prevent: a check reading only a rect cannot judge whether \
             the control would respond. Convert the call site. Control lines seen: {}. Trace: {}.",
            list(&missing),
            list(&control_names(&trace, SECTION)),
            session.trace_path().display()
        )));
    }
    if !wrong.is_empty() {
        return Ok(Some(format!(
            "★★★ THE ARROWS ARE GREYED IN THE WRONG PLACES: {}.\nThe rule is `!sorted && n > 0` \
             for up and `!sorted && n < last` for down, applied by allocating the button inside \
             `ui.add_enabled_ui(..)` — NOT by filtering the press afterwards, which is the defect \
             this check was written against. If EVERY arrow reports `enabled=false`, the \
             Keep-sorted flag is set on `{FIELD}` and the whole list is locked, which this \
             fixture's field does not carry. Trace: {}.",
            list(&wrong),
            session.trace_path().display()
        )));
    }
    report.note("all six arrows report the state their position calls for");

    // --- 4: the rows are reachable, not merely traced ----------------------
    //
    // See the module header: row 2 is below the panel's scroll viewport at
    // every window height this machine can produce, so the assertion is that
    // SOME row's arrows are on screen rather than that all of them are.
    let reachable = ["row0", "row1", "row2"]
        .into_iter()
        .filter(|row| declared(&trace, ui_rect, &format!("{SECTION}.{row}.up")).is_some())
        .count();
    if reachable == 0 {
        return Ok(Some(format!(
            "★★★ THE SECTION SHIPPED UNREACHABLE: all three rows traced their enabled state and \
             NONE declared a visible rectangle, so every row is clipped out of the panel's \
             scroll viewport. A `ui-rect-clipped` line per row says by how much. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "{reachable} of 3 option rows are inside the panel's scroll viewport at {VIEWPORT}"
    ));

    Ok(None)
}

/// The last `enabled=` a named control reported, or `None` if it reported none.
fn state_of(trace: &Trace, name: &str) -> Option<bool> {
    trace
        .events(CONTROL)
        .filter(|line| line.get("name") == Some(name))
        .last()
        .and_then(|line| match line.get("enabled") {
            Some("true") => Some(true),
            Some("false") => Some(false),
            _ => None,
        })
}

/// Every control name seen under `prefix`, for a failure message.
fn control_names(trace: &Trace, prefix: &str) -> Vec<String> {
    let mut names: Vec<String> = trace
        .events(CONTROL)
        .filter_map(|line| line.get("name"))
        .filter(|name| name.starts_with(prefix))
        .map(str::to_owned)
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}
