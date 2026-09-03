//! `new_document_sizes_the_page` — picking A3 landscape must produce an A3
//! landscape page, not an A4 portrait one with an A3 label.
//!
//! # What this is about
//!
//! `file.new` makes an A4 page and asks nothing, which is what Acrobat and
//! Inkscape both do. `file.new_from_template` is the other half of
//! `RIBBON_IA.md` §5.1's row — the command that asks — and it could not exist
//! until 2026-08-18, because nothing in `pdfcer-core` wrote a `/MediaBox` and
//! the only shell-side implementation was one checked-in template asset per
//! size. That plan is recorded and refused in `app::blank`'s §3a; it could not
//! have answered a custom size at any number of assets.
//!
//! `EditSession::set_media_box` shipped, so the implementation is: parse the
//! one template, resize page 0, rewrite the whole file, **re-parse it**, and
//! hand the result over as an ordinary new document with nothing pending and
//! nothing undoable.
//!
//! # ★ The failure this exists for, and why the unit tests cannot see it
//!
//! There are four places the size can be lost, and each looks correct from the
//! one next to it:
//!
//! | where | what it looks like |
//! |---|---|
//! | the radio does not reach `sheet_pt` | the summary line reads right and the page is portrait |
//! | `Action::NewSized` carries the wrong pair | the dialog is right and the document is transposed |
//! | `set_media_box` is called and the rewrite drops it | the request traces perfectly and the page is 595 × 842 |
//! | the re-parse reads a different page | everything traces perfectly and the canvas shows A4 |
//!
//! The workspace's unit tests cover the first half of that chain and cannot
//! reach the second: `sheet_pt` is pinned in `dialogs::new_document::tests`,
//! and what happens after `to_full_bytes` is a property of a real parse of
//! real bytes. So this check reads `result_w` / `result_h` from the
//! `new-document-sized` trace line, which `app::blank` emits **after** the
//! re-parse from `pages[0].media_box` — the page as a reader of the file will
//! see it, not the rectangle that was asked for.
//!
//! # What it drives
//!
//! File tab → New from template… → open the size list → **A3** → **Landscape**
//! → Create. Then it asserts the resulting page is 420 × 297 mm within a
//! millimetre, in that order — the wrong way round is the transposition defect
//! and is reported as such rather than as "the wrong size".
//!
//! A3 rather than A4, deliberately: A4 is what the dialog opens on and what
//! `file.new` makes, so a build that ignored every control would still produce
//! it. A3 landscape differs from the default in **both** dimensions and in
//! orientation.
//!
//! # No fixture, and that is the point
//!
//! It launches with no `--pdf`. `file.new_from_template` is registered with no
//! `enabled_when` because an operator with nothing open is the one it exists
//! for, and a check that needed a document open would be testing a state the
//! command is least likely to be used from.

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The ribbon control, and the tab it lives on.
const SUBJECT: &str = "ribbon.item.file.new_from_template";
const TAB_ID: &str = "file";
const TAB: &str = "ribbon.tab.file";

/// The dialog's own regions.
const BODY: &str = "new-document.body";
const SIZE: &str = "new-document.size";
const SIZE_ITEM_PREFIX: &str = "new-document.size.item.";
const LANDSCAPE: &str = "new-document.landscape";
const CREATE: &str = "new-document.create";

/// The line `app::blank::document_sized` emits after the re-parse.
const SIZED_EVENT: &str = "new-document-sized";

/// The index of A3 in `pdfcer_core::paper::PaperSize::ALL`.
///
/// The list is A0, A1, A2, A3, … — largest-first, which is the engine's own
/// ordering and is deliberate: this operator's sheets are A1 and A3, and
/// burying them under A4 would make the common case the hard one. Spelled out
/// as a constant with the ordering stated so that a reordering of `ALL` fails
/// here with a readable reason rather than silently checking A2.
const A3_INDEX: usize = 3;

/// A3's short side and long side, in millimetres.
const A3_SHORT_MM: f64 = 297.0;
const A3_LONG_MM: f64 = 420.0;

/// How close is close enough, in millimetres.
///
/// A3 is 420 × 297 mm exactly by definition, and the points are derived from
/// that, so the round trip should be exact to well under a tenth of a
/// millimetre. One millimetre is a tolerance that cannot mask a real defect —
/// the smallest wrong answer available is A4 (210 × 297), which is 123 mm out.
const TOLERANCE_MM: f64 = 1.0;

pub struct NewDocumentSizesThePage;

impl Check for NewDocumentSizesThePage {
    fn name(&self) -> &'static str {
        "new_document_sizes_the_page"
    }

    fn defect(&self) -> &'static str {
        "the size chooser reports the sheet it was asked for and makes an A4 page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
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
            "input is disabled (--no-input), and this check is five clicks. Reported as SKIPPED \
             rather than passed — a check that did not run has learned nothing.",
        ));
    }

    // ★ No fixture. See the module header: the command is offered with nothing
    // open, on purpose, and that is the state to drive it from.
    let mut spec = LaunchSpec::new(&exe, ctx.out("new_document_size.trace.txt"));
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
        "launched {} as pid {} with no document",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(30);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so {}={} did not reach the process. Captured stderr is \
             at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    // ★ Nothing may have made a document yet.
    //
    // A New that fires on its own is `view.app_initiative`'s specified default
    // — Never — broken in the way that matters most, since this command
    // REPLACES what is open. On an empty shell it would be invisible; the
    // trace is the only place it shows.
    if trace.events(SIZED_EVENT).next().is_some() {
        return Ok(Some(format!(
            "`{SIZED_EVENT}` appears in the trace before anything was clicked. A document was \
             made without being asked for, and this command replaces what is open."
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- A. the File tab ----------------------------------------------------
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so no click \
             reached the ribbon."
        )));
    }

    // --- B. the control -----------------------------------------------------
    let trace = session.trace()?;
    let Some(control) = declared(&trace, ui_rect, SUBJECT) else {
        return Ok(Some(format!(
            "the File tab is active and none of its controls is `{SUBJECT}`. The command is \
             registered and the manifest places it beside `file.new` in the File band, so this \
             is either an overflowed ribbon or an item dropped for naming an unregistered \
             command. Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    if !control.is_substantial() {
        return Ok(Some(format!(
            "`{SUBJECT}` was declared at {control:?}, which has no usable area — the control is \
             laid out and not on screen."
        )));
    }
    driver.click_at(session.frame()?.declared_center(control))?;
    session.settle(15);

    let trace = session.trace()?;
    let Some(body) = declared(&trace, ui_rect, BODY) else {
        return Ok(Some(format!(
            "the click on `{SUBJECT}` published no `{BODY}` region, so the dialog did not open. \
             The command is dispatched to `DialogsState::open_new_document`; a registered command \
             with no arm traces `command-unimplemented`, and the shell trace is at {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the dialog opened at {body:?}"));

    // --- C. pick A3 ---------------------------------------------------------
    let size = declared(&trace, ui_rect, SIZE).ok_or_else(|| {
        Error::new(format!(
            "the dialog is open and published no `{SIZE}` region, so there is no size list to \
             choose from."
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(size))?;
    session.settle(12);

    let trace = session.trace()?;
    let target = format!("{SIZE_ITEM_PREFIX}{A3_INDEX}");
    let Some(entry) = declared(&trace, ui_rect, &target) else {
        let entries = declared_names(&trace, ui_rect, SIZE_ITEM_PREFIX);
        return Err(Error::new(format!(
            "the click on `{SIZE}` published no `{target}`. The popup did not open, or opened and \
             closed within the settle. Entries declared: {}. Reported as SKIPPED: this is a \
             harness timing question, not an application claim.",
            list(&entries)
        )));
    };
    driver.click_at(session.frame()?.declared_center(entry))?;
    session.settle(12);

    // --- D. turn it landscape -----------------------------------------------
    let trace = session.trace()?;
    let landscape = declared(&trace, ui_rect, LANDSCAPE).ok_or_else(|| {
        Error::new(format!(
            "the dialog published no `{LANDSCAPE}` region, so there is no orientation control."
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(landscape))?;
    session.settle(12);

    // --- E. create ----------------------------------------------------------
    //
    // The Create button is ABSENT rather than greyed when the size is out of
    // range — but a standard size is always in range, so its absence here is a
    // defect and not a state.
    let trace = session.trace()?;
    let Some(create) = declared(&trace, ui_rect, CREATE) else {
        return Ok(Some(format!(
            "the dialog is showing a standard size and published no `{CREATE}` region. That \
             button is absent only when the size is out of range, and A3 is not — so either the \
             validity check has become wrong for standard sizes, or the button stopped being \
             drawn."
        )));
    };
    driver.click_at(session.frame()?.declared_center(create))?;
    // Longer than a widget settle: this parses a template, rewrites the whole
    // file, re-parses it, and then adopts a new document — which drops the
    // render worker's texture and rebuilds every panel.
    session.settle(30);

    // --- F. the verdict -----------------------------------------------------
    let trace = session.trace()?;
    let Some(sized) = trace.events(SIZED_EVENT).last() else {
        return Ok(Some(format!(
            "the click on `{CREATE}` produced no `{SIZED_EVENT}` line, so no sized document was \
             made. `Action::NewSized` is declined when the open document has unsaved edits — this \
             run opened no document at all, so that guard cannot be it."
        )));
    };

    let read = |key: &str| -> Option<f64> { sized.get(key).and_then(|v| v.parse::<f64>().ok()) };
    let (Some(w_pt), Some(h_pt)) = (read("result_w"), read("result_h")) else {
        return Err(Error::new(format!(
            "the `{SIZED_EVENT}` line carries no readable `result_w`/`result_h`. Those fields are \
             what this check measures; without them nothing can be concluded. Line: {sized:?}."
        )));
    };
    let mm = |pt: f64| pt * 25.4 / 72.0;
    let (w_mm, h_mm) = (mm(w_pt), mm(h_pt));
    report.note(format!(
        "the re-parsed page is {w_mm:.1} × {h_mm:.1} mm ({w_pt:.1} × {h_pt:.1} pt)"
    ));

    // ★ The transposition is reported as ITSELF, not as "the wrong size".
    //
    // A page that is 297 × 420 has the right sheet and the wrong orientation,
    // and that is one specific defect — the radio not reaching `sheet_pt`, or
    // the action carrying the pair the other way round. Collapsing it into
    // "expected 420 × 297, got 297 × 420" would leave the reader to notice the
    // digits are the same.
    if (w_mm - A3_SHORT_MM).abs() <= TOLERANCE_MM && (h_mm - A3_LONG_MM).abs() <= TOLERANCE_MM {
        return Ok(Some(format!(
            "★ the page is A3 PORTRAIT ({w_mm:.1} × {h_mm:.1} mm) after Landscape was clicked. \
             The sheet is right and the orientation is not, so the size list is reaching the \
             document and the orientation radio is not — look at `NewDocumentDialog::sheet_pt`'s \
             two callers and at whether `Action::NewSized` carries the pair the right way round."
        )));
    }

    if (w_mm - A3_LONG_MM).abs() > TOLERANCE_MM || (h_mm - A3_SHORT_MM).abs() > TOLERANCE_MM {
        return Ok(Some(format!(
            "★ A3 landscape was chosen and the page is {w_mm:.1} × {h_mm:.1} mm — it should be \
             {A3_LONG_MM} × {A3_SHORT_MM}. If it reads 210 × 297 the document is the untouched A4 \
             template, so `set_media_box` was not called, its write was dropped by the rewrite, \
             or the re-parse read a different page. `app::blank::document_sized` is the whole of \
             that path and it is thirty lines."
        )));
    }

    Ok(None)
}
