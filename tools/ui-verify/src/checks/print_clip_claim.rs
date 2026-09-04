//! `print_clip_claim_follows_the_preview` — the commit button's count must be
//! corrected by what the preview has already looked at.
//!
//! # What this is about
//!
//! Operator request O113. The preview's clip hatch became ink-aware: on a 1:1
//! CAD sheet whose overhang is empty paper nothing is hatched, and the caption
//! reads *"This sheet hangs over the printable area, but nothing is printed
//! there — the overhang is blank."*
//!
//! `Job::clipped()` stayed geometric, so the commit button went on reading
//! *"Print — 1 sheet will be clipped"* over a picture showing nothing lost.
//! Both sentences were true and they read as contradicting each other.
//!
//! The fix — `crates/pdfcer-gui/src/dialogs/print/verdicts.rs` — remembers the
//! blank/not-blank verdict per sheet as the preview renders it, and labels the
//! button with `geometric − known_blank`, with unexamined sheets still counted.
//!
//! # ★★★ Why no unit test in the workspace can observe this
//!
//! Two links, and neither is reachable from a test:
//!
//! 1. **The verdict is produced inside `paint`**, which needs an `egui::Ui`, a
//!    real device geometry from a driver, and a rasterised page. The
//!    arithmetic that consumes the verdict is pure and is proved in
//!    `dialogs::print::verdicts_tests`; the *recording* is not.
//! 2. **The cache key is only interesting when it is live.** A key that
//!    silently never matches produces the old geometric count — which is a
//!    correct answer to a different question, and on a job where nothing is
//!    blank it is also the *right* answer. So a cache that never works looks
//!    exactly like a cache that works, on every machine where the fixture does
//!    not clip.
//!
//! # What it measures
//!
//! Two trace lines the dialog emits every frame it is open:
//!
//! ```text
//! print-preview canvas=[…] … tex=1 overhang=blank-band claim=none:0
//! print-plan    printer="…" … clipped=Some(1) claim=none:0 …
//! ```
//!
//! `clipped=` is `Job::clipped()`, the geometric count, unchanged. `claim=` is
//! `<state>:<count>` — what the button actually says. `overhang=` is what the
//! ink test found in the band of the sheet on screen.
//!
//! Three assertions, in increasing strength:
//!
//! | # | condition | requirement |
//! |---|---|---|
//! | 1 | always | `claim` count ≤ `clipped` count — the correction may never *invent* a clip |
//! | 2 | `clipped=Some(0)` | `claim=none:0` — nothing clipped, nothing said |
//! | 3 | `overhang=blank-band` | `claim` state is **not** `geometric` — the verdict landed and moved the number |
//!
//! ★ Assertion 3 is the one the request is about, and it is stated as "not
//! geometric" rather than as "none" on purpose: a multi-sheet job in which one
//! blank sheet has been examined and four have not is correctly `at-most`, not
//! `none`. Requiring `none` would fail a correct build on any job longer than
//! one sheet.
//!
//! # ★ When it SKIPS, and why that is the honest verdict
//!
//! The scale mode defaults to **Fit**, which scales a page to the printable
//! area and therefore does not clip. The operator's case is 1:1, and the scale
//! radios publish **no `ui-rect` region**, so this harness cannot switch them:
//! it publishes rects for the paper combo, the Properties button and the
//! splitter, and for nothing else in the dialog.
//!
//! So on most machines the fixture will not clip at all, `overhang=fits`,
//! assertion 3 is vacuous, and the check reports SKIPPED with the values it
//! read. It has learned that the two lines exist and that assertions 1 and 2
//! hold; it has not exercised the correction, and saying so is the only honest
//! verdict. The same three-state discipline `print_dialog` applies to a
//! machine with no printers.
//!
//! **What would make it bite every time**: a `ui-rect` region on the scale
//! radios, so the harness could choose Actual size and force the 1:1 geometry
//! this request is about. That is a change to `dialogs::print::tabs` and it is
//! the right next step for this check — recorded here rather than done,
//! because it widens the published-region surface and belongs in its own
//! commit.
//!
//! # What it deliberately does NOT do
//!
//! **It never presses the commit button**, and no future edit may make it do
//! so. Same rule and same reason as `print_dialog` and `print_paper`: that
//! button is the one control in the application that consumes paper and cannot
//! be undone, and a harness that can start a print job will eventually start
//! one by accident.

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The ribbon control that opens the dialog, and the tab it lives on.
const SUBJECT: &str = "ribbon.item.file.print";
const TAB_ID: &str = "file";
const TAB: &str = "ribbon.tab.file";

/// The trace event the dialog emits once, when it is built.
const OPEN_EVENT: &str = "print-open";

/// The per-frame line carrying `clipped=` and `claim=`.
const PLAN_EVENT: &str = "print-plan";

/// The per-frame line carrying `overhang=` and `claim=`.
const PREVIEW_EVENT: &str = "print-preview";

/// What a verdict-free claim looks like: the pre-O113 behaviour.
const GEOMETRIC: &str = "geometric";

/// The verdict that says the sheet on screen overhangs onto empty paper.
const BLANK_BAND: &str = "blank-band";

pub struct PrintClipClaimFollowsThePreview;

impl Check for PrintClipClaimFollowsThePreview {
    fn name(&self) -> &'static str {
        "print_clip_claim_follows_the_preview"
    }

    fn defect(&self) -> &'static str {
        "the commit button announces a clip over a preview that shows nothing lost, because the count is geometric and the hatch is not"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Split a `claim=<state>:<count>` field into its two halves.
///
/// The state is what a capture cannot supply: `geometric:2` and `at-most:2`
/// are the same number and a different sentence on the button.
fn split_claim(field: &str) -> Option<(&str, usize)> {
    let (state, count) = field.split_once(':')?;
    Some((state, count.parse().ok()?))
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. `file.print` is gated on `doc.open`, so with nothing open the control is \
             greyed and there is no dialog to reach. A large-format 1:1 sheet such as \
             fixtures/a1-titleblock.pdf is the fixture this check is about.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is two clicks. Reported as SKIPPED \
             rather than passed — a check that did not run has learned nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("print_clip_claim.trace.txt"));
    spec.pdf = Some(pdf);
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
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so {}={} did not reach the process.",
            ctx.profile.vocab.start_event, ctx.profile.diag_env.0, ctx.profile.diag_env.1,
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- open the dialog ----------------------------------------------------
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so no click \
             reached the ribbon."
        )));
    }
    let Some(control) =
        crate::checks::driving::declared_or_in_overflow(&session, &driver, ui_rect, SUBJECT)?
    else {
        let trace = session.trace()?;
        return Err(Error::new(format!(
            "the File tab is active and neither it nor its overflow declares `{SUBJECT}`. \
             Controls declared: {}. That is `print_dialog`'s defect, not this one.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    driver.click_at(session.frame()?.declared_center(control))?;
    // Enumerating printers touches the spooler, which BLOCKS on a network
    // printer. The same settle `print_dialog` and `print_paper` use, for the
    // same reason.
    session.settle(40);

    let trace = session.trace()?;
    let Some(open) = trace.events(OPEN_EVENT).next() else {
        return Err(Error::new(format!(
            "the click on `{SUBJECT}` produced no `{OPEN_EVENT}` line, so the dialog never \
             opened. That is `print_dialog`'s subject; nothing about the clip claim can be \
             learned here."
        )));
    };
    if open.get("unavailable").unwrap_or("<absent>") != "None" {
        return Err(Error::new(
            "the spooler refused on this machine, so there is no device geometry and therefore \
             no clip to claim anything about. Reported as SKIPPED.",
        ));
    }

    // --- the two lines ------------------------------------------------------
    let Some(plan) = trace.last(PLAN_EVENT) else {
        return Err(Error::new(format!(
            "the dialog is open and emitted no `{PLAN_EVENT}` line."
        )));
    };
    let Some(preview) = trace.last(PREVIEW_EVENT) else {
        return Err(Error::new(format!(
            "the dialog is open and emitted no `{PREVIEW_EVENT}` line, so the preview column \
             drew nothing. `print_layout` is where that is diagnosed."
        )));
    };

    let clipped_field = plan.get("clipped").unwrap_or("<absent>").to_owned();
    let claim_field = plan.get("claim").unwrap_or("<absent>").to_owned();
    let overhang = preview.get("overhang").unwrap_or("<absent>").to_owned();
    report.note(format!(
        "clipped={clipped_field} claim={claim_field} overhang={overhang}"
    ));

    let Some((state, count)) = split_claim(&claim_field) else {
        return Ok(Some(format!(
            "`{PLAN_EVENT}` carries claim={claim_field}, which is not the `<state>:<count>` \
             shape this check reads. Either the field was renamed or the claim was dropped from \
             the trace, and with it the only headless evidence of which sentence the button is \
             showing."
        )));
    };

    // `clipped=Some(2)` / `clipped=None`. `None` means there is no job, which
    // the preview column already reports in its own words.
    let geometric: usize = clipped_field
        .strip_prefix("Some(")
        .and_then(|rest| rest.strip_suffix(')'))
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);

    // --- assertion 1: the correction may never invent a clip ----------------
    if count > geometric {
        return Ok(Some(format!(
            "the button claims {count} sheets against a geometric count of {geometric}. The \
             correction is only ever allowed to SUBTRACT sheets the preview has examined and \
             found blank; a number above the geometric count is a clip nothing measured."
        )));
    }

    // --- assertion 2: nothing clipped, nothing said -------------------------
    if geometric == 0 && count != 0 {
        return Ok(Some(format!(
            "no sheet's page box exceeds the printable area (clipped={clipped_field}) and the \
             button still claims {count} ({claim_field})."
        )));
    }

    // --- assertion 3: the verdict landed ------------------------------------
    if overhang == BLANK_BAND {
        if state == GEOMETRIC {
            return Ok(Some(format!(
                "the preview found the overhang of the sheet on screen to be blank paper \
                 (overhang={BLANK_BAND}) and the button is still making the uncorrected \
                 geometric claim ({claim_field}). That is O113's exact defect: the caption says \
                 nothing is printed out there and the button says a sheet will be clipped. \
                 Either the verdict is not being recorded in `preview::paint`, or its cache key \
                 never matches — the two look identical from outside, and both produce this."
            )));
        }
        report.note(format!(
            "the blank-band verdict reached the claim: overhang={BLANK_BAND} claim={claim_field} \
             against a geometric count of {geometric}"
        ));
        return Ok(None);
    }

    Err(Error::new(format!(
        "the sheet on screen reports overhang={overhang}, so the correction this check exists \
         to verify was never exercised. That is the expected result on most machines: the scale \
         mode defaults to Fit, which does not clip, and the scale radios publish no `ui-rect` \
         region for this harness to switch them with. Assertions 1 and 2 held \
         (clipped={clipped_field} claim={claim_field}). Reported as SKIPPED, because a check \
         that did not exercise its subject has learned nothing about it — see this module's \
         header for the one change that would make it bite every run."
    )))
}
