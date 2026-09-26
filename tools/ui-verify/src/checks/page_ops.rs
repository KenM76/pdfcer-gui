//! `page_ops_round_trip` — **the check for the tab that did nothing**.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_ops.md`.

use std::path::Path;

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, UNIMPLEMENTED_EVENT, declared,
    declared_names, list, list_str, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// **Review**, and the choice is load-bearing rather than conventional.
const MODE: &str = "review";

/// The tab carrying every verb under test. Shown in Review and Edit, not Read.
const PAGES_TAB: (&str, &str) = ("ribbon.tab.pages", "pages");

/// The tab carrying Save a copy. Shown in every mode.
const FILE_TAB: (&str, &str) = ("ribbon.tab.file", "file");

/// **Phase B's control.** Right rather than left so the expected `/Rotate` value
/// is `90`, which is the shortest unambiguous string to find in the bytes.
const ROTATE: (&str, &str) = ("ribbon.item.pages.rotate_right", "pages.rotate_right");

/// **Phase C's control.** Down rather than up because the current page is page 1
/// and `move_up` on page 1 is the *refusal* this check deliberately does not
/// exercise — `panels::pages::ops` owns that boundary and asserts it headlessly.
const MOVE_DOWN: (&str, &str) = ("ribbon.item.pages.move_down", "pages.move_down");

/// **Phase D's control**, and the destructive one.
const DELETE: (&str, &str) = ("ribbon.item.pages.delete", "pages.delete");

/// **Phase A2's control.**
const EXTRACT: (&str, &str) = ("ribbon.item.pages.extract", "pages.extract");

/// `extract path=… pages=… bytes=… asked=…` — the extraction reached a write.
const EXTRACT_APPLIED: &str = "extract";

/// The command that puts the whole run on disk.
const SAVE: (&str, &str) = ("ribbon.item.file.save_copy", "file.save_copy");

/// `open ok pages=N path=…` — the page count, from the flattened page tree, in
/// whichever process emitted it.
const OPEN_EVENT: &str = "open";

/// The field on [`OPEN_EVENT`] that carries the count.
const PAGES_FIELD: &str = "pages";

/// `rotate-pages page=… n=… epoch=… disclosures=…` — `vector_edit`'s line for
/// the rotate arm. Its presence means the **engine** ran, not merely that a
/// control was pressed.
const ROTATE_APPLIED: &str = "rotate-pages";

/// `reorder-pages page=0 n=… epoch=… …`
const REORDER_APPLIED: &str = "reorder-pages";

/// `delete-pages page=… n=… epoch=… …`
const DELETE_APPLIED: &str = "delete-pages";

/// `pages-resync was=… now=… renumbered=… page=… epoch=…` — **this check's
/// sharpest in-process oracle.**
const RESYNC_EVENT: &str = "pages-resync";

/// `save-copy path=… bytes=… appended=… …`
const SAVED_EVENT: &str = "save-copy";

/// `save-copy-failed path=… detail=…`
const FAILED_EVENT: &str = "save-copy-failed";

/// The seam that answers the save dialog. Shared with `save_copy_round_trip`.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// What phase H looks for in the saved copy's bytes.
const ROTATED_90: &[u8] = b"/Rotate 90";

/// What phase H requires the **source** not to contain, so its evidence is
/// unambiguous.
const ANY_ROTATE: &[u8] = b"/Rotate";

/// The fewest pages this check can run against.
const MIN_PAGES: usize = 3;

/// See the module documentation.
pub struct PageOpsRoundTrip;

impl Check for PageOpsRoundTrip {
    fn name(&self) -> &'static str {
        "page_ops_round_trip"
    }

    fn defect(&self) -> &'static str {
        "The whole Pages tab does nothing: rotate, move and delete are registered, drawn, \
         offered by a context menu and bound to chords, and every press traces \
         `command-unimplemented` — or reaches the engine and leaves the shell's page vector, its \
         rasters and its selections describing a document that no longer exists"
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

/// A cheap content digest — length plus FNV-1a over the bytes.
fn digest(bytes: &[u8]) -> (usize, u64) {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (bytes.len(), hash)
}

/// How many times `needle` occurs in `haystack`.
fn occurrences(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

/// How many times the shell has reported `id` invoked.
fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// The page count the application reported when it opened its document.
fn opened_pages(trace: &Trace) -> Option<usize> {
    trace.last(OPEN_EVENT)?.get_usize(PAGES_FIELD)
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
             not show that tab in this mode — `shell::manifest`'s Review tab list is \
             `file, view, pages, markup, measure` — or the tab strip is too narrow and it has \
             moved into the overflow menu, which this check cannot open because a menu's \
             contents are not published as regions. Tabs declared: {}.",
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
             DID land, so pointer input works and this is not the input channel; the likely \
             cause is that the tab moved between the frame that declared its rect and the frame \
             that received the click."
        )));
    }
    Ok(())
}

/// Locate a declared control and refuse a degenerate rectangle.
fn control(trace: &Trace, ui_rect: &str, name: &str) -> Result<LRect> {
    let rect = declared(trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the active tab publishes its controls' rects and none of them is `{name}`. Either \
             this build does not have that command on that tab, or the band overflowed and the \
             group was not drawn — which `save_copy_round_trip` records happening to the Markup \
             tab on a 2384 pt-wide sheet. Controls declared: {}.",
            list(&declared_names(trace, ui_rect, ITEM_PREFIX))
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

/// Click a band control and confirm the **shell** reported the invoke.
fn click_command(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
    settle: u32,
) -> Result<()> {
    let rect = control(&session.trace()?, ui_rect, region)?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(settle);
    if invokes(session, id)? <= before {
        return Err(Error::new(format!(
            "`{region}` DID NOT TAKE THE CLICK: it was declared at {rect:?} and the click \
             produced no `{INVOKE_EVENT} id={id}`. A document with pages is open, so a greyed \
             control is the wrong reading for the `doc.pages` gate every `pages.*` command \
             carries. Commands the shell reported invoked this run: {}.",
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

/// Build the failure text for a page verb that was invoked and did nothing.
fn no_effect(session: &Session, (region, id): (&str, &str), applied: &str) -> Result<String> {
    let trace = session.trace()?;
    let unimplemented = trace
        .events(UNIMPLEMENTED_EVENT)
        .any(|l| l.get("id") == Some(id));
    Ok(format!(
        "`{region}` WAS INVOKED AND THE ENGINE NEVER RAN. The shell traced \
         `{INVOKE_EVENT} id={id}` and there is no `{applied}` line.\n{}\n{}\nTrace: {}.",
        if unimplemented {
            format!(
                "★ The application traced `{UNIMPLEMENTED_EVENT} id={id}`, which is \
                 `dispatch_command`'s fall-through: the command arrived and dispatch had NO ARM \
                 for it. That is the exact state this check was written against — look at \
                 `app/dispatch.rs`'s `\"{id}\"` arm."
            )
        } else {
            format!(
                "There is no `{UNIMPLEMENTED_EVENT}` for it, so the command did reach an arm and \
                 the failure is further down: `page_operands` may have returned `None`, the \
                 `Action` may not be raised, or `vector_edit` may have refused — it traces \
                 `…-refused` with a reason when it does."
            )
        },
        match trace.events("pages-declined").last() {
            Some(l) => format!("The dispatch arm declined: `{}`.", l.raw),
            None => "There is no `pages-declined` line, so the arm found an operand.".to_owned(),
        },
        session.trace_path().display()
    ))
}

/// The most recent resync line, or a failure sentence explaining its absence.
fn resync_after(session: &Session, since: usize) -> Result<Option<crate::trace::TraceLine>> {
    Ok(session.trace()?.events(RESYNC_EVENT).nth(since).cloned())
}

/// How many resync lines have been emitted so far.
fn resyncs(session: &Session) -> Result<usize> {
    Ok(session.trace()?.events(RESYNC_EVENT).count())
}

/// Launch one process with both diagnostic channels armed and the save seam set.
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
        "launched {} on {} as pid {}",
        spec.exe.display(),
        pdf.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Every `pages.*` command is gated on `doc.pages`, so with nothing open the \
             controls are correctly greyed and this check would be measuring the gate rather \
             than the feature.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is six clicks across two processes. \
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

    let source = std::fs::read(&pdf)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", pdf.display())))?;
    let before_digest = digest(&source);

    // Phase H's precondition, checked BEFORE anything is driven so the SKIP
    // costs nothing. A fixture that already carries a `/Rotate` entry would
    // make "the copy contains `/Rotate 90`" evidence of the fixture rather than
    // of this run's edit, and a check whose evidence is ambiguous is worse than
    // one that did not run.
    let rotates_in_source = occurrences(&source, ANY_ROTATE);
    if rotates_in_source > 0 {
        return Err(Error::new(format!(
            "{} already contains {rotates_in_source} `/Rotate` entr(y/ies). Phase H proves the \
             rotation reached the file by finding `/Rotate 90` in the saved copy and NOT in the \
             source; on this document that evidence would be indistinguishable from the \
             fixture's own furniture. Point --pdf at a document with no page rotation — \
             `D:\\Dev\\pdfcer\\fixtures\\synthetic\\pageops\\four-pages.pdf` is the one this project uses.",
            pdf.display()
        )));
    }

    let target = ctx.out("page-ops-round-trip.pdf");
    let _ = std::fs::remove_file(&target);
    if target == pdf {
        return Ok(Some(
            "the copy's path IS the fixture's path, so phases F and G would compare a file with \
             itself and prove nothing. This is a harness defect rather than an application one, \
             and it is reported as a failure so it cannot be mistaken for a pass."
                .to_owned(),
        ));
    }

    // =======================================================================
    // The editing process. Scoped, so it is killed before the second one
    // launches — two pdfcer windows competing for the foreground would make
    // every click after the first one a race.
    // =======================================================================
    let pages_before;
    {
        let session = launch(ctx, report, &pdf, &target, "page_ops.trace.txt")?;
        let driver = Driver::new(session.window());

        // --- PHASE A: the baseline every later count is measured against ---
        pages_before = opened_pages(&session.trace()?).ok_or_else(|| {
            Error::new(format!(
                "the application traced no `{OPEN_EVENT} ok {PAGES_FIELD}=N` line, so this check \
                 has no baseline. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
        if pages_before < MIN_PAGES {
            return Err(Error::new(format!(
                "{} has {pages_before} page(s) and this check needs at least {MIN_PAGES}: one to \
                 rotate and keep, one to delete, and one left over so the engine's \
                 \"§7.7.3.3 requires at least one page\" refusal is nowhere near.",
                pdf.display()
            )));
        }
        report.note(format!(
            "fixture {} — {pages_before} pages, {} bytes, digest {:016x}, no `/Rotate` anywhere",
            pdf.display(),
            before_digest.0,
            before_digest.1
        ));

        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, PAGES_TAB)?;

        // --- PHASE A2: extract the current page to a new document --------
        //
        // First, and its result is deliberately thrown away: phase E's save
        // writes to the **same** path, because `pick_save_path` has one
        // diagnostic seam and both verbs go through it. What this phase can
        // therefore prove is the link nothing else in the workspace covers —
        // **that a ribbon click reaches the picker and the picker's answer
        // reaches a write** — and it proves it against the file rather than
        // against the trace, by reading the bytes while the process is still
        // running.
        //
        // What it deliberately does NOT prove is which pages are in that file.
        // Counting them needs a page-tree walk this crate has no parser for,
        // and re-opening it in a second process would cost a third launch to
        // assert something `app::actions::pages`' unit tests already assert by
        // writing a file and loading it back. The division is stated rather
        // than implied: the JOIN is here, the CONTENT is there.
        let _ = std::fs::remove_file(&target);
        click_command(&session, &driver, ui_rect, EXTRACT, 30)?;
        let trace = session.trace()?;
        let Some(extracted) = trace.last(EXTRACT_APPLIED) else {
            return Ok(Some(no_effect(&session, EXTRACT, EXTRACT_APPLIED)?));
        };
        if extracted.get_usize("pages") != Some(1) {
            return Ok(Some(format!(
                "THE EXTRACTION TOOK THE WRONG NUMBER OF PAGES: `{}`. Nothing is picked in the \
                 Pages panel and one page is on screen, so `panels::pages::ops::operands`' \
                 documented rule — act on the current page — should have named exactly one. A \
                 build that fell back to the whole document would report `pages={pages_before}`.",
                extracted.raw
            )));
        }
        match std::fs::read(&target) {
            Ok(bytes) if bytes.starts_with(b"%PDF-") => {
                report.note(format!(
                    "★ extract: a ribbon click reached the picker and the picker's answer \
                     reached a write — {} bytes of freestanding PDF at {}, `{}`",
                    bytes.len(),
                    target.display(),
                    extracted.raw
                ));
            }
            Ok(bytes) => {
                return Ok(Some(format!(
                    "THE EXTRACTION WROTE SOMETHING THAT IS NOT A PDF: {} bytes at {} beginning \
                     {:?}. `pageops::extract` returns the complete bytes of a freestanding PDF — \
                     own header, own cross-reference table, own trailer — so a file that does \
                     not start `%PDF-` means the wrong buffer was written.",
                    bytes.len(),
                    target.display(),
                    String::from_utf8_lossy(&bytes[..bytes.len().min(8)])
                )));
            }
            Err(e) => {
                return Ok(Some(format!(
                    "★ THE EXTRACTION WROTE NO FILE. The application traced `{}` — so the arm \
                     ran, the picker answered and `pageops::extract` produced bytes — and \
                     nothing is readable at {}: {e}.\n\n\
                     {SAVE_PATH_ENV} named that path, so the picker's answer and the write's \
                     destination have come apart.",
                    extracted.raw,
                    target.display()
                )));
            }
        }

        // --- PHASE B: rotate the current page ----------------------------
        let seen = resyncs(&session)?;
        click_command(&session, &driver, ui_rect, ROTATE, 24)?;
        if session.trace()?.last(ROTATE_APPLIED).is_none() {
            return Ok(Some(no_effect(&session, ROTATE, ROTATE_APPLIED)?));
        }
        let Some(line) = resync_after(&session, seen)? else {
            return Ok(Some(format!(
                "THE ROTATION DID NOT REACH THE PAGE VECTOR. `{ROTATE_APPLIED}` was traced, so \
                 the engine ran — and `{RESYNC_EVENT}` was not, which means \
                 `app::actions::pages::resync` compared the page vector the shell holds against \
                 `EditSession::pages()` and found them identical. The canvas renders from that \
                 vector, so the sheet on screen would keep drawing the way it was."
            )));
        };
        // The assertion no other check in the suite makes.
        if line.get("renumbered") != Some("0") {
            return Ok(Some(format!(
                "★ A ROTATION WAS TREATED AS A RENUMBERING: `{}`.\n\n\
                 A rotation rewrites one `/Rotate` entry per page and adds or removes no page, \
                 so every index still names the same sheet — `canvas::interact`'s measured rule, \
                 one structure up. Reporting `renumbered=1` here makes the resync throw away the \
                 operator's canvas selection on every turn of a sheet, which is silent, valid, \
                 and costs them work they had done.",
                line.raw
            )));
        }
        report.note(format!("rotate: `{}`", line.raw));

        // --- PHASE C: send the current page one place later --------------
        let seen = resyncs(&session)?;
        click_command(&session, &driver, ui_rect, MOVE_DOWN, 24)?;
        if session.trace()?.last(REORDER_APPLIED).is_none() {
            return Ok(Some(no_effect(&session, MOVE_DOWN, REORDER_APPLIED)?));
        }
        let Some(line) = resync_after(&session, seen)? else {
            return Ok(Some(format!(
                "THE REORDER DID NOT REACH THE PAGE VECTOR. `{REORDER_APPLIED}` was traced and \
                 `{RESYNC_EVENT}` was not, so the shell's page vector still describes the OLD \
                 order — every index in the application now names the wrong sheet, and the next \
                 phase's Delete would remove one nobody chose."
            )));
        };
        if line.get("renumbered") != Some("1") {
            return Ok(Some(format!(
                "★ A REORDER WAS NOT TREATED AS A RENUMBERING: `{}`.\n\n\
                 Every page survives a reorder, so a resync watching only the page COUNT would \
                 report exactly this — and would leave the canvas selection and every cached \
                 strip raster pointing at sheets that have moved. The signal that separates the \
                 two is the sequence of page-object identities; see \
                 `app::actions::pages::resync`.",
                line.raw
            )));
        }
        if line.get_usize("now") != Some(pages_before) {
            return Ok(Some(format!(
                "A REORDER CHANGED THE PAGE COUNT: `{}`. It moved pages; it must remove none.",
                line.raw
            )));
        }
        report.note(format!("move down: `{}`", line.raw));

        // --- PHASE D: delete the current page ----------------------------
        let seen = resyncs(&session)?;
        click_command(&session, &driver, ui_rect, DELETE, 28)?;
        if session.trace()?.last(DELETE_APPLIED).is_none() {
            return Ok(Some(no_effect(&session, DELETE, DELETE_APPLIED)?));
        }
        let Some(line) = resync_after(&session, seen)? else {
            return Ok(Some(format!(
                "★ THE DELETE DID NOT REACH THE PAGE VECTOR. `{DELETE_APPLIED}` was traced — the \
                 engine removed the page — and `{RESYNC_EVENT}` was not, so the shell still \
                 holds {pages_before} pages. The panel would keep saying so, the status bar \
                 would keep saying `n/{pages_before}`, and the canvas would keep rendering a \
                 `Page` whose object the engine has FREED."
            )));
        };
        let expected = pages_before - 1;
        if line.get_usize("now") != Some(expected) {
            return Ok(Some(format!(
                "THE DELETE LEFT THE WRONG NUMBER OF PAGES: `{}`, where {expected} were \
                 expected. One page was on screen and nothing was picked in the Pages panel, so \
                 `panels::pages::ops::operands`' documented rule — act on the current page — \
                 should have named exactly one.",
                line.raw
            )));
        }
        if line.get("renumbered") != Some("1") {
            return Ok(Some(format!(
                "A DELETE WAS NOT TREATED AS A RENUMBERING: `{}`. Every index above the removed \
                 page now names a different sheet, and the canvas selection that survives is a \
                 pointer at objects nobody chose with `format.delete` one keystroke away.",
                line.raw
            )));
        }
        report.note(format!("delete: `{}`", line.raw));

        // --- PHASE E: save a copy ------------------------------------------
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        click_command(&session, &driver, ui_rect, SAVE, 40)?;
        let trace = session.trace()?;
        let Some(saved) = trace.last(SAVED_EVENT) else {
            return Ok(Some(format!(
                "NOTHING WAS WRITTEN, so none of the three edits can be followed onto disk. \
                 `{}` was invoked and no `{SAVED_EVENT}` line followed. {}\nThat is \
                 `save_copy_round_trip`'s subject rather than this one's — read its verdict \
                 first. Trace: {}.",
                SAVE.1,
                match trace.last(FAILED_EVENT) {
                    Some(l) => format!("The write was ATTEMPTED and refused: `{}`.", l.raw),
                    None => format!("There is no `{FAILED_EVENT}` either, so no write was tried."),
                },
                session.trace_path().display()
            )));
        };
        report.note(format!("the copy was written: `{}`", saved.raw));
        if saved.get_usize("appended").unwrap_or(0) == 0 {
            return Ok(Some(format!(
                "THE SAVE APPENDED NOTHING after three structural edits: `{}`. \
                 `save_incremental`'s contract says `appended=0` happens only for an empty dirty \
                 set, so either the write ran against a session that does not carry the edits or \
                 it wrote the base document's bytes.",
                saved.raw
            )));
        }
    }

    // =======================================================================
    // PHASES F, G and H — what is on disk, asserted by the harness rather than
    // by the process that wrote it
    // =======================================================================
    if !target.is_file() {
        return Ok(Some(format!(
            "THE SAVED FILE IS NOT WHERE IT WAS ASKED FOR. The application traced \
             `{SAVED_EVENT}` and nothing exists at {}.",
            target.display()
        )));
    }
    let copy = std::fs::read(&target)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", target.display())))?;
    report.artifact(target.clone());

    // --- PHASE F: the document that was opened is untouched ---------------
    //
    // It matters more for this check than for `save_copy`'s, and the reason is
    // the verb: a page DELETE is the one operation in this application that
    // destroys an operator's sheets. If it reached the file they opened, the
    // sheets are gone from the only copy they had.
    let after_bytes = std::fs::read(&pdf)
        .map_err(|e| Error::new(format!("cannot re-read {}: {e}", pdf.display())))?;
    let after_digest = digest(&after_bytes);
    if after_digest != before_digest {
        return Ok(Some(format!(
            "★★ THE DOCUMENT THAT WAS OPENED HAS BEEN MODIFIED, and a page was DELETED during \
             this run. {} was {} bytes (digest {:016x}) before and is {} bytes (digest {:016x}) \
             after.\n\nThe operator's sheets are gone from the file they opened. \
             `file.save_copy`'s tooltip promises *\"The original is never overwritten unless you \
             pick it\"*, and {SAVE_PATH_ENV} named somewhere else.",
            pdf.display(),
            before_digest.0,
            before_digest.1,
            after_digest.0,
            after_digest.1
        )));
    }
    report.note(format!(
        "★ the opened document is byte-identical after a run that deleted one of its pages — {} \
         bytes, digest {:016x}",
        after_digest.0, after_digest.1
    ));

    // --- PHASE G: the copy is an APPENDED revision, not a rewrite -----------
    if !copy.starts_with(&source) {
        let shared = copy
            .iter()
            .zip(source.iter())
            .take_while(|(a, b)| a == b)
            .count();
        return Ok(Some(format!(
            "THE COPY IS A FULL REWRITE, NOT AN APPENDED UPDATE. The source is {} bytes and the \
             copy is {} bytes; they agree for the first {shared} and then diverge. That is \
             `save_copy_round_trip`'s phase E rather than this check's subject — read its \
             verdict — but it is asserted here too because phase H's evidence depends on it: \
             `/Rotate 90` found in a full rewrite could have come from anywhere in the document.",
            source.len(),
            copy.len()
        )));
    }

    // --- PHASE H: the rotation is IN THE FILE ---------------------------
    let rotated = occurrences(&copy, ROTATED_90);
    if rotated == 0 {
        return Ok(Some(format!(
            "★ THE ROTATION IS NOT IN THE SAVED FILE. The source carries no `/Rotate` entry at \
             all and the copy carries no `{}` either, so nothing in the appended revision says \
             any page was turned.\n\n\
             The application traced `{ROTATE_APPLIED}`, which means the engine ran and the \
             session recorded the command — so this is the save's or the session's finding, not \
             the ribbon's. The plausible build it catches is one that spun the RASTER rather \
             than the page: the sheet turns on screen, every count is right, and the file the \
             operator sends out is unrotated.",
            String::from_utf8_lossy(ROTATED_90)
        )));
    }
    report.note(format!(
        "★ the saved copy's appended revision carries `{}` {rotated} time(s), and the source \
         carries no `/Rotate` at all — the turn an OS-injected click asked for is in the file",
        String::from_utf8_lossy(ROTATED_90)
    ));

    // =======================================================================
    // PHASE I — THE ROUND TRIP: re-open the file that came out
    // =======================================================================
    let reopened_pages = {
        let session = launch(
            ctx,
            report,
            &target,
            &ctx.out("page-ops-unused.pdf"),
            "page_ops.reopen.trace.txt",
        )?;
        opened_pages(&session.trace()?).ok_or_else(|| {
            Error::new(format!(
                "the second process traced no `{OPEN_EVENT} ok {PAGES_FIELD}=N` line for the \
                 saved copy, so this check has no round-trip oracle. Trace: {}.",
                session.trace_path().display()
            ))
        })?
    };

    let expected = pages_before - 1;
    if reopened_pages != expected {
        return Ok(Some(format!(
            "★★ THE DELETE IS NOT IN THE SAVED FILE. The fixture had {pages_before} pages; one \
             was deleted and the shell's own resync agreed; and a SECOND process, opening the \
             saved copy from disk through the same `Document::load` an operator's File ▸ Open \
             uses, reports {reopened_pages} — {expected} were expected.\n\n\
             A save that writes a file is not the same claim as a save that writes the EDIT. The \
             plausible build this catches writes the **base document's** bytes: it produces a \
             valid PDF, it leaves the original alone (phase F passes) and it trivially begins \
             with the original's bytes (phase G passes, because it IS them)."
        )));
    }
    report.note(format!(
        "★★ ROUND TRIP PROVEN: a second process opened the saved copy from disk and its page \
         tree has {reopened_pages} pages, one fewer than the fixture's {pages_before}. The \
         delete, the reorder that decided WHICH sheet was deleted, and the rotation that \
         survived both are in the file the second process read"
    ));

    report.note(
        "NOT covered here: the four chords (`[`, `]`, `Alt+Up`, `Alt+Down`). Synthetic \
         keystrokes do not reach the target window from this session (see find_bar), so they are \
         covered by the manifest's keymap test and by the single dispatcher every route shares, \
         and the gap is on the record rather than implied by a green result",
    );
    report.note(
        "NOT covered here: the page tile's context menu, which offers these same six verbs. An \
         egui popup declares no ui-rect regions, so there is nothing to aim a click at — and \
         both routes reach `PdfcerApp::dispatch_command`, which is what one choke point is for",
    );
    report.note(
        "NOT covered here: what is INSIDE the extracted file. `pages.extract` and \
         `file.save_copy` share one PDFCER_DIAG_SAVE_PATH seam, so phase E overwrote phase A2's \
         file. The join — click to picker to write, landing a freestanding PDF — is phase A2's; \
         the content is covered by `app::actions::pages`' unit tests, which write an extraction \
         and load it back, including one carrying an unsaved rotation",
    );

    let _ = std::fs::remove_file(ctx.out("page-ops-unused.pdf"));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names this check greps for are the ones `egui-shell` builds, and the
    /// ids are the ones the application registers.
    #[test]
    fn the_selectors_match_the_shells_own_spelling() {
        for (region, id) in [ROTATE, MOVE_DOWN, DELETE, EXTRACT, SAVE] {
            assert_eq!(region, format!("ribbon.item.{id}"));
            assert!(region.starts_with(ITEM_PREFIX), "{region}");
        }
        assert_eq!(PAGES_TAB.0, format!("ribbon.tab.{}", PAGES_TAB.1));
        assert_eq!(FILE_TAB.0, format!("ribbon.tab.{}", FILE_TAB.1));
        // The three verbs live on Pages and the save lives on File, so the
        // check has to cross between two tabs. If these ever became the same
        // tab the second `click_tab` would be a no-op the shell does not report
        // and phase E would SKIP for a confusing reason.
        assert_ne!(PAGES_TAB.1, FILE_TAB.1);
        for (_, id) in [ROTATE, MOVE_DOWN, DELETE, EXTRACT] {
            assert!(
                id.starts_with(PAGES_TAB.1),
                "a command id names its owning tab, so `{id}` must share `{}`'s prefix",
                PAGES_TAB.1
            );
        }
        assert_eq!(MODE, "review");
    }

    /// **The occurrence counter really counts, and really finds nothing when
    /// there is nothing.**
    #[test]
    fn the_occurrence_counter_reads_both_directions() {
        assert_eq!(occurrences(b"a/Rotate 90 b/Rotate 90", ROTATED_90), 2);
        assert_eq!(occurrences(b"/Rotate 180", ROTATED_90), 0);
        assert_eq!(
            occurrences(b"/Rotate 180", ANY_ROTATE),
            1,
            "the source precondition must notice a rotation this check does not expect"
        );
        assert_eq!(occurrences(b"", ROTATED_90), 0);
        assert_eq!(
            occurrences(b"/Rot", ROTATED_90),
            0,
            "a haystack shorter than the needle must not index out of bounds"
        );
        // …and the overlap case, so the windowing is not accidentally skipping.
        assert_eq!(occurrences(b"aaaa", b"aa"), 3);
    }

    /// **The digest notices a single changed byte and a truncation.**
    #[test]
    fn the_digest_notices_a_single_changed_byte_and_a_truncation() {
        let a = b"%PDF-1.7 four sheets";
        let mut b = a.to_vec();
        b[9] ^= 0x01;
        assert_ne!(digest(a), digest(&b), "one flipped bit must change it");
        assert_ne!(
            digest(a),
            digest(&a[..a.len() - 1]),
            "a truncation must change it, which is what the length is in the tuple for"
        );
        assert_eq!(digest(a), digest(a), "and it must be stable");
    }

    /// The two streams are parsed out of one file without contaminating each
    /// other, and every field this check reads is read from the line the
    /// application really writes.
    #[test]
    fn every_field_this_check_reads_is_parsed_from_a_real_line() {
        let text = "pdfcer-diag start argv1=None\n\
                    pdfcer-diag open ok pages=4 path=\"D:\\\\jobs\\\\Sheet 1.pdf\"\n\
                    egui-shell-diag ribbon-command-invoked id=pages.delete handler=310\n\
                    pdfcer-diag rotate-pages page=0 n=1 epoch=1 disclosures=none\n\
                    pdfcer-diag pages-resync was=4 now=4 renumbered=0 page=1 epoch=1\n\
                    pdfcer-diag delete-pages page=0 n=1 epoch=3 disclosures=none\n\
                    pdfcer-diag pages-resync was=4 now=3 renumbered=1 page=1 epoch=3";
        let app = Trace::parse(text, "pdfcer-diag");
        let shell = Trace::parse(text, driving::SHELL_TRACE_PREFIX);

        assert!(app.started("start"));
        assert!(
            app.events(INVOKE_EVENT).next().is_none(),
            "the shell's line must not be read as the application's"
        );
        assert!(
            shell
                .events(INVOKE_EVENT)
                .any(|l| l.get("id") == Some(DELETE.1))
        );

        // The page count survives a path with SPACES in it. The application
        // Debug-quotes the path for exactly this reason; `pages=` sits BEFORE
        // it on the line, which is why this parses either way — asserted so a
        // future reordering of that line's fields cannot silently break the
        // baseline this whole check is measured against.
        assert_eq!(opened_pages(&app), Some(4));

        // The two resync lines are distinguishable by position, which is how
        // the phases tell "the line my click produced" from "a line from an
        // earlier phase" — the same count-not-presence rule `invokes` follows.
        let resyncs: Vec<&crate::trace::TraceLine> = app.events(RESYNC_EVENT).collect();
        assert_eq!(resyncs.len(), 2);
        assert_eq!(resyncs[0].get("renumbered"), Some("0"));
        assert_eq!(resyncs[0].get_usize("now"), Some(4));
        assert_eq!(resyncs[1].get("renumbered"), Some("1"));
        assert_eq!(resyncs[1].get_usize("now"), Some(3));
        assert_eq!(
            app.last(ROTATE_APPLIED).and_then(|l| l.get_usize("n")),
            Some(1)
        );
        assert_eq!(
            app.last(DELETE_APPLIED).and_then(|l| l.get("epoch")),
            Some("3")
        );
    }
}
