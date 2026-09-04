//! `a_file_can_be_attached_and_taken_back_out` — **the whole round trip, through
//! the real binary, with no native dialog answered by hand.**
//!
//! # What this proves
//!
//! `attach_file`, `list_attachments`, `extract_attachment` and `detach_file`
//! have existed in `pdfcer-core` for months with **no command, no menu item and
//! no panel** — a capability that does not exist as far as the operator is
//! concerned. The panel shipped on 2026-08-28. This is what says it stayed
//! working.
//!
//! # ★★★ Why the round trip rather than "a file was attached"
//!
//! Because every step of this feature is **invisible on the page**. An embedded
//! file changes no pixel; a screenshot of a document with three attachments and
//! one with none are the same picture. So the only evidence available is what
//! the program says it did — and the failure this guards is the one where each
//! step reports success and the bytes never arrive.
//!
//! Four claims, in order, each of which can be true while the next is false:
//!
//! | # | claim | line |
//! |---|---|---|
//! | 1 | the operator's file reached the engine | `attach-file-read name=… bytes=N` |
//! | 2 | the engine wrote it into the document | `attach-file page=0 n=1 epoch=…` |
//! | 3 | **the panel reads it back out of the session** | `attachments-panel count=1` |
//! | 4 | **the bytes come back out byte-for-byte** | the saved copy compares equal |
//!
//! ★★ Claim 4 is the one that cannot be faked. A build that stored the path
//! instead of the bytes, or that wrote a truncated stream, or that saved the
//! wrong attachment on a multi-row list, passes 1 to 3 and fails here. It is
//! also the only claim in this suite that compares **file contents** rather
//! than a trace line, and it is worth the exception: the subject of the feature
//! IS the bytes.
//!
//! # The two seams, and why they are two
//!
//! `PDFCER_DIAG_ATTACH_PATH` answers the *attach* picker and
//! `PDFCER_DIAG_ATTACHMENT_SAVE_PATH` answers the *save-a-copy* picker. Sharing
//! one variable would make exactly this check unwritable — the round trip needs
//! to name an input file and an output file in one session — which is why the
//! shell declares two rather than reusing `PDFCER_DIAG_SAVE_PATH`.
//!
//! ★ Both are answered by the application, not by synthetic input: a native
//! modal is a window this harness cannot reach, and every other picker in this
//! suite is answered the same way.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | open the panel on a document with no attachments | `attachments-panel count=0` |
//! | B | click **Attach file…** | `attach-file-read name=… bytes=N`, then `attach-file page=0 n=1` |
//! | C | read the census again | `count=1 document_level=1` |
//! | D | click **Save a copy…** | `attachment-saved bytes=N renamed=…` |
//! | E | compare the saved bytes with the file attached in B | identical |
//! | F | click **Remove** | `detach-file …`, then `count=0` |
//!
//! # ★ Phase A is a precondition, not a formality
//!
//! `count=0` is asserted before anything is attached, so phase C's `count=1`
//! cannot be satisfied by a fixture that arrived carrying an attachment — the
//! same rule `a_note_can_be_written_onto_a_shape_that_exists` follows for
//! `with_note`. If the fixture does carry one, this SKIPS with the reason
//! rather than measuring the wrong thing.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Supplied at launch: **Edit mode, and nothing else.**
///
/// # ★★★ `edit.attachments` was here until a smoke launch showed it CLOSING the
/// panel
///
/// The command is a **toggle** — `app::panels::toggle_panel` closes a panel that
/// `dock::is_on_screen` reports as showing, and that predicate is *mounted **and**
/// its side visible **and** it is the active tab*. Edit's default arrangement
/// mounts Attachments as the last tab of its right-hand stack, and the last tab
/// is the active one.
///
/// So the launch invoked it and the trace answered
/// `panel-closed id=edit.attachments closed=true`. **Every phase below would
/// have failed on a correct build**, at phase A, reporting that the panel was
/// not on screen — which it had been, until this check shut it.
///
/// ⇒ ★★ It is the fourth "cannot pass" in this suite and the only one a
/// **reading** could not have found: an audit that walked all eleven checks
/// two hours earlier marked this one SOUND, correctly, because which tab a
/// stack activates by default is a property of the running program and not of
/// the source. **Launching it offscreen for seven seconds found it.**
///
/// ★ The convention this now follows is `properties_metadata`'s, cited by
/// `bookmark_add` and `dimension_groups` before it: **ask whether the surface is
/// already drawing, and only press the toggle if it is not.**
/// ★★★ **AND the panel, through the seam — corrected 2026-09-01.**
///
/// This was `"mode.edit"` alone, on the reasoning recorded above: ask whether
/// the panel is already drawing, and press the ribbon toggle only if it is not.
/// The asking is right and it is kept. What was wrong is the fallback: the
/// ribbon shows **one tab at a time**, so `ribbon.item.edit.attachments` is not
/// declared unless the Edit tab happens to be the one showing — and when it is
/// not, the toggle cannot be pressed and this check SKIPPED.
///
/// It had been skipping. A driven check that skips has stopped working and
/// nothing will tell you; that is a rule this project already had, filed to
/// `D:/dev/rag/egui/a_driven_check_that_skips_has_stopped_working_and_nothing_will_tell_you.md`,
/// and it went on being true here for however long the saved Edit layout has
/// not had this panel showing.
///
/// ⇒ The seam reaches the command wherever it lives. The ask-then-toggle logic
/// below is untouched and is now the SECOND route rather than the only one.
const INVOKE: &str = "mode.edit,edit.attachments";
/// The ribbon control that toggles the panel, for the case where Edit's saved
/// arrangement does not have it showing.
const PANEL_ITEM: &str = "ribbon.item.edit.attachments";
/// The panel's per-frame census.
const CENSUS: &str = "attachments-panel";
/// The line the panel writes when the picker has answered and the file is read.
const ATTACH_READ: &str = "attach-file-read";
/// The funnel's line for the attach verb.
const ATTACHED: &str = "attach-file";
/// The line the save path writes on success.
const SAVED: &str = "attachment-saved";
/// The funnel's line for the detach verb.
const DETACHED: &str = "detach-file";
/// The region the Attach control publishes.
const ATTACH_REGION: &str = "attachments.attach";
/// The region the first row's Save publishes.
const SAVE_REGION: &str = "attachments.save";
/// The region the first row's Remove publishes.
const REMOVE_REGION: &str = "attachments.remove";
/// The seam that answers the attach picker.
const ATTACH_ENV: &str = "PDFCER_DIAG_ATTACH_PATH";
/// The seam that answers the save-a-copy picker.
const SAVE_ENV: &str = "PDFCER_DIAG_ATTACHMENT_SAVE_PATH";

/// See the module documentation.
pub struct AFileCanBeAttachedAndTakenBackOut;

impl Check for AFileCanBeAttachedAndTakenBackOut {
    fn name(&self) -> &'static str {
        "a_file_can_be_attached_and_taken_back_out"
    }

    fn defect(&self) -> &'static str {
        "a file can be attached to a document and cannot be got back out of it, or is reported as \
         attached and is not in the file at all — a failure with no visible symptom, because an \
         embedded file changes no pixel on any page"
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

/// The `attachments-panel` line's count, or `None` if the panel did not draw.
fn census(session: &Session) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .last(CENSUS)
        .and_then(|line| line.get_usize("count")))
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks three panel controls. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // ★ The file attached is the FIXTURE ITSELF.
    //
    // Deliberate, and not laziness: it is a real file of a known size that is
    // certain to exist wherever this check runs, and attaching a PDF to a PDF is
    // the commonest real case — a revision, a spec sheet, a source drawing. It
    // also makes phase E's comparison meaningful at a size where a truncation
    // bug shows up, which a three-byte scratch file would not.
    let source = pdf.clone();
    let saved_to = ctx.out("attachment-roundtrip.bin");
    let _ = std::fs::remove_file(&saved_to);

    let mut spec = LaunchSpec::new(&exe, ctx.out("attachments.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ATTACH_ENV.to_owned(), source.to_string_lossy().into_owned()));
    spec.env
        .push((SAVE_ENV.to_owned(), saved_to.to_string_lossy().into_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with {ATTACH_ENV}={} and {SAVE_ENV}={}",
        exe.display(),
        session.pid(),
        source.display(),
        saved_to.display()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- A: the panel, and nothing in it -----------------------------------
    //
    // ★ Raised only if it is not already drawing. See [`INVOKE`]: pressing the
    // toggle over an open panel shuts the subject of the check, which is what
    // this one did on every run until 2026-08-29.
    if census(&session)?.is_none() {
        let trace = session.trace()?;
        if let Some(item) = declared(&trace, ui_rect, PANEL_ITEM) {
            driver.click_at(session.frame()?.declared_center(item))?;
            session.settle(20);
        }
    }
    let trace = session.trace()?;
    let Some(before) = census(&session)? else {
        return Err(Error::new(format!(
            "the panel traced no `{CENSUS}` line after `{INVOKE}` and after pressing \
             `{PANEL_ITEM}`, so it is not on screen and every control below is absent for that \
             reason rather than the one under test. ★ If the trace carries \
             `panel-closed id=edit.attachments`, this check pressed a toggle over an already-open \
             panel and shut its own subject — the defect this INVOKE was corrected for. Regions \
             beginning `attachments`: {}.",
            list(&declared_names(&trace, ui_rect, "attachments"))
        )));
    };
    if before != 0 {
        return Err(Error::new(format!(
            "the fixture already carries {before} attachment(s), so `count` cannot be the oracle \
             for what this check attaches. Use a document with none."
        )));
    }

    // --- B: attach ----------------------------------------------------------
    let attach = declared(&trace, ui_rect, ATTACH_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{ATTACH_REGION}` region. Regions beginning `attachments`: {}.",
            list(&declared_names(&trace, ui_rect, "attachments"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(attach))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(read) = trace.events(ATTACH_READ).last() else {
        return Ok(Some(format!(
            "clicking Attach produced no `{ATTACH_READ}` line, so the picker did not answer or the \
             file was not read. ★ Suspect the seam first: `{ATTACH_ENV}` is how a native modal is \
             answered in this harness, and a build that stopped consulting it would open a real \
             dialog this check cannot reach and would then look like a hang. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the file was read: `{}`", read.raw));
    if trace.events(ATTACHED).count() == 0 {
        return Ok(Some(format!(
            "the file was read and no `{ATTACHED}` line followed, so the engine never wrote it. A \
             refused `vector_edit` traces `{ATTACHED}-refused` — look for that first; the engine \
             refuses an encrypted document, a certified one, and a name tree it cannot extend."
        )));
    }

    // --- C: the panel reads it back ----------------------------------------
    let after = census(&session)?.unwrap_or(0);
    if after != 1 {
        return Ok(Some(format!(
            "the engine attached the file and the panel lists {after}. The panel reads the \
             SESSION, so a zero here means it is reading the file on disk instead — an operator \
             would attach a file and see nothing until they saved."
        )));
    }
    report.note("★ the panel read the attachment back out of the session");

    // --- D & E: save a copy, and compare the bytes -------------------------
    let trace = session.trace()?;
    let save = declared(&trace, ui_rect, SAVE_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{SAVE_REGION}` region on the first row. Regions: {}.",
            list(&declared_names(&trace, ui_rect, "attachments"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(save))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(saved) = trace.events(SAVED).last() else {
        return Ok(Some(format!(
            "clicking Save a copy produced no `{SAVED}` line. A decline traces \
             `attachment-save-declined`, a cancelled picker traces `attachment-save-cancelled`, \
             and a failed write traces `attachment-save-failed` — one of those three will say \
             which. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the save reported: `{}`", saved.raw));

    // ★★★ THE ASSERTION THAT CANNOT BE FAKED.
    //
    // Everything above is the program's own account of itself. This is the
    // bytes. A build that stored the path rather than the stream, truncated the
    // write, or saved the wrong row passes every line above and fails here.
    let written = std::fs::read(&saved_to).map_err(|err| {
        Error::new(format!(
            "the save reported success and {} cannot be read: {err}. That is a harness or \
             filesystem fault rather than a verdict on the program, so it is a SKIP.",
            saved_to.display()
        ))
    })?;
    let original = std::fs::read(&source).map_err(|err| {
        Error::new(format!(
            "cannot re-read the source fixture {}: {err}.",
            source.display()
        ))
    })?;
    if written != original {
        return Ok(Some(format!(
            "THE BYTES DID NOT SURVIVE THE ROUND TRIP. Attached {} ({} bytes); got back {} bytes. \
             Every line the program wrote reported success, which is the whole reason this check \
             compares the file rather than the trace: an embedded file changes no pixel, so a \
             truncated or substituted stream has no visible symptom at all.",
            source.display(),
            original.len(),
            written.len()
        )));
    }
    report.note(format!(
        "★★★ {} bytes came back identical to what went in",
        written.len()
    ));

    // --- F: remove ----------------------------------------------------------
    let trace = session.trace()?;
    let remove = declared(&trace, ui_rect, REMOVE_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{REMOVE_REGION}` region on the first row. ★ A **page-level** attachment
             deliberately has no Remove — it is removed as a comment — so if the row under test \
             is one of those this is the panel behaving correctly and the check aiming wrongly. \
             Regions: {}.",
            list(&declared_names(&trace, ui_rect, "attachments"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(remove))?;
    session.settle(24);

    let trace = session.trace()?;
    if trace.events(DETACHED).count() == 0 {
        return Ok(Some(format!(
            "clicking Remove produced no `{DETACHED}` line. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let removed = census(&session)?.unwrap_or(usize::MAX);
    if removed != 0 {
        return Ok(Some(format!(
            "the attachment was removed and the panel still lists {removed}. The count must be \
             exactly 0 — one was attached and one was removed — and 'fewer than before' would \
             pass on a build that removed the wrong one from a longer list."
        )));
    }
    report.note("★★ the attachment is gone from the session's own listing");
    Ok(None)
}
