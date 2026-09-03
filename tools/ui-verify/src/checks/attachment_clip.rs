//! `an_attachment_moves_between_two_open_documents` — **the gap the
//! verb-coverage gate found on its first honest run.**
//!
//! # Where this came from
//!
//! Nobody reported it. `tools/gates/check-verb-coverage.sh` shipped on
//! 2026-09-01 and named five verbs `pdfcer-core` implements that this shell
//! called nowhere and had written no sentence about. Three of them were
//! `copy_attachment`, `cut_attachment` and `paste_attachment`, shipped in
//! `Pass 173.0` — so an embedded file could not be moved from one open document
//! to another, which is an odd thing to be missing now that pdfcer is
//! multi-document.
//!
//! ⇒ **This check is the answer to "how would anyone have known?"** and the
//! shape is worth keeping: the gate found the gap, and the gate cannot close
//! it, because a gate reads source and a capability is a thing you drive.
//!
//! ## ★★★ Why it must cross a document boundary
//!
//! A same-document copy-then-paste would exercise every line of the code and
//! **would not test the defect**. The defect was *"an attachment cannot be moved
//! between two open documents"*; a check confined to one document is a check
//! that passes on a build where the clipboard is a per-document field.
//!
//! That is this project's standing failure mode wearing a new hat — a check
//! whose subject is narrower than the report it answers.
//!
//! ## ★★ And why the disclosure is asserted by its ABSENCE here
//!
//! `attach_file` builds its name-tree patch with
//! `entries.retain(|(k, _)| k != &name_bytes)` before pushing, so a same-named
//! attachment is **replaced** — silently, recoverable only from the earlier
//! revision. The panel says so before the press, and the region
//! `attachments.paste.replaces` is that sentence.
//!
//! In the *second* document there is no file of that name, so the note must
//! **not** be drawn. Asserting the absence is the more useful half: a build that
//! drew the warning unconditionally would be crying wolf on every paste, which
//! is how an operator learns to stop reading warnings — and it would pass any
//! check that only asserted the warning appears when it should.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | attach a file to document 1 | `attach-file` and a census of 1 |
//! | B | press Copy | `attachment-copied name=… bytes=…` |
//! | C | open document 2 and go to its Attachments panel | a census of 0 |
//! | D | the Paste control is drawn, and the replace note is NOT | `attachments.paste` declared, `attachments.paste.replaces` absent |
//! | E | press Paste | `paste-attachment-requested … replacing=false`, then a census of 1 |
//!
//! ★ Step D's first half is also an R9 assertion: before step B the clipboard
//! is empty and `attachments.paste` must be **absent**, not greyed. That is
//! checked at the top, and it is the control point — without it, a build that
//! always drew the button would pass every later step.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then the panel, through the harness seam.
///
/// ★★ NOT `click_mode_segment` plus a ribbon click. The ribbon shows one tab at
/// a time and this check leaves the operator on whichever tab the mode selector
/// last drew — which on the first run was File, so
/// `ribbon.item.edit.attachments` was simply not on screen and the check
/// skipped on a build where the feature worked. A region absent because it is
/// on another tab looks exactly like one absent because the feature is missing.
///
/// The seam reaches the command wherever it lives, which is the same reason
/// `checks::attachments` uses it.
const INVOKE: &str = "mode.edit,edit.attachments";
/// The command that opens the panel.
const PANEL_ITEM: &str = "ribbon.item.edit.attachments";
/// The panel's own census line — how this check counts what a document holds.
const CENSUS: &str = "attachments-panel"; // ui-text-exempt: a trace event name, never displayed
/// The line the attach writes when it has read the file.
const ATTACHED: &str = "attach-file"; // ui-text-exempt: a trace event name, never displayed
/// ★ The line the Copy button writes.
const COPIED: &str = "attachment-copied"; // ui-text-exempt: a trace event name, never displayed
/// ★ The line this check exists to read.
const PASTED: &str = "paste-attachment-requested"; // ui-text-exempt: a trace event name
/// The Attach control, used to put a file in document 1 in the first place.
const ATTACH_REGION: &str = "attachments.attach"; // ui-text-exempt: a trace region name
/// The Copy control.
const COPY_REGION: &str = "attachments.copy"; // ui-text-exempt: a trace region name
/// The Paste control — absent, per R9, when the clipboard holds nothing.
const PASTE_REGION: &str = "attachments.paste"; // ui-text-exempt: a trace region name
/// ★★★ The paste's verdict about replacement, as ONE of a pair.
///
/// The first version of this check asserted `attachments.paste.replaces` was
/// **absent** in the second document, and FAILED against a correct build —
/// because `ui_rect` is a change log and the panel had legitimately declared
/// that name several frames earlier, in the FIRST document, where a file of
/// that name really was present. A region that stops being drawn does not
/// un-declare itself.
///
/// ⇒ The panel now publishes exactly one of two names every frame, and this
/// check reads whichever came last. **An absence assertion became a presence
/// assertion**, which is the only kind a change log can answer. The finding
/// is already in `D:/dev/rag/egui/` under
/// `a_change_log_ui_rect_trace_cannot_report_that_a_widget_stopped_being_drawn.md`,
/// and this reproduced it within the hour.
const REPLACES_REGION: &str = "attachments.paste.replaces"; // ui-text-exempt: a trace region name
/// The other half of the pair — nothing would be displaced.
const FRESH_REGION: &str = "attachments.paste.fresh"; // ui-text-exempt: a trace region name
/// The document tab strip's per-tab region prefix.
const TAB: &str = "doc-tab."; // ui-text-exempt: a trace region name prefix
/// The seam that answers the attach picker without a human.
const ATTACH_ENV: &str = "PDFCER_DIAG_ATTACH_PATH"; // ui-text-exempt: an environment variable name
/// The seam that answers the OPEN picker. A fourth copy of this constant, and
/// the duplication is deliberate in this harness: a shared one would make every
/// check that names it recompile when any other changed its meaning, and the
/// name is the application's contract rather than this crate's.
const OPEN_PATH_ENV: &str = "PDFCER_DIAG_OPEN_PATH"; // ui-text-exempt: an environment variable name

pub struct AnAttachmentMovesBetweenTwoOpenDocuments;

impl Check for AnAttachmentMovesBetweenTwoOpenDocuments {
    fn name(&self) -> &'static str {
        "an_attachment_moves_between_two_open_documents"
    }

    fn defect(&self) -> &'static str {
        "an embedded file cannot be moved from one open document to another — the Attachments \
         panel attaches from disk and removes, and has no clipboard at all, so the only route \
         between two documents is to save the file out and attach it again"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "five steps across two documents, each reading a rectangle the step before it resolved" // ui-text-exempt: a lint justification, never displayed
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses four controls in two documents.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a document to attach a file to."))?;
    let second = ctx.second_pdf.clone().ok_or_else(|| {
        Error::new(
            "no second document. Pass --second-pdf <path>, DIFFERENT from --pdf. A same-document \
             copy-and-paste would exercise every line of this feature and would not test the \
             defect, which is that an attachment could not cross a document boundary.",
        )
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // The file to attach is this harness's own binary path? No — something
    // small and certainly present. The fixture the check is given is ideal: it
    // exists, it is a few kilobytes, and its name is stable.
    let payload = second.clone();

    let mut spec = LaunchSpec::new(&exe, ctx.out("attachment-clip.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((ATTACH_ENV.to_owned(), payload.display().to_string()));
    spec.env
        .push((OPEN_PATH_ENV.to_owned(), second.display().to_string()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    // --- open the Attachments panel ----------------------------------------
    //
    // ★★ ASK WHETHER IT IS ALREADY DRAWING, and only press the toggle if it is
    // not. `attachments`' own header records why: the ribbon item is a TOGGLE
    // and the dock layout persists, so pressing it over an already-open panel
    // shuts the subject of the check — which is what that check did on every
    // run until 2026-08-29, and what `the_line_weight_switch_reaches_the_resize`
    // started doing the day the exit hook made layout persistence reliable.
    //
    // ★ The ribbon item may also be absent from THIS tab: the ribbon shows one
    // tab at a time, so `ribbon.item.edit.attachments` is not declared unless
    // the Edit tab is the one showing. Absence is therefore not a failure, and
    // the census below is the real precondition.
    if census(&session)?.is_none() {
        let trace = session.trace()?;
        if let Some(item) = declared(&trace, ui_rect, PANEL_ITEM) {
            driver.click_at(session.frame()?.declared_center(item))?;
            session.settle(30);
        }
    }
    if census(&session)?.is_none() {
        let trace = session.trace()?;
        return Err(Error::new(format!(
            "the Attachments panel traced no `{CENSUS}` line, so it is not on screen and every \
             control below is absent for that reason rather than the one under test. Regions \
             beginning `attachments`: {}.",
            list(&declared_names(&trace, ui_rect, "attachments"))
        )));
    }

    // ★★★ THE CONTROL POINT. With nothing on the clipboard the Paste control
    // must be ABSENT — R9's rule that an unavailable capability renders nothing
    // rather than a greyed stub. Without this, a build that drew the button
    // unconditionally would satisfy every step below.
    if declared(&session.trace()?, ui_rect, PASTE_REGION).is_some() {
        return Ok(Some(format!(
            "★★★ THE PASTE CONTROL IS DRAWN WITH AN EMPTY CLIPBOARD: `{PASTE_REGION}` is \
             declared before anything has been copied. R9 reserves greying for the TEMPORARILY \
             unavailable, explained on hover — an operator who has copied nothing is not \
             waiting for anything, so the control must not be there at all. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ with an empty clipboard the Paste control is absent, not greyed");

    // --- A: attach a file to document 1 ------------------------------------
    let trace = session.trace()?;
    let Some(attach) = declared(&trace, ui_rect, ATTACH_REGION) else {
        return Err(Error::new(format!(
            "the panel declares no `{ATTACH_REGION}`, so this check cannot put a file in the \
             first document. Regions beginning `attachments`: {}.",
            list(&declared_names(&trace, ui_rect, "attachments"))
        )));
    };
    driver.click_at(session.frame()?.declared_center(attach))?;
    session.settle(50);

    let trace = session.trace()?;
    if trace.events(ATTACHED).count() == 0 {
        return Err(Error::new(format!(
            "no `{ATTACHED}` line, so nothing was attached to the first document and this \
             check has no subject. That is `attachments`' own territory, not this one's. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ a file is attached to the first document");

    // --- B: copy it ---------------------------------------------------------
    let trace = session.trace()?;
    let Some(copy) = declared(&trace, ui_rect, COPY_REGION) else {
        return Ok(Some(format!(
            "★★★ THERE IS NO COPY CONTROL: the panel lists an attachment and declares no \
             `{COPY_REGION}`. Regions beginning `attachments`: {}.\n\
             This is the defect exactly — `EditSession::copy_attachment` has existed since \
             `Pass 173.0` and the panel offered attach, save and remove only, so the sole \
             route from one document to another was to write the file to disk and attach it \
             again. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "attachments")),
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_center(copy))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(copied) = trace.last(COPIED) else {
        return Ok(Some(format!(
            "★★ THE COPY BUTTON IS DRAWN AND DOES NOTHING: `{COPY_REGION}` was clicked and no \
             `{COPIED}` line followed. R9 calls an offered control that does nothing the \
             misleading kind of placeholder — worse than an absent one, because the operator \
             now believes the file is on the clipboard. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the copy happened: `{}`", copied.raw));

    // --- C: open the second document ---------------------------------------
    //
    // Through the ribbon's Open, whose picker is answered by `OPEN_PATH_ENV`.
    // The tab strip is the oracle: two tabs means two documents.
    let trace = session.trace()?;
    let Some(open_item) = declared(&trace, ui_rect, "ribbon.item.file.open") else {
        return Err(Error::new(
            "the ribbon does not declare `ribbon.item.file.open`, so the second document \
             cannot be opened.",
        ));
    };
    driver.click_at(session.frame()?.declared_center(open_item))?;
    session.settle(60);

    let trace = session.trace()?;
    let tabs = declared_names(&trace, ui_rect, TAB);
    if tabs.len() < 2 {
        return Err(Error::new(format!(
            "the second document did not open — {} tab(s): {}. That is `document_tabs`' \
             subject, not this one's.",
            tabs.len(),
            list(&tabs)
        )));
    }

    // The panel follows the active document, and the active document is the one
    // just opened. Its census must be zero: the payload was attached to the
    // FIRST document, and if this reads non-zero the two tabs are showing one
    // document's state and the rest of this check would be measuring nothing.
    let trace = session.trace()?;
    let count = trace
        .last(CENSUS)
        .and_then(|l| l.get("count").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if count != "0" {
        return Err(Error::new(format!(
            "the second document's Attachments panel reports count={count}, and this check \
             needs a document with none — it is about to assert that no replacement warning is \
             drawn, which is only meaningful when the destination is empty of that name. Pass \
             a --second-pdf with no embedded files."
        )));
    }
    report.note("★★ the second document is open and holds no attachments");

    // --- D: the Paste control is there, and the warning is not --------------
    let trace = session.trace()?;
    let Some(paste) = declared(&trace, ui_rect, PASTE_REGION) else {
        return Ok(Some(format!(
            "★★★ THERE IS NO PASTE CONTROL IN THE SECOND DOCUMENT: an attachment is on the \
             clipboard and `{PASTE_REGION}` is not declared. Regions beginning `attachments`: \
             {}.\n\
             If the Copy line was traced and this is absent, the clipboard is per-document — \
             which is the defect this check exists for, in its purest form. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "attachments")),
            session.trace_path().display()
        )));
    };
    // ★★ WHICHEVER CAME LAST, never "is one absent". See `REPLACES_REGION`.
    let verdict = declared_names(&trace, ui_rect, "attachments.paste.")
        .into_iter()
        .rfind(|n| n == REPLACES_REGION || n == FRESH_REGION);
    match verdict.as_deref() {
        Some(name) if name == REPLACES_REGION => {
            return Ok(Some(format!(
                "★★ THE REPLACEMENT WARNING IS DRAWN WHEN NOTHING WOULD BE REPLACED: \
                 `{REPLACES_REGION}` is the paste's latest verdict in a document whose \
                 census is 0.\n\
                 The note exists because `attach_file` retains-then-pushes, so a \
                 SAME-NAMED attachment is silently displaced. Drawn unconditionally it is \
                 worse than absent: an operator who sees it on every paste stops reading \
                 it, and it will be there on the one paste that does destroy something. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        }
        Some(_) => {}
        None => {
            return Ok(Some(format!(
                "★★ THE PASTE STATES NO VERDICT: neither `{REPLACES_REGION}` nor \
                 `{FRESH_REGION}` has ever been declared, so the control cannot say \
                 whether it is about to displace a file. One of the pair must be \
                 published every frame -- that is what makes the answer readable from a \
                 change-log trace at all. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    report.note("★★★ the Paste control crossed to the second document, with no false warning");

    // --- E: paste, and it reaches the engine --------------------------------
    driver.click_at(session.frame()?.declared_center(paste))?;
    session.settle(60);

    let trace = session.trace()?;
    let Some(pasted) = trace.last(PASTED) else {
        return Ok(Some(format!(
            "★★★ THE PASTE COMMITTED NOTHING: `{PASTE_REGION}` was clicked and no `{PASTED}` \
             line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if pasted.get("replacing") == Some("true") {
        return Ok(Some(format!(
            "★★ THE PASTE THINKS IT IS REPLACING SOMETHING: `{}`, in a document whose census \
             was 0. The `replacing` flag decides which of two outcome sentences the operator \
             reads, and getting it wrong in this direction tells them a file was displaced \
             when none was. Trace: {}.",
            pasted.raw,
            session.trace_path().display()
        )));
    }
    let after = trace
        .last(CENSUS)
        .and_then(|l| l.get("count").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if after == "0" {
        return Ok(Some(format!(
            "★★★ THE ENGINE WAS CALLED AND THE DOCUMENT DID NOT CHANGE: `{}` was traced and \
             the panel still reports count=0. The verb refused — most likely \
             `AttachmentTreeUnsupported`, which this shell cannot ask about in advance because \
             `AttachmentNotes` does not report the tree's shape. Read the status line in the \
             trace. Trace: {}.",
            pasted.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ …and the file landed in the second document: `{}`, count now {after}",
        pasted.raw
    ));
    Ok(None)
}

/// The panel's `count=`, or `None` when the panel has not drawn.
///
/// ★ `None` and `Some(0)` are different answers and the distinction is the
/// whole of the ask-then-toggle rule above: *"the panel is not on screen"* and
/// *"the panel is on screen and this document has no attachments"* look
/// identical to any check that collapses them, and the remedies are opposite.
fn census(session: &Session) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .last(CENSUS)
        .and_then(|l| l.get("count").map(str::to_owned))
        .and_then(|c| c.parse().ok()))
}
