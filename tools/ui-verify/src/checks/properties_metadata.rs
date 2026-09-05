//! `properties_metadata_round_trips` — typing a title reaches the document,
//! and undo takes it back out of the box as well as out of the file.
//!
//! # ⚠ NOT RUN by the session that last edited this file — 2026-09-05
//!
//! It was **passing** before that session and it has **not been re-run since**:
//! the machine's pointer and keyboard belonged to another track, and a driven
//! run cannot share them. What changed here is which control opens the surface
//! (see [`open_document_properties`]); nothing about the four assertions moved.
//! ⇒ A reader looking for evidence that the metadata panel still works after
//! the move will not find it in this file. Run it.
//!
//! # ★★★ The surface MOVED — the operator, 2026-09-05
//!
//! > *"the document properties are still always visible in the properties tab.
//! > it needs to get out of there and be in its own document properties tab."*
//!
//! The four `/Info` editors were the last section of the **Properties** panel
//! and are now a panel of their own, `crate::panels::docprops`, opened by
//! `file.document_properties` on File ▸ Document. So this check clicks a
//! different ribbon item and brings a different dock tab to the front. It
//! asserts the same four things about the same four boxes.
//!
//! ★ **The region names did not change** — `properties.info` and
//! `properties.info.N`. That was the moving session's decision and it is stated
//! at the constants: one change at a time, so that the next run's verdict is
//! readable rather than being a race between two edits neither of which was
//! driven.
//!
//! # The gap this closed originally
//!
//! `file.properties`' own tooltip commissioned this from S3 until the split:
//! *"The document's own title, author, subject and keywords, and the properties
//! of whatever is selected on the page."* Only the second half existed, on a
//! recorded blocker — *"needs a `/Info` accessor that `pdfcer-core` does not
//! expose on `Document` at all"* — which was true when written and false when
//! read.
//!
//! # ★ The assertion that is the whole check, and it is the SECOND one
//!
//! Not *"a commit was traced"*. That proves the keystrokes arrived and the
//! action was raised, and it is satisfied by a build where the value never
//! reaches the document at all.
//!
//! The real assertion is that **tabbing away a second time commits nothing**.
//! The panel's commit rule is *focus left AND the draft differs from what the
//! document holds*, so a second departure from an untouched field is silent
//! **only if the value is genuinely in the document now**. If it is not, the
//! draft still differs, and every focus change writes it again — a field that
//! looks edited, produces an undo entry per glance, and holds a value the file
//! does not have.
//!
//! # ★ And the third: undo has to reach the BOX, not only the file
//!
//! The drafts are re-seeded whenever `doc.edit_epoch` moves, and that is what
//! makes `Ctrl+Z` work here. Without it the box would still show the title the
//! document no longer has, and the next focus change would write it straight
//! back — **an undo the panel silently reverses**, which is worse than an undo
//! that does nothing because the operator watched it succeed.
//!
//! That failure is invisible to every unit test in the crate: `InfoDrafts::sync`
//! can be tested, `set_info_field` can be tested, and the epoch bump can be
//! tested, and the defect lives in whether the three are connected across a
//! frame boundary. It is the same shape as every other check in this
//! directory.
//!
//! # What this does NOT prove
//!
//! That the bytes in a saved file are right. `save_copy` and a second process
//! reading it back is what proves that, and `checks::save_copy` already owns
//! that shape. This proves the panel and the session agree, which is the link
//! that did not exist yesterday.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode this runs in. `file` is in every mode's tab list, so Review is
/// chosen only because the rest of the harness uses it.
const MODE: &str = "review";
/// The panel's own region.
///
/// ★ Still `properties.info` after the section became the Document properties
/// panel on 2026-09-05 — see `crate::panels::docprops`'s `REGION`, which
/// carries the reason the name was left alone.
const SECTION: &str = "properties.info";
/// The prefix of the per-field editor regions; the field's index in
/// `InfoField::all()` is appended. Index 0 is `/Title`, index 1 is `/Author`.
const FIELD: &str = "properties.info.";
/// The ribbon item that opens the panel, and the dock tab it mounts as.
///
/// ★★ It is a **toggle**, unlike the `file.properties` control this check used
/// to press: its question is *"is this panel open?"*, so it falls through
/// `app::dispatch`'s guard arm to `toggle_panel`. That is why the click below
/// is guarded by *"only if the section is not already on screen"* — pressing it
/// with the panel up would close the thing under test. The guard predates the
/// move and was written for the same hazard.
const COMMAND: &str = "file.document_properties";
/// The trace the panel emits when it decides to commit a draft.
const COMMITTED: &str = "info-field-commit";
/// The label `vector_edit` traces when `set_info_field` succeeded.
const APPLIED: &str = "set-info-field";
/// The keystrokes that spell the title. Every letter is already in `sys::vk`.
const TITLE_KEYS: [u16; 5] = [vk::T, vk::I, vk::T, vk::L, vk::E];

/// See the module documentation.
pub struct PropertiesMetadataRoundTrips;

impl Check for PropertiesMetadataRoundTrips {
    fn name(&self) -> &'static str {
        "properties_metadata_round_trips"
    }

    fn defect(&self) -> &'static str {
        "the Document properties panel shows no metadata, or shows boxes that do not reach the \
         file — so a title typed into one is written again on every focus change, or survives \
         an undo that removed it from the document, leaving the panel and the file disagreeing \
         about what the operator's document is called"
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

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon \
             control and two text fields, types five letters and presses Ctrl+Z. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("properties_metadata.trace.txt"));
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
    // ★★ MAXIMISE — at the harness's default 1,100 pt window the File tab's
    // last two groups fold away entirely and this check reports a lost
    // command. See `about.rs` for the measurement; three checks shared it.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: the panel on screen --------------------------------------------
    //
    // Only if it is not already, because `file.document_properties` is a panel
    // TOGGLE and pressing it when the panel is open would close the thing under
    // test. In Review's default arrangement the panel is mounted as the LAST
    // tab of the inspector stack, so on a fresh profile it is mounted, behind
    // a sibling, and publishing nothing — which is the case this click is for.
    if declared(&session.trace()?, ui_rect, SECTION).is_none() {
        open_document_properties(&session, &driver, ui_rect)?;
    }
    let trace = session.trace()?;
    if declared(&trace, ui_rect, SECTION).is_none() {
        return Ok(Some(format!(
            "no `{SECTION}` region after opening the Document properties panel, so the \
             metadata form did not draw. ★ Since 2026-09-05 this surface is a panel of its \
             own (`{COMMAND}`) rather than a section of the Properties panel: if the ribbon \
             item was pressed and nothing came up, check that `Panel::DocumentProperties` is \
             mounted by this mode's arrangement, not only that the section still draws. \
             Regions beginning `properties`: {}.",
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    let fields = declared_names(&trace, ui_rect, FIELD);
    if fields.len() < 2 {
        return Ok(Some(format!(
            "the section drew {} editable field(s) and the engine's `InfoField::all()` has \
             four. This check needs at least two — one to type into and one to move focus to. \
             Fields declared: {}.",
            fields.len(),
            list(&fields)
        )));
    }
    report.note(format!("{} metadata fields drawn", fields.len()));

    let title = declared(&trace, ui_rect, &format!("{FIELD}0")).ok_or_else(|| {
        Error::new(format!(
            "no `{FIELD}0` region — the Title field is not on screen."
        ))
    })?;
    let author = declared(&trace, ui_rect, &format!("{FIELD}1")).ok_or_else(|| {
        Error::new(format!(
            "no `{FIELD}1` region — the Author field is not on screen, and this check moves \
             focus to it in order to make Title commit."
        ))
    })?;

    // --- 2: type a title, then leave the field -----------------------------
    driver.click_at(session.frame()?.declared_center(title))?;
    session.settle(10);
    for key in TITLE_KEYS {
        driver.press(key)?;
        session.settle(3);
    }
    session.settle(8);

    // ★ Committing is what LEAVING the field does, not what typing does — the
    // rule is `lost_focus`, shared with the Forms panel and the canvas form
    // editor, because `TextEdit::changed()` fires per keystroke and one typed
    // word must not be a dozen undo entries. So the commit is provoked by
    // clicking the next field, which is what an operator does.
    let committed_before = session.trace()?.events(COMMITTED).count();
    driver.click_at(session.frame()?.declared_center(author))?;
    session.settle(18);

    let trace = session.trace()?;
    if trace.events(COMMITTED).count() <= committed_before {
        return Ok(Some(format!(
            "five letters were typed into the Title field and focus was moved to Author, and \
             no new `{COMMITTED}` line was traced. Either the keystrokes never reached the \
             field, or the commit rule never fired — and the two are told apart by whether \
             ANY `{COMMITTED}` line exists in this run at all."
        )));
    }
    if let Some(refusal) = trace
        .events(&format!("{APPLIED}-refused"))
        .filter_map(|l| l.get("detail").map(str::to_owned))
        .last()
    {
        return Ok(Some(format!(
            "the panel raised its action and the engine REFUSED it: {refusal}. The shell half \
             works; this is a `pdfcer-core` verdict and belongs in a request."
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the panel decided to commit (`{COMMITTED}` was traced) and no `{APPLIED}` line \
             followed, so the `Action` was raised and its apply arm never ran, or ran and \
             could not borrow the session. Nothing reached the document."
        )));
    }
    report.note("the typed title reached the document through the action funnel");

    // --- 3: ★ and it is REALLY in the document -----------------------------
    //
    // Go back to Title and leave it again without typing. The commit rule's
    // second condition is *the draft differs from what the document holds*, so
    // this is silent only if the value is genuinely stored. A build where the
    // action was raised and dropped commits again here, for ever, once per
    // focus change.
    let committed_before = trace.events(COMMITTED).count();
    driver.click_at(session.frame()?.declared_center(title))?;
    session.settle(10);
    driver.click_at(session.frame()?.declared_center(author))?;
    session.settle(16);

    let trace = session.trace()?;
    if trace.events(COMMITTED).count() > committed_before {
        return Ok(Some(
            "★ moving through the Title field WITHOUT TYPING committed again. The commit rule \
             is 'focus left AND the draft differs from the document', so a second commit means \
             the document does not hold what the panel just wrote to it. Every glance at this \
             field now produces an undo entry the operator did not earn, and the box shows a \
             title the file does not have."
                .to_owned(),
        ));
    }
    report.note("a second departure from the untouched field wrote nothing — the value is stored");

    // --- 4: ★ undo takes it out of the BOX, not only out of the file -------
    let applied_before = trace.events(APPLIED).count();
    let committed_before = trace.events(COMMITTED).count();
    // ★★ The QAT BUTTON, not `Ctrl+Z`, and the substitution is the fix for a
    // permanent SKIP.
    //
    // Synthetic chords do not reach the target window from this session —
    // `find_bar` and `page_ops` both record it, and `page_ops` puts the gap on
    // its own record rather than implying coverage it does not have. So this
    // phase skipped on every run it ever made, which means the assertion it
    // exists for — *does an undo take the value out of the BOX, or only out of
    // the file?* — has never once been made.
    //
    // Clicking the quick-access Undo is the same act. `undo_redo_round_trip`
    // already proves the chord and the button reach one dispatcher, and this
    // check is not about how the undo was raised: it is about what the PANEL
    // does when one lands. A phase that skips for ever because it insisted on
    // one of two equivalent routes is a phase that has quietly opted out.
    //
    // The chord stays as the fallback, so a build where the button vanishes
    // still has somewhere to go before it gives up.
    if let Some(rect) = declared(&trace, ui_rect, "ribbon.qat.edit.undo") {
        driver.click_at(session.frame()?.declared_center(rect))?;
    } else {
        driver.press_chord(&[vk::CONTROL], vk::Z)?;
    }
    session.settle(20);
    let trace = session.trace()?;
    if trace.last("undo").is_none() && trace.events(APPLIED).count() == applied_before {
        return Err(Error::new(
            "neither the quick-access Undo control nor Ctrl+Z produced an undo line, so step 4 \
             would be asserting nothing. Reported as SKIPPED rather than FAILED: on this \
             route that is the harness's input, not the panel.",
        ));
    }

    // Now move through the field again. If the drafts did NOT re-seed on the
    // epoch bump, the box still holds the typed title while the document holds
    // the original — so leaving the field writes it straight back, and the undo
    // the operator watched succeed is silently reversed.
    driver.click_at(session.frame()?.declared_center(title))?;
    session.settle(10);
    driver.click_at(session.frame()?.declared_center(author))?;
    session.settle(16);

    let trace = session.trace()?;
    if trace.events(COMMITTED).count() > committed_before {
        return Ok(Some(
            "★ after Ctrl+Z, moving through the Title field WROTE THE OLD VALUE BACK. The \
             panel's drafts did not re-seed when the edit epoch moved, so the box kept a title \
             the document no longer has — and the next focus change put it back. An undo the \
             operator watched succeed, silently reversed by looking at the panel."
                .to_owned(),
        ));
    }
    report.note("after undo the box follows the document — the drafts re-seed on the epoch");
    Ok(None)
}

/// Bring the Document properties panel up from the ribbon.
///
/// `file.document_properties`, on **File ▸ Document** — the band `RIBBON_IA.md`
/// §5.1 heads *"inspection of what is inside the file"*, which is where Fonts
/// and Properties already sit. A document's title is inside the file.
///
/// ★ It was `ribbon.item.file.properties` until 2026-09-05, when the operator
/// asked for the metadata to leave the selection inspector. The **first**
/// control in the Document band is now the one whose label is the phrase he
/// used, which is also why the check aims at a specific id rather than at "the
/// first item in the band": an item-position search would have silently kept
/// passing against the old control.
fn open_document_properties(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.file").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.file` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let item_region = format!("ribbon.item.{COMMAND}");
    let Some(item) = declared_or_in_overflow(session, driver, ui_rect, &item_region)? else {
        return Err(Error::new(format!(
            "the File tab declares no `{item_region}`, on the band or in the overflow, so the \
             Document properties panel cannot be opened. Items the File tab did declare: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.file."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);
    Ok(())
}
