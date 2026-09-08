//! `a_text_file_becomes_pages` — **File ▸ Import text as pages, driven: a
//! `.txt` on disk becomes real pages in the open document.**
//!
//! The other half of the operator's 2026-09-04 ask — *"we should have
//! export/import for that"* — which was half a feature for two days because
//! `pdfcer-core` could not create a page, only copy one. `blank_document` and
//! `place_text` shipped as `Pass 252.0`; this is what says the shell reached
//! them.
//!
//! ## ★★★ The five links, and four of them have no test anywhere else
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | the ribbon item exists and dispatches | `reach::every_registered_command_is_routed_or_argued` — that an arm EXISTS, not that it runs |
//! | 2 | the picker's answer reaches the window | **nothing** |
//! | 3 | **the window's four choices reach the template** | `dialogs::import_text::tests` — the conversion, given a dialog |
//! | 4 | **Import raises the action and the apply arm reads the file** | **nothing** |
//! | 5 | **the engine's report becomes a receipt the operator can read** | `actions::importtext::tests` — the sentences, given a report |
//!
//! **Link 4 is the one that would ship as silence.** The dialog is its own OS
//! window; a button whose published rect is right and whose click lands on the
//! main window instead does nothing, says nothing, and looks exactly like a
//! feature that was never wired.
//!
//! ## ★★ `frame_of`, never `session.frame()` — this is a DIALOG
//!
//! A dialog is a separate viewport with its own client rect. This project spent
//! a driven run discovering that: every in-dialog click landed hundreds of
//! points away and the symptom was *silence*. `driving::frame_of` resolves the
//! frame the region was published in, and it is safe on main-window regions
//! too, so there is no reason to reach for the other one.
//!
//! ## ★ The picker is an OS dialog, so it is bypassed by the env seam
//!
//! `PDFCER_DIAG_TEXT_IMPORT_PATH` — the same seam `pick_form_data_source`,
//! `pick_document` and `pick_image_source` all carry, and for the same reason:
//! a native file dialog is a window this harness cannot type into, so without
//! it the check could press the ribbon item and get no further.
//!
//! ⚠ **The seam is not a shortcut around the feature.** Everything after the
//! picker — the window, the four choices, the button, the action, the file
//! read, the engine call and the receipt — is the real thing.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, frame_of, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Fire the command at start-up; the picker is answered by the env seam below.
const INVOKE: &str = "file.import_text";
/// The line the picker writes, naming where its answer came from.
const PICKED_EVENT: &str = "text-import-picked";
/// The line the window writes when Import is pressed, carrying every choice.
const REQUESTED_EVENT: &str = "import-text-requested";
/// ★★★ The line the apply arm writes when the engine has made the pages.
const APPLIED_EVENT: &str = "import-text-applied";
/// The window's body, published so a failure can say whether it opened at all.
const BODY_REGION: &str = "import-text.body";
/// The Import button.
const IMPORT_REGION: &str = "import-text.import";
/// The env seam that answers the picker.
const PATH_ENV: &str = "PDFCER_DIAG_TEXT_IMPORT_PATH";

/// The text this check imports.
///
/// ★★ Chosen so the **receipt has something to say beyond the page count**: the
/// em dashes are characters WinAnsi can encode (so the import must not refuse),
/// and the blank lines make paragraphs the placer has to decide about. A file of
/// bare ASCII words would exercise the chain and prove nothing about the
/// disclosures.
///
/// ★ Written by the check rather than committed as a fixture, because it is
/// three lines and because a fixture would have to be found and read to know
/// what the assertions below mean.
const REGISTER: &str = "Drawing register\n\nSheet 01 \u{2014} site plan, rev C\nSheet 02 \u{2014} foundations, rev A\nSheet 03 \u{2014} steelwork, rev B\n\nIssued for construction.\n";

/// See the module documentation.
pub struct ATextFileBecomesPages;

impl Check for ATextFileBecomesPages {
    fn name(&self) -> &'static str {
        "a_text_file_becomes_pages"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Import text as pages opens a window, the operator chooses a sheet and presses \
         Import, and nothing arrives in the document — the button reaches no verb, or the apply \
         arm never reads the file, or the engine refuses and the refusal is swallowed. All three \
         look identical from a chair: a window that closes and a document that did not change"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses a button in a dialog window, \
             which is a real pointer gesture.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. An import is an INSERT — `place_text` refuses \
             `NoPageToInsertBeside` on a document with no page — so this check needs a document \
             open, and the ribbon item is greyed on `doc.pages` for the same reason.",
        )
    })?;

    // The text to import, written beside the trace so a failure can be read
    // against the exact bytes that produced it.
    let source = ctx.out("import-text.register.txt");
    std::fs::write(&source, REGISTER)
        .map_err(|why| Error::new(format!("could not write {}: {why}", source.display())))?;
    report.artifact(source.clone());

    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("import-text.trace.txt"));
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
        .push((PATH_ENV.to_owned(), source.to_string_lossy().into_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with {INVOKE} and the picker answered from {PATH_ENV}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);

    // --- 1: did the picker's answer arrive? ---------------------------------
    let trace = session.trace()?;
    let Some(picked) = trace.events(PICKED_EVENT).last() else {
        return Ok(Some(format!(
            "THE COMMAND REACHED NO PICKER: no `{PICKED_EVENT}` line after `{INVOKE}`. \
             `app::files::pick_text_source` traces unconditionally on both routes, so its \
             absence means the dispatch arm never ran — check that \
             `dispatch::exchange::claims` names `file.import_text` and that the command is \
             registered. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if picked.get("source") != Some("env") {
        return Err(Error::new(format!(
            "the picker answered from `{}` rather than from the environment seam, so a NATIVE \
             file dialog is open and this run cannot continue. {PATH_ENV} was set; check \
             `app::files::from_env`. Line: `{}`.",
            picked.get("source").unwrap_or("?"),
            picked.raw
        )));
    }
    report.note("★ the picker took the file from the environment seam");

    // --- 2: did the window open? --------------------------------------------
    let trace = session.trace()?;
    let Some(body) = declared(&trace, ui_rect, BODY_REGION) else {
        return Ok(Some(format!(
            "THE IMPORT WINDOW DID NOT OPEN: the picker answered and no `{BODY_REGION}` region \
             followed. `dialogs::open::open_import_text` sets the dialog and \
             `dialogs::show` draws it; one of the two did not happen. Regions declared: {}. \
             Trace: {}.",
            list(&declared_names(&trace, ui_rect, "import-text")),
            session.trace_path().display()
        )));
    };
    let _ = body;
    report.note("★ the window opened");

    // --- 3: ★★★ PRESS IMPORT ------------------------------------------------
    let trace = session.trace()?;
    let Some(button) = declared(&trace, ui_rect, IMPORT_REGION) else {
        return Ok(Some(format!(
            "THE WINDOW OPENED AND PUBLISHED NO IMPORT BUTTON: no `{IMPORT_REGION}`. It is \
             published with `ui_rect_visible`, so an absent region means it is not on screen — \
             the window's opening size may be too short for its content. Trace: {}.",
            session.trace_path().display()
        )));
    };
    // ★★ `frame_of`, not `session.frame()` — see the module header. The import
    // window is its own viewport and its client rect is not the main window's.
    let frame = frame_of(&session, &trace, ui_rect, IMPORT_REGION)?;
    let driver = Driver::new(session.window());
    driver.click_at(frame.declared_at(button, 0.5, 0.5))?;
    session.settle(40);

    // --- 4: did the button raise the action, with the CHOICES on it? --------
    let trace = session.trace()?;
    let Some(requested) = trace.events(REQUESTED_EVENT).last() else {
        return Ok(Some(format!(
            "★★★ IMPORT WAS PRESSED AND RAISED NOTHING: no `{REQUESTED_EVENT}` line. The press \
             was made at the centre of the rect the application declared, in that region's own \
             viewport frame.\n\
             If the click landed on the MAIN window instead, this is the dialog-frame trap — but \
             this check uses `driving::frame_of` precisely to avoid it, so look instead at \
             whether `ImportTextDialog::show` consumes `import_requested` in the same frame it \
             is set. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★ the window raised the action: `{}`",
        requested.raw
    ));
    // ★ The choices are asserted, not just the press. A build whose chooser
    // never reached the template raises an identical action from an identical
    // click and makes pages of the wrong size — which reads as a rendering
    // problem three steps later.
    if requested.get("sheet") != Some("a4") {
        return Ok(Some(format!(
            "the window raised `sheet={}` and it opens on A4. Either the default moved or the \
             chooser is not reaching the template — `dialogs::import_text::tests::\
             the_window_opens_on_a4_whatever_order_the_engine_lists_its_sheets_in` covers the \
             first half in process. Line: `{}`.",
            requested.get("sheet").unwrap_or("?"),
            requested.raw
        )));
    }

    // --- 5: ★★★ DID PAGES ARRIVE? -------------------------------------------
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        return Ok(Some(format!(
            "★★★ THE ACTION WAS RAISED AND NO PAGES ARRIVED: `{}` and no `{APPLIED_EVENT}` \
             line.\n\
             The apply arm runs, so either the file could not be read, or it was not UTF-8, or \
             `place_text` refused — and all three are recorded on the status row rather than \
             here. Read the disclosure line in the trace: `NoColumn` and `PageTooShort` mean \
             the margins leave no room on the chosen sheet, and `Unmappable` names every \
             character the face could not write. Trace: {}.",
            requested.raw,
            session.trace_path().display()
        )));
    };
    let pages: usize = applied
        .get("pages")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if pages == 0 {
        return Ok(Some(format!(
            "★★★ THE IMPORT REPORTED ZERO PAGES: `{}`. Seven lines of text became no sheets, \
             which `place_text` refuses by name (`EmptyText`, `NoWordsToPlace`) rather than \
             reporting — so a zero here means the report was read wrongly, not that the engine \
             made nothing. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the engine made {pages} page(s): `{}`",
        applied.raw
    ));

    // ★★ The undo promise, asserted rather than reported. Seven lines is one
    // page, far inside `MAX_UNDO_DEPTH`, so the fold MUST have worked — and a
    // build reporting `coalesced=0` here would be one whose promise of a single
    // Ctrl+Z is false on every import, which is the failure
    // `text::importtext::many_undo_steps` exists to disclose and which nobody
    // would notice until they pressed undo.
    if applied.get("coalesced") != Some("1") {
        return Ok(Some(format!(
            "★★ A ONE-PAGE IMPORT WAS NOT FOLDED INTO ONE UNDO ENTRY: `{}`. `coalesced` is \
             false, so the operator is told this will take several presses of Undo to reverse — \
             on an import of seven lines. The engine folds up to `MAX_UNDO_DEPTH` commands; \
             something is defeating that. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ and it is one Ctrl+Z, which is what the receipt promises");

    Ok(None)
}
