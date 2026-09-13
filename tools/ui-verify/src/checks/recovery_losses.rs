//! **A document pdfcer had to rebuild says what the rebuild could not keep**,
//! and a document whose rebuild kept everything says nothing — the driven half
//! of the dropped-object disclosure.
//!
//! # What this file is for
//!
//! When a PDF's cross-reference table is unusable, `pdfcer-core` rebuilds one by
//! scanning the whole file for anything that looks like the start of an object.
//! Some of what that scan finds cannot be read back, and those candidates are
//! **dropped**. Until engine `fb6e004` there was no way for a caller to ask
//! which ones: a recovery that silently lost a page's content stream handed back
//! a shorter document and told nobody. `RecoveryReport::objects_dropped` now
//! carries an object number and a reason for each, and the engine's own filing
//! for it recorded that the disclosure *"is not yet visible to anyone"* — a
//! statement about **this shell**.
//!
//! It is visible now, in Document properties, under the recovery note:
//!
//! * the recovery note itself — heading and census — as `properties.recovery`;
//! * the dropped-object block under it, two sentences and a list of object
//!   numbers, as `properties.recovery-dropped`.
//!
//! The strings are unit-tested from every corner, including through the real
//! engine on a fixture authored for it. What no unit test can see is whether
//! either block is **connected to a running window** — whether the panel draws
//! them where an operator can read them. That is this file's whole subject, and
//! it is standing rule R1.
//!
//! # ★★★ The control launch is the check, and it is a RECOVERED file
//!
//! The obvious control would have been a document with a sound index, which is
//! what the neighbouring `load_anomalies` checks use. It would have been wrong
//! here. On a sound file `Document::recovery()` is `None`, so the entire
//! neighbourhood — note, census, dropped block — correctly draws nothing, and
//! "the dropped block is absent" is then satisfied by at least three states that
//! are not the one under test:
//!
//! 1. the properties panel never opened;
//! 2. the document was never recovered, so nothing nearby drew at all;
//! 3. the block is correctly driven by `objects_dropped`.
//!
//! An assertion satisfied by all three is not a measurement of which one
//! shipped. So the control is `fixtures/recovered-no-losses.pdf`: the same
//! damage, the same recovery path, the same `RecoveryReason`, differing from its
//! sibling in **exactly one property** — the scan found nothing it could not
//! keep. Both launches require `properties.recovery`, which disposes of (1) and
//! (2) and leaves exactly one reading of the difference between them.
//!
//! ⚠ Both fixtures' properties are asserted **through the engine** by the
//! shell's own tests, in the suite that runs on every `cargo test`:
//! `the_recovery_fixture_drops_the_two_objects_it_was_built_to_drop` and
//! `the_control_fixture_for_the_dropped_disclosure_really_recovers_and_drops_nothing`.
//! That is deliberate placement. If an engine bump changed what the scan finds,
//! the absence assertion below would go red and blame the application for
//! something true of the fixture — so the tripwire lives where it costs a second
//! rather than a driven sweep.
//!
//! # What this does NOT prove
//!
//! That the **wording** is right. The trace publishes a region name and a
//! rectangle, not a string, so a build that printed the wrong object numbers, or
//! described a routine false positive in the language reserved for a real loss,
//! would pass. Those are asserted in `crate::text::panels::docprops`' own tests
//! against the same report — deliberately, because a string is exactly what a
//! unit test CAN see. What it cannot see is the window.

use std::path::Path;

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The damaged file: no index, and a scan that finds two things it cannot keep.
///
/// Built by `fixtures/recovered-with-losses.PROVENANCE.py`, which records why
/// each of the two exists — one is a real truncated object, the other is the
/// bytes `9 0 obj` sitting inside the content stream's own drawn text, which the
/// scan is obliged to try. Those are the two different stories the disclosure
/// has to keep apart, and the fixture carries both on purpose.
const DAMAGED: &str = "recovered-with-losses.pdf";

/// The control: the same damage, nothing dropped. See the module header.
const CONTROL: &str = "recovered-no-losses.pdf";

/// The recovery note — heading and census. Required on **both** launches, and
/// that is what makes the control's silence about drops a measurement rather
/// than an ambiguity.
const RECOVERY_REGION: &str = "properties.recovery";

/// The dropped-object block. Required on [`DAMAGED`], forbidden on [`CONTROL`].
const DROPPED_REGION: &str = "properties.recovery-dropped";

/// The metadata form's region — the panel's other content, used here only as
/// proof that the panel itself came up.
const PANEL_OPEN_WITNESS: &str = "properties.info";

/// The mode this drives in. `file` is in every mode's tab list; Review is
/// chosen because the rest of the harness uses it.
const MODE: &str = "review";

/// The trace line the shell emits once a document is open. Its **absence** is
/// how a launch that opened nothing is told apart from a build that discloses
/// nothing.
const STATUS_LINE: &str = "status";

// ---------------------------------------------------------------------------

/// See the module documentation.
pub struct RecoveryLossesAreListedInDocumentProperties;

impl Check for RecoveryLossesAreListedInDocumentProperties {
    fn name(&self) -> &'static str {
        "recovery_losses_are_listed_in_document_properties"
    }

    fn defect(&self) -> &'static str {
        "pdfcer rebuilt a damaged document's index by scanning it, could not read back some of \
         what the scan found, and threw those away without saying so — so an operator whose \
         drawing came back with a blank page, a missing detail or a vanished annotation has no \
         way to learn that pdfcer knows which object went. Or the opposite: the losses block is \
         pinned to every recovered file, announcing missing objects on documents that lost \
         nothing, which trains him to stop reading the one disclosure that will eventually matter"
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
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, the File tab and \
             the Document properties item, twice. Reported as SKIPPED rather than passed: a \
             check that did not run has learned nothing.",
        ));
    }

    // --- 1: the damaged file, and the block it earns -----------------------
    let session = launch(ctx, &exe, DAMAGED, "recovery_losses.damaged.trace.txt")?;
    report.note(format!(
        "launched {} on fixtures/{DAMAGED} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // ★★ MAXIMISE — at the harness's default 1,100 pt window the File tab's
    // last two groups fold away entirely and a check reports a lost command.
    // `about.rs` holds the measurement; several checks already share it.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    open_properties(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    // ★ The document has to be OPEN before the presence or absence of a
    // disclosure means anything. A launch that opened nothing traces no
    // `status` line at all, and every region on a document-dependent surface is
    // then legitimately missing. Two sweeps in this project produced confident,
    // detailed, entirely wrong defect reports from exactly that.
    if trace.last(STATUS_LINE).is_none() {
        return Err(Error::new(format!(
            "the application launched and never traced a `{STATUS_LINE}` line, so it opened no \
             document. ⚠ fixtures/{DAMAGED} is deliberately damaged — no xref, no trailer, no \
             startxref — so a refusal to open it is itself worth investigating, but it is a \
             loader finding and not a finding about this panel."
        )));
    }
    if declared(&trace, ui_rect, PANEL_OPEN_WITNESS).is_none() {
        return Err(Error::new(format!(
            "the Document properties panel did not come up — no `{PANEL_OPEN_WITNESS}` region — \
             so nothing can be concluded about the recovery block inside it. Regions beginning \
             `properties`: {}.",
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    if declared(&trace, ui_rect, RECOVERY_REGION).is_none() {
        return Ok(Some(format!(
            "fixtures/{DAMAGED} was opened by rebuild-by-scan — `pdfcer inspect` reports \
             reason=StartxrefNotFound for it — and the Document properties panel declares no \
             `{RECOVERY_REGION}` region, so the recovery disclosure itself never drew. The \
             operator is editing a document whose index pdfcer invented, and is not being told. \
             ★ Check `recovery_note`'s early return: it is `doc.session.document().recovery()`, \
             and if that is now `None` for this file the finding is in the loader, not here. \
             Regions beginning `properties`: {}.",
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    if declared(&trace, ui_rect, DROPPED_REGION).is_none() {
        return Ok(Some(format!(
            "the recovery note is drawn on fixtures/{DAMAGED} and the block naming what the \
             rebuild could NOT keep is not. That file's scan drops two objects — asserted \
             through the engine by the shell's \
             `the_recovery_fixture_drops_the_two_objects_it_was_built_to_drop` — so the losses \
             exist and the disclosure of them does not. ★ Check `dropped_objects_note`'s early \
             return: it must be `report.objects_dropped.is_empty()` and nothing else. A recovery \
             that quietly discards a page's content stream is the loader silence decision 145 \
             was written to end, appearing one layer down. Regions beginning `properties`: {}.",
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    report.note("the damaged file discloses both that it was rebuilt and what the rebuild lost");
    drop(session);

    // --- 2: ★★ THE CONTROL, and it is the half that makes this a check -----
    let session = launch(ctx, &exe, CONTROL, "recovery_losses.control.trace.txt")?;
    report.note(format!(
        "control launch on fixtures/{CONTROL} as pid {} — the same damage, nothing dropped",
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    open_properties(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    if trace.last(STATUS_LINE).is_none() {
        return Err(Error::new(format!(
            "the control launch opened no document — no `{STATUS_LINE}` line — so its silence \
             about dropped objects proves nothing. fixtures/{CONTROL}."
        )));
    }
    if declared(&trace, ui_rect, PANEL_OPEN_WITNESS).is_none() {
        return Err(Error::new(format!(
            "the control launch's Document properties panel did not come up — no \
             `{PANEL_OPEN_WITNESS}` region — so the absence of a losses block in it is the \
             absence of the whole panel, and proves nothing."
        )));
    }
    // ★★★ The positive witness. Without this line the assertion below would be
    // satisfied by a build where the entire recovery disclosure had stopped
    // drawing, which is a worse defect wearing the same green tick.
    if declared(&trace, ui_rect, RECOVERY_REGION).is_none() {
        return Err(Error::new(format!(
            "the control launch's panel is open and declares no `{RECOVERY_REGION}` region, so \
             this document is not showing a recovery disclosure at all — and the absence of a \
             losses block under a note that is not there is not evidence about the losses block. \
             ⚠ fixtures/{CONTROL} must be a RECOVERED file; if it has acquired a sound \
             cross-reference table it has stopped being a control and the shell's \
             `the_control_fixture_for_the_dropped_disclosure_really_recovers_and_drops_nothing` \
             will say so in one second."
        )));
    }
    if let Some(rect) = declared(&trace, ui_rect, DROPPED_REGION) {
        return Ok(Some(format!(
            "fixtures/{CONTROL} was rebuilt by scanning and the scan kept everything it found — \
             asserted through the engine by the shell's \
             `the_control_fixture_for_the_dropped_disclosure_really_recovers_and_drops_nothing` \
             — and the panel drew `{DROPPED_REGION}` at {rect:?} anyway. That is a block naming \
             missing objects on a document that is missing none. ★ Check \
             `dropped_objects_note`'s first statement: it must return on an empty \
             `objects_dropped` BEFORE it draws or publishes anything. R9 — a disclosure with \
             nothing to disclose renders nothing, not a reassuring zero — and R8b rule 4, \
             because a warning that cries wolf on healthy files is the one an operator learns to \
             skip past."
        )));
    }
    report.note("the lossless recovery discloses the rebuild and says nothing about losses");

    Ok(None)
}

// ---------------------------------------------------------------------------
// shared
// ---------------------------------------------------------------------------

/// A launch on the real desktop, because this one is going to be clicked.
///
/// ⚠ Unlike `load_anomalies`' status-bar half there is no off-desktop variant
/// here: every assertion in this file needs the properties panel open, the panel
/// is opened by clicking, and a click needs a window a pointer can reach. So
/// this check cannot run while the operator is at the machine, and that is a
/// cost rather than an oversight.
fn launch(ctx: &CheckContext, exe: &Path, fixture: &str, trace_name: &str) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace_name));
    spec.pdf = Some(crate::checks::driving::repo_fixture(
        fixture,
        "This check pins its own two documents and ignores --pdf: its whole method is one \
         build's positive reading on a file whose rebuilt index lost objects, denied on a file \
         whose rebuilt index lost none, and a suite-wide fixture is neither of those.",
    )?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    Session::launch(&spec, ctx.profile.trace_prefix)
}

/// Bring the Document properties panel to the front, if it is not already.
///
/// ★ Reuses `properties_metadata`'s opener rather than spelling the two clicks
/// again. It is the same ribbon item and the same toggle hazard — pressing
/// `file.document_properties` while the panel is up CLOSES it — and a second
/// copy of that guard would be a second place for the next ribbon move to have
/// to be applied.
fn open_properties(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    if declared(&session.trace()?, ui_rect, PANEL_OPEN_WITNESS).is_some() {
        return Ok(());
    }
    crate::checks::properties_metadata::open_document_properties(session, driver, ui_rect)
}
