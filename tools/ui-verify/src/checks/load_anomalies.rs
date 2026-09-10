//! **A PDF that contradicts itself says so, and a PDF that does not stays
//! quiet** — the driven half of the load-anomaly disclosure.
//!
//! # What this file is for
//!
//! `crate::app::status::anomalies` (in the shell) turns the engine's
//! `Document::load_anomalies()` into two operator-facing surfaces:
//!
//! * a one-line **census** in the status bar — *"1 duplicate dictionary key"* —
//!   published as the trace region `status-group:load-anomalies`;
//! * a **row per anomaly** in the Document properties panel, naming the object,
//!   the key, the value pdfcer used and the value it left — published as
//!   `properties.load-anomalies` with `properties.load-anomalies.N` per row.
//!
//! Every part of that is unit-tested, including against the real engine loader
//! on a fixture authored for it. What no unit test can see is whether the two
//! surfaces are **connected to a running window**: whether the status bar draws
//! the line on the frame the document opens, and whether the panel draws its
//! rows where an operator can read them. That is this file's whole subject, and
//! it is the standing rule R1 — *verify by driving the binary, not by a passing
//! test*.
//!
//! # ★★★ The second launch is the check
//!
//! Each of the two checks below launches **twice**: once on
//! `fixtures/contradicts-itself.pdf`, asserting the disclosure is THERE, and
//! once on `fixtures/four-pages.pdf`, asserting it is NOT.
//!
//! Without the control launch, "the region is declared" is satisfied by a build
//! that declares it on every file — a heading permanently pinned to the status
//! bar and a section permanently pinned to the panel, saying *"this file
//! contradicts itself"* about files that do not. That is a worse defect than
//! the one the check is for, it is the defect R8b's *fuzzy, never sneaky*
//! clause is most alert to (a disclosure that cries wolf trains an operator to
//! stop reading it), and it wears exactly the same green tick.
//!
//! The control fixture's cleanliness is not assumed either: the shell's own
//! `the_control_fixtures_a_driven_run_uses_are_genuinely_clean` asserts through
//! the engine that both files this module names really do load without an
//! anomaly. If that ever changed, the absence assertion here would go red and
//! blame the program for something that is true of the fixture — so the
//! tripwire lives in the crate that runs on every `cargo test`, not here.
//!
//! # Why two checks and not one
//!
//! Because they cost different things. The status-bar half needs **no pointer
//! and no keyboard**: the bar draws on the first frame after the document
//! opens, so the harness launches, waits, and reads the trace. The panel half
//! has to click a mode segment, a ribbon tab and a ribbon item.
//!
//! Fusing them would make the cheap, always-runnable half inherit the
//! expensive half's `--no-input` SKIP — and a SKIP is not red, so the whole
//! disclosure would silently stop being evidence on every run where the
//! machine's pointer belonged to somebody else. That is a failure mode this
//! project has paid for repeatedly. Split, the status-bar half runs on every
//! sweep including the ones nobody can watch.
//!
//! # What these do NOT prove
//!
//! That the *wording* is right. The trace publishes a region name and a
//! rectangle, not a string, so a build that drew the census clause in the wrong
//! order, or spelled the discarded value where the kept one belongs, would pass
//! both. The wording is asserted in `crate::app::status::anomalies`' own tests,
//! against the same fixture, through the same engine call — deliberately,
//! because a string is exactly what a unit test CAN see. What it cannot see is
//! the window, and that is what is here.

use std::path::{Path, PathBuf};

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The fixture that contradicts itself: one doubled `/PageMode` in the
/// catalogue, `/UseOC` first and `/UseOutlines` second.
///
/// Built by `fixtures/contradicts-itself.PROVENANCE.py`, which computes its own
/// xref offsets so that the file's cross-reference table is **sound** — that
/// matters, because a scanned-and-rebuilt file lights `Document::recovery()`
/// instead, which is a different disclosure with a different status region, and
/// a check reading the wrong one would be green about a surface it never
/// touched.
const CONTRADICTS: &str = "contradicts-itself.pdf";

/// The control: a file with nothing to disclose.
///
/// ⚠ Asserted clean through the engine by the shell's own
/// `the_control_fixtures_a_driven_run_uses_are_genuinely_clean`. Do not swap it
/// for another fixture without adding the new name there; an absence assertion
/// against an unverified control is an assertion about nothing.
const CLEAN: &str = "four-pages.pdf";

/// The status bar's census region.
const STATUS_REGION: &str = "status-group:load-anomalies";
/// The Document properties panel's anomaly block.
const PANEL_REGION: &str = "properties.load-anomalies";
/// The prefix of the per-anomaly row regions inside that block.
const ROW_PREFIX: &str = "properties.load-anomalies.";
/// The metadata form's region — the panel's *other* content, used here only as
/// proof that the panel itself is open on the control launch.
const PANEL_OPEN_WITNESS: &str = "properties.info";
/// The mode the panel half drives in. `file` is in every mode's tab list;
/// Review is chosen because the rest of the harness uses it.
const MODE: &str = "review";
/// The trace line the shell emits once a document is open. Its **absence** is
/// how a launch that opened nothing is told apart from a build that discloses
/// nothing.
const STATUS_LINE: &str = "status";

/// Where the window is put for the two no-input launches, as
/// `PDFCER_DIAG_VIEWPORT` takes it: `x,y,w,h` in desktop pixels.
///
/// ★★ **Off the desktop, on purpose.** Neither status-bar launch reads a single
/// pixel — the whole verdict comes from the trace — so the window does not need
/// to be anywhere a human could see it, and putting it where a human cannot
/// means this check can run while the operator is working. Every on-screen
/// alternative either covers his window or races him for it.
///
/// ⚠ **The position only survives because [`launch_quiet`] sets
/// `LaunchSpec::place` to `false`.** `Session::place` moves every launched
/// window to `(780, 40)` unconditionally, so before that flag existed this
/// constant's first two numbers were discarded and the paragraph above was
/// simply untrue — the window would have landed in the middle of the operator's
/// screen while a doc comment said it was nowhere near him. The flag was added
/// with this check, 2026-09-09, and the two must not be separated.
///
/// The size is the harness's usual 1400x900 and is not arbitrary: a narrower
/// window folds status-bar groups away, and a folded group publishes no region
/// — which would read as "the disclosure is missing" when it is merely elided.
/// The `SAFE_ORIGIN + size` arithmetic that binds an on-screen check does not
/// bind here precisely because nothing is ever aimed at this window.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

// ---------------------------------------------------------------------------
// 1: the status bar
// ---------------------------------------------------------------------------

/// See the module documentation.
pub struct LoadAnomaliesReachTheStatusBar;

impl Check for LoadAnomaliesReachTheStatusBar {
    fn name(&self) -> &'static str {
        "load_anomalies_reach_the_status_bar"
    }

    fn defect(&self) -> &'static str {
        "a PDF whose catalogue contradicts itself opens silently — pdfcer picked one of two \
         values for a key and told nobody — so an operator reading a titleblock cannot know \
         that the file said something else a few bytes earlier. Or the opposite: the census \
         line is pinned to the status bar for every file, announcing a contradiction in \
         documents that have none, which trains him to stop reading it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_status_bar(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive_status_bar(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = resolve_exe(ctx)?;
    let ui_rect = ui_rect_event(ctx)?;

    // --- 1: the file that contradicts itself -------------------------------
    let session = launch_quiet(
        ctx,
        &exe,
        CONTRADICTS,
        "load_anomalies.contradicts.trace.txt",
    )?;
    report.note(format!(
        "launched {} on fixtures/{CONTRADICTS} as pid {} — no input is sent, and the window is \
         placed off the desktop",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(45);

    let trace = session.trace()?;
    // ★ The document has to be OPEN before the presence or absence of a
    // disclosure means anything. A launch that opened nothing — a mistyped
    // path, a relative path resolved against the wrong directory — traces no
    // `status` line at all, and every region on a document-dependent surface is
    // then legitimately missing. Two sweeps in this project produced confident,
    // detailed, entirely wrong defect reports from exactly that.
    if trace.last(STATUS_LINE).is_none() {
        return Err(Error::new(format!(
            "the application launched and never traced a `{STATUS_LINE}` line, so it opened no \
             document. The verdict below would be about an empty shell, not about the fixture. \
             Check that fixtures/{CONTRADICTS} exists and that an absolute path was passed."
        )));
    }
    if declared(&trace, ui_rect, STATUS_REGION).is_none() {
        return Ok(Some(format!(
            "fixtures/{CONTRADICTS} was opened and the status bar declares no `{STATUS_REGION}` \
             region, so the census line never drew. The engine reports one `DuplicateDictKey` \
             for this file — asserted by the shell's own \
             `the_contradicting_fixture_produces_exactly_one_anomaly_through_the_engine` — so \
             the anomaly exists and the disclosure of it does not. Status regions declared: {}.",
            list(&declared_names(&trace, ui_rect, "status-group:"))
        )));
    }
    report.note("the census line is on the bar for the contradicting file");
    drop(session);

    // --- 2: ★★ THE CONTROL, and it is the half that makes this a check -----
    let session = launch_quiet(ctx, &exe, CLEAN, "load_anomalies.clean.trace.txt")?;
    report.note(format!(
        "control launch on fixtures/{CLEAN} as pid {}",
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(45);

    let trace = session.trace()?;
    if trace.last(STATUS_LINE).is_none() {
        return Err(Error::new(format!(
            "the control launch opened no document — no `{STATUS_LINE}` line — so its silence \
             about load anomalies proves nothing. fixtures/{CLEAN}."
        )));
    }
    if let Some(rect) = declared(&trace, ui_rect, STATUS_REGION) {
        return Ok(Some(format!(
            "fixtures/{CLEAN} loads without a single anomaly — asserted through the engine by \
             the shell's `the_control_fixtures_a_driven_run_uses_are_genuinely_clean` — and the \
             status bar declared `{STATUS_REGION}` at {rect:?} anyway. The census line is on \
             for files that have nothing to disclose. ★ Check `status::anomalies::status_line`'s \
             `None` arm and its caller in `status::disclosure`: a disclosure that appears on \
             clean files is not a cosmetic defect, it is the one that makes an operator stop \
             reading the bar."
        )));
    }
    report.note("the census line is absent on the clean file, so it is driven by the document");

    Ok(None)
}

// ---------------------------------------------------------------------------
// 2: the Document properties panel
// ---------------------------------------------------------------------------

/// See the module documentation.
pub struct LoadAnomaliesAreListedInDocumentProperties;

impl Check for LoadAnomaliesAreListedInDocumentProperties {
    fn name(&self) -> &'static str {
        "load_anomalies_are_listed_in_document_properties"
    }

    fn defect(&self) -> &'static str {
        "the status bar says the file contradicted itself and there is nowhere to find out \
         WHERE. Document properties draws no anomaly rows — so the operator is told a value \
         was chosen for him and can never see which two values were on the table, which is the \
         only form of that disclosure he could act on"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_panel(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive_panel(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = resolve_exe(ctx)?;
    let ui_rect = ui_rect_event(ctx)?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, the File tab and \
             the Document properties item, twice. Reported as SKIPPED rather than passed: a \
             check that did not run has learned nothing. ★ The status-bar half of the same \
             disclosure, `load_anomalies_reach_the_status_bar`, needs no input and did run.",
        ));
    }

    // --- 1: the contradicting file, panel open -----------------------------
    let session = launch_visible(
        ctx,
        &exe,
        CONTRADICTS,
        "load_anomalies_panel.contradicts.trace.txt",
    )?;
    report.note(format!(
        "launched {} on fixtures/{CONTRADICTS} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // ★★ MAXIMISE — at the harness's default 1,100 pt window the File tab's
    // last two groups fold away entirely and a check reports a lost command.
    // `about.rs` holds the measurement; three checks already share it.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    open_properties(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    if declared(&trace, ui_rect, PANEL_OPEN_WITNESS).is_none() {
        return Err(Error::new(format!(
            "the Document properties panel did not come up — no `{PANEL_OPEN_WITNESS}` region — \
             so nothing can be concluded about the anomaly rows inside it. Regions beginning \
             `properties`: {}.",
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    if declared(&trace, ui_rect, PANEL_REGION).is_none() {
        return Ok(Some(format!(
            "the Document properties panel is open on a file the engine reports one \
             `DuplicateDictKey` for, and it declares no `{PANEL_REGION}` region — so the long \
             form of the disclosure is not drawn. The status bar's census tells the operator a \
             choice was made; this panel is the only place that says which two values it was \
             between. Regions beginning `properties`: {}.",
            list(&declared_names(&trace, ui_rect, "properties"))
        )));
    }
    let rows = declared_names(&trace, ui_rect, ROW_PREFIX);
    if rows.len() != 1 {
        return Ok(Some(format!(
            "the anomaly block drew {} row region(s) and this fixture is authored for exactly \
             one contradiction. ★ A block with a heading and no rows under it is the shape a \
             regression takes when `rows()` starts returning empty strings, or when the loop \
             stops publishing: the heading still says the file contradicted itself and there is \
             nothing under it. Rows declared: {}.",
            rows.len(),
            list(&rows)
        )));
    }
    report.note("one anomaly row is drawn, matching the one anomaly the engine reports");
    drop(session);

    // --- 2: ★★ the control, with the panel OPEN ----------------------------
    //
    // This is stronger than the status bar's control. There, the absence could
    // in principle be explained by the bar not drawing at all; here the panel is
    // provably open — `properties.info` is declared — and the anomaly block is
    // still not in it. That distinguishes "the block is document-driven" from
    // "the surface it lives on was closed".
    let session = launch_visible(ctx, &exe, CLEAN, "load_anomalies_panel.clean.trace.txt")?;
    report.note(format!(
        "control launch on fixtures/{CLEAN} as pid {}",
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
        return Err(Error::new(format!(
            "the control launch's Document properties panel did not come up — no \
             `{PANEL_OPEN_WITNESS}` region — so the absence of an anomaly block below it is the \
             absence of the whole panel, and proves nothing."
        )));
    }
    if let Some(rect) = declared(&trace, ui_rect, PANEL_REGION) {
        return Ok(Some(format!(
            "fixtures/{CLEAN} has no load anomalies and the Document properties panel drew \
             `{PANEL_REGION}` at {rect:?} anyway — a heading telling the operator the file \
             contradicted itself, on a file that did not. Check `load_anomalies_note`'s early \
             return: it must be `rows.is_empty()`, taken BEFORE anything is drawn."
        )));
    }
    let rows = declared_names(&trace, ui_rect, ROW_PREFIX);
    if !rows.is_empty() {
        return Ok(Some(format!(
            "the clean file declared no anomaly block and {} row region(s) under it: {}. That is \
             rows drawn outside their own block, which no reading of `load_anomalies_note` \
             produces — suspect a stale trace or a second panel publishing the same prefix \
             before believing the geometry.",
            rows.len(),
            list(&rows)
        )));
    }
    report.note("the clean file's panel is open and carries no anomaly block");

    Ok(None)
}

// ---------------------------------------------------------------------------
// shared
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

/// **Resolve a fixture from this repository, refusing to guess.**
///
/// ★ Resolved from `CARGO_MANIFEST_DIR` at COMPILE TIME and not from
/// `--source-root`, which defaults to `crates` because its job is the staleness
/// comparison. `root.join("fixtures")` gave `crates/fixtures/…`, a directory
/// that does not exist, and the checks that used it SKIPPED for ever while
/// looking healthy. The reasoning is `checks::protect::repo_fixture`'s and is
/// copied rather than re-derived.
fn repo_fixture(name: &str) -> Result<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join(name);
    if !path.is_file() {
        return Err(Error::new(format!(
            "the fixture {} is missing. This check pins its own two documents and ignores \
             --pdf: its whole method is one build's positive reading on a file that contradicts \
             itself, denied on a file that does not, and a suite-wide fixture is neither.",
            path.display()
        )));
    }
    Ok(path)
}

/// A launch with the diagnostic channel on, the window off the desktop, and no
/// input ever sent to it.
fn launch_quiet(
    ctx: &CheckContext,
    exe: &Path,
    fixture: &str,
    trace_name: &str,
) -> Result<Session> {
    let mut spec = base_spec(ctx, exe, fixture, trace_name)?;
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), OFFSCREEN.to_owned()));
    }
    // ★ Without this the line above is decoration: `Session::place` would move
    // the window to `(780, 40)` the moment it appeared. See `OFFSCREEN`.
    spec.place = false;
    Session::launch(&spec, ctx.profile.trace_prefix)
}

/// A launch on the real desktop, because this one is going to be clicked.
fn launch_visible(
    ctx: &CheckContext,
    exe: &Path,
    fixture: &str,
    trace_name: &str,
) -> Result<Session> {
    let spec = base_spec(ctx, exe, fixture, trace_name)?;
    Session::launch(&spec, ctx.profile.trace_prefix)
}

fn base_spec(
    ctx: &CheckContext,
    exe: &Path,
    fixture: &str,
    trace_name: &str,
) -> Result<LaunchSpec> {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace_name));
    spec.pdf = Some(repo_fixture(fixture)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    Ok(spec)
}

/// Bring the Document properties panel to the front, if it is not already.
///
/// ★ Reuses `properties_metadata`'s opener rather than spelling the two clicks
/// again. It is the same ribbon item and the same toggle hazard — pressing
/// `file.document_properties` while the panel is up CLOSES it — and two copies
/// of that guard would be two places for the next ribbon move to have to be
/// applied.
fn open_properties(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    if declared(&session.trace()?, ui_rect, PANEL_OPEN_WITNESS).is_some() {
        return Ok(());
    }
    crate::checks::properties_metadata::open_document_properties(session, driver, ui_rect)
}
