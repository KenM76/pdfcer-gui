//! `checks::redaction` — mark a page, apply the redaction, and prove the text
//! is **gone from the bytes**
//!
//! # The defect this exists to catch
//!
//! `SALVAGE.md` records the core team's `Pass 72.0` warning in the strongest
//! terms any note in this project uses:
//!
//! > **A shell calling `redact::apply_redactions` directly and writing the
//! > bytes ships an unverified redaction and will not know.** … `pdfcer`'s
//! > `redact-apply` does exactly that at HEAD and exits `SUCCESS` on a file it
//! > never verified.
//!
//! *"Will not know"* is the whole problem, and it is why this check exists in a
//! harness rather than only in the crate's own test suite. A build that marked
//! a page, reported four regions removed, wrote a file and removed **nothing**
//! would:
//!
//! * pass every unit test that asserts the pipeline is called;
//! * emit a `redact-written` trace line indistinguishable from a correct one,
//!   because a trace line is written by the code under test, about itself;
//! * produce a file that opens, has the right page count, and looks right.
//!
//! The only thing that separates it from a correct build is **what is in the
//! bytes on disk**, read by something that is not the program that wrote them.
//!
//! # ★ The falsification, and where it is
//!
//! `HANDOFF.md` §2's sharpest lesson is defect 8: *"a test that checks a
//! relation rather than a magnitude is satisfied by any absurdity in the right
//! direction."* An absence check is the extreme case — it passes on an empty
//! file, on a file that was never written, and on a grep looking for the wrong
//! string.
//!
//! So the same byte scan is run **three** times and only one of the three is
//! the actual verdict:
//!
//! | run | over | must be | what a wrong answer means |
//! |---|---|---|---|
//! | 1 | the **fixture**, before anything | `SECRET` **present** | the instrument can register the secret at all. This is the falsifying phase: a build that marks and reports success without removing anything fails at run 3, and a *check* that could not see the secret in the first place fails here rather than passing vacuously |
//! | 2 | the **output** | `SURVIVOR` **present** | the scan is still a valid instrument on *this* file. Page 2 is untouched, so its text must still be findable — and if the writer had compressed the streams, this fails and says so, rather than letting run 3 pass because a deflate stream hid everything |
//! | 3 | the **output** | `SECRET` **absent** | **the verdict** |
//!
//! Runs 1 and 2 are not belt and braces. Without run 1 the check passes on a
//! fixture that never contained the secret; without run 2 it passes on any
//! output whose bytes the scan cannot read. Both were failure modes available
//! to the first draft.
//!
//! # ★ …and a fourth oracle, in a second process
//!
//! A raw byte scan is the harness's own reading of the file. The other honest
//! question is what **pdfcer** makes of it, and `checks::save_copy` established
//! the shape: re-open the written file in a second process and read the answer
//! out of its trace.
//!
//! Here that is `file.copy_page_text`, which runs the same extraction
//! `pdfcer extract-text` does, over the page the redaction covered. On the
//! fixture it traces `text-copied source=page chars=N`; on the redacted output
//! the same click must trace `text-copy-declined source=page
//! reason=nothing-to-copy`, because there is nothing left on that page to copy.
//!
//! That is the assertion an operator would actually make — *"I opened the file
//! and tried to copy the text out"* — made by a process that did not perform
//! the redaction and holds none of its state.
//!
//! # The fixture is generated, and every byte in it is the harness's
//!
//! Two pages, uncompressed, one distinctive string on each:
//!
//! * page 1 draws [`SECRET`], and is the page that gets marked;
//! * page 2 draws [`SURVIVOR`], and is never touched.
//!
//! Generated into the run's output directory rather than committed, for
//! `checks::save_copy`'s reason about stray files — and **two pages rather than
//! one**, which is the design decision that makes this check falsifiable
//! without a keyboard. A one-page fixture with the whole page marked has no
//! survivor, so a build that emitted a blank document would pass every absence
//! assertion. The second page is the negative control, and it is reached
//! without typing anything: the mark covers page 1 only.
//!
//! **No keyboard is used anywhere in this check.** Synthetic keystrokes do not
//! reach the target window from the session that writes them on this machine
//! (`HANDOFF.md` §8 records the investigation and the failed reproduction), so
//! the marking route this check drives is the one that needs no text entry:
//! *Mark whole page*. The search-and-mark route — which needs a query typed
//! into a field — is therefore **not covered here**, and that is stated rather
//! than left to be discovered. Its rule is unit-tested in
//! `crates/pdfcer-gui/src/app/actions/apply.rs`; what is not verified by driving
//! is the field itself.
//!
//! # What each phase would fail on
//!
//! | phase | fails when |
//! |---|---|
//! | A | the fixture's own text is not extractable by pdfcer, so the check has no baseline |
//! | B | `edit.redact` opens no panel, or the panel's controls publish no rects |
//! | C | marking traces no mark, so nothing after it means anything |
//! | D | the apply report never appears, or reports `verified=false` on a clean fixture |
//! | E | **the confirm control is live before the acknowledgement is given** — the gate that stands between an operator and the one irreversible operation in the program |
//! | F | no file was written |
//! | G | **the source file changed** — a redaction that wrote over the document it came from |
//! | H | the secret survives in the output, or the survivor does not |
//! | I | a second process can still extract the redacted page's text |

use std::path::{Path, PathBuf};

use super::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list,
    list_str, shell_trace,
};
use super::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode whose ribbon carries `edit.redact`.
///
/// Edit, and it is not interchangeable with the `review` every other driving
/// check uses: redaction is authoring, so `RIBBON_IA.md` puts it on the Edit
/// tab and Read and Review are not shown that tab at all. A check that stayed
/// in Review would find no control and SKIP, reporting a missing feature that
/// is merely on a tab it did not open.
const MODE: &str = "edit";

/// The Edit tab.
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");

/// The File tab, for the extraction oracle.
const FILE_TAB: (&str, &str) = ("ribbon.tab.file", "file");

/// The control that opens the marking panel.
const REDACT: (&str, &str) = ("ribbon.item.edit.redact", "edit.redact");

/// The control that copies the current page's text — the extraction oracle,
/// used once on the fixture and once on the redacted output.
const COPY_PAGE_TEXT: (&str, &str) = ("ribbon.item.file.copy_page_text", "file.copy_page_text");

/// The panel's whole-page marking control.
const WHOLE_PAGE_REGION: &str = "redact-whole-page";

/// The panel's control that opens the apply report.
const APPLY_REGION: &str = "redact-apply";

/// The dialog's mandatory acknowledgement checkbox.
const ACK_REGION: &str = "redact-apply-ack";

/// The dialog's confirm control.
///
/// ★ Declared by the application **only while it is enabled**, which is what
/// makes phase E possible at all: its absence from the trace is positive
/// evidence that the gate is closed, rather than the absence of evidence a
/// disabled-but-drawn control would leave.
const CONFIRM_REGION: &str = "redact-apply-confirm";

/// Every region name the redaction surfaces publish, for a SKIP reason.
const REGION_PREFIX: &str = "redact-";

/// `redact-panel marks=N pages=M epoch=E` — the marking census.
const PANEL_EVENT: &str = "redact-panel";

/// `redact-prepared marks=… verified=… …` — the removal ran in memory.
const PREPARED_EVENT: &str = "redact-prepared";

/// `redact-refused reason=…` — the apply declined before anything was written.
const REFUSED_EVENT: &str = "redact-refused";

/// `redact-written path=… verified=… …` — bytes reached the file system.
const WRITTEN_EVENT: &str = "redact-written";

/// `redact-write-failed path=… detail=…`.
const WRITE_FAILED_EVENT: &str = "redact-write-failed";

/// `text-copied source=page chars=N` — pdfcer's own extraction found text.
const COPIED_EVENT: &str = "text-copied";

/// `text-copy-declined source=page reason=nothing-to-copy` — it found none.
const COPY_DECLINED_EVENT: &str = "text-copy-declined";

/// The seam that answers the save dialog instead of opening it.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// **The string the redaction must remove**, drawn on page 1.
///
/// Deliberately long, upper-case and unlike anything a PDF producer emits: a
/// short token could be absent from an output file by luck, and a proof that
/// can pass by luck proves nothing. It is also well over
/// `redact::proof::MIN_VERIFIABLE_LEN`, so the raw-byte half of the
/// application's own proof has something to say about it.
const SECRET: &str = "CONFIDENTIALWITNESSALPHA";

/// **The string that must survive**, drawn on page 2.
///
/// The negative control, and the reason the fixture has two pages. Without it,
/// a build that wrote an empty document would satisfy every absence assertion
/// in this check.
const SURVIVOR: &str = "UNTOUCHEDWITNESSBETA";

/// See the module documentation.
pub struct RedactionRemovesAndProvesIt;

impl Check for RedactionRemovesAndProvesIt {
    fn name(&self) -> &'static str {
        "redaction_removes_and_proves_it"
    }

    fn defect(&self) -> &'static str {
        "Applying a redaction reports success and leaves the marked text in the saved file, \
         writes over the document it came from, or offers its irreversible confirm control \
         before the operator has acknowledged anything — none of which a passing test suite can \
         see, because the only honest evidence is the bytes on disk read by a second process"
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

// ---------------------------------------------------------------------------
// The instrument
// ---------------------------------------------------------------------------

/// Whether `hay` contains `needle` as a byte subsequence.
///
/// The harness's **own** scan, deliberately not shared with the application's.
/// `crate::redact::proof`'s local `contains` carries the same argument from the
/// other side: *"an absence proof that shared its search routine with the code
/// it is auditing would be a weaker proof."* Here the separation is stronger
/// still — this one is in a different crate, in a different process, over bytes
/// read back from the file system.
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > hay.len() {
        return false;
    }
    hay.windows(needle.len()).any(|w| w == needle)
}

/// A cheap content digest — length plus FNV-1a.
///
/// `checks::save_copy`'s, for its reason: the question is *"did this file
/// change"*, the adversary is a bug rather than a forger, and the length is
/// part of the digest so a truncation cannot hide behind a collision.
fn digest(bytes: &[u8]) -> (usize, u64) {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (bytes.len(), hash)
}

/// **Build the two-page fixture.**
///
/// A classic single-revision PDF with a correct cross-reference table, one
/// uncompressed content stream per page, and nothing else. Assembled here
/// rather than committed so that every byte in it is one this check put there —
/// which is what makes *"the secret is in the bytes"* and *"the secret is not
/// in the bytes"* two readings of the same instrument rather than two
/// assumptions about somebody's producer.
///
/// **Uncompressed on purpose.** The check's verdict is a byte scan, and a
/// `/FlateDecode` content stream would hide the secret from it — which is a
/// false pass. Phase H's survivor assertion is what would catch that if a
/// future writer compressed the *output*; keeping the input uncompressed is
/// what stops it arising in the first place.
fn fixture_bytes() -> Vec<u8> {
    let page_content = |text: &str| format!("BT /F1 18 Tf 40 120 Td ({text}) Tj ET");
    let stream = |text: &str| {
        let c = page_content(text);
        format!("<< /Length {} >>\nstream\n{c}\nendstream", c.len())
    };
    let page = |contents: u32| {
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 7 0 R >> >> /Contents {contents} 0 R >>"
        )
    };
    let bodies: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>".to_owned(),
        page(5),
        page(6),
        stream(SECRET),
        stream(SURVIVOR),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
    ];

    let mut buf = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in bodies.iter().enumerate() {
        offsets.push(buf.len());
        buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref_at = buf.len();
    let n = bodies.len() + 1;
    buf.extend_from_slice(format!("xref\n0 {n}\n0000000000 65535 f \n").as_bytes());
    for off in &offsets {
        buf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size {n} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n").as_bytes(),
    );
    buf
}

// ---------------------------------------------------------------------------
// Driving
// ---------------------------------------------------------------------------

/// Launch one process with both diagnostic channels armed and the save seam
/// set.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &Path,
    target: &Path,
    trace_name: &str,
) -> Result<Session> {
    let mut spec = LaunchSpec::new(
        ctx.resolve_exe().ok_or_else(|| {
            Error::new(format!(
                "no binary to drive. Pass --exe, or build the profile's default at {}.",
                ctx.profile.default_exe
            ))
        })?,
        ctx.out(trace_name),
    );
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} on {} as pid {} with {SAVE_PATH_ENV}={}",
        spec.exe.display(),
        pdf.display(),
        session.pid(),
        target.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the process \
             and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// How many times the shell has reported `id` invoked.
///
/// A count rather than a presence: this check clicks four different controls
/// across two processes, and *"has it ever been invoked?"* would be answered
/// `true` by a click made ten seconds earlier.
fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// Click a ribbon tab and confirm the shell reported it.
fn click_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region in `{MODE}`. Either this build does \
             not show that tab in this mode, or the tab strip is too narrow and it has moved into \
             the overflow menu — which this check cannot open. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let before = shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count()
        <= before
    {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{TAB_EVENT} tab={id}` line. The mode click \
             DID land, so pointer input works and this is not the input channel."
        )));
    }
    Ok(())
}

/// Locate a declared region and refuse a degenerate rectangle.
fn region(trace: &Trace, ui_rect: &str, name: &str, prefix: &str) -> Result<LRect> {
    let rect = declared(trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{name}` region, so there is nothing to aim at. \
             Regions declared under `{prefix}`: {}.",
            list(&declared_names(trace, ui_rect, prefix))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{name}` was declared at {rect:?}, which has no usable area. A click aimed at a \
             degenerate rectangle proves nothing."
        )));
    }
    Ok(rect)
}

/// Click a ribbon band control and confirm the **shell** reported the invoke.
fn click_command(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (name, id): (&str, &str),
    settle: u32,
) -> Result<()> {
    // ★★ **Three places a ribbon control can be**, and this asked about one.
    //
    // `region` reads a declared rect out of the trace, which is right for the
    // panel's and the dialog's controls — they are always drawn when their
    // surface is. It is wrong for a **ribbon item**: at the harness's 1,100 pt
    // window the band runs out of width, and an item may instead be inside a
    // collapsed group's popup or behind the overflow button. Neither publishes
    // a rect until it is opened.
    //
    // `file.copy_page_text` lives in the File tab's *Export* group, which
    // collapses at that width — so this check SKIPped with *"the application
    // declared no `ribbon.item.file.copy_page_text` region"*, which was true
    // and not the whole truth: `ribbon.group.file.export.collapsed` was in the
    // same trace.
    //
    // ★ `declared_or_in_overflow` is the one statement of "where can a ribbon
    // command be" and it already knew all three. This is the **second** copy of
    // that lookup found and removed on 2026-08-27; `settings_headings` was the
    // first, blind since the same ribbon change. A rule stated twice is a rule
    // that drifts, and this is what the drift looks like: nothing failed, the
    // checks simply stopped being able to begin.
    let rect = crate::checks::driving::declared_or_in_overflow(session, driver, ui_rect, name)?
        .ok_or_else(|| {
            Error::new(format!(
                "`{name}` was not on the band, in any collapsed group, or in the overflow, so \
                 there is nothing to aim at. Regions declared under `{ITEM_PREFIX}`: {}.",
                crate::checks::driving::list(&crate::checks::driving::declared_names(
                    &session.trace().unwrap_or_default(),
                    ui_rect,
                    ITEM_PREFIX
                ))
            ))
        })?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(settle);
    if invokes(session, id)? <= before {
        return Err(Error::new(format!(
            "`{name}` DID NOT TAKE THE CLICK: it was declared at {rect:?} and the click produced \
             no `{INVOKE_EVENT} id={id}`. A document with pages is open, so a greyed control is \
             the wrong reading for a `doc.pages` gate. Commands the shell reported invoked this \
             run: {}.",
            list_str(
                &shell_trace(session)?
                    .events(INVOKE_EVENT)
                    .filter_map(|l| l.get("id"))
                    .collect::<Vec<&str>>()
            )
        )));
    }
    Ok(())
}

/// Click a region the application published that is **not** a ribbon control.
///
/// The panel's and the dialog's own controls, which produce no
/// `ribbon-command-invoked` because they are not commands. There is therefore
/// no input-channel confirmation available here, and that is why every caller
/// asserts an application-side effect immediately afterwards: an unconfirmed
/// click plus an unchanged application is reported as a SKIP by the caller
/// rather than as a failure of the feature.
fn click_region(session: &Session, driver: &Driver, rect: LRect, settle: u32) -> Result<()> {
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(settle);
    Ok(())
}

/// The most recent marking census, as `(marks, pages)`.
fn census(trace: &Trace) -> Option<(usize, usize)> {
    let line = trace.last(PANEL_EVENT)?;
    Some((line.get_usize("marks")?, line.get_usize("pages")?))
}

/// **Copy the current page's text through the ribbon, and report what pdfcer's
/// own extraction found.**
///
/// `Ok(Some(chars))` when it copied something, `Ok(None)` when it declined
/// because there was nothing to copy. Used **twice** — once on the fixture and
/// once on the redacted output in a second process — which is the whole reason
/// it is a function: the two answers have to come from the identical sequence,
/// or the comparison at the end is between two different measurements.
fn extracted_chars(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    what: &str,
) -> Result<Option<usize>> {
    let copies_before = session.trace()?.events(COPIED_EVENT).count();
    let declines_before = session.trace()?.events(COPY_DECLINED_EVENT).count();
    click_tab(session, driver, ui_rect, FILE_TAB)?;
    click_command(session, driver, ui_rect, COPY_PAGE_TEXT, 16)?;

    let trace = session.trace()?;
    if trace.events(COPIED_EVENT).count() > copies_before {
        return Ok(trace.last(COPIED_EVENT).and_then(|l| l.get_usize("chars")));
    }
    if trace.events(COPY_DECLINED_EVENT).count() > declines_before {
        return Ok(None);
    }
    Err(Error::new(format!(
        "`{}` was invoked on {what} and traced neither `{COPIED_EVENT}` nor \
         `{COPY_DECLINED_EVENT}`, so this check has no extraction oracle. Trace: {}.",
        COPY_PAGE_TEXT.1,
        session.trace_path().display()
    )))
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions ----------------------------------------------------
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is eight clicks across two processes. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    // The fixture is generated rather than taken from `--pdf`, and that is not
    // negotiable for this check: its whole verdict is a byte scan for two
    // strings, and a document somebody else authored contains neither.
    let fixture: PathBuf = ctx.out("redaction-fixture.pdf");
    let source = fixture_bytes();
    std::fs::write(&fixture, &source)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", fixture.display())))?;
    let before = digest(&source);

    // ★ RUN 1 of the byte scan — the falsifying phase. See the module header's
    // table: if the instrument cannot see the secret in the file that was just
    // written from a constant containing it, then every absence it reports
    // afterwards is worthless, and this is where that is caught.
    if !contains(&source, SECRET.as_bytes()) {
        return Ok(Some(format!(
            "★ THE INSTRUMENT CANNOT SEE THE SECRET. `{SECRET}` was written into the fixture at \
             {} and a byte scan of the same file does not find it, so this check's absence \
             assertions could not fail and would pass against a build that removed nothing. This \
             is a harness defect rather than an application one, and it is reported as a FAILURE \
             so it cannot be mistaken for a pass.",
            fixture.display()
        )));
    }
    if !contains(&source, SURVIVOR.as_bytes()) {
        return Ok(Some(
            "★ the fixture does not contain its own negative control, so a build that emitted an \
             empty document would satisfy every assertion below. Harness defect, reported as a \
             failure."
                .to_owned(),
        ));
    }
    report.note(format!(
        "fixture {} — {} bytes, digest {:016x}; both `{SECRET}` (page 1) and `{SURVIVOR}` \
         (page 2) are present in its bytes",
        fixture.display(),
        before.0,
        before.1
    ));

    let target = ctx.out("redaction-applied.pdf");
    let _ = std::fs::remove_file(&target);
    if target == fixture {
        return Ok(Some(
            "the output path IS the fixture's path, so phase G would compare a file with itself. \
             Harness defect, reported as a failure so it cannot be mistaken for a pass."
                .to_owned(),
        ));
    }

    // =======================================================================
    // The marking process. Scoped, so `Drop` kills it before the second one
    // launches — two pdfcer windows competing for the foreground would make
    // every click after the first one a race.
    // =======================================================================
    let extracted_before;
    {
        let session = launch(ctx, report, &fixture, &target, "redaction.trace.txt")?;
        let driver = Driver::new(session.window());

        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);

        // --- PHASE A: pdfcer can read page 1's text ------------------------
        //
        // The baseline for phase I, and a precondition in its own right: a
        // fixture whose text this build cannot extract is one the redaction
        // search could not have found either, and every number below would be
        // measuring the extractor rather than the removal.
        extracted_before =
            extracted_chars(&session, &driver, ui_rect, "the fixture")?.ok_or_else(|| {
                Error::new(
                    "pdfcer extracted no text at all from page 1 of the fixture, so this check has \
                     no baseline and its phase-I assertion would pass on a build that removed \
                     nothing. The fixture draws one Helvetica string in an uncompressed content \
                     stream; if that is not extractable, the extractor is the subject and this \
                     check is not it."
                        .to_owned(),
                )
            })?;
        report.note(format!(
            "phase A: pdfcer's own extraction reads {extracted_before} character(s) from page 1 of \
             the fixture"
        ));

        // --- PHASE B: the marking panel -----------------------------------
        click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
        click_command(&session, &driver, ui_rect, REDACT, 20)?;
        let trace = session.trace()?;
        let whole_page =
            region(&trace, ui_rect, WHOLE_PAGE_REGION, REGION_PREFIX).map_err(|e| {
                Error::new(format!(
                    "{e}\n\
                     `{}` was invoked and the marking panel published no controls. Either the \
                     panel did not mount, or `Panel::Redact` is not reachable from this mode's \
                     dock arrangement.",
                    REDACT.1
                ))
            })?;
        if let Some((marks, _)) = census(&trace)
            && marks != 0
        {
            return Ok(Some(format!(
                "the panel reports {marks} redaction mark(s) on a freshly generated fixture that \
                 has none. The census is reading something other than this document."
            )));
        }

        // --- PHASE C: mark page 1 ------------------------------------------
        click_region(&session, &driver, whole_page, 18)?;
        let trace = session.trace()?;
        let Some((marks, pages)) = census(&trace) else {
            return Err(Error::new(format!(
                "the panel traced no `{PANEL_EVENT}` line after the whole-page click, so this \
                 check cannot tell a mark that was made from a click that missed. Trace: {}.",
                session.trace_path().display()
            )));
        };
        if marks == 0 {
            return Err(Error::new(format!(
                "the click on `{WHOLE_PAGE_REGION}` produced no mark (`{PANEL_EVENT} marks=0`). \
                 There is no input-channel confirmation for a panel control — it is not a command \
                 — so this is reported as a SKIP rather than as a failure of marking: the click \
                 may never have been delivered."
            )));
        }
        report.note(format!(
            "phase C: {marks} mark(s) on {pages} page(s) — `{}`",
            trace
                .last(PANEL_EVENT)
                .map_or_else(String::new, |l| l.raw.clone())
        ));

        // --- PHASE D: the apply report -------------------------------------
        let apply = region(&session.trace()?, ui_rect, APPLY_REGION, REGION_PREFIX)?;
        click_region(&session, &driver, apply, 20)?;
        let trace = session.trace()?;
        let Some(prepared) = trace.last(PREPARED_EVENT) else {
            if let Some(refused) = trace.last(REFUSED_EVENT) {
                return Ok(Some(format!(
                    "★ THE APPLY WAS REFUSED on a fixture that should redact cleanly: `{}`. Every \
                     variant of that refusal means no file was produced, which is the correct \
                     behaviour for the condition it names — so the finding is the condition, not \
                     the refusal.",
                    refused.raw
                )));
            }
            return Err(Error::new(format!(
                "the apply control was clicked and traced neither `{PREPARED_EVENT}` nor \
                 `{REFUSED_EVENT}`, so the dialog never opened and this check has no report to \
                 confirm. Trace: {}.",
                session.trace_path().display()
            )));
        };
        let prepared_line = prepared.raw.clone();
        let prepared_marks = prepared.get_usize("marks").unwrap_or(0);
        let verified = prepared.get("verified") == Some("true");
        report.note(format!("phase D: `{prepared_line}`"));
        if prepared_marks == 0 {
            return Ok(Some(format!(
                "★ THE REMOVAL APPLIED NO MARKS. The panel reported {marks} mark(s) and the apply \
                 reports `marks=0`, so the two censuses disagree — which is the un-saved-mark \
                 trap `crate::redact` §1.2 exists to close: applying against the BASE revision \
                 rather than the session graph applies nothing and reports success. `{prepared_line}`"
            )));
        }
        if !verified {
            return Ok(Some(format!(
                "★ THE ABSENCE PROOF DID NOT COME BACK CLEAN on a synthetic two-page fixture with \
                 one Helvetica string per page: `{prepared_line}`. A residual here means the \
                 redacted text still occurs somewhere in the output bytes, which on this fixture \
                 cannot be a coincidence."
            )));
        }

        // --- PHASE E: ★ the gate ------------------------------------------
        //
        // The confirm control is declared by the application only while it is
        // ENABLED, so its absence now is positive evidence that the
        // acknowledgement is doing its job — and its presence would be the
        // finding: the one irreversible operation in the program, reachable
        // before the operator has agreed to anything.
        if declared(&session.trace()?, ui_rect, CONFIRM_REGION).is_some() {
            return Ok(Some(format!(
                "★ THE CONFIRM CONTROL IS LIVE BEFORE ANYTHING WAS ACKNOWLEDGED. \
                 `{CONFIRM_REGION}` was declared with the report freshly opened and neither \
                 checkbox ticked, which means an operator can commit a permanent removal with a \
                 single click on a dialog they have not read. The gate is \
                 `RedactDialog::ready_to_confirm`."
            )));
        }
        report.note(
            "phase E: the confirm control is not offered until the acknowledgement is given"
                .to_owned(),
        );

        let ack = region(&session.trace()?, ui_rect, ACK_REGION, REGION_PREFIX)?;
        click_region(&session, &driver, ack, 16)?;
        let confirm =
            region(&session.trace()?, ui_rect, CONFIRM_REGION, REGION_PREFIX).map_err(|e| {
                Error::new(format!(
                    "{e}\n\
                     The acknowledgement was clicked and the confirm control is still not \
                     offered. Either the click missed the checkbox — there is no input-channel \
                     confirmation for one — or the report carries residuals this fixture should \
                     not produce, in which case the second acknowledgement is also required."
                ))
            })?;

        // --- PHASE F: write ------------------------------------------------
        click_region(&session, &driver, confirm, 24)?;
        let trace = session.trace()?;
        let Some(written) = trace.last(WRITTEN_EVENT) else {
            if let Some(failed) = trace.last(WRITE_FAILED_EVENT) {
                return Ok(Some(format!(
                    "★ THE WRITE WAS REFUSED AFTER THE OPERATOR CONFIRMED: `{}`. Every variant of \
                     that refusal means no file was produced — including the one that matters \
                     most, a proof that failed between the buffer and the syscall.",
                    failed.raw
                )));
            }
            return Ok(Some(format!(
                "★ NOTHING WAS WRITTEN. The confirm control was clicked and there is no \
                 `{WRITTEN_EVENT}` and no `{WRITE_FAILED_EVENT}` line, so either the click did \
                 not land on a control that had just been declared, or the save seam \
                 {SAVE_PATH_ENV} was not consulted and a real modal is waiting behind this \
                 window. Trace: {}.",
                session.trace_path().display()
            )));
        };
        report.note(format!("phase F: `{}`", written.raw));
        if written.get("verified") != Some("true") {
            return Ok(Some(format!(
                "★ the file was written with `verified=false`: `{}`. On this fixture that is a \
                 residual the application could not classify, and the operator has been handed a \
                 file it does not vouch for.",
                written.raw
            )));
        }
    }

    // =======================================================================
    // The evidence: the bytes, read by the harness, from the file system
    // =======================================================================
    if !target.is_file() {
        return Ok(Some(format!(
            "★ the application traced a successful write and there is no file at {}. The trace is \
             written by the code under test, about itself; this is the half that is not.",
            target.display()
        )));
    }
    let output = std::fs::read(&target)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", target.display())))?;

    // --- PHASE G: ★ the source is untouched -------------------------------
    //
    // The most damaging thing this shell could do, and the one that cannot be
    // undone by anybody: overwriting the file the content was removed from
    // destroys the only remaining copy of it.
    let after = std::fs::read(&fixture)
        .map_err(|e| Error::new(format!("cannot re-read {}: {e}", fixture.display())))?;
    if digest(&after) != before {
        return Ok(Some(format!(
            "★★ THE REDACTION WROTE OVER THE DOCUMENT THAT WAS OPENED. {} was {} bytes with \
             digest {:016x} before the run and is {} bytes with digest {:016x} after it. That \
             destroys the only remaining copy of the content the operator was removing, on the \
             one operation least able to survive a mistake. `crate::redact` §4 and \
             `crate::dialogs::redact::suggested_path` are what this is supposed to be prevented \
             by.",
            fixture.display(),
            before.0,
            before.1,
            digest(&after).0,
            digest(&after).1
        )));
    }
    report.note("phase G: the document that was opened is byte-for-byte unchanged".to_owned());

    // --- PHASE H: ★ RUNS 2 AND 3 of the byte scan -------------------------
    //
    // Run 2 first, deliberately. It establishes that the scan is a valid
    // instrument on THIS file before run 3 uses it to report an absence — see
    // the module header's table. A writer that compressed its streams would
    // hide both strings equally, and without this the check would read that as
    // a successful redaction.
    if !contains(&output, SURVIVOR.as_bytes()) {
        return Ok(Some(format!(
            "★ THE NEGATIVE CONTROL IS MISSING FROM THE OUTPUT. `{SURVIVOR}` is drawn on page 2, \
             which no mark covered, and it is not in {} — so either the redaction destroyed a \
             page it was never asked to touch, or the output's streams are compressed and this \
             check's byte scan can no longer see anything in them. Both readings make the absence \
             assertion below meaningless, which is why this is reported before it rather than \
             after.",
            target.display()
        )));
    }
    if contains(&output, SECRET.as_bytes()) {
        return Ok(Some(format!(
            "★★ THE REDACTED TEXT IS STILL IN THE SAVED FILE. `{SECRET}` was marked, the \
             application reported it removed and verified absent, and a byte scan of {} finds it. \
             This is the failure `SALVAGE.md`'s Pass 72.0 note is about, arriving through the \
             one path that was supposed to make it impossible.",
            target.display()
        )));
    }
    report.note(format!(
        "phase H: {} is {} bytes — `{SECRET}` is absent from them and `{SURVIVOR}` is present",
        target.display(),
        output.len()
    ));

    // =======================================================================
    // PHASE I: ★ a SECOND PROCESS, which performed no redaction and holds
    // none of the first one's state, cannot extract the redacted page's text
    // =======================================================================
    {
        let session = launch(
            ctx,
            report,
            &target,
            // A second save path, set and never used. Harmless, and cheaper
            // than a second launch helper that differs in one field.
            &ctx.out("redaction-unused.pdf"),
            "redaction.reopen.trace.txt",
        )?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);

        let canvas = session
            .trace()?
            .last(ctx.profile.vocab.canvas_event)
            .map(|l| l.raw.clone());
        let Some(canvas) = canvas else {
            return Ok(Some(format!(
                "★ the redacted file at {} did not draw in a fresh process — the canvas traced \
                 nothing at all. A redaction that produces a file pdfcer cannot open has removed \
                 the content and the document with it.",
                target.display()
            )));
        };
        report.note(format!("phase I: the redacted file re-opens — `{canvas}`"));

        match extracted_chars(&session, &driver, ui_rect, "the redacted output")? {
            None => {
                report.note(
                    "phase I: a second process extracts NOTHING from the redacted page — \
                     `text-copy-declined reason=nothing-to-copy`"
                        .to_owned(),
                );
            }
            Some(chars) => {
                return Ok(Some(format!(
                    "★★ A SECOND PROCESS STILL EXTRACTS {chars} CHARACTER(S) FROM THE REDACTED \
                     PAGE. pdfcer read {extracted_before} character(s) from page 1 of the fixture \
                     and reads {chars} from page 1 of {} — through the same extraction \
                     `pdfcer extract-text` uses, which is the tool an operator would reach for \
                     to get the text back out. The byte scan in phase H found nothing, so the \
                     text has survived in a form that scan cannot see.",
                    target.display()
                )));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two strings this check turns on are in the fixture it generates.**
    ///
    /// The falsifying phase, asserted at build time as well as at run time.
    /// Phase 1 of the byte scan reports a harness defect if this is ever false;
    /// this catches the same thing without launching anything, which is where a
    /// developer who has just edited [`fixture_bytes`] will see it.
    #[test]
    fn the_generated_fixture_contains_both_strings() {
        let bytes = fixture_bytes();
        assert!(
            contains(&bytes, SECRET.as_bytes()),
            "the secret is not in the fixture, so every absence assertion in this check would \
             pass vacuously"
        );
        assert!(
            contains(&bytes, SURVIVOR.as_bytes()),
            "the negative control is not in the fixture, so a build that emitted an empty \
             document would pass"
        );
        assert!(
            bytes.starts_with(b"%PDF-1.7"),
            "the fixture must be a PDF at all"
        );
        assert!(
            bytes.ends_with(b"%%EOF\n"),
            "…with a trailer the parser can find"
        );
    }

    /// ★ **The two strings cannot be confused with each other, or with
    /// anything a producer emits.**
    ///
    /// A shared prefix would make the survivor's presence satisfy a scan for
    /// the secret, which would turn the check's verdict into its own negative
    /// control. Asserted rather than eyeballed because the two constants sit
    /// four lines apart and are deliberately similar in shape.
    #[test]
    fn the_secret_and_the_survivor_are_unrelated_strings() {
        assert!(!SECRET.contains(SURVIVOR));
        assert!(!SURVIVOR.contains(SECRET));
        assert!(!SECRET.starts_with(&SURVIVOR[..4]));
        assert!(
            SECRET.len() >= 8 && SURVIVOR.len() >= 8,
            "a short token can be absent from a file by luck, and a proof that can pass by luck \
             proves nothing"
        );
    }

    /// **The instrument registers a presence and an absence.**
    ///
    /// `contains` is the whole verdict of phases 1–3, and a scan that always
    /// answered `false` would make the check pass against every build. Both
    /// directions, plus the empty-needle case, which must be `false` — the
    /// mathematically correct answer (`true`) would make every run report a
    /// leak.
    #[test]
    fn the_byte_scan_answers_both_ways() {
        assert!(contains(b"abcdefg", b"cde"));
        assert!(!contains(b"abcdefg", b"xyz"));
        assert!(!contains(b"abc", b"abcd"), "a needle longer than the hay");
        assert!(!contains(b"abc", b""), "an empty needle matches nothing");
        assert!(contains(b"abc", b"abc"), "the whole hay");
    }

    /// The digest notices a flipped bit and a truncation.
    ///
    /// Phase G's whole verdict rests on it, and phase G is the one that catches
    /// the most damaging failure available to this feature.
    #[test]
    fn the_digest_notices_a_change() {
        let a = fixture_bytes();
        let mut b = a.clone();
        let last = b.len() - 10;
        b[last] ^= 0x01;
        assert_ne!(digest(&a), digest(&b), "a flipped bit");
        b.truncate(a.len() - 1);
        assert_ne!(digest(&a), digest(&b), "a truncation");
        assert_eq!(digest(&a), digest(&fixture_bytes()), "and is stable");
    }

    /// The region and command names match what the application publishes.
    ///
    /// Spelling, and it is not a formality: these strings are matched literally
    /// against the application's `ui-rect` declarations, so a rename on either
    /// side silently un-aims every click this check makes and the check reports
    /// a missing feature that is merely spelled differently.
    #[test]
    fn the_selectors_match_the_applications_own_names() {
        assert_eq!(REDACT.0, format!("{ITEM_PREFIX}{}", REDACT.1));
        assert_eq!(
            COPY_PAGE_TEXT.0,
            format!("{ITEM_PREFIX}{}", COPY_PAGE_TEXT.1)
        );
        for name in [WHOLE_PAGE_REGION, APPLY_REGION, ACK_REGION, CONFIRM_REGION] {
            assert!(
                name.starts_with(REGION_PREFIX),
                "`{name}` is not under the prefix this check lists for its SKIP reasons, so a \
                 failure to find it would report an unhelpfully empty list"
            );
        }
        assert_eq!(EDIT_TAB.0, format!("ribbon.tab.{}", EDIT_TAB.1));
        assert_eq!(FILE_TAB.0, format!("ribbon.tab.{}", FILE_TAB.1));
    }
}
