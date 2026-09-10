//! `match_the_pages_picks_the_sheet_from_the_document` — **operator request
//! O167, driven.**
//!
//! # The report
//!
//! Ken, 2026-09-10, in the same sentence as O166:
//!
//! > *"we also need the option to auto select paper size based on the page
//! > sizes in the pdf."*
//!
//! The Paper control's second entry — **Match the pages in this document** —
//! is that option. Choosing it measures every page of the job at its rotated
//! extent, picks the smallest enumerated sheet that holds them all with each
//! page free to lie either way round, and asks the driver for that sheet.
//! When nothing holds them it takes the largest available and says so.
//!
//! # ★★★ The claim this check exists for, and why the obvious one is not it
//!
//! Four things could be asserted after clicking that entry, and three of them
//! are worth almost nothing on their own:
//!
//! | Evidence | What it actually proves |
//! |---|---|
//! | `pick=auto` | the click landed on the entry |
//! | `auto=matched` | the decision ran |
//! | `paper=Form(8)` | its answer was turned into a request the driver will see |
//! | ★ `largest=` fits `sheet=` | **the sheet has something to do with this document** |
//!
//! A build that resolved auto to the first form in the driver's list would
//! emit the first three, correctly, and be completely wrong. The operator's
//! words were *"based on the page sizes in the pdf"*, and only the fourth row
//! is about the pages. That is why `largest=` was added to `print-plan` on
//! 2026-09-10 — while writing this check, which is the fifth time in this
//! project that sitting down to write a driven check found a trace that could
//! not tell apart the two states the check existed for.
//!
//! So the assertion is the **invariant**, not the value:
//!
//! > `auto=matched` means the largest page fits the planned sheet either way
//! > round. `auto=toobig` means it does not.
//!
//! That holds on any machine, against any driver's paper list, with any
//! fixture. Nothing here hard-codes A3, and nothing here needs to know what
//! printer this PC has — which is the difference between a check that runs on
//! the operator's machine and one that runs on mine.
//!
//! ## ★★★ The band is the application's DECLARED tolerance — and the first
//! run is what taught this check that
//!
//! This section is written out of a correction, because the corrected version
//! is the one worth reading and the reasoning that produced the wrong one is
//! the reason somebody would write it again.
//!
//! **What was assumed.** `largest=` and `sheet=` do not come from the same
//! place: the fit decision is made against what the driver said when it
//! *enumerated* its forms, and `sheet=` is what `printer_caps_for` returns when
//! the job is planned *for* the chosen form — a second call into a second
//! device context. So the first version of this check treated any small
//! disagreement as driver noise and reported **SKIP**: "I could not tell".
//!
//! **What the first driven run measured.** On the benchmark CAD drawing:
//!
//! ```text
//! before: pick=device auto=off    paper=DeviceDefault sheet=792.00x612.00   largest=none
//! after:  pick=auto   auto=matched paper=Form(8)      sheet=1190.40x841.68 largest=1191.00x842.00
//! ```
//!
//! The page is **0.60 pt bigger than the sheet** and the application said
//! `matched`. Under the assumed model that was inconclusive noise, and the run
//! reported SKIP. It is nothing of the sort. `autopaper::fits` is
//! `pw <= sw + FIT_TOLERANCE_PT` with `FIT_TOLERANCE_PT = 2.0`, and the
//! application overhangs by design: CAD producers write A3 as a clean
//! 1191×842 pt and the driver calls the same sheet 1190.4×841.68. **A page
//! 0.6 pt over an A3 form is a correct match by the rule the application
//! states about itself**, and a harness that rendered that as "I could not
//! tell" was hiding the application's declared behaviour behind its own
//! uncertainty.
//!
//! **What it does now.** The comparison is asymmetric, because the two
//! outcomes make opposite claims and only one of them tolerates an overhang:
//!
//! | Outcome | The claim | This check fails it when |
//! |---|---|---|
//! | `matched` | *"this sheet holds the pages"* | the page is over by **more than [`OVERHANG_PT`]** |
//! | `toobig` | *"nothing here holds them"* | the page has **more than [`OVERHANG_PT`] to spare** |
//!
//! [`OVERHANG_PT`] is the application's 2 pt tolerance plus 4 pt for the
//! two-different-calls effect that was assumed to be the whole story. It is
//! **6 pt, about 2 mm** — two orders of magnitude below the distance between
//! adjacent sheets (A4 to A3 is 246 pt on the short edge), so the thing this
//! check exists to catch cannot hide inside it. The build that resolves auto
//! to the front of the driver's list would have reported `sheet=792.00x612.00`
//! against `largest=1191.00x842.00`: **399 pt out**, not 0.6.
//!
//! ⚠ **There is no SKIP band any more, and that is deliberate.** A SKIP that
//! fires on the ordinary case is worse than no check: it is a green-adjacent
//! verdict nobody investigates, produced every single run. The one thing this
//! check is genuinely unable to decide has its own SKIP with its own sentence
//! (`auto=nobasis`), and it is a state the *application* declared, not one the
//! harness inferred.
//!
//! # ★★★ Falsified, not merely green (2026-09-10)
//!
//! *"A check that cannot fail is not evidence"* is a standing lesson in this
//! project, and this check was made to fail on purpose before it was believed.
//!
//! **What was planted.** The exact defect the module header argues about:
//! `autopaper::choose`'s matched arm was switched from
//! *"the smallest enumerated form that holds every page"* to
//! `forms.first()` — the front of the driver's list, chosen without looking at
//! the document. Two lines. Everything else left alone: the pages were still
//! measured, `largest_page_pt` still filled from them, `mixed` still computed,
//! the resolution to a `PaperChoice::Form` untouched.
//!
//! **What came back**, and this is the whole argument for `largest=` existing:
//!
//! ```text
//! after choosing auto: pick=auto auto=matched paper=Form(9)
//!                      sheet=841.68x595.20 largest=1191.00x842.00
//! ```
//!
//! **All three of the obvious assertions held.** The operator chose the
//! policy (`pick=auto`). The decision ran (`auto=matched`). Its answer reached
//! the driver (`paper=Form(9)`). A check built on those three would have
//! reported PASS on a build that put an A3 site plan on an A4 sheet. The one
//! that caught it was the fourth: `1191.00x842.00` does not lie on
//! `841.68x595.20`, **349.32 pt out**, and the check said so in those words and
//! named `autopaper::choose` as one of the two places to look. It was the right
//! answer.
//!
//! The file was then restored from a kept copy — never `git checkout`, which
//! discards uncommitted work in the same file — verified by checksum, and the
//! check re-run green.
//!
//! ⚠ If a future edit to `autopaper::choose` or to the paper combo makes this
//! check awkward, **re-run that falsification rather than trusting the green.**
//! It costs two release builds and about four minutes, and it is the only thing
//! that has ever demonstrated that this check discriminates the state it names
//! from the three states that look like it.
//!
//! # ★ What this check deliberately CANNOT establish
//!
//! **It never presses Print.** Four print checks state that rule in their own
//! words rather than by reference, because the day somebody adds a sixth by
//! copying one of these files, the copied file is what they will read: this
//! suite runs unattended on the machine whose default printer is the
//! operator's plotter, and a harness that can start a print job will
//! eventually start one by accident.
//!
//! **It cannot read the disclosure sentence.** Under R8b rule 4 an inference
//! is reported off-canvas and never drawn onto the page, and auto selection is
//! an inference — so a sentence under the combo names the sheet chosen *and*
//! the largest page measured, and a second sentence names the tray flag when
//! the job is mixed. Those are `ui.label`s with no published rect; the harness
//! reads rectangles and trace lines, not text. What stands in for them:
//! `paper_auto_matched`, `paper_auto_too_big`, `paper_auto_no_basis` and
//! `paper_auto_mixed` are `ui_text` entries under the string gate, and
//! `auto_paper_line` is unit-tested against each outcome. That is a weaker
//! claim than driving it and is written down as one.
//!
//! **Its `toobig` arm is weaker than its `matched` arm, and knowingly so.**
//! `matched` names a specific sheet and this check can say whether that sheet
//! holds the document. `toobig` claims something about *every* sheet the
//! driver has — that none of them does — and the driver's form list is not on
//! the trace, so the strongest thing assertable from outside is that the sheet
//! reported really is too small. A build that took the FIRST form rather than
//! the largest would satisfy that whenever the first form is also too small.
//! Closing it would mean putting the whole enumerated list on the trace, which
//! is a diagnostic line per form on every frame; the arithmetic is asserted in
//! `autopaper::choose`'s unit tests instead. On this machine and fixture the
//! outcome is `matched`, so the strong arm is the one that runs — but that is
//! a property of the fixture, not of the check.
//!
//! **It does not assert that the sheet chosen is the SMALLEST that fits.**
//! `largest=` and `sheet=` are two of the driver's sheets; the list of all the
//! others is not on the trace, so "no smaller sheet would also have held it"
//! is not answerable from here. It is the property `autopaper::choose`'s unit
//! tests assert directly, against a fixture list, and this project's standing
//! division of labour is that arithmetic is asserted where it lives and the
//! chain in front of it is asserted by driving the binary.
//!
//! # Every way this reports SKIP
//!
//! No binary; `--no-input`; no `--pdf` (Print is gated on a document being
//! open); no start line; the ribbon control not declared; the dialog not
//! opening; the spooler refusing; the device enumerating no forms (the combo
//! is not drawn at all then, deliberately); the popup not opening or closing
//! within a settle; the auto entry not declared; no `print-plan` line; the
//! click not moving `pick=`; `auto=nobasis`; and the slack-band case above.
//!
//! Each of those is a state in which nothing was learned about the
//! application, and each says so in its own sentence rather than through a
//! shared one — because a reader of a SKIP wants to know what to do next, and
//! "something went wrong" is not that.

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_in, declared_names, frame_for, list,
    shell_trace,
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

/// The per-frame line carrying `pick=`, `auto=`, `paper=`, `sheet=`, `largest=`.
const PLAN_EVENT: &str = "print-plan";

/// The line reporting what the selected device said about itself.
const FEATURES_EVENT: &str = "print-features";

/// The paper combo, closed.
const PAPER: &str = "print.paper";

/// ★ The auto entry's own region, **outside** the numbered
/// `print.paper.item.N` namespace.
///
/// That is not a naming preference, it is the reason `print_paper_changes_the_plan`
/// still measures what it says it measures. Auto sits second on screen, so the
/// obvious thing would have been to give it index 1 and push the driver's forms
/// up by one — which would have left that check clicking a policy while its own
/// report said it had clicked a sheet, still green. `REGION_PAPER_AUTO`'s doc in
/// the application carries the full argument.
const PAPER_AUTO: &str = "print.paper.auto";

/// The first entry, "from the printer's own settings" — clicked at the end to
/// prove auto is a policy the operator can leave, not a latch.
const PAPER_DEVICE: &str = "print.paper.item.0";

/// How far a page may hang over its sheet, in points, before the application's
/// verdict about it is wrong rather than merely generous.
///
/// **2 pt of it is the application's own `FIT_TOLERANCE_PT`**, mirrored here
/// deliberately: `autopaper::fits` is `pw <= sw + FIT_TOLERANCE_PT`, so a page
/// up to 2 pt bigger than a form is a *correct* `matched` and a check that
/// called it wrong would be failing the application for doing what it says it
/// does. CAD producers write A3 as a clean 1191×842 pt; the driver calls the
/// same sheet 1190.4×841.68. That 0.6 pt is the whole reason the tolerance
/// exists, and the first driven run of this check measured exactly it.
///
/// **4 pt of it is the two-different-calls allowance** — the fit decision is
/// made against the driver's *enumerated* form size and `sheet=` comes back
/// from `printer_caps_for`'s device context, and a driver may round the two
/// differently.
///
/// ⚠ **It is a ceiling on wrongness, not a tolerance on the comparison.** It
/// is never used to abandon a verdict; see the module header for why the
/// original SKIP band was removed. 6 pt is about 2 mm, two orders of magnitude
/// below the 246 pt that separates A4 from A3 — the sheet-from-the-front-of-
/// the-list defect this check exists for misses by hundreds of points.
///
/// ⚠ **If the application's `FIT_TOLERANCE_PT` grows, this must grow with
/// it**, or a correct `matched` starts failing here. The two cannot be shared:
/// this is a separate crate and that constant is private. The dependency is
/// therefore stated, and [`the_band_is_at_least_the_applications_own_tolerance`]
/// pins the direction so a shrink here is caught by the compiler's own test
/// run rather than by a confusing red on the operator's machine.
const OVERHANG_PT: f64 = 6.0;

/// The application's `FIT_TOLERANCE_PT`, restated so the relationship above can
/// be asserted rather than only described.
///
/// ⚠ Mirrored by hand from `crates/pdfcer-gui/src/dialogs/print/autopaper.rs`.
/// It is not imported because `ui-verify` does not depend on the application's
/// crate — it drives the built binary, which is the entire point of it. A
/// mirrored constant is a claim about someone else's source, and this project's
/// standing lesson about those is that they get a test which fails in the
/// direction that matters.
const APP_FIT_TOLERANCE_PT: f64 = 2.0;

/// **The two bounds on [`OVERHANG_PT`], asserted at COMPILE TIME.**
///
/// These were a `#[test]` until clippy pointed out that a comparison between
/// two constants is decided by the compiler and a test of it can never fail at
/// run time. It is right, and the right answer is not to silence it: a
/// relationship that is knowable at compile time should **break the build**,
/// not produce a red test somebody has to run first. So they are `const _`
/// items, and a violation is a compile error naming this file.
///
/// Both directions matter, which is why both are here:
///
/// - **Too small** and a *correct* `matched` fails this check — the
///   application permits a page to overhang its form by `FIT_TOLERANCE_PT`,
///   so any value below that makes the check red on a build doing exactly what
///   it documents. That is the failure the first driven run walked into from
///   the other side, and it cost a rewrite of the oracle.
/// - **Too large** and a sheet that is a whole size wrong is waved through,
///   which is the only defect this check exists for. The bound is stated
///   against the closest adjacent pair in ordinary use, A4 to A3 on the short
///   edge — 246 pt.
///
/// ⚠ The lower bound is a claim about **another crate's private constant**,
/// mirrored by hand into [`APP_FIT_TOLERANCE_PT`]. If that constant moves and
/// this one is not moved with it, this assertion still holds — it pins the
/// relationship, not the value. The tripwire for the value is the driven run
/// itself: a `matched` failing by a point or two is what a widened application
/// tolerance looks like from here.
const _: () = assert!(
    OVERHANG_PT >= APP_FIT_TOLERANCE_PT,
    "OVERHANG_PT is below the tolerance the application declares; a correct match would fail"
);
const _: () = assert!(
    OVERHANG_PT * 10.0 < 841.89 - 595.28,
    "OVERHANG_PT is not an order of magnitude below the A4-to-A3 step; a wrong sheet could pass"
);

pub struct MatchThePagesPicksTheSheetFromTheDocument;

impl Check for MatchThePagesPicksTheSheetFromTheDocument {
    fn name(&self) -> &'static str {
        "match_the_pages_picks_the_sheet_from_the_document"
    }

    fn defect(&self) -> &'static str {
        "choosing Match the pages selects a sheet that has nothing to do with the document's page sizes"
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

/// One `print-plan` line, reduced to the five fields this check reasons about.
#[derive(Debug, Clone)]
struct Plan {
    /// What the operator chose: `device`, `form` or `auto`.
    pick: String,
    /// How the auto decision came out: `off`, `nobasis`, `matched`, `toobig`.
    auto: String,
    /// The RESOLVED request — `DeviceDefault` or `Form(n)`, Debug-formatted.
    paper: String,
    /// The physical sheet the plan was built against, as `WxH` or `none`.
    sheet: String,
    /// The largest page the auto decision measured, as `WxH` or `none`.
    largest: String,
}

#[allow(clippy::too_many_lines)]
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
             greyed and there is no dialog to reach.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is four clicks. Reported as SKIPPED \
             rather than passed — a check that did not run has learned nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("print_auto_paper.trace.txt"));
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
            "the trace has no `{}` line, so {}={} did not reach the process. Captured stderr is \
             at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- A. open the dialog -------------------------------------------------
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

    // Through the overflow when the ribbon has folded it there — at the
    // harness's window width the File tab correctly folds Print away, and a
    // lookup that read only the tab surface once stood as a FAIL for days.
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
    // printer, and this dialog also enumerates the device's paper forms.
    session.settle(40);

    let trace = session.trace()?;
    let Some(open) = trace.events(OPEN_EVENT).next() else {
        return Err(Error::new(format!(
            "the click on `{SUBJECT}` produced no `{OPEN_EVENT}` line, so the dialog never \
             opened. That is `print_dialog`'s subject; nothing about paper can be learned here."
        )));
    };
    if open.get("unavailable").unwrap_or("<absent>") != "None" {
        return Err(Error::new(
            "the spooler refused on this machine, so there is no device to enumerate paper for. \
             Reported as SKIPPED: a refused enumeration proves nothing either way.",
        ));
    }

    // --- B. the device has something to choose from -------------------------
    //
    // ★ ONE form is enough here, and that is the difference from
    // `print_paper_changes_the_plan`, which needs two because it must switch
    // BETWEEN sheets. Auto has to decide even when the list has one entry —
    // the answer is then either "that one" or "that one, and it is too small"
    // — and both are decisions worth asserting.
    let features = trace.events(FEATURES_EVENT).last().ok_or_else(|| {
        Error::new(format!(
            "the dialog opened and emitted no `{FEATURES_EVENT}` line, so the per-device read \
             that fills the paper list did not run."
        ))
    })?;
    let forms: usize = features
        .get("forms")
        .and_then(|n| n.parse().ok())
        .unwrap_or_default();
    report.note(format!(
        "the selected device enumerated {forms} paper form(s); form_source={}",
        features.get("form_source").unwrap_or("<absent>")
    ));
    if forms == 0 {
        return Err(Error::new(
            "the selected device enumerated no paper forms, so the combo is not drawn at all and \
             there is no auto entry to click. Reported as SKIPPED: that is the shell answering a \
             legal driver answer correctly, and it is asserted by the string gate rather than \
             here.",
        ));
    }

    // --- C. the plan before anything is touched -----------------------------
    let before = plan(&session)?;
    report.note(format!(
        "before: pick={} auto={} paper={} sheet={} largest={}",
        before.pick, before.auto, before.paper, before.sheet, before.largest
    ));
    if before.auto != "off" {
        return Ok(Some(format!(
            "★ the dialog opened with `auto={}` before anything was clicked. Auto selection is \
             the operator's choice and the dialog is supposed to open on `off` unless it was \
             remembered — in which case `pick=` would read `auto`, and it reads `{}`. A \
             decision that runs when nobody asked for it will also run on a document nobody \
             looked at.",
            before.auto, before.pick
        )));
    }

    // --- D. choose it -------------------------------------------------------
    //
    // ★★ FROM HERE THE REGIONS ARE IN THE DIALOG'S OWN OS WINDOW. Its
    // `ui-rect` rectangles are relative to ITS client area, so `session.frame()`
    // is the wrong origin for every one of them and produces coordinates that
    // look entirely reasonable and land several hundred points away. The frame
    // is re-resolved per click rather than hoisted, because the operator can
    // move the window and so can this harness's own clicks nudge it.
    let trace = session.trace()?;
    let (paper, paper_vp) = declared_in(&trace, ui_rect, PAPER).ok_or_else(|| {
        Error::new(format!(
            "the dialog published no `{PAPER}` region. Pages & Layout is the default tab, so this \
             is not a tab problem — and the device enumerated {forms} form(s), so it is not an \
             empty list either."
        ))
    })?;
    driver.click_at(frame_for(&session, &trace, paper_vp.as_deref())?.declared_center(paper))?;
    session.settle(12);

    let trace = session.trace()?;
    let Some((auto_entry, auto_vp)) = declared_in(&trace, ui_rect, PAPER_AUTO) else {
        let entries = declared_names(&trace, ui_rect, "print.paper.");
        return Err(Error::new(format!(
            "the click on `{PAPER}` published no `{PAPER_AUTO}` region. Regions under \
             `print.paper.`: {}. If the list opened and this one is absent, the auto entry is not \
             being drawn and that is a FAILURE rather than this SKIP — but the popup closing \
             within the settle looks identical from here, so it is reported as a harness timing \
             question. Re-run; if it recurs with entries listed above and no auto among them, the \
             entry is missing.",
            list(&entries)
        )));
    };
    driver
        .click_at(frame_for(&session, &trace, auto_vp.as_deref())?.declared_center(auto_entry))?;
    // Longer than a widget settle: the choice re-measures every page of the
    // job and then re-reads the device geometry through `printer_caps_for`,
    // which opens an information device context.
    session.settle(25);

    let after = plan(&session)?;
    report.note(format!(
        "after choosing auto: pick={} auto={} paper={} sheet={} largest={}",
        after.pick, after.auto, after.paper, after.sheet, after.largest
    ));

    // --- E. the four claims, in order of what they would mean ---------------
    if after.pick != "auto" {
        return Err(Error::new(format!(
            "the click on `{PAPER_AUTO}` left `pick={}`, so it did not land on the entry. \
             Reported as SKIPPED rather than failed: nothing was learned about what auto does, \
             because auto was never chosen. Trace: {}.",
            after.pick,
            session.trace_path().display()
        )));
    }
    if after.auto == "off" {
        return Ok(Some(
            "★ `pick=auto` and `auto=off`: the operator chose Match the pages and the decision \
             never ran. The combo is recording the choice and nothing is acting on it — the \
             sheet is still whatever it was, silently. `PrintDialog::show` must recompute \
             `auto_paper` from `device.paper` at the top of every frame."
                .to_owned(),
        ));
    }
    if after.auto == "nobasis" {
        return Err(Error::new(format!(
            "the decision ran and reported `nobasis` — no sheet to choose from, or no page to \
             measure. The device enumerated {forms} form(s), so this is a document with no \
             measurable pages. Reported as SKIPPED: the application answered honestly and there \
             is nothing to compare. Drive this check with a different --pdf."
        )));
    }
    if after.paper == before.paper && after.paper.starts_with("DeviceDefault") {
        return Ok(Some(format!(
            "★★ `pick=auto auto={}` and the resolved request is still `{}`. The decision ran, \
             produced an answer, and the answer never reached the driver — the job will be \
             printed on whatever the device was standing on, and the sentence under the combo \
             will name a sheet nobody asked for. `PrintDialog::effective_device` must take \
             `self.auto_paper.resolved()`.",
            after.auto, after.paper
        )));
    }

    // --- F. ★★★ the claim that is about the DOCUMENT ---------------------
    let Some(largest) = size(&after.largest) else {
        return Ok(Some(format!(
            "`auto={}` and `largest={}`: the decision reports an outcome and no page measurement. \
             Those two cannot both be true — `Matched` and `TooBig` each carry the page the \
             choice was made for. A `none` here means the outcome was synthesised from something \
             other than the document.",
            after.auto, after.largest
        )));
    };
    let Some(sheet) = size(&after.sheet) else {
        return Err(Error::new(format!(
            "`sheet={}` after choosing auto, so the job was not planned and there is no geometry \
             to compare against. Reported as SKIPPED. Trace: {}.",
            after.sheet,
            session.trace_path().display()
        )));
    };

    let clearance = fit_clearance(largest, sheet);
    report.note(format!(
        "largest page {:.2}x{:.2} against planned sheet {:.2}x{:.2}: clearance {clearance:.2} pt \
         either way round",
        largest.0, largest.1, sheet.0, sheet.1
    ));

    // ★★★ The verdict is ASYMMETRIC, and the asymmetry is the finding.
    //
    // `matched` and `toobig` make opposite claims and only one of them is
    // allowed to overhang. An earlier version of this check compared
    // `clearance.abs()` against a band and SKIPped inside it, on the theory
    // that a small disagreement was driver noise — and its first driven run
    // SKIPped on -0.60 pt, which is not noise at all but the application's
    // declared 2 pt tolerance doing exactly its job on a CAD drawing whose
    // producer wrote A3 as a clean 1191x842. See the module header. A harness
    // that renders the application's stated behaviour as "I could not tell"
    // produces a verdict nobody investigates, every single run.
    match after.auto.as_str() {
        "matched" if clearance > -OVERHANG_PT => {
            report.note(format!(
                "★★★ Match the pages chose a sheet from THIS document: the largest page \
                 ({:.2}x{:.2}) lies on the planned sheet ({:.2}x{:.2}) — {} — and the request \
                 that reached the driver is `{}` (mixed={})",
                largest.0,
                largest.1,
                sheet.0,
                sheet.1,
                // ★ Said in words rather than as a signed number, because a
                // NEGATIVE clearance is a correct pass here and "with -0.60 pt
                // to spare" reads like a defect being waved through. It is the
                // producer-rounding case the application's tolerance is for,
                // and the report should say so in the same breath as the
                // number, not leave a reader to work it out.
                if clearance >= 0.0 {
                    format!("{clearance:.2} pt to spare")
                } else {
                    format!(
                        "over by {:.2} pt, inside the {APP_FIT_TOLERANCE_PT} pt producer-rounding \
                         tolerance the application declares",
                        -clearance
                    )
                },
                after.paper,
                plan_field(&session, "mixed").unwrap_or_else(|| "<absent>".to_owned()),
            ));
        }
        "matched" => {
            return Ok(Some(format!(
                "★★★ `auto=matched` and the largest page does NOT fit the planned sheet: \
                 {:.2}x{:.2} on {:.2}x{:.2}, over by {:.2} pt on the tighter axis even allowing \
                 the page to lie either way round AND allowing the {OVERHANG_PT} pt this check \
                 tolerates for the application's own producer-rounding slack. The dialog will \
                 say it matched the document and the drawing will be clipped or silently \
                 shrunk. Either `autopaper::choose` picked a sheet its own `fits` test rejects, \
                 or the form it chose is not the form the job was planned for — compare \
                 `paper={}` with the enumerated list. ★ A discrepancy of a few hundred points \
                 here means auto resolved to a sheet chosen without reference to the document \
                 at all, which is exactly what this check was built to catch.",
                largest.0, largest.1, sheet.0, sheet.1, -clearance, after.paper
            )));
        }
        "toobig" if clearance < OVERHANG_PT => {
            report.note(format!(
                "★★ Match the pages reported honestly that nothing fits: the largest page \
                 ({:.2}x{:.2}) does not lie on the sheet it reported ({:.2}x{:.2}) — short by \
                 {:.2} pt — and the request that reached the driver is `{}`. ⚠ That the sheet \
                 reported is the LARGEST the driver has is not established here; see the \
                 module header.",
                largest.0, largest.1, sheet.0, sheet.1, -clearance, after.paper
            ));
        }
        "toobig" => {
            return Ok(Some(format!(
                "★★ `auto=toobig` and the largest page DOES fit the planned sheet: \
                 {:.2}x{:.2} on {:.2}x{:.2}, with {clearance:.2} pt to spare — more room than \
                 the {OVERHANG_PT} pt this check allows for rounding between the driver's two \
                 answers. The operator is being told their drawing is bigger than any paper the \
                 printer has, and it is not. A false shortfall is worse than none: it will send \
                 them to a different printer they did not need.",
                largest.0, largest.1, sheet.0, sheet.1
            )));
        }
        other => {
            return Ok(Some(format!(
                "`auto={other}` is not a token this build is supposed to emit for a decision that \
                 ran. `autopaper::outcome_token` has four spellings and three of them are handled \
                 above; a fifth means the outcome type grew a variant and this check was not told."
            )));
        }
    }

    // --- G. and it can be left again ----------------------------------------
    //
    // ★ Cheap, and it asserts a property nothing above does: that auto is a
    // POLICY the operator can change their mind about, not a latch. A build
    // that resolved auto once and then held `device.paper` at the resolved
    // `Form(n)` would pass every assertion above and would have quietly
    // replaced the operator's choice with its own — the next time they opened
    // the list, the entry selected would be a sheet they never picked.
    //
    // Failure to reopen the popup is a SKIP with its own sentence rather than
    // a silent omission of the claim: a check that quietly drops an assertion
    // it could not make reads as coverage it does not have.
    let trace = session.trace()?;
    let Some((paper, paper_vp)) = declared_in(&trace, ui_rect, PAPER) else {
        report.note(
            "could not reopen the paper list to check that auto can be left again; that claim is \
             NOT part of this pass",
        );
        return Ok(None);
    };
    driver.click_at(frame_for(&session, &trace, paper_vp.as_deref())?.declared_center(paper))?;
    session.settle(12);
    let trace = session.trace()?;
    let Some((device, device_vp)) = declared_in(&trace, ui_rect, PAPER_DEVICE) else {
        report.note(
            "the reopened list published no `print.paper.item.0`; the leave-again claim is NOT \
             part of this pass",
        );
        return Ok(None);
    };
    driver.click_at(frame_for(&session, &trace, device_vp.as_deref())?.declared_center(device))?;
    session.settle(25);

    let back = plan(&session)?;
    report.note(format!(
        "after leaving auto: pick={} auto={} paper={} largest={}",
        back.pick, back.auto, back.paper, back.largest
    ));
    if back.pick == "auto" {
        return Err(Error::new(format!(
            "the click on `{PAPER_DEVICE}` left `pick=auto`, so it did not land. The leave-again \
             claim was not tested. Reported as SKIPPED rather than failed."
        )));
    }
    if back.auto != "off" || back.largest != "none" {
        return Ok(Some(format!(
            "★ leaving auto left the decision behind: `pick={} auto={} largest={}`. It is a \
             policy, and a policy that is switched off must stop computing — otherwise the \
             sentence under the combo describes a choice the operator has just undone, and \
             `paper={}` may still carry a form nobody selected.",
            back.pick, back.auto, back.largest, back.paper
        )));
    }
    report.note(
        "★ and it can be left again: choosing the printer's own settings put `auto` back to \
         `off` and forgot the measurement",
    );

    Ok(None)
}

/// The five fields of the most recent `print-plan` line.
///
/// The **last** line rather than the first: the dialog emits one per frame, so
/// the first describes the state it opened in and only the last describes the
/// state after a click. Reading the first would make every assertion above
/// trivially true, which is a mistake this project has made once and is worth
/// naming at every site that could make it again.
fn plan(session: &Session) -> Result<Plan> {
    let trace = session.trace()?;
    let line = trace.events(PLAN_EVENT).last().ok_or_else(|| {
        Error::new(format!(
            "the dialog is open and the trace has no `{PLAN_EVENT}` line, so nothing reports what \
             the job was planned against."
        ))
    })?;
    let field = |key: &str| line.get(key).unwrap_or("<absent>").to_owned();
    Ok(Plan {
        pick: field("pick"),
        auto: field("auto"),
        paper: field("paper"),
        sheet: field("sheet"),
        largest: field("largest"),
    })
}

/// One arbitrary field of the most recent `print-plan` line, for a report note.
///
/// Separate from [`plan`] because it is used only for prose: `mixed=` is worth
/// printing beside a pass so a reader knows whether the job was single-size,
/// but no assertion turns on it and adding it to [`Plan`] would suggest one
/// does.
fn plan_field(session: &Session, key: &str) -> Option<String> {
    let trace = session.trace().ok()?;
    trace
        .events(PLAN_EVENT)
        .last()
        .and_then(|l| l.get(key).map(ToOwned::to_owned))
}

/// Parse a `WxH` size token, or `None` for `none` and anything unparseable.
///
/// ★ The application writes these with [`autopaper::size_token`], two decimal
/// places and no whitespace, deliberately so that this function can be four
/// lines rather than string surgery over a `Debug` tuple. `sheet=` used to be
/// `Some((1190.4, 841.68))` and was changed on 2026-09-10 for exactly that
/// reason — this project has already shipped a driven check that reported the
/// opposite of the truth because it was parsing a `Debug` spelling.
fn size(token: &str) -> Option<(f64, f64)> {
    let (w, h) = token.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

/// How much room the page has on the sheet, in points, **either way round**.
///
/// Positive means it fits with that much to spare on the tighter axis;
/// negative means it is over by that much. Signed rather than boolean so that
/// a failure report can say *how far* wrong the answer is, which is the
/// difference between "the sheet is one size down" and "the sheet is unrelated
/// to this document".
///
/// ★ Both orientations are tried, and that is not a convenience: `dmPaperSize`
/// names a physical piece of paper and `dmOrientation` is a separate field with
/// its own control in this dialog. A3 and "A3 landscape" are not two sheets. A
/// clearance that respected page orientation would call an A3 sheet wrong for a
/// landscape A3 drawing, which is the commonest CAD export there is — and it
/// would do so as a FAILURE, against an application that was right.
fn fit_clearance(page: (f64, f64), sheet: (f64, f64)) -> f64 {
    let one = (sheet.0 - page.0).min(sheet.1 - page.1);
    let turned = (sheet.1 - page.0).min(sheet.0 - page.1);
    one.max(turned)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A size token round-trips**, in the spelling the application writes.
    ///
    /// The application's own test asserts the same round trip from its side.
    /// Both exist because they are two different claims: that one writes what
    /// it means, and that this one reads what was written. A single test on
    /// either side would leave the boundary itself untested, which is the
    /// shape of defect this whole harness is for.
    #[test]
    fn a_size_token_parses_the_way_the_application_writes_it() {
        assert_eq!(size("1190.40x841.68"), Some((1190.40, 841.68)));
        assert_eq!(size("595.28x841.89"), Some((595.28, 841.89)));
        assert_eq!(size("none"), None);
        assert_eq!(size("<absent>"), None);
        assert_eq!(size("Some((1190.4, 841.68))"), None);
    }

    /// **A page that fits reports positive clearance, and a turned page fits
    /// the same sheet.**
    ///
    /// The second half is the one worth having: a landscape A4 page on a
    /// portrait A4 sheet must report room to spare, not a shortfall, because
    /// which way the image lies is `dmOrientation` and not `dmPaperSize`.
    #[test]
    fn a_turned_page_lies_on_the_same_sheet() {
        let a4 = (595.28, 841.89);
        assert!(fit_clearance(a4, a4) >= 0.0);
        assert!(fit_clearance((841.89, 595.28), a4) >= 0.0);
        // A5 on A4: comfortable room either way round.
        assert!(fit_clearance((419.53, 595.28), a4) > 100.0);
    }

    /// **A page too big reports how far over it is, as a negative number.**
    ///
    /// The sign carries the verdict and the magnitude carries the diagnosis,
    /// so both are asserted. An A3 page on an A4 sheet is over by the
    /// difference on the tighter axis — not by the difference in area, and not
    /// by zero.
    #[test]
    fn a_page_too_big_reports_how_far_over_it_is() {
        let a4 = (595.28, 841.89);
        let a3 = (841.89, 1190.55);
        let clearance = fit_clearance(a3, a4);
        assert!(clearance < 0.0, "clearance was {clearance}");
        assert!(
            (clearance + 348.66).abs() < 1.0,
            "A3 on A4 should be over by ~348.66 pt on the tighter axis, not {clearance}"
        );
    }

    /// **A page overhanging by less than the application's tolerance is a
    /// PASS, and one overhanging by a whole size is not.**
    ///
    /// The measured case, pinned: A3 written by a CAD producer as a clean
    /// 1191x842 against the driver's 1190.4x841.68 is `matched` and correct.
    /// The defect case beside it: the same page against a Letter sheet, which
    /// is what "auto resolved to the front of the driver's list" looks like on
    /// this machine.
    #[test]
    fn the_measured_producer_rounding_case_passes_and_the_wrong_sheet_does_not() {
        let cad_a3 = (1191.0, 842.0);
        let driver_a3 = (1190.4, 841.68);
        let letter_landscape = (792.0, 612.0);

        let ok = fit_clearance(cad_a3, driver_a3);
        assert!(
            ok < 0.0 && ok > -OVERHANG_PT,
            "the measured case was -0.60 pt and must be a pass, not {ok}"
        );

        let wrong = fit_clearance(cad_a3, letter_landscape);
        assert!(
            wrong < -OVERHANG_PT,
            "a Letter sheet for an A3 drawing must fail, and it measured {wrong}"
        );
        assert!(
            wrong < -100.0,
            "the wrong-sheet case should miss by hundreds of points, not {wrong} — if this ever \
             tightens, the check has lost its discriminating power"
        );
    }
}
