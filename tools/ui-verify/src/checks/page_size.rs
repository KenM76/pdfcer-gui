//! `resizing_a_sheet_changes_the_paper_in_the_saved_file` — picking a sheet
//! size for the page an operator is looking at must change **that page's
//! `/MediaBox` in the file he then saves**, and must leave the page beside it
//! alone.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_size.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// The fixture, pinned. See the module header: this check needs **more than
/// one page**, so that the sheet it does not touch can be the control.
const FIXTURE: &str = "fixtures/cropped-sheets.pdf";

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
const A6_INDEX: usize = 6;

/// What the window must say it picked — `pdfcer_core::paper::PaperSize::id`,
/// which the engine documents as *"ASCII, lowercase, hyphenated, and must not
/// change once shipped"*.
const EXPECTED_SIZE_ID: &str = "a6";

/// Points per millimetre — 72 points per inch ÷ 25.4 mm per inch.
const PT_PER_MM: f64 = 72.0 / 25.4;

/// A6 portrait, in points: 105 × 148 mm.
const A6_PT: (f64, f64) = (105.0 * PT_PER_MM, 148.0 * PT_PER_MM);

/// How close the read-back must be, in points.
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

/// The canvas line naming the visible area it frames.
const CANVAS_EVENT: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed

/// The visible area (`/CropBox`) the canvas last framed page 0 by, as width and height.
fn visible_area(trace: &crate::trace::Trace) -> Option<(f64, f64)> {
    let crop = trace
        .events(CANVAS_EVENT)
        .filter(|l| l.get("page") == Some("0"))
        .last()?
        .get("crop")?
        .to_owned();
    let v: Vec<f64> = crop.split(',').filter_map(|n| n.parse().ok()).collect();
    (v.len() == 4).then(|| (v[2] - v[0], v[3] - v[1]))
}

/// Every `page-size-document` line in `trace`, by page index.
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
fn open_and_census(
    session: &Session,
    driver: &ScriptedPointer,
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
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;

    // The fixture is PINNED and any `--pdf` is ignored. See the module
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
    // Worth stating because the SKIP was honest and useless in the same
    // breath: it named the missing region precisely and its own guidance even
    // offered the right diagnosis as an alternative reading. A check that
    // cannot establish its own preconditions reports the absence of its subject
    // as though it were the absence of the feature.
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), "mode.edit".to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("page_size.a.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
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
    // The outcome event must be ABSENT before anything is clicked. Rule 4 of
    // `checks::mod`: never treat a presence as evidence without showing the
    // channel was silent beforehand.
    if trace.events(COMMIT_EVENT).next().is_some() {
        return Ok(Some(format!(
            "`{COMMIT_EVENT}` appears in the trace before anything was clicked, so nothing this \
             check reads afterwards can be attributed to its own gesture."
        )));
    }

    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let driver = &pointer;

    let before = open_and_census(&session, driver, ui_rect, report, "the fixture")?;
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
        click_dialog_region(&session, driver, ui_rect, SIZE_COMBO)?;
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
    click_dialog_region(&session, driver, ui_rect, &entry)?;
    session.settle(10);
    // Portrait explicitly: the window opens on the operand sheet's own
    // orientation, so the assertion below is about a sheet this check chose.
    click_dialog_region(&session, driver, ui_rect, PORTRAIT)?;
    session.settle(10);
    click_dialog_region(&session, driver, ui_rect, APPLY)?;
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

    // **Did the entry this check clicked turn out to be A6?**
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
    crate::checks::ocr::click_tab(&session, driver, ui_rect, "file")?;
    crate::checks::save_copy::click_command(
        &session,
        driver,
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
    // The verdict process needs `mode.edit` for the same reason phase A does,
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
    spec2
        .env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec2.place = false;
    let pointer2 = ScriptedPointer::attach(&mut spec2, ctx.out("page_size.d.pointer.txt"))?;

    let verdict = Session::launch(&spec2, ctx.profile.trace_prefix)?;
    report.artifact(pointer2.path().to_path_buf());
    report.note(format!("the saved copy is open in pid {}", verdict.pid()));
    report.artifact(verdict.trace_path().to_path_buf());
    verdict.settle(30);

    let after = open_and_census(&verdict, &pointer2, ui_rect, report, "the SAVED copy")?;

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

    // The visible area must follow the paper. Page 0's `/CropBox` matched its
    // old sheet; left behind, every viewer shows the old size (O250).
    let Some(seen) = visible_area(&verdict.trace()?) else {
        return Err(Error::new(format!(
            "the verdict process published no `{CANVAS_EVENT}` line for page 0 with a `crop=`, so the visible area cannot be judged."
        )));
    };
    if (seen.0 - A6_PT.0).abs() > TOLERANCE_PT || (seen.1 - A6_PT.1).abs() > TOLERANCE_PT {
        return Ok(Some(format!(
            "★★★ THE VISIBLE AREA DID NOT FOLLOW THE PAPER. The saved page 0 is A6, and the canvas frames it by a visible area of {:.2} x {:.2}. Its `/CropBox` matched the old sheet, so it should have moved with the resize; left behind, every viewer shows the page at its old size and the resize looks like it did nothing.",
            seen.0, seen.1
        )));
    }
    report.note(format!(
        "and its visible area followed: the canvas frames page 0 at {:.2} x {:.2}",
        seen.0, seen.1
    ));

    // -- PHASE D′: THE NEGATIVE CONTROL -------------------------------------
    //
    // Without this the check has no dynamic range. Its positive arm is
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
fn click_dialog_region(
    session: &Session,
    driver: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = driving::declared_in(&trace, ui_rect, name).ok_or_else(|| {
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
    driver.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A6 is 105 × 148 mm, and this check's constant is that conversion
    /// rather than a rounded copy of it.**
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

    /// **The fixture can carry the defect**, which is the property that
    /// makes this check able to fail at all.
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
            text.contains("/CropBox"),
            "{FIXTURE} carries no /CropBox, so a visible area left behind by the resize could not be told from one that moved"
        );
        assert!(
            !text.contains("297.63") && !text.contains("419.52"),
            "{FIXTURE} appears to already contain an A6 sheet, which would let this check pass \
             on a build that writes nothing"
        );
    }
}
