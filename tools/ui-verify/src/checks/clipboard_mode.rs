//! `a_paste_review_may_not_do_says_so` — **the other side of the driven
//! sweep's finding A1: the chord gets through, and the refusal is a sentence.**
//!
//! ## ⚠ WRITTEN 2026-09-05 AND **NOT RUN BY THE SESSION THAT WROTE IT**
//!
//! Said here, in its own header, rather than left for an absent result to
//! imply. That session worked headlessly by instruction: another track held the
//! machine's pointer and keyboard, and this harness drives both. **No line
//! below has been observed against a running binary.** It is registered so the
//! next sweep picks it up, and its first run should be treated as calibration
//! rather than as a verdict — this project has three recorded cases of an
//! articulate, plausible failure message being about nothing at all.
//!
//! ## What this is for
//!
//! The sweep of 2026-09-05 found, as its first failure:
//!
//! ```text
//! chord-command      chord="Ctrl+C" id=edit.copy  via=clipboard-event
//! chord-command      chord="Ctrl+V" id=edit.paste via=clipboard-event
//! chord-not-offered  id=edit.paste mode=review
//! ```
//!
//! **Copy was offered in Review and paste was not.** The cause was
//! `app::modes::capability::offers_command`, which gates a chord on *"does this
//! mode show the tab that owns this command?"* — a proxy for *"may this mode do
//! this?"* that is wrong for a verb whose answer depends on what the operator
//! is pointing at. Paste lives on the Edit tab; Review is not shown it; so the
//! mode whose entire purpose is marking up somebody else's drawing could copy a
//! comment and had nowhere to put it.
//!
//! The fix pushes the four clipboard chords through blind and lets
//! `app::dispatch::clipboard` decide per press, which it was already doing
//! correctly and had never been reached from Review to do.
//!
//! ## ★★★ Why THIS check, and not just the repaired sticky-note one
//!
//! `copying_a_sticky_note_carries_the_whole_comment` owns the **grant**: a
//! comment copied in Review pastes in Review. It cannot own the **refusal**,
//! and the refusal is where opening a gate does its damage:
//!
//! | | before the fix | after it, if nobody wrote the sentence |
//! |---|---|---|
//! | Review, comment on the clipboard | chord refused; `chord-not-offered` traced | pastes — the grant |
//! | Review, **page geometry** on the clipboard | chord refused; `chord-not-offered` traced | dispatcher returns; **nothing traced on any surface** |
//!
//! The second row is strictly worse than the state it replaced. A chord stopped
//! at the gate at least left a line in the trace; a chord that reaches a
//! dispatcher and silently returns leaves the operator with a keystroke that
//! does nothing, no sentence, and — since `command-declined` is a diagnostic
//! channel — nothing they could show anybody either. That is this project's
//! founding defect class, and it is exactly what a fix for A1 would introduce
//! if it stopped at the gate.
//!
//! ⇒ So this check asserts **three things that must all hold at once**, and
//! none of the three is redundant:
//!
//! | # | assertion | the build it fails on |
//! |---|---|---|
//! | 1 | no `chord-not-offered` for `edit.paste` in Review | the build that shipped until 2026-09-05 — the chord never reaches the dispatcher |
//! | 2 | `command-declined … reason=mode-cannot-paste-here` | a build that opened the gate and also dropped the *effect* gate, i.e. Review silently pasting a drawing's geometry into somebody else's sheet |
//! | 3 | the `⊗` decline region is on screen | a build that opened the gate, kept the effect gate, and left the refusal in the trace where no operator can read it |
//!
//! ★ Assertion 2 is the one that would be tempting to drop as "internal". It is
//! not: without it, assertion 1 alone passes on a build where **paste
//! succeeded**, because a successful paste also emits no `chord-not-offered`.
//! The pair is what pins *reached the dispatcher AND was refused there*.
//!
//! ## ★★ How it gets a content clip into Review without Review copying one
//!
//! Review cannot select page content — that is `edit_content`, and Review does
//! not have it. So the operand is prepared **in Edit**, with `Ctrl+A`, and the
//! mode is changed afterwards. That is not a contrivance for the harness; it is
//! the operator's own path. Copy a detail out of a drawing, switch to Review to
//! mark up the sheet you were sent, press `Ctrl+V` out of habit.
//!
//! `Ctrl+A` rather than a canvas click, deliberately: a click needs a document
//! whose geometry is under a known point, and `RESUME.md`'s fixture table
//! records three separate occasions on which a coordinate aimed at the wrong
//! document produced an articulate failure about nothing. `edit.select_all`
//! needs only that the page has content, and its own trace line says whether it
//! got any.
//!
//! ## What it does NOT assert, said rather than implied
//!
//! **Which sentence is drawn.** The harness cannot read the text a panel
//! renders — no AccessKit reader, no OCR, no text extraction from a capture —
//! so it asserts that the decline REGION was published, which is the strongest
//! claim available from out here. The wording is pinned headlessly by
//! `text::clipboard::tests::every_mode_refusal_names_a_mode_the_selector_actually_offers`
//! and by
//! `app::status::decline::tests::a_mode_refusal_reads_like_no_other_decline`.
//!
//! **The Read case.** Read refuses all four verbs and does so through the same
//! two gates, so it would be a second run of the same code path for no new
//! fact. Review is the mode the defect was reported in and the mode where the
//! grant and the refusal differ, which makes it the one worth a launch.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment, declared, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// Where the operand is prepared: the only mode that may select page content.
const COPY_MODE: &str = "edit";
/// Where the paste is attempted: the mode the defect was reported in.
const PASTE_MODE: &str = "review";
/// The fixture, pinned. Any `--pdf` is ignored and the check says so.
///
/// ★ Pinned because `Ctrl+A` must find something: this check's subject is what
/// happens to a **content** clip, and on a page with no page content
/// `edit.select_all` copies nothing, the paste gate takes the markup branch,
/// Review permits it, and the check would report a pass having exercised the
/// opposite case. A real A1 CAD sheet cannot be empty.
const FIXTURE: &str = "fixtures/a1-titleblock.pdf";

/// `clipboard-copy kind=selection page=… objects=… annots=… thin=… bytes=…`.
const COPY_EVENT: &str = "clipboard-copy";
/// `chord-not-offered id=… mode=…` — the gate this defect lived in.
const CHORD_NOT_OFFERED: &str = "chord-not-offered";
/// `command-declined id=… reason=…` — the dispatcher's own refusal line.
const COMMAND_DECLINED: &str = "command-declined";
/// The command `Ctrl+V` resolves to.
const PASTE_COMMAND: &str = "edit.paste";
/// The `reason=` the paste's mode gate publishes.
const PASTE_REFUSED_REASON: &str = "mode-cannot-paste-here";
/// The status bar's worded-decline region.
///
/// Matched literally, and `app::status::decline`'s own constant says so on the
/// other side: renaming it silently un-aims this assertion.
const DECLINE_REGION: &str = "status-group:decline";

/// See the module documentation.
pub struct APasteReviewMayNotDoSaysSo;

impl Check for APasteReviewMayNotDoSaysSo {
    fn name(&self) -> &'static str {
        "a_paste_review_may_not_do_says_so"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+V in Review is refused by the mode gate before it reaches the clipboard at all — \
         so a reviewer who copied a comment has nowhere to put it — or the gate was opened and \
         the refusal that remains for page content is a silent return, which is a keystroke \
         that does nothing and says nothing"
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
            "input is disabled (--no-input). This check clicks two mode segments and presses \
             Ctrl+A, Ctrl+C and Ctrl+V. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is not on disk. This check cannot use an arbitrary \
             document: it needs a page with page CONTENT on it, or Ctrl+A copies nothing and \
             the paste takes the markup branch — the opposite of the case under test."
        )));
    }
    // ★ A sweep that supplied `--pdf` and had it thrown away must be told so: a
    // run that silently ignored a flag is indistinguishable from one that
    // honoured it.
    if ctx.pdf.is_some() {
        report.note(
            "★ this check IGNORES --pdf and pins fixtures/a1-titleblock.pdf: it needs a page \
             carrying page content, because a clip of NO content would exercise the markup \
             branch of the paste gate, which Review permits",
        );
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("clipboard-mode.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {} on {FIXTURE}", session.pid()));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: in Edit, put page CONTENT on the clipboard -----------------------
    click_mode_segment(&session, &driver, ui_rect, COPY_MODE)?;
    session.settle(20);
    driver.press_chord(&[vk::CONTROL], vk::A)?;
    session.settle(12);
    driver.press_chord(&[vk::CONTROL], vk::C)?;
    session.settle(20);

    let trace = session.trace()?;
    let copy = trace.events(COPY_EVENT).last();
    let objects: usize = copy
        .as_ref()
        .and_then(|l| l.get("objects"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if objects == 0 {
        // SKIP, not FAIL. This check's subject is what Review does with a
        // content clip; a check that could not get one is not judging its own
        // subject, and saying "the paste was refused" about an empty clipboard
        // would be a pass for the wrong reason.
        return Err(Error::new(format!(
            "Ctrl+A then Ctrl+C in {COPY_MODE} put no page content on the clipboard, so there \
             is no content clip for {PASTE_MODE} to refuse. Last `{COPY_EVENT}` line: {}. \
             Either `edit.select_all` selected nothing on this fixture, or a focused widget \
             owned Ctrl+A. SKIPPED rather than failed. Trace: {}.",
            copy.map_or_else(|| "none".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "{objects} page object(s) copied in {COPY_MODE}: {}",
        copy.map_or_else(String::new, |l| l.raw.clone())
    ));

    // --- 2: switch to Review and press Ctrl+V --------------------------------
    click_mode_segment(&session, &driver, ui_rect, PASTE_MODE)?;
    session.settle(20);
    driver.press_chord(&[vk::CONTROL], vk::V)?;
    session.settle(24);

    let trace = session.trace()?;

    // --- 3: the chord REACHED the dispatcher --------------------------------
    //
    // The A1 assertion, and the one that fails on the build that shipped.
    if let Some(refused) = trace
        .events(CHORD_NOT_OFFERED)
        .find(|l| l.get("id") == Some(PASTE_COMMAND))
    {
        return Ok(Some(format!(
            "★★★ CTRL+V NEVER REACHED THE CLIPBOARD IN {}: `{}`.\n\
             `{PASTE_COMMAND}` is refused by the MODE GATE, and `edit.copy` is not — so an \
             operator in the mode whose entire purpose is marking up somebody else's drawing \
             can copy a comment and has nowhere to put it. This is the state the driven sweep \
             of 2026-09-05 recorded as finding A1. Look at \
             `app::modes::capability::offers_command` and its escape list, NOT at \
             `canvas::clipboard`: the dispatcher already gates the effect correctly on what is \
             on the clipboard and is never reached. Trace: {}.",
            PASTE_MODE.to_uppercase(),
            refused.raw,
            session.trace_path().display()
        )));
    }

    // --- 4: …and was refused THERE, on what the clipboard holds --------------
    //
    // ★ Without this, assertion 3 alone passes on a build where the paste
    // SUCCEEDED — a successful paste also emits no `chord-not-offered` — which
    // would be Review quietly adding a drawing's geometry to somebody else's
    // sheet. The gate opened in the wrong direction is a worse defect than the
    // one it was opened to fix.
    let declined = trace.events(COMMAND_DECLINED).find(|l| {
        l.get("id") == Some(PASTE_COMMAND) && l.get("reason") == Some(PASTE_REFUSED_REASON)
    });
    let Some(declined) = declined else {
        return Ok(Some(format!(
            "★★★ {} PASTED PAGE CONTENT, OR SAID NOTHING AT ALL. The chord reached the shell \
             (no `{CHORD_NOT_OFFERED}` line) and no `{COMMAND_DECLINED} id={PASTE_COMMAND} \
             reason={PASTE_REFUSED_REASON}` followed. {PASTE_MODE} authors MARKUP and does not \
             author page content — `MODES_AND_PANELS.md` Part 1 — so a clip of {objects} page \
             object(s) must be refused there. `{COMMAND_DECLINED}` lines in the trace: {}. \
             Trace: {}.",
            PASTE_MODE.to_uppercase(),
            list(
                &trace
                    .events(COMMAND_DECLINED)
                    .map(|l| l.raw.clone())
                    .collect::<Vec<_>>()
            ),
            session.trace_path().display()
        )));
    };
    report.note(format!("the dispatcher refused it: {}", declined.raw));

    // --- 5: …and the operator was TOLD ---------------------------------------
    //
    // The half that makes opening the gate safe. See the header's table: a
    // dispatcher that refuses silently is worse than the gate that refused
    // loudly, because `command-declined` is a diagnostic channel and an
    // operator cannot read it.
    if declared(&trace, ui_rect, DECLINE_REGION).is_none() {
        return Ok(Some(format!(
            "★★★ THE REFUSAL IS SILENT. {PASTE_MODE} correctly refused the paste — `{}` — and \
             `{DECLINE_REGION}` was never published, so nothing on screen says so. From the \
             operator's chair that is a keystroke that does nothing, which is this project's \
             founding defect class, and the trace line above is not a surface. \
             `app::dispatch::clipboard`'s paste gate must call \
             `app::status::decline::record_mode_refusal`. ⚠ It must NOT call \
             `app::actions::record_note`, which draws under `⚑ About your last edit:` for a \
             press where nothing happened. Status regions that did publish: {}. Trace: {}.",
            declined.raw,
            list(&driving::declared_names(&trace, ui_rect, "status-group:")),
            session.trace_path().display()
        )));
    }

    report.note(
        "★★ Ctrl+V reached the dispatcher in Review, the dispatcher refused a CONTENT clip \
         there, and the refusal is on the status bar's worded-decline row",
    );
    Ok(None)
}
