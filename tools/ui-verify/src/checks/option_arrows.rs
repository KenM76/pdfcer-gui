//! `the_option_arrows_are_greyed_only_at_the_ends_of_the_list` — the Move-up
//! and Move-down buttons beside a drop-down's choices are dead at the ends of
//! the list and live everywhere else.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/option_arrows.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
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

    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("option-arrows.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with the seam selecting {FIELD}; only the scripted wheel is sent",
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
             admits a widget on the strength of its rectangle alone, so the miss is a fact about \
             the NAME or the fixture and not about the arrows: either the name does not match a \
             fully-qualified field in the file, or no widget of that field has a place on a \
             rendered page. It is NOT a statement about appearance streams — a widget with no \
             `/AP` is in this census and is selectable; it is only unfillable. {METHOD}",
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

    // --- 4: every row is reachable by scrolling the panel -----------------
    //
    // The rows sit below the panel's fold at every window height the monitor
    // allows, so the panel is wheeled through the scripted pointer, as the
    // operator scrolls it, until all three rows have been on screen.
    let Some(section) = declared(&trace, ui_rect, SECTION) else {
        unreachable!("step 2 returned when the section was not declared");
    };
    let aim = WindowPoint::centre_of(section);
    let mut seen = [false; 3];
    for notch in 0..=10 {
        let now = session.trace()?;
        for (row, was) in seen.iter_mut().enumerate() {
            *was |= declared(&now, ui_rect, &format!("{SECTION}.row{row}.up")).is_some();
        }
        if seen.iter().all(|s| *s) {
            report.note(format!(
                "all 3 option rows came on screen; {notch} wheel step(s) over the section"
            ));
            return Ok(None);
        }
        pointer.wheel(&session, aim, -2.0, false)?;
        session.settle(8);
    }
    let missed: Vec<String> = (0..3)
        .filter(|row| !seen[*row])
        .map(|row| format!("row{row}"))
        .collect();
    Ok(Some(format!(
        "★★★ OPTION ROWS NEVER REACHED: {} never declared a visible rectangle while the panel \
         was wheeled ten steps over `{SECTION}`, so the operator cannot scroll them into view. \
         A `ui-rect-clipped` line per row says by how much. Trace: {}.",
        list(&missed),
        session.trace_path().display()
    )))
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
