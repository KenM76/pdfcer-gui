//! `a_save_that_would_produce_blank_pages_is_refused` — **the operator's own
//! bug report, driven through the running binary.**
//!
//! ⬜ **NOT DRIVEN BY ITS AUTHOR.** This module was written on 2026-09-05 and
//! **has not been run once.** The canvas-input track owned the pointer for the
//! whole of the session that produced it and two driven runs corrupt each
//! other, so the suite was deliberately not invoked. Every sentence below about
//! what the trace will contain is a *prediction from the source*, not an
//! observation, and R1 is explicit that a passing unit test is not a substitute
//! for one. The first session with a free machine should run this and correct
//! whatever it got wrong — including, quite possibly, the `INVOKE` chain.
//!
//! # What this is for
//!
//! The operator, 2026-09-05:
//!
//! > *"I tested deleting pages from a pdf. when I open the document in Acrobat
//! > there are blank pages at the end of the document equalling the number of
//! > pages I deleted."*
//!
//! `pdfcer-core` v0.38.0's `delete_pages` decrements `/Count` on the removed
//! page's **immediate parent** and on no ancestor above it, so on any document
//! whose page tree has more than one level the root goes on declaring the
//! pre-delete page count. Acrobat builds its page list from the root `/Count`;
//! pdfcer walks `/Kids`. `crate::pagetree` — in the shell — refuses the save
//! rather than handing him the file.
//!
//! ## ★★★ Why this needs a driven check at all, when the unit tests are green
//!
//! Because the unit tests can only prove that `write_copy` refuses when it is
//! called with a stale document. They cannot prove that **pressing the delete
//! command and then pressing save in the running program** reaches `write_copy`
//! at all — and the whole class of defect this project keeps finding is a
//! correct mechanism that nothing calls. `check-verb-coverage.sh` exists
//! because of exactly that, and `RESUME.md`'s standing note is blunter: *"a note
//! is not a mechanism"*.
//!
//! The chain being asserted here has four links, and the failure of any one of
//! them is invisible to `cargo test`:
//!
//! 1. `pages.delete` has a dispatch arm and reaches `EditSession::delete_pages`;
//! 2. `file.save_copy` reaches [`crate::app::save::write_copy`];
//! 3. the guard runs there, on the funnel, and refuses;
//! 4. **the operator is told**, on an off-canvas surface, in a sentence.
//!
//! ## ★★★ The fixture is PINNED, and a flat one would make this vacuous
//!
//! `--pdf` is ignored. This check opens `fixtures/nested-page-tree.pdf` and
//! nothing else, and says so in its notes when a `--pdf` was supplied and
//! thrown away — because a sweep that silently ignored a flag is
//! indistinguishable from one that honoured it
//! (`three_clicks_round_a_hole_measure_the_hole`'s standing rule).
//!
//! The pin is not convenience. On a **flat** page tree the immediate parent
//! *is* the root, so a writer that updates only the immediate parent updates
//! the root by accident and **the defect cannot occur**. Run against
//! `fixtures/four-pages.pdf` this check would watch a page delete, watch a
//! clean save, and pass — reporting that a build carrying the defect in full is
//! sound. That exact substitution was performed on the unit-test half on
//! 2026-09-05 and it did precisely that, including printing a sentence
//! asserting the engine had been fixed. So the fixture is pinned, and the
//! `levels=` field of the `save-pagetree` trace line is read back and asserted
//! ≥ 3 before any verdict is reached.
//!
//! ## The oracle: three things, and the third is the one no trace can fake
//!
//! | # | assertion | what its failure means |
//! |---|---|---|
//! | 1 | `save-pagetree … levels=4 bad=2` appears | the guard ran, on a tree deep enough to exhibit the defect, and saw it |
//! | 2 | **no file exists at the target** | the guard traced a refusal and wrote the file anyway |
//! | 3 | the `status-group:edit-disclosure` region is on screen | he was **told**, rather than left with a save that silently did nothing |
//!
//! ★★ Assertion 3 is the one this project has learned to insist on. A refusal
//! with no sentence is this shell's founding defect shape — a control that is
//! pressed and does nothing — and it is worse here than usual, because the
//! operator has just deleted pages and is pressing save: a silence reads as
//! *"it saved"*, and he goes looking for a file that is not there.
//!
//! ★ The region carries a **rect**, not the text. `status::disclosure` publishes
//! `ui_rect(region, rect)` and no more, so this check can prove a sentence is on
//! screen and cannot prove which sentence. The words are asserted headlessly in
//! `crate::text::pagetree::tests`, which is the right split: the catalog owns
//! the wording, the harness owns the reachability.
//!
//! ## What this does NOT cover
//!
//! **That Acrobat shows blank pages.** That claim is the diagnosis, it was made
//! with `pdfcer dump-object` and an independent page-tree walk, and it is
//! recorded in the engine request. Nothing in this repository can drive Acrobat.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Delete a page, then save a copy — the operator's own two presses.
///
/// ★ `mode.edit` first, for the reason every driven check in this suite gives:
/// driving from a named mode makes the run reproducible rather than dependent
/// on whatever mode the last session left behind. It is also load-bearing here
/// rather than merely tidy — `pages.delete` changes page content, and Read's
/// posture is that the document is not the operator's to alter.
///
/// ★★ `pages.delete` with nothing picked acts on the **current page**, which is
/// page 1 — a defined answer rather than a disabled state
/// (`app::dispatch::pages`' own note). Page 1 hangs from node `A1` under `A`
/// under the root, so removing it leaves two ancestors stale and not one, which
/// is what makes `bad=2` below a stronger assertion than `bad>=1`.
const INVOKE: &str = "mode.edit,pages.delete,file.save_copy";
/// The variable that answers the save picker.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
/// The line `app::actions::pages` writes when pages leave the document.
const DELETED: &str = "pages-deleted";
/// The line `app::save::write_copy` writes on **every** save, clean or not.
const AUDITED: &str = "save-pagetree";
/// The line it writes when it refuses.
const REFUSED: &str = "save-refused-pagetree";
/// The line a successful save writes — its presence here would be the defect.
const SAVED: &str = "save-copy";
/// The status bar's rule-4 disclosure row.
const DISCLOSURE: &str = "status-group:edit-disclosure";
/// The fixture, pinned. See the module header.
const FIXTURE: &str = "fixtures/nested-page-tree.pdf";
/// The shallowest page tree on which this check's question is meaningful.
///
/// Root, an intermediate node, another intermediate node, leaves = 4 levels in
/// the fixture. Three is the floor at which an ancestor **above** the immediate
/// parent exists at all, and below it a build with no upward walk produces a
/// correct file.
const MIN_LEVELS: u32 = 3;

/// See the module documentation.
pub struct ASaveThatWouldProduceBlankPagesIsRefused;

impl Check for ASaveThatWouldProduceBlankPagesIsRefused {
    fn name(&self) -> &'static str {
        "a_save_that_would_produce_blank_pages_is_refused"
    }

    fn defect(&self) -> &'static str {
        "deleting pages and saving produces a file that opens in Acrobat with blank pages at the \
         end, one for each page removed — and pdfcer's own reader cannot see anything wrong with \
         it, so the shell hands it over without a word"
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

/// Read one `key=value` field out of a trace line.
///
/// The trace is `key=value` pairs separated by spaces, and every value this
/// check reads is a bare integer or `true`/`false` — no quoting, no spaces. A
/// field that is absent returns `None` rather than a default, because a build
/// whose trace line lost a field must be distinguishable from one whose field
/// is zero.
fn field<'a>(raw: &'a str, key: &str) -> Option<&'a str> {
    raw.split_whitespace()
        .find_map(|pair| pair.strip_prefix(key)?.strip_prefix('='))
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

    // ★★★ The fixture is PINNED and `--pdf` is discarded — see the header. Said
    // out loud in the notes, because a sweep that silently ignored a flag is
    // indistinguishable from one that honoured it.
    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is not on disk. Regenerate it with \
             `python tools/gen-nested-page-tree-fixture.py`; this check cannot use an arbitrary \
             document, because on a flat page tree the defect under test CANNOT OCCUR and the \
             check would pass against a build carrying it in full."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was supplied and is IGNORED: this check pins {FIXTURE}, whose three-level \
             page tree is the only shape on which an ancestor above the immediate parent can go \
             stale"
        ));
    }

    // ★ Deleted first, and that is not tidiness: a file left by a previous run
    // would make assertion 2 fail on a build that is behaving perfectly, and —
    // worse — a file the run itself wrote would be indistinguishable from one
    // that was already there.
    let target = ctx.out("pagetree-guard-copy.pdf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "{} could not be removed before the run, so a file found afterwards would prove \
             nothing. SKIPPED rather than failed.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("pagetree-guard.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env.push((
        SAVE_PATH_ENV.to_owned(),
        target.to_string_lossy().into_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} on the pinned nested fixture",
        exe.display(),
        session.pid()
    ));
    session.settle(90);

    let trace = session.trace()?;

    // --- link 1: the delete happened ---------------------------------------
    let Some(deleted) = trace.events(DELETED).last() else {
        return Ok(Some(format!(
            "★ NO PAGE WAS DELETED: `{INVOKE}` was invoked and no `{DELETED}` line appeared, so \
             nothing below tests anything. Either `pages.delete` has no dispatch arm — which is \
             defect D1's shape and has happened to six page verbs in this shell before — or the \
             command was refused by the mode. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the delete reached the document: `{}`",
        deleted.raw
    ));

    // --- link 2 and 3: the guard ran, and on a deep enough tree -------------
    let Some(audited) = trace.events(AUDITED).last() else {
        return Ok(Some(format!(
            "★★★ THE PAGE-TREE GUARD DID NOT RUN: pages were deleted, a save was invoked, and no \
             `{AUDITED}` line appeared.\n\
             That line is written on EVERY save, clean or not, for exactly this reason — a clean \
             audit and an audit that never ran are otherwise the same silence. So its absence \
             means either `file.save_copy` never reached `app::save::write_copy`, or the guard \
             was removed from it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the guard ran: `{}`", audited.raw));

    match field(&audited.raw, "walked") {
        Some("true") => {}
        other => {
            return Ok(Some(format!(
                "★★ THE GUARD COULD NOT WALK THE FILE IT WAS ABOUT TO WRITE: `{AUDITED}` reports \
                 walked={other:?}.\n\
                 A save whose own output does not re-parse is a writer defect of a different \
                 kind, and the guard deliberately does NOT refuse on it (a skip narrows the \
                 evidence rather than fabricating it) — so this check cannot reach a verdict. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        }
    }

    let levels: u32 = field(&audited.raw, "levels")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if levels < MIN_LEVELS {
        return Ok(Some(format!(
            "★★★ THE DOCUMENT UNDER TEST HAS A {levels}-LEVEL PAGE TREE AND THIS CHECK IS \
             MEANINGLESS ON ANYTHING SHALLOWER THAN {MIN_LEVELS}.\n\
             On a flat tree the immediate parent IS the root, so a writer with no upward walk at \
             all produces a correct file and every assertion below would pass against the broken \
             build. Either the pinned fixture was regenerated flat, or the shell opened a \
             different document from the one this check handed it. Regenerate with \
             `python tools/gen-nested-page-tree-fixture.py`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the page tree is {levels} levels deep, so it CAN carry the defect — which is the \
         thing a flat fixture silently could not"
    ));

    // --- the verdict --------------------------------------------------------
    let refused = trace.events(REFUSED).last();
    let saved = trace.events(SAVED).last();
    let bad: u32 = field(&audited.raw, "bad")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if bad == 0 {
        // Not a failure of this shell. Either the engine was fixed — in which
        // case the file on disk should be sound and this check has outlived its
        // subject — or the delete did not damage the tree for some reason this
        // check should say rather than guess.
        let file_is_sound = std::fs::read(&target).is_ok_and(|b| b.starts_with(b"%PDF-"));
        return Err(Error::new(format!(
            "the guard found nothing wrong after a page delete on a {levels}-level tree \
             (`{}`), and a file {} written.\n\
             SKIPPED rather than failed: this is what a FIXED `pdfcer-core` looks like. If \
             `delete_pages` now decrements /Count on every ancestor, close \
             `request_delete_pages_leaves_ancestor_count_stale_on_a_nested_page_tree.md`, delete \
             this check, and delete the engine half of \
             `app::save::tests::deleting_a_page_from_a_nested_document_is_caught_at_the_save`. \
             The guard itself stays. Trace: {}.",
            audited.raw,
            if file_is_sound { "WAS" } else { "was NOT" },
            session.trace_path().display()
        )));
    }

    if refused.is_none() {
        return Ok(Some(format!(
            "★★★ THE GUARD SAW THE DAMAGE AND DID NOT REFUSE: `{}` reports bad={bad}, and there \
             is no `{REFUSED}` line{}.\n\
             The audit is being computed and thrown away. That is worse than not having the \
             guard: the shell now knows the file it is writing is damaged and writes it anyway. \
             Trace: {}.",
            audited.raw,
            saved.map_or_else(String::new, |s| format!(" — instead there is `{}`", s.raw)),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the save was refused: `{}`",
        refused.expect("checked above").raw
    ));

    // --- assertion 2: nothing on disk ---------------------------------------
    if let Ok(bytes) = std::fs::read(&target) {
        return Ok(Some(format!(
            "★★★ THE SAVE WAS REFUSED AND {} BYTES REACHED THE DISK ANYWAY, at {}.\n\
             The refusal happens BEFORE `std::fs::write` by construction — the guard returns \
             `Err` and the write is the next statement — so a file here means either a second \
             write path exists or the refusal is being reported after the fact. A refusal that \
             left a partial file would be the same failure with a smaller file. Trace: {}.",
            bytes.len(),
            target.display(),
            session.trace_path().display()
        )));
    }
    report.note("★★ and nothing reached the disk — the damaged file was never written".to_owned());

    // --- assertion 3: he was TOLD -------------------------------------------
    let trace = session.trace()?;
    if declared(&trace, ui_rect, DISCLOSURE).is_none() {
        return Ok(Some(format!(
            "★★★ THE SAVE WAS REFUSED IN SILENCE: no `{DISCLOSURE}` region was declared.\n\
             He has just deleted pages and pressed save. A save that produces no file and no \
             sentence is indistinguishable from one that worked, so he goes looking for a file \
             that is not there — and the file he does NOT have is the one thing he cannot \
             discover by looking at the page. `app::save::page_tree_refusal_note` is the call \
             site; `crate::text::pagetree` is the sentence. Regions beginning `status-group`: \
             {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "status-group")),
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ and he was TOLD — the status bar's rule-4 disclosure row is on screen. Its WORDS \
         are asserted headlessly in `crate::text::pagetree::tests`; this region carries a rect \
         and no text, so what is proved here is that a sentence is reachable, not which one."
            .to_owned(),
    );

    Ok(None)
}
