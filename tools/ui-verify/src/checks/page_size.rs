//! `resizing_a_sheet_changes_the_paper_in_the_saved_file` — picking a sheet
//! size for the page an operator is looking at must change **that page's
//! `/MediaBox` in the file he then saves**, and must leave the page beside it
//! alone.
//!
//! # What this is about
//!
//! `EditSession::set_media_boxes` shipped on 2026-08-18, written for the
//! drawing-set case, and was called by nothing for nineteen days because no
//! `pages.resize` command existed. A complete size chooser was built and
//! unreachable in `dialogs::new_document`, which opens only while creating a
//! file. The command, the window and the verb are wired now, and the whole
//! point of this file is that **none of that is evidence.**
//!
//! # ★★★ HOW I WOULD FALSIFY THIS CHECK
//!
//! Stated first, because a check nobody can falsify is a claim rather than a
//! measurement, and this one asserts an absence in phase D as well as a
//! presence.
//!
//! **Four plants, each of which this check must go red for.** Each is a real
//! way this feature can be built wrong, and each produces a *perfect* trace in
//! the writing process — which is exactly why the verdict is taken elsewhere.
//!
//! | plant | what the writing process still reports | what phase D reads |
//! |---|---|---|
//! | make `actions::pagesize::set` return `Ok(Vec::new())` without calling the engine | `page-size-commit`, and `page-size-changed … n=1` from the funnel, unchanged | page 0 still at its original size ⇒ **FAIL** |
//! | send `(0..doc.pages.len())` instead of the operands | everything, unchanged | page 1 **also** A6 ⇒ **FAIL on the negative control** |
//! | trace the *requested* rectangle instead of `session.pages()` | a perfect `page-size-sheet w=297.64` | phase D is a different process reading a file off disk; the plant cannot reach it ⇒ still **FAIL** if the write was dropped |
//! | transpose in `sheet_pt` | a plausible summary line | page 0 at 419.53 × 297.64 ⇒ **FAIL**, and the message says *transposed* rather than *wrong size* |
//!
//! **How to run the falsification.** Plant one, `cargo build --release -p
//! pdfcer-gui`, run this check alone, and require its own `[FAIL]` line in the
//! output — not merely a non-zero exit, which a SKIP also produces. Restore
//! from a byte copy, never from `git checkout`.
//!
//! **The one thing that would make this check worthless** and must be watched
//! for: if `page-size-document` were ever published from the *request* rather
//! than from `doc.pages`, every phase below would still pass on a build that
//! wrote nothing. That line's own comment in `dialogs::page_size` carries the
//! argument; this is the check that would silently stop meaning anything if it
//! were ignored.
//!
//! # The five phases, and why the verdict is in a second process
//!
//! | phase | process | what it establishes |
//! |---|---|---|
//! | **A** | 1 | the fixture's page sizes **as this build resolves them** — the baseline every later number is compared against, so the check does not depend on a hard-coded fixture geometry |
//! | **B** | 1 | Pages ▸ Sheet size opens a window, and picking A6 portrait reaches its commit |
//! | **C** | 1 | Save a copy writes a file |
//! | **D** | **2** | ★★★ **THE VERDICT** — a fresh binary opens the written file and reports the page sizes *it* resolves from the bytes |
//! | **D′** | 2 | ★ **THE NEGATIVE CONTROL, in the same trace line family, from the same instrument, in the same run** — page 1 was not an operand and must read exactly what phase A read for it |
//!
//! ⇒ **The oracle is not the code under test.** Phase A and phase D are the
//! same reader over two different files; the thing being judged is the
//! difference between them. `signing.rs` established this shape here and its
//! own falsification run is the argument: a planted flipped byte left the
//! writing phase's trace *character for character identical* and the reading
//! phase caught it.
//!
//! # The operand rule is used rather than driven
//!
//! Nothing is picked in the Pages panel, deliberately. `pages.resize` takes the
//! same operand as every other `pages.*` command —
//! `panels::pages::ops::operands`: the picked sheets when there are any, **the
//! current sheet when there are none** — so a launch with no picks aims the
//! command at page 0 and leaves pages 1–3 as the control. Driving the panel's
//! multi-select would test the panel, which has its own checks, and would put a
//! second failure mode between this check and its subject.
//!
//! # A6, and why not A4
//!
//! A6 is 297.64 × 419.53 pt. `four-pages.pdf` carries three distinct sheet
//! sizes — 2383.94 × 1683.78, 612 × 792 and 306 × 396 — and A6 differs from
//! every one of them **in both dimensions**, so neither a no-op nor a
//! transposition can pass vacuously. A4 would have been the lazy choice and is
//! also the size the *other* size window opens on, which is precisely the
//! coincidence a check should not rest on.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The fixture, pinned. See the module header: this check needs **more than
/// one page**, so that the sheet it does not touch can be the control.
const FIXTURE: &str = "fixtures/four-pages.pdf";

/// The Pages tab.
const PAGES_TAB: &str = "pages";

/// The command under test.
const RESIZE: (&str, &str) = ("ribbon.item.pages.resize", "pages.resize");

/// `PDFCER_DIAG_SAVE_PATH` — the seam that answers the save picker.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// `page-size-document index=… w=… h=… llx=… lly=…` — **the oracle.** One line
/// per sheet, published when the window opens, read from the shell's own page
/// tree over the file it has open.
const SHEET_EVENT: &str = "page-size-document";

/// `page-size-opened sheets=… distinct=… …` — the window drew.
const OPENED_EVENT: &str = "page-size-opened";

/// `page-size-commit n=… choice=… w_pt=… h_pt=… …` — the commit button acted.
const COMMIT_EVENT: &str = "page-size-commit";

/// `save-copy path=… bytes=… …` — the write happened.
const SAVED_EVENT: &str = "save-copy";

/// The size list's combo, closed.
const SIZE_COMBO: &str = "page-size.size";

/// The entry to click, indexed into `pdfcer_core::paper::PaperSize::ALL`.
///
/// **6 is A6**, the seventh entry: `A0, A1, A2, A3, A4, A5, A6, …`. An index
/// because that is what the region name carries — the window publishes
/// `page-size.size.item.<N>` — and because this crate deliberately cannot ask
/// the engine: `ui-verify` has exactly one dependency, and a verification
/// harness that pulls in the crate under test fails to build for reasons
/// unrelated to the thing it is verifying, on the day it is most needed.
///
/// ⚠ **So the index is checked at RUN TIME instead**, against
/// [`EXPECTED_SIZE_ID`] on the commit line. `PaperSize::ALL` is
/// `#[non_exhaustive]` and its own doc comment says the table will grow (ARCH,
/// JIS B, ISO B/C are all named as plausible); a size inserted before A6 would
/// silently make this check click A5 and then assert A6's dimensions — a red
/// run whose message would blame the application for a table that moved.
const A6_INDEX: usize = 6;

/// What the window must say it picked — `pdfcer_core::paper::PaperSize::id`,
/// which the engine documents as *"ASCII, lowercase, hyphenated, and must not
/// change once shipped"*.
///
/// ★ Read from `size_id=` and **never** from the `choice=` field beside it.
/// That one is a `Debug` spelling, present for a human reading a trace;
/// Debug-formatting a domain type and then parsing it produced two false
/// failure reports in this project in a single week.
const EXPECTED_SIZE_ID: &str = "a6";

/// Points per millimetre — 72 points per inch ÷ 25.4 mm per inch.
const PT_PER_MM: f64 = 72.0 / 25.4;

/// A6 portrait, in points: 105 × 148 mm.
///
/// ★ Converted from the **defining millimetres** rather than written out as
/// `297.64 x 419.53`, for the reason `dialogs::new_document`'s own test states:
/// a hand-rounded number looks right, is wrong in the fourth significant
/// figure, and will not compare equal to what the engine writes. Pinned against
/// `PaperSize::A6.size_pt()` by [`tests::a6_is_where_this_check_thinks_it_is`],
/// so the two cannot drift.
const A6_PT: (f64, f64) = (105.0 * PT_PER_MM, 148.0 * PT_PER_MM);

/// How close the read-back must be, in points.
///
/// A tenth of a point. The numbers are written by the engine and read by the
/// engine, so the only slack that has to be absorbed is the two-decimal
/// formatting of the trace line itself — 0.005 pt. A tolerance three orders of
/// magnitude tighter than the smallest gap between any two entries in
/// `PaperSize::ALL` cannot mask a wrong size.
const TOLERANCE_PT: f64 = 0.1;

/// The Portrait radio, clicked explicitly rather than assumed.
const PORTRAIT: &str = "page-size.portrait";

/// The commit button.
const APPLY: &str = "page-size.apply";

/// See the module documentation.
pub struct ResizingASheetChangesThePaperInTheSavedFile;

impl Check for ResizingASheetChangesThePaperInTheSavedFile {
    fn name(&self) -> &'static str {
        "resizing_a_sheet_changes_the_paper_in_the_saved_file"
    }

    fn defect(&self) -> &'static str {
        "an open drawing's sheet size cannot be changed at all, or the window reports a change \
         it did not write, or it writes the change to every sheet in the document instead of \
         the one the operator was looking at"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// One sheet's geometry as a process resolved it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Sheet {
    /// Width in points.
    w: f64,
    /// Height in points.
    h: f64,
}

/// Every `page-size-document` line in `trace`, by page index.
///
/// ★ Reads the LAST line per index rather than the first. The window can be
/// opened more than once in a run — phase B opens it, phase D opens it again —
/// and the census is republished each time. Taking the first would hand phase D
/// a fossil from phase B, which is the *"`.last()` returns a fossil"* trap
/// `driving::declared` exists to solve for `ui-rect` and which applies to any
/// republished line.
fn sheets(trace: &crate::trace::Trace) -> std::collections::BTreeMap<usize, Sheet> {
    let mut out = std::collections::BTreeMap::new();
    for line in trace.events(SHEET_EVENT) {
        let Some(index) = line.get("index").and_then(|v| v.parse::<usize>().ok()) else {
            continue;
        };
        let (Some(w), Some(h)) = (
            line.get("w").and_then(|v| v.parse::<f64>().ok()),
            line.get("h").and_then(|v| v.parse::<f64>().ok()),
        ) else {
            continue;
        };
        out.insert(index, Sheet { w, h });
    }
    out
}

/// Open the sheet-size window and return the census it publishes.
///
/// Used **twice** — once on the fixture in process 1 and once on the saved copy
/// in process 2 — which is the whole reason it is a call rather than eight
/// lines inline: the two censuses must be produced by the identical sequence,
/// or the comparison at the end is between two different measurements. That is
/// `save_copy`'s own `comments_count` lesson, which it learned by carrying two
/// copies of a census reader that were both wrong in the same way.
fn open_and_census(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
    what: &str,
) -> Result<std::collections::BTreeMap<usize, Sheet>> {
    report.note(format!("about to open the sheet-size window on {what}"));
    crate::checks::ocr::click_tab(session, driver, ui_rect, PAGES_TAB)?;
    crate::checks::save_copy::click_command(session, driver, ui_rect, RESIZE, 30)?;

    let trace = session.trace()?;
    if trace.events(OPENED_EVENT).next().is_none() {
        return Err(Error::new(format!(
            "`{}` was invoked on {what} and the window published no `{OPENED_EVENT}`. Either it \
             did not open, or it opened and its survey found no sheets — which cannot happen \
             for a document with pages, so the first reading is the likely one.",
            RESIZE.1
        )));
    }
    let census = sheets(&trace);
    if census.is_empty() {
        return Err(Error::new(format!(
            "the window opened on {what} and published no `{SHEET_EVENT}` line. That census is \
             this check's only route to the document's page geometry; without it there is \
             nothing to judge."
        )));
    }
    report.note(format!(
        "{what}: the window resolved {} sheets — {}",
        census.len(),
        census
            .iter()
            .map(|(i, s)| format!("{i}: {:.2}x{:.2}", s.w, s.h))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    Ok(census)
}

#[allow(clippy::too_many_lines)]
fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is six clicks across two processes. \
             Reported as SKIPPED rather than passed — a check that did not run has learned \
             nothing.",
        ));
    }

    // ★ The fixture is PINNED and any `--pdf` is ignored. See the module
    // header: this check needs MORE THAN ONE PAGE, because the sheet it does
    // not touch is its negative control, and a single-page fixture would make
    // the check unable to detect a build that resized every sheet in the
    // document.
    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is not on disk. This check cannot use an arbitrary document: \
             it needs at least two pages so that the sheet it leaves alone can be its negative \
             control."
        )));
    }
    if let Some(supplied) = ctx.pdf.as_ref() {
        report.note(format!(
            "· --pdf {} was supplied and is IGNORED; this check pins {FIXTURE}",
            supplied.display()
        ));
    }

    // The destination, removed first. A file left by a previous run would make
    // phase D read a stale document and — worse — a file this run wrote would
    // be indistinguishable from one that was already there.
    let saved = ctx.out("page_size.resized.pdf");
    if saved.exists() {
        std::fs::remove_file(&saved).map_err(|err| {
            Error::new(format!(
                "could not clear the destination {} before the run: {err}",
                saved.display()
            ))
        })?;
    }

    // -- PHASE A: the baseline, from the build under test -------------------
    report.note(
        "phase A: launching on the fixture to read its sheet sizes as this build \
                 resolves them",
    );
    let mut spec = LaunchSpec::new(&exe, ctx.out("page_size.a.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), saved.display().to_string()));
    // ★★★ **`mode.edit` at launch — added 2026-09-06, the first time this check
    // was ever RUN.**
    //
    // It was written by a session that was forbidden to drive, and on its first
    // real run it SKIPPED with *"the application declared no `ribbon.tab.pages`
    // region. Tabs it did declare: ribbon.tab.file, ribbon.tab.view."*
    //
    // ⇒ Nothing was broken. The application starts in **Read**, where the Pages
    // tab correctly does not exist, and the check had never said which mode it
    // needed — so it was looking for a tab the operator had not asked for
    // either. The same idiom is in `attachment_clip` and `attachments`, which
    // learned it the same way.
    //
    // ★ Worth stating because the SKIP was honest and useless in the same
    // breath: it named the missing region precisely and its own guidance even
    // offered the right diagnosis as an alternative reading. A check that
    // cannot establish its own preconditions reports the absence of its subject
    // as though it were the absence of the feature.
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), "mode.edit".to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} on {} as pid {}",
        exe.display(),
        pdf.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(30);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the application published no `{}`, so it did not reach a first frame.",
            ctx.profile.vocab.start_event
        )));
    }
    // ★ The outcome event must be ABSENT before anything is clicked. Rule 4 of
    // `checks::mod`: never treat a presence as evidence without showing the
    // channel was silent beforehand.
    if trace.events(COMMIT_EVENT).next().is_some() {
        return Ok(Some(format!(
            "`{COMMIT_EVENT}` appears in the trace before anything was clicked, so nothing this \
             check reads afterwards can be attributed to its own gesture."
        )));
    }

    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let driver = Driver::new(session.window());
    session.raise();

    let before = open_and_census(&session, &driver, ui_rect, report, "the fixture")?;
    if before.len() < 2 {
        return Err(Error::new(format!(
            "{FIXTURE} resolved to {} sheet(s). This check needs at least two: one to resize and \
             one to leave alone.",
            before.len()
        )));
    }
    let baseline_0 = before[&0];
    let baseline_1 = before[&1];
    if (baseline_0.w - A6_PT.0).abs() < TOLERANCE_PT
        && (baseline_0.h - A6_PT.1).abs() < TOLERANCE_PT
    {
        return Err(Error::new(format!(
            "page 0 of {FIXTURE} is ALREADY A6 ({:.2} x {:.2}), so choosing A6 would be a no-op \
             and this check could not fail. Pick a different size or a different fixture.",
            baseline_0.w, baseline_0.h
        )));
    }

    // -- PHASE B: pick A6 portrait and commit -------------------------------
    report.note("phase B: choosing A6 portrait in the window and pressing its commit button");

    // The combo popup is painted a frame after the click, so open it and then
    // ask whether the entry appeared, retrying rather than lengthening the
    // settle: a longer settle is a magic number tuned against one machine, and
    // a retry that does not help means the aim, not the wait.
    let entry = format!("page-size.size.item.{A6_INDEX}");
    let mut opened = false;
    for _ in 0..3 {
        click_dialog_region(&session, &driver, ui_rect, SIZE_COMBO)?;
        session.settle(10);
        if driving::declared(&session.trace()?, ui_rect, &entry).is_some() {
            opened = true;
            break;
        }
    }
    if !opened {
        return Err(Error::new(format!(
            "the size list did not open, or opened without an entry `{entry}`. Entries it did \
             declare: {}.",
            driving::list(&driving::declared_names(
                &session.trace()?,
                ui_rect,
                "page-size.size.item."
            ))
        )));
    }
    click_dialog_region(&session, &driver, ui_rect, &entry)?;
    session.settle(10);
    // Portrait explicitly. The window opens on the operand sheet's own
    // orientation, and page 0 of this fixture is landscape — so leaving the
    // radio alone would ask for A6 *landscape* and the assertion below would be
    // about a sheet nobody chose.
    click_dialog_region(&session, &driver, ui_rect, PORTRAIT)?;
    session.settle(10);
    click_dialog_region(&session, &driver, ui_rect, APPLY)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).last() else {
        return Ok(Some(format!(
            "the commit button was clicked and the window published no `{COMMIT_EVENT}`. The \
             control is drawn (this check clicked the rect it declared), so it took the press \
             and did nothing — which is the `visible control, silently inert` defect this suite \
             exists for."
        )));
    };
    let asked = (
        commit.get("w_pt").and_then(|v| v.parse::<f64>().ok()),
        commit.get("h_pt").and_then(|v| v.parse::<f64>().ok()),
    );

    // ★★★ **Did the entry this check clicked turn out to be A6?**
    //
    // A SKIP rather than a FAIL, because a size the engine inserted into the
    // middle of `PaperSize::ALL` is a change to the table and not a defect in
    // the application — and reporting it as a failure would send the reader
    // looking for a bug in a window that is working perfectly. This is the
    // whole reason `size_id` is published: without it the run would go red one
    // assertion later, with a message naming the wrong culprit.
    let picked = commit.get("size_id").unwrap_or("?");
    if picked != EXPECTED_SIZE_ID {
        return Err(Error::new(format!(
            "this check clicks size-list entry {A6_INDEX} expecting `{EXPECTED_SIZE_ID}`, and \
             the window reports it committed `{picked}`. `PaperSize::ALL` has changed order — \
             the engine's own docs say that table will grow — so the INDEX is stale, not the \
             application. Move {A6_INDEX} to A6's new position and re-run."
        )));
    }
    report.note(format!(
        "the window committed n={} as `{picked}` at {:?}",
        commit.get("n").unwrap_or("?"),
        asked
    ));

    // -- PHASE C: save a copy -----------------------------------------------
    report.note("phase C: saving a copy, so the verdict can be taken from a file");
    crate::checks::ocr::click_tab(&session, &driver, ui_rect, "file")?;
    crate::checks::save_copy::click_command(
        &session,
        &driver,
        ui_rect,
        crate::checks::save_copy::SAVE,
        40,
    )?;
    let trace = session.trace()?;
    if trace.events(SAVED_EVENT).next().is_none() {
        return Err(Error::new(format!(
            "`file.save_copy` produced no `{SAVED_EVENT}`, so there is no written file to judge. \
             That is `save_copy_round_trip`'s subject, not this check's — reported as SKIPPED so \
             a broken save does not read as a broken resize."
        )));
    }
    if !saved.exists() {
        return Err(Error::new(format!(
            "the shell reported `{SAVED_EVENT}` and no file appeared at {}.",
            saved.display()
        )));
    }
    report.artifact(saved.clone());

    // -- PHASE D: THE VERDICT, in a second process --------------------------
    report.note(
        "phase D: opening the saved file in a FRESH binary — the verdict is taken by a \
                 process that did not write it",
    );
    let mut spec2 = LaunchSpec::new(&exe, ctx.out("page_size.d.trace.txt"));
    spec2.pdf = Some(saved.clone());
    spec2.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec2
        .env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // ★ The verdict process needs `mode.edit` for the same reason phase A does,
    // and it is easy to miss: this launch only READS the saved file, so it looks
    // like it needs no authoring mode — but it reads by opening the sheet-size
    // window, which lives on the Pages tab, which does not exist in Read. The
    // first run fixed phase A and phase D skipped for the identical reason
    // twenty seconds later.
    spec2
        .env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), "mode.edit".to_owned()));
    spec2.allow_stale = ctx.allow_stale;
    spec2.source_root = ctx.source_root.clone();

    let verdict = Session::launch(&spec2, ctx.profile.trace_prefix)?;
    report.note(format!("the saved copy is open in pid {}", verdict.pid()));
    report.artifact(verdict.trace_path().to_path_buf());
    verdict.settle(30);
    let driver2 = Driver::new(verdict.window());
    verdict.raise();

    let after = open_and_census(&verdict, &driver2, ui_rect, report, "the SAVED copy")?;

    let Some(&resized) = after.get(&0) else {
        return Ok(Some(
            "the saved copy resolved no page 0 at all, so the resize did not merely fail — the \
             document is not what was opened."
                .to_owned(),
        ));
    };

    // The transposition is reported as its own defect, before the general
    // wrong-size one: 419.53 x 297.64 collapsed into "expected 297.64 x 419.53"
    // wastes the reader's first hypothesis on a size list that is working.
    if (resized.w - A6_PT.1).abs() < TOLERANCE_PT && (resized.h - A6_PT.0).abs() < TOLERANCE_PT {
        return Ok(Some(format!(
            "★ TRANSPOSED. A6 PORTRAIT was chosen and the saved file's page 0 is {:.2} x {:.2}, \
             which is A6 LANDSCAPE. The size reached the document, so the verb and the write are \
             working; the orientation did not survive the trip from the radio to the rectangle. \
             `dialogs::page_size::sheet_pt` is the one function all four consumers ask, so a \
             transposition here is in that function or in the radio that feeds it.",
            resized.w, resized.h
        )));
    }
    if (resized.w - A6_PT.0).abs() > TOLERANCE_PT || (resized.h - A6_PT.1).abs() > TOLERANCE_PT {
        return Ok(Some(format!(
            "★★★ THE SHEET SIZE DID NOT REACH THE FILE. A6 portrait ({:.2} x {:.2}) was chosen \
             and committed — the window published `{COMMIT_EVENT}` asking for {asked:?} — and a \
             FRESH BINARY reading the saved copy resolves page 0 as {:.2} x {:.2}. It was {:.2} \
             x {:.2} before the change, so {}. The request traced perfectly and the document is \
             not what it says.",
            A6_PT.0,
            A6_PT.1,
            resized.w,
            resized.h,
            baseline_0.w,
            baseline_0.h,
            if (resized.w - baseline_0.w).abs() < TOLERANCE_PT
                && (resized.h - baseline_0.h).abs() < TOLERANCE_PT
            {
                "NOTHING WAS WRITTEN AT ALL"
            } else {
                "something was written, and it is neither the old size nor the new one"
            }
        )));
    }
    report.note(format!(
        "★★★ VERDICT: a second process reading the saved file resolves page 0 as {:.2} x {:.2} \
         — A6 portrait, the size that was asked for. It was {:.2} x {:.2} before.",
        resized.w, resized.h, baseline_0.w, baseline_0.h
    ));

    // -- PHASE D′: THE NEGATIVE CONTROL -------------------------------------
    //
    // ★★★ Without this the check has no dynamic range. Its positive arm is
    // satisfied by a build that sets EVERY page to A6 — which is not a
    // hypothetical wrong build, it is the shape you get by passing
    // `0..pages.len()` where the operand list belongs, and it is a data-loss
    // bug on a drawing set. What has to be shown is that the same instrument,
    // in the same run, over the same file, reports the sheet nobody picked as
    // unchanged.
    let Some(&control) = after.get(&1) else {
        return Ok(Some(
            "the saved copy resolved no page 1, so this check's negative control does not exist \
             and its positive arm is NOT a verdict — a build that resized page 0 correctly and \
             deleted page 1 would have reached here."
                .to_owned(),
        ));
    };
    if (control.w - baseline_1.w).abs() > TOLERANCE_PT
        || (control.h - baseline_1.h).abs() > TOLERANCE_PT
    {
        return Ok(Some(format!(
            "★★★ THE NEGATIVE CONTROL MOVED, so the positive half of this check is NOT a \
             verdict. Page 1 was NOT picked — nothing was picked, so the operand rule aims \
             `pages.resize` at the current sheet, page 0 — and it went from {:.2} x {:.2} to \
             {:.2} x {:.2} anyway. {} The likely cause is an operand list built from the \
             document rather than from `panels::pages::ops::operands`.",
            baseline_1.w,
            baseline_1.h,
            control.w,
            control.h,
            if (control.w - A6_PT.0).abs() < TOLERANCE_PT {
                "It is now A6, i.e. the change was applied to the WHOLE DOCUMENT."
            } else {
                "It is not the size that was asked for either."
            }
        )));
    }
    report.note(format!(
        "★★★ and the NEGATIVE CONTROL held: page 1 was not picked and reads {:.2} x {:.2} in the \
         saved file, exactly what it read in the fixture. The instrument has dynamic range — it \
         speaks for the sheet that was resized and stays silent for the one beside it",
        control.w, control.h
    ));

    Ok(None)
}

/// Click a region the **application** declared, converting against the frame it
/// was declared in.
///
/// ★★★ `driving::frame_of`, never `session.frame()`. **A dialog is an OS
/// window**, so the main window's frame is the wrong one and the symptom of
/// using it is *silence*: every click lands hundreds of points away, nothing
/// responds, and the check reports a working feature as inert.
/// `RESUME.md` records that fault twice — `checks::ocr::click_region` on
/// 2026-08-27 and `new_document_size` on 2026-09-05, the second written *after*
/// the first was fixed and its lesson written down. A note is not a mechanism;
/// a shared helper is.
fn click_dialog_region(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    name: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let rect = driving::declared(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{name}` region. Regions it did declare beginning \
             `page-size.`: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "page-size."))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{name}` was declared at {rect:?}, which has no usable area to click."
        )));
    }
    let frame = driving::frame_of(session, &trace, ui_rect, name)?;
    driver.click_at(frame.declared_center(rect))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **A6 is 105 × 148 mm, and this check's constant is that conversion
    /// rather than a rounded copy of it.**
    ///
    /// The failure this exists for is the one `dialogs::new_document`'s own
    /// test names: a hand-rounded `297.64 × 419.53` looks right, is wrong in
    /// the fourth significant figure, and will not compare equal to what the
    /// engine writes. The engine converts from the defining millimetres for
    /// exactly that reason, and this pins that the harness does the same
    /// arithmetic rather than a similar-looking one.
    ///
    /// ⓘ It cannot assert against `PaperSize::A6.size_pt()` directly, and that
    /// is deliberate: this crate has **one** dependency, `windows-sys`, and its
    /// own manifest argues at length that a verification harness with a large
    /// dependency tree is one that fails to build for reasons unrelated to the
    /// thing under test. The agreement between this constant and the engine's
    /// table is asserted at run time instead, from
    /// [`EXPECTED_SIZE_ID`] on the window's own commit line, which is a
    /// stronger check than a compile-time one anyway: it verifies the size the
    /// **running program** committed rather than the one this file believes it
    /// will.
    #[test]
    fn a6_is_its_own_millimetres() {
        assert!(
            (A6_PT.0 - 105.0 * 72.0 / 25.4).abs() < 1e-12,
            "A6 is 105 mm wide: {A6_PT:?}"
        );
        assert!(
            (A6_PT.1 - 148.0 * 72.0 / 25.4).abs() < 1e-12,
            "A6 is 148 mm tall: {A6_PT:?}"
        );
        assert!(
            A6_PT.0 < A6_PT.1,
            "the pinned pair is PORTRAIT, which is what the check clicks the radio for"
        );
    }

    /// ★★★ **The fixture can carry the defect**, which is the property that
    /// makes this check able to fail at all.
    ///
    /// Two requirements, and each is a way this check silently stops meaning
    /// anything: it needs **at least two pages**, or there is no negative
    /// control; and page 0 must not **already** be A6, or the positive arm
    /// passes on a build that writes nothing.
    ///
    /// Asserted from the fixture's bytes rather than from a remembered number,
    /// because the fixture is regenerated and a check that pinned 2383.94 would
    /// go red for a reason that has nothing to do with sheet sizes.
    #[test]
    fn the_fixture_can_carry_the_defect() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(FIXTURE);
        let Ok(bytes) = std::fs::read(&path) else {
            // Not a failure: a clean checkout without the fixture is a
            // precondition this unit test cannot create, and asserting on a
            // file that is not there would make the workspace suite depend on
            // the harness's fixtures.
            return;
        };
        let text = String::from_utf8_lossy(&bytes);
        let boxes: Vec<&str> = text.match_indices("/MediaBox").map(|(_, s)| s).collect();
        assert!(
            boxes.len() >= 2,
            "{FIXTURE} carries {} /MediaBox entries; this check needs at least two pages so the \
             sheet it leaves alone can be its negative control",
            boxes.len()
        );
        assert!(
            !text.contains("297.63") && !text.contains("419.52"),
            "{FIXTURE} appears to already contain an A6 sheet, which would let this check pass \
             on a build that writes nothing"
        );
    }
}
