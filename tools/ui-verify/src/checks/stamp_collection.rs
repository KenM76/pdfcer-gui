//! **Acrobat custom stamps, driven end to end** — both halves of
//! `OPERATOR_REQUESTS.md` **O169** against the running binary.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/stamp_collection.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, frame_of, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The collection: three stamps, one of them dynamic, category *Site Review*.
const COLLECTION: &str = "stamp-collection.pdf";

/// The control: an ordinary four-page document with no name tree.
const PLAIN: &str = "four-pages.pdf";

/// The disclosure section's region in Document properties.
const SECTION_REGION: &str = "properties.stamp-collection";
/// The prefix of the per-stamp row regions inside it.
const ROW_PREFIX: &str = "properties.stamp-collection.";
/// How many rows the fixture's name tree must produce.
const STAMPS: usize = 3;
/// The metadata form's region — the panel's *other* content, used here only as
/// proof that the panel itself is open on the control launch.
const PANEL_OPEN_WITNESS: &str = "properties.info";
/// The mode this drives in. `file` is in every mode's tab list; Review is
/// chosen because the rest of the harness uses it.
const MODE: &str = "review";
/// The trace line the shell emits once a document is open. Its **absence** is
/// how a launch that opened nothing is told apart from a build that discloses
/// nothing — two sweeps in this project produced confident, detailed, entirely
/// wrong defect reports from exactly that.
const STATUS_LINE: &str = "status";

// ---------------------------------------------------------------------------
// The author half's vocabulary
// ---------------------------------------------------------------------------

/// How many pages `PLAIN` has, and therefore how many stamps the plan holds.
const SOURCE_PAGES: usize = 4;

/// The ribbon item that opens the authoring window.
const RIBBON_ITEM: &str = "ribbon.item.file.stamp_collection";
/// The command behind it, as `command-unimplemented` would name it.
const COMMAND_ID: &str = "file.stamp_collection";
/// The authoring window's own region.
const DIALOG: &str = "dialog:stamp-collection";
/// Its Save button.
const DIALOG_SAVE: &str = "stamp-collection.save";
/// The line the window traces as it opens, carrying what it read from the
/// document: the page count, whether this file already is a collection, and
/// how many names it found.
const DIALOG_OPENED: &str = "stamp-collection-open";
/// The line the window traces when Save is pressed, carrying the plan.
const REQUESTED: &str = "stamp-collection-requested";
/// The line `crate::stamps::write` traces after the bytes are on disk.
const WROTE: &str = "stamp-collection";
/// The environment seam that answers the save picker.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// What the authored collection is called on disk, under `--out`.
const WRITTEN_NAME: &str = "stamp_collection.authored.pdf";

/// See the module documentation.
pub struct AStampCollectionDisclosesItself;

impl Check for AStampCollectionDisclosesItself {
    fn name(&self) -> &'static str {
        "a_stamp_collection_discloses_itself"
    }

    fn defect(&self) -> &'static str {
        "an operator opens an Acrobat stamp collection somebody sent him and pdfcer shows him a \
         document of unrelated pictures — no category, no stamp names, no sign that the file is \
         a stamp set at all, because every one of those facts lives in metadata and a name tree \
         that no rendering can show. Or the opposite: the Stamp collection heading is pinned \
         into Document properties for every document, telling him his CAD drawings are Acrobat \
         stamp sets"
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
    let exe = resolve_exe(ctx)?;
    let ui_rect = ui_rect_event(ctx)?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, the File tab and \
             the Document properties item, twice. Reported as SKIPPED rather than passed: a \
             check that did not run has learned nothing.",
        ));
    }

    // --- 1: the collection, panel open -------------------------------------
    let session = launch(
        ctx,
        &exe,
        COLLECTION,
        "stamp_collection.collection.trace.txt",
    )?;
    report.note(format!(
        "launched {} on fixtures/{COLLECTION} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // MAXIMISE — at the harness's default 1,100 pt window the File tab's
    // last two groups fold away entirely and a check reports a lost command.
    // `about.rs` holds the measurement; four checks now share it.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    open_properties(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    if trace.last(STATUS_LINE).is_none() {
        return Err(Error::new(format!(
            "the application launched and never traced a `{STATUS_LINE}` line, so it opened no \
             document. Every verdict below would be about an empty shell rather than about \
             fixtures/{COLLECTION}."
        )));
    }
    if declared(&trace, ui_rect, PANEL_OPEN_WITNESS).is_none() {
        return Err(Error::new(format!(
            "the Document properties panel did not come up — no `{PANEL_OPEN_WITNESS}` region — \
             so nothing can be concluded about the stamp section inside it. Regions beginning \
             `properties`: {}.",
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    if declared(&trace, ui_rect, SECTION_REGION).is_none() {
        return Ok(Some(format!(
            "the Document properties panel is open on a file whose catalog carries a `/Names` → \
             `/Pages` tree with {STAMPS} stamps in it, and it declares no `{SECTION_REGION}` \
             region — so pdfcer opened an Acrobat stamp collection and said nothing about it. \
             Every fact that makes this file a collection is invisible in the page view, which \
             is precisely why it is owed off-canvas. Check `panels::docprops::stamps::section`'s \
             early return: it must be `crate::stamps::is_collection`, which asks the NAME TREE \
             and not the title. Regions beginning `properties`: {}.",
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    let rows = declared_names(&trace, ui_rect, ROW_PREFIX);
    if rows.len() != STAMPS {
        return Ok(Some(format!(
            "the stamp section drew {} row region(s) and this fixture's name tree holds exactly \
             {STAMPS}. ★ A heading and a count with nothing under them is the shape this \
             regression takes: the section still says '{STAMPS} stamps, 1 of them dynamic' and \
             the list contradicting it is empty. Rows declared: {}.",
            rows.len(),
            list(&rows)
        )));
    }
    report.note(format!(
        "{STAMPS} stamp rows are drawn, matching the {STAMPS} entries the fixture's name tree \
         holds"
    ));
    drop(session);

    // --- 2: the control, with the panel OPEN ----------------------------
    //
    // Stronger than an absence read off a closed panel: `properties.info` is
    // declared, so the panel is provably up, and the stamp section is still not
    // in it. That distinguishes "the section is document-driven" from "the
    // surface it lives on was never opened".
    let session = launch(ctx, &exe, PLAIN, "stamp_collection.plain.trace.txt")?;
    report.note(format!(
        "control launch on fixtures/{PLAIN} as pid {}",
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
    if declared(&trace, ui_rect, PANEL_OPEN_WITNESS).is_none() {
        return Err(Error::new(
            "the control launch's Document properties panel did not come up — no \
             `properties.info` region — so the absence of a stamp section below it is the \
             absence of the whole panel, and proves nothing.",
        ));
    }
    if let Some(rect) = declared(&trace, ui_rect, SECTION_REGION) {
        return Ok(Some(format!(
            "fixtures/{PLAIN} has no `/Names` → `/Pages` tree and Document properties drew \
             `{SECTION_REGION}` at {rect:?} anyway — a heading telling the operator that an \
             ordinary four-page document is an Acrobat stamp set. Check that the early return in \
             `panels::docprops::stamps::section` tests `is_collection` (the name tree) and not \
             the presence of a `/Title`, which every document he opens has."
        )));
    }
    let rows = declared_names(&trace, ui_rect, ROW_PREFIX);
    if !rows.is_empty() {
        return Ok(Some(format!(
            "the control declared no stamp section and {} row region(s) under it: {}. That is \
             rows drawn outside their own block, which no reading of `section` produces — \
             suspect a stale trace or a second panel publishing the same prefix before believing \
             the geometry.",
            rows.len(),
            list(&rows)
        )));
    }
    report.note("the ordinary document's panel is open and carries no stamp section");

    Ok(None)
}

// ---------------------------------------------------------------------------
// Harness plumbing — the same shapes `load_anomalies` uses, and why
// ---------------------------------------------------------------------------

fn resolve_exe(ctx: &CheckContext) -> Result<PathBuf> {
    ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })
}

fn ui_rect_event(ctx: &CheckContext) -> Result<&'static str> {
    ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })
}

/// This module's fixtures, resolved by [`crate::checks::driving::repo_fixture`].
///
/// See that function for why the path is compile-time resolved. The sentence
/// kept here is the one about this check's own method.
fn repo_fixture(name: &str) -> Result<PathBuf> {
    crate::checks::driving::repo_fixture(
        name,
        "This check pins its own two documents and ignores --pdf: its whole method is one \
         build's positive reading on a stamp collection, denied on a document that is not one, \
         and a suite-wide fixture is neither.",
    )
}

/// **Launch on a document this check names, with whatever extra seams it needs.**
fn launch_on(
    ctx: &CheckContext,
    exe: &Path,
    pdf: &Path,
    trace_name: &str,
    extra_env: &[(&str, String)],
) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace_name));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    for (key, value) in extra_env {
        spec.env.push(((*key).to_owned(), value.clone()));
    }
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    Session::launch(&spec, ctx.profile.trace_prefix)
}

/// [`launch_on`], for one of this file's two pinned fixtures and no extra seams.
fn launch(ctx: &CheckContext, exe: &Path, fixture: &str, trace_name: &str) -> Result<Session> {
    launch_on(ctx, exe, &repo_fixture(fixture)?, trace_name, &[])
}

/// Bring the Document properties panel to the front, if it is not already.
fn open_properties(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    if declared(&session.trace()?, ui_rect, PANEL_OPEN_WITNESS).is_some() {
        return Ok(());
    }
    crate::checks::properties_metadata::open_document_properties(session, driver, ui_rect)
}

// ===========================================================================
// The author half — O169's write side
// ===========================================================================

/// **An ordinary document becomes a stamp collection Acrobat will accept.**
pub struct StampCollectionReachesTheEngine;

impl Check for StampCollectionReachesTheEngine {
    fn name(&self) -> &'static str {
        "stamp_collection_reaches_the_engine"
    }

    fn defect(&self) -> &'static str {
        "the operator picks Save as stamp collection, the window comes up, he presses Save — and \
         either nothing reaches disk at all, or bytes reach disk that Acrobat will not show as a \
         stamp set. ★ The symptom is the same in both cases and it is the worst kind: Acrobat \
         reads its stamps folder once at startup and reports nothing, so what he sees is an \
         empty Stamps menu and no error anywhere, on either side"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_author(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the round trip is one causal chain — open, plan, save, disk, reopen, disclose — and \
              every step's failure sentence names the step before it. Splitting it into helpers \
              would put the else branches in a different file from the act they explain, which \
              is what makes a failing driven check readable at all."
)]
fn drive_author(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = resolve_exe(ctx)?;
    let ui_rect = ui_rect_event(ctx)?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, the File tab, a \
             ribbon item and a Save button, then launches again. Reported as SKIPPED rather than \
             passed: a check that did not run has learned nothing.",
        ));
    }

    // Cleared before the run, not merely named. A collection left behind by
    // an earlier run would let a build that writes NOTHING satisfy every
    // assertion from `target.exists()` onward — including the round trip, which
    // would cheerfully reopen last week's file and report four stamps. That is
    // `a_driven_check_that_does_not_establish_its_preconditions_measures_the_previous_run`
    // in the Rust RAG, and it is the reason this is the first thing that runs.
    let target = ctx.out(WRITTEN_NAME);
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the run, so a collection written by an earlier run could be \
             mistaken for this one's — and the round trip would pass on it.",
            target.display()
        )));
    }

    // --- 1: the source document --------------------------------------------
    let source = repo_fixture(PLAIN)?;
    let session = launch_on(
        ctx,
        &exe,
        &source,
        "stamp_collection.author.trace.txt",
        &[(SAVE_PATH_ENV, target.display().to_string())],
    )?;
    report.note(format!(
        "launched {} on fixtures/{PLAIN} as pid {}, save picker answered with {}",
        exe.display(),
        session.pid(),
        target.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // Maximised for `about.rs`'s measured reason: at the harness's default
    // 1,100 pt window the File tab's last groups fold away and a check reports
    // a lost command that is merely off the band.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    let trace = session.trace()?;
    if trace.last(STATUS_LINE).is_none() {
        return Err(Error::new(format!(
            "the application launched and never traced a `{STATUS_LINE}` line, so it opened no \
             document. Everything below would be about an empty shell rather than about \
             fixtures/{PLAIN}."
        )));
    }
    let tab = declared(&trace, ui_rect, "ribbon.tab.file").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.file` region in {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    // --- 2: the window ------------------------------------------------------
    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, RIBBON_ITEM)? else {
        return Ok(Some(format!(
            "the File tab declares no `{RIBBON_ITEM}`, on the band or in the overflow — so the \
             capability is unreachable however well it works. ★ Under R8 a command's presence in \
             the registry is the only way the shell learns it exists, so an item missing here is \
             a registration that did not happen, not a layout accident. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.file."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(24);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, DIALOG).is_none() {
        let unimplemented = trace
            .events("command-unimplemented")
            .any(|l| l.get("id") == Some(COMMAND_ID));
        return Ok(Some(if unimplemented {
            format!(
                "`{COMMAND_ID}` was clicked and traced `command-unimplemented` — drawn, on the \
                 ribbon, and with no dispatch arm behind it."
            )
        } else {
            format!(
                "`{COMMAND_ID}` was clicked and no `{DIALOG}` region appeared. The arm ran and \
                 built no window, or declined it. ⚠ It declines on no-document and on an empty \
                 document, and fixtures/{PLAIN} is neither — so a decline here is the guard \
                 misreading a document that has {SOURCE_PAGES} pages."
            )
        }));
    }

    // --- 3: what the window READ from the document ------------------------
    //
    // The plan is one row per page, every row included, so `pages=` is also the
    // stamp count that must survive to disk. Asserted here rather than only at
    // the far end because a shortfall introduced HERE and a shortfall
    // introduced in the writer produce the same file, and only the trace tells
    // them apart.
    let Some(opened) = trace.last(DIALOG_OPENED) else {
        return Ok(Some(format!(
            "the window drew and traced no `{DIALOG_OPENED}` line, so it never read the document \
             it is about. Its rows are one per page and its category is seeded from the file; a \
             window that read neither is showing the operator a plan about nothing."
        )));
    };
    report.note(format!("window opened: `{}`", opened.raw));
    match opened.get_usize("pages") {
        Some(SOURCE_PAGES) => {}
        Some(other) => {
            return Ok(Some(format!(
                "the window planned {other} stamp(s) for a {SOURCE_PAGES}-page document. Every \
                 row is included by default, so this number IS the collection — a plan short of \
                 a page writes a valid file with a sheet quietly missing from it, which is the \
                 one defect in this feature with no visible symptom at all."
            )));
        }
        None => {
            return Err(Error::new(format!(
                "the `{DIALOG_OPENED}` line carries no readable `pages=` count: `{}`",
                opened.raw
            )));
        }
    }
    if opened.get_usize("reopened") != Some(0) {
        return Ok(Some(format!(
            "the window reported `reopened=` on fixtures/{PLAIN}, which has no `/Names` → \
             `/Pages` tree and is asserted through the engine not to be a collection. A build \
             that thinks every document is a re-opened collection will seed its names from an \
             empty list and its category from a tree that is not there: `{}`.",
            opened.raw
        )));
    }

    // --- 4: press Save ------------------------------------------------------
    let Some(button) = declared(&trace, ui_rect, DIALOG_SAVE) else {
        return Ok(Some(format!(
            "the window declares no `{DIALOG_SAVE}` region, so there is nothing to press. \
             Regions beginning `stamp-collection`: {}.",
            list(&declared_names(&trace, ui_rect, "stamp-collection"))
        )));
    };
    // ⚠ `frame_of`, never `session.frame()`. This window is its own viewport
    // and therefore its own OS window; the main frame's origin would put the
    // click somewhere on the document. That mistake has been made in this
    // harness before and it lands silently — a click at a plausible coordinate
    // on the wrong window is not an error, it is a marquee drag.
    driver.click_at(frame_of(&session, &trace, ui_rect, DIALOG_SAVE)?.declared_center(button))?;
    session.settle(40);

    // --- 5: what was requested, and what was written ------------------------
    let trace = session.trace()?;
    let Some(requested) = trace.last(REQUESTED) else {
        return Ok(Some(format!(
            "Save was pressed and no `{REQUESTED}` line followed, so the button is drawn and \
             inert — or it is greyed, which happens on an empty category and on zero included \
             stamps. The category is seeded from the collection, then the title, then the file \
             stem, and fixtures/{PLAIN} has a stem; so a greyed Save here means the seeding \
             chain broke rather than that the operator left a field blank."
        )));
    };
    report.note(format!("requested: `{}`", requested.raw));
    if requested.get_usize("stamps") != Some(SOURCE_PAGES) {
        return Ok(Some(format!(
            "the window read {SOURCE_PAGES} pages and then asked for a different number of \
             stamps: `{}`. Between reading and requesting, this shell did nothing at all — no \
             row was unticked and no name was typed — so the two numbers disagreeing is a plan \
             that mutates while nobody is touching it.",
            requested.raw
        )));
    }
    // `category=` is Debug-formatted, so it arrives quoted; `TraceLine::get`
    // strips the quotes. Its VALUE is not asserted — the seeding chain prefers
    // the document's own `/Title` over the file stem, and pinning one of those
    // here would make this check fail the day the fixture gains a title, which
    // is a property of the fixture rather than of the feature. What matters is
    // that it is not empty, because an empty one greys Save, and Save was not
    // grey.
    report.note(format!(
        "category seeded as {:?}",
        requested.get("category").unwrap_or("<absent>")
    ));

    let Some(wrote) = trace.last(WROTE) else {
        let say = |event: &str| {
            trace
                .last(event)
                .map_or_else(|| "none".to_owned(), |l| l.raw.clone())
        };
        return Ok(Some(format!(
            "the plan was requested and no `{WROTE}` line followed, so nothing reached disk. \
             declined={} cancelled={} unavailable={} failed={}. ★ `unavailable` means this build \
             has no picker seam and the harness could not answer the save dialog — that is a \
             harness-facing defect, not an operator-facing one, and it is reported here rather \
             than skipped because the two look identical from outside.",
            say("stamp-collection-declined"),
            say("stamp-collection-cancelled"),
            say("stamp-collection-unavailable"),
            say("stamp-collection-failed"),
        )));
    };
    report.note(format!("wrote: `{}`", wrote.raw));
    for (field, expected) in [("stamps", SOURCE_PAGES), ("pages", SOURCE_PAGES)] {
        let Some(actual) = wrote.get_usize(field) else {
            return Err(Error::new(format!(
                "the `{WROTE}` line carries no readable `{field}=` count: `{}`",
                wrote.raw
            )));
        };
        if actual != expected {
            return Ok(Some(format!(
                "the engine reported `{field}={actual}` for a plan of {expected}. ★★ `stamps=` \
                 and `pages=` are printed side by side precisely so this can be seen: a build \
                 that carried the right pages and named the wrong ones writes a perfectly good \
                 PDF, of the right size, with the right page count, and Acrobat shows an empty \
                 menu. Line: `{}`.",
                wrote.raw
            )));
        }
    }
    if wrote.get_usize("skipped") != Some(0) {
        return Ok(Some(format!(
            "the engine skipped at least one page: `{}`. ⚠ `skipped=` is expected to be zero \
             forever and is printed for exactly this reason — a number that is always zero is a \
             tripwire. A non-zero one means this shell handed the engine a name list and a page \
             list that disagreed.",
            wrote.raw
        )));
    }

    // --- 6: the disclosure and the disk agree -------------------------------
    //
    // ⚠ `path=` in the trace is Debug-formatted, so it arrives with its
    // separators doubled. It is deliberately NOT parsed and compared: this
    // check knows the path because it SET it, and a machine reading a
    // `{:?}` field is the shape that once made a driven check report the
    // opposite of the truth while quoting the truth in its own message.
    if !target.exists() {
        return Ok(Some(format!(
            "the shell traced a successful write and {} does not exist. The receipt and the disk \
             disagree, which is the worst of the failures available here: the operator has been \
             told a file was written, and told to restart Acrobat to see it.",
            target.display()
        )));
    }
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the collection was written to {} and cannot be read back: {e}",
            target.display()
        ))
    })?;
    if !bytes.starts_with(b"%PDF") {
        return Ok(Some(format!(
            "{} was written and does not begin `%PDF` in {} bytes. Something reached disk and it \
             is not a document.",
            target.display(),
            bytes.len()
        )));
    }
    report.note(format!("{} bytes on disk, beginning %PDF", bytes.len()));
    drop(session);

    // --- 7: THE ROUND TRIP ----------------------------------------------
    //
    // Reopen what pdfcer wrote, with the reader the header argues is an
    // independent oracle, and require it to disclose a collection of exactly
    // the planned size. This is the only assertion in the file that asks the
    // question Acrobat asks: not "are the names in the bytes" but "are they
    // reachable from the catalog's `/Names` → `/Pages` tree".
    let session = launch_on(
        ctx,
        &exe,
        &target,
        "stamp_collection.roundtrip.trace.txt",
        &[],
    )?;
    report.note(format!(
        "reopened the authored collection as pid {}",
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
        return Ok(Some(format!(
            "pdfcer wrote {} and then could not open it — no `{STATUS_LINE}` line on the second \
             launch. A collection its own reader refuses is one Acrobat will refuse too, and the \
             operator has already been told it was written.",
            target.display()
        )));
    }
    if declared(&trace, ui_rect, PANEL_OPEN_WITNESS).is_none() {
        return Err(Error::new(format!(
            "the round trip's Document properties panel did not come up — no \
             `{PANEL_OPEN_WITNESS}` region — so the absence of a stamp section below it is the \
             absence of the whole panel and proves nothing about the file that was written."
        )));
    }
    if declared(&trace, ui_rect, SECTION_REGION).is_none() {
        return Ok(Some(format!(
            "pdfcer wrote {}, reported {SOURCE_PAGES} stamps in it, reopened it — and Document \
             properties declares no `{SECTION_REGION}` region, which means its own reader does \
             not see a `/Names` → `/Pages` tree in the file. ★ The reader is calibrated against \
             a collection built independently of pdfcer, so the disagreement is about the BYTES: \
             they are a PDF of the right pages with no stamp structure Acrobat can find. Regions \
             beginning `properties`: {}.",
            target.display(),
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    let rows = declared_names(&trace, ui_rect, ROW_PREFIX);
    if rows.len() != SOURCE_PAGES {
        return Ok(Some(format!(
            "the authored collection reopens as a collection of {} stamp(s) and {SOURCE_PAGES} \
             were planned, requested and reported written. The name tree that survived the round \
             trip is not the one the operator was shown. Rows declared: {}.",
            rows.len(),
            list(&rows)
        )));
    }
    report.note(format!(
        "the file pdfcer wrote reopens as a stamp collection of {SOURCE_PAGES} stamps — the \
         plan, the receipt and the bytes all agree"
    ));

    Ok(None)
}
