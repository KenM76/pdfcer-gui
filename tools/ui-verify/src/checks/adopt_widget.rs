//! `adopt_widget` — **insert a form's pages to CREATE orphaned widgets, then
//! register one back into the document.**
//!
//! # ★★ The only check in this suite that needs two features, because the
//! fixture is a STATE and not a file
//!
//! An orphaned widget is a `/Widget` annotation in a page's `/Annots` that no
//! `/AcroForm` field reaches. There is no fixture PDF that contains one and
//! there should not be: it is not a shape a producer writes, it is a shape
//! **pdfcer itself makes**. `EditSession::insert_pages` copies everything
//! reachable from a page, and a page's `/Annots` reaches its widgets — but
//! `/AcroForm` is a **catalog** entry, so it is not in the set of objects being
//! copied. A source with 12 fields inserted into another document arrives as 13
//! inert boxes and no form at all.
//!
//! So this check drives Insert pages first, not as setup but as **half the
//! subject**: the defect it exists to catch is exactly *"pdfcer made these and
//! then could not undo it"*, and a hand-authored fixture would prove the
//! registration works on a shape pdfcer never produces.
//!
//! It is also why the two halves cannot be split into two checks. The state
//! only exists inside one session.
//!
//! # What an operator meets if this regresses
//!
//! A box drawn on the page with a border and a background, indistinguishable
//! from the field beside it, that swallows every keystroke. This project's
//! recurring failure — a visible control that is silently inert — arriving
//! through a **document** instead of a ribbon.
//!
//! # ★ Why the button's LABEL is asserted and not only its presence
//!
//! `adopt_preview` shipped so the row could read *"Register as `Address`"*
//! instead of *"Register"*, and the engine's framing is the reason it matters:
//! for a merged field-widget the name is **in the file and not on screen** —
//! the widget belongs to no field, so no field row names it. A button that says
//! only *"Register"* is a guess the operator is being asked to accept.
//!
//! The harness cannot read a label off the screen, so the panel traces its
//! decision per row and this check asserts on that. Presence alone would pass
//! on a build where the preview was never asked and every row said *"Register"*
//! — which is precisely the state this morning's request was filed about.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | Edit ▸ Insert pages, with `PDFCER_DIAG_INSERT_PATH` set to a form | `insert-pages` traced, `orphans>0` in the disclosure |
//! | B | open Forms ▸ Tab order | `tab-order-unclaimed` census with a non-zero count |
//! | C | read the per-row preview trace | at least one row knows the name it would register under |
//! | D | press the first row's Register | `adopt-widget-requested`, then `adopt-widget … epoch=` |
//! | E | re-read the census | one fewer unclaimed widget than before |

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_or_in_overflow,
    frame_of, list, live_names,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The form whose pages are inserted to manufacture the orphans.
///
/// # ★ `demo-form.pdf`, and the first choice was wrong for a reason worth
/// keeping
///
/// It was `multi-widget-form.pdf`, picked because the engine's measurement
/// ("11 of 13 merged, 2 bare kids") suggested a fixture with both shapes. The
/// driven run answered `orphaned_widgets = orphaned_widgets_unrecoverable = 3`
/// — **every** orphan was a bare kid, so the row that resolves a name never
/// appeared and the phase asserting it could not pass.
///
/// The fixture's own name says why, once you know what the two shapes are. A
/// *multi-widget* field is one field with several `/Kids`, and a kid IS a bare
/// kid the moment `insert_pages` drops its `/Parent`. A **merged**
/// field-widget (§12.7.3.1) is the opposite arrangement: one dictionary serving
/// as both, which is what a form with one widget per field is made of.
///
/// So the fixture that exercises the recoverable path is the plain one. Worth
/// recording rather than just changing, because "pick the fixture whose name
/// mentions the feature" is the intuition that produced the wrong answer here.
const SOURCE: &str = "forms/demo-form.pdf";
/// The environment seam that answers the Insert-pages file picker.
const INSERT_PATH_ENV: &str = "PDFCER_DIAG_INSERT_PATH";
/// The mode the Forms panel is reached in.
const MODE: &str = "edit";
/// The command that opens the insert window, and the tab it lives on.
///
/// ★ `pages.insert_from_file` on the **Pages** tab, not `edit.insert_pages`.
/// The first draft of this check guessed the latter and skipped with a list of
/// the nine controls the Edit tab actually declares — which is `crate::checks`
/// rule 5 working: a reason that says what it *did* find turns a wrong guess
/// into a one-line correction instead of an investigation.
const INSERT_TAB: &str = "pages";
const INSERT_ITEM: &str = "ribbon.item.pages.insert_from_file";
/// The insert window's commit control.
const INSERT_BUTTON: &str = "insert-pages.insert";
/// The command that shows the Forms panel.
const PANEL_ITEM: &str = "ribbon.item.view.panel_forms";
/// The funnel's success line for the insert.
const INSERTED: &str = "insert-pages";
/// The per-page census the Tab-order section traces.
const TAB_CENSUS: &str = "forms-tab-page";
/// What the register rows trace about each unclaimed widget.
const ROW: &str = "adopt-row";
/// The press.
const REQUESTED: &str = "adopt-widget-requested";
/// The funnel's success line for the registration.
const ADOPTED: &str = "adopt-widget";
/// The panel body, which is what gets scrolled and what a row must end up
/// inside.
const PANEL_BODY: &str = "dock.body.view.panel_forms";
/// How far this check will scroll looking for a row before giving up.
///
/// A bound rather than a loop, because "scroll until you find it" over a list
/// that does not contain it is a hang, and a hang in a suite is a failure with
/// no message.
const MAX_SCROLL: usize = 12;

/// See the module documentation.
pub struct AdoptWidgetPutsAFormControlBack;

impl Check for AdoptWidgetPutsAFormControlBack {
    fn name(&self) -> &'static str {
        "adopt_widget_puts_a_form_control_back"
    }

    fn defect(&self) -> &'static str {
        "inserting a form's pages leaves boxes that draw exactly like form fields and swallow \
         every keystroke, with no way to register them — or a Register control that is drawn and \
         does not reach the document, or one that never asks what name it would use and so \
         offers the operator a guess to accept"
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
            "input is disabled (--no-input). This check clicks a mode segment, three ribbon \
             controls and two panel controls. Reported as SKIPPED rather than passed: a check \
             that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let source = engine_fixture(SOURCE).ok_or_else(|| {
        Error::new(format!(
            "the engine fixture `{SOURCE}` is not on disk. This check manufactures its own \
             subject by inserting a real form's pages, so there is nothing to fall back to."
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("adopt_widget.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // The picker's answer, supplied rather than clicked. A native file dialog
    // is an OS modal this harness cannot drive, and the seam exists for exactly
    // that — see `app::files`, which documents one seam per verb.
    spec.env.push((
        INSERT_PATH_ENV.to_owned(),
        source.to_string_lossy().into_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}, with {INSERT_PATH_ENV} pointing at {}",
        exe.display(),
        session.pid(),
        source.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- A: manufacture the orphans ----------------------------------------
    open_from_tab(&session, &driver, ui_rect, INSERT_TAB, INSERT_ITEM)?;
    session.settle(24);
    let trace = session.trace()?;
    let button = declared(&trace, ui_rect, INSERT_BUTTON).ok_or_else(|| {
        Error::new(format!(
            "the Insert-pages window declared no `{INSERT_BUTTON}` region, so it did not open or \
             did not read the source. Regions beginning `insert-pages`: {}.",
            list(&declared_names(&trace, ui_rect, "insert-pages"))
        ))
    })?;
    driver.click_at(frame_of(&session, &trace, ui_rect, INSERT_BUTTON)?.declared_center(button))?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(inserted) = trace.events(INSERTED).last() else {
        return Ok(Some(format!(
            "the Insert button was pressed and the funnel traced no `{INSERTED}` line, so no \
             pages were inserted and this check has no subject to register."
        )));
    };
    let disclosures = inserted.get("disclosures").unwrap_or_default();
    report.note(format!("inserted: {disclosures}"));
    if !disclosures.contains("re-registering") {
        return Ok(Some(format!(
            "the insert succeeded and its disclosure said nothing about form controls needing \
             re-registering: {disclosures:?}. Either `{SOURCE}` no longer carries widgets, or \
             `InsertOutcome::orphaned_widgets` came back zero — in which case pdfcer has started \
             merging `/AcroForm` on this path and this check's whole premise has changed."
        )));
    }

    // --- B: the Tab-order section, which is where they are listed ----------
    open_from_tab(&session, &driver, ui_rect, "view", PANEL_ITEM)?;
    session.settle(24);
    open_tab_order(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    let Some(before) = unclaimed_total(&trace) else {
        return Ok(Some(format!(
            "the Forms panel drew and traced no `{TAB_CENSUS}` line, so the Tab-order section \
             did not open or did not walk the pages."
        )));
    };
    if before == 0 {
        return Ok(Some(String::from(
            "the insert reported orphaned widgets and the Tab-order section counted ZERO \
             unclaimed. The disclosure and the listing disagree about the same document, which \
             means one of them is walking it differently — and the listing is what the operator \
             is offered a remedy from.",
        )));
    }
    report.note(format!("{before} unclaimed widget(s) listed"));

    // --- C: ★ the preview was ASKED, per row -------------------------------
    let rows: Vec<_> = trace.events(ROW).collect();
    if rows.is_empty() {
        return Ok(Some(format!(
            "{before} unclaimed widget(s) are listed and no `{ROW}` line was traced, so no row \
             asked `adopt_preview` what it would do. Every row then reads \"Register\" and the \
             operator is accepting a name they have never seen."
        )));
    }
    let named = rows
        .iter()
        .filter(|l| l.get("named").is_some_and(|v| v == "1"))
        .count();
    report.note(format!(
        "{} row(s) previewed, {named} of them knowing the name they would register under",
        rows.len()
    ));
    if named == 0 {
        return Ok(Some(format!(
            "all {} row(s) previewed and NONE resolved a name, so every button reads \
             \"Register\" rather than \"Register as …\". The engine measured 11 of 13 orphans as \
             merged field-widgets carrying their own `/T`; zero here means the preview is being \
             asked with the wrong argument, or the source's widgets are all bare kids and this \
             fixture can no longer exercise the case it was chosen for.",
            rows.len()
        )));
    }

    // --- D: press one ------------------------------------------------------
    let names = live_names(&trace, ui_rect, "tab-order.register.");
    let Some(first) = names.first() else {
        return Ok(Some(format!(
            "{before} unclaimed widget(s) are listed and no `tab-order.register.*` region is on \
             screen, so the rows were computed and never laid out — or were laid out past the \
             bottom of the panel, which is where the Bookmarks authoring row was found on \
             2026-08-19. Regions declared: {}.",
            list(&declared_names(&trace, ui_rect, "tab-order"))
        )));
    };
    // ★★ SCROLLED INTO VIEW before it is clicked.
    //
    // The rows sit inside the panel's scroll area, and a dock panel is a few
    // hundred points tall. A region below the fold is published at its
    // **content** position — which is correct, and is outside the panel's own
    // rectangle — so clicking its centre lands on whatever is drawn there
    // instead, and the check reports the control as inert.
    //
    // That is a *harness* limitation reporting as an application defect, which
    // this suite produced three times on 2026-08-19 before `Driver::scroll_at`
    // existed. So: scroll until the row is inside the panel, and say so if it
    // never gets there rather than clicking a coordinate outside the window.
    let panel = declared(&trace, ui_rect, PANEL_BODY)
        .ok_or_else(|| Error::new(format!("no `{PANEL_BODY}` region to scroll.")))?;
    let mut rect = declared(&trace, ui_rect, first)
        .ok_or_else(|| Error::new(format!("the `{first}` region went away between phases.")))?;
    let mut turns = 0;
    while !panel.contains_rect(rect) && turns < MAX_SCROLL {
        driver.scroll_at(session.frame()?.declared_center(panel), -3)?;
        session.settle(10);
        turns += 1;
        rect = declared(&session.trace()?, ui_rect, first)
            .ok_or_else(|| Error::new(format!("`{first}` stopped being drawn while scrolling.")))?;
    }
    if !panel.contains_rect(rect) {
        return Ok(Some(format!(
            "after {MAX_SCROLL} wheel turns the `{first}` row is still outside the panel \
             ({rect:?} against {panel:?}), so it cannot be reached by scrolling either. A \
             control an operator cannot get to is a control that does not exist, whatever \
             its click arm does."
        )));
    }
    report.note(format!("the row is on screen after {turns} wheel turn(s)"));
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(REQUESTED).last().is_none() {
        return Ok(Some(format!(
            "clicking `{first}` traced no `{REQUESTED}` line, so the Register control is drawn \
             and inert — or is greyed, which on a row whose preview resolved a name would mean \
             the button and the preview disagree."
        )));
    }
    if let Some(refusal) = trace.events(&format!("{ADOPTED}-refused")).last() {
        return Ok(Some(format!(
            "★ the row previewed successfully and the CALL refused: {}. `adopt_preview` and \
             `adopt_widget` share one guard set in the engine precisely so this cannot happen, \
             so reaching it means they have been given different arguments — the likeliest cause \
             is the name box's contents not travelling with the press.",
            refusal.get("detail").unwrap_or_default()
        )));
    }
    let Some(applied) = trace.events(ADOPTED).last() else {
        return Ok(Some(format!(
            "the row raised its action and the funnel never traced `{ADOPTED}`, so it was queued \
             and never applied."
        )));
    };
    report.note(format!(
        "registered at epoch {}: {}",
        applied.get("epoch").unwrap_or_default(),
        applied.get("disclosures").unwrap_or_default()
    ));

    // --- E: and the listing agrees ------------------------------------------
    let Some(after) = unclaimed_total(&trace) else {
        return Ok(Some(format!(
            "the panel stopped tracing `{TAB_CENSUS}` after the registration."
        )));
    };
    if after + 1 != before {
        return Ok(Some(format!(
            "{before} widget(s) were unclaimed before the registration and {after} after it, \
             where {} was expected. The engine reported success, so the widget is registered and \
             the panel is not reading it back — which an operator meets as pressing Register and \
             watching the row stay exactly where it was.",
            before - 1
        )));
    }
    report.note(format!(
        "{after} unclaimed remain — the registered widget left the list, which is the round trip"
    ));

    let shot = ctx.out("adopt_widget.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!(
                "the window could not be captured ({e}); the trace assertions above still hold"
            ));
        }
    }
    Ok(None)
}

/// Resolve a fixture under the engine repository's synthetic corpus.
///
/// ★ The path is derived, not configured. `D:\Dev\pdfcer` is READ-ONLY to this
/// project and its corpus is the only place these shapes exist, so the check
/// reads from it and writes nowhere near it. Returning `None` rather than
/// panicking is what turns a missing corpus into a SKIP with a reason instead
/// of a crash in the middle of a suite.
fn engine_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(rel);
    path.is_file().then_some(path)
}

/// Every page's unclaimed count, summed.
///
/// ★ Summed across the pages of the **latest** frame rather than taken from one
/// page. The insert puts the form's sheets somewhere in the middle of the
/// document, and which page they land on is the insert's business, not this
/// check's — a check that hard-coded a page index would break on a change to
/// the insert position that is not a defect.
fn unclaimed_total(trace: &crate::trace::Trace) -> Option<usize> {
    let lines: Vec<_> = trace.events(TAB_CENSUS).collect();
    if lines.is_empty() {
        return None;
    }
    // The last frame's worth: the panel traces one line per page per frame, so
    // walking backwards to the lowest page number collects exactly one frame.
    let mut total = 0usize;
    let mut seen: Vec<&str> = Vec::new();
    for line in lines.iter().rev() {
        let page = line.get("page").unwrap_or_default();
        if seen.contains(&page) {
            break;
        }
        seen.push(page);
        total += line
            .get("unclaimed")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or_default();
    }
    Some(total)
}

/// Click a ribbon tab, then an item on it, following the overflow if the item
/// went there.
fn open_from_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    tab: &str,
    item: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let tab_region = declared(&trace, ui_rect, &format!("ribbon.tab.{tab}")).ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.{tab}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab_region))?;
    session.settle(14);
    let found = declared_or_in_overflow(session, driver, ui_rect, item)?.ok_or_else(|| {
        Error::new(format!(
            "no `{item}` region on the {tab} tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                &format!("ribbon.item.{tab}.")
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(found))?;
    session.settle(20);
    Ok(())
}

/// Expand the Tab-order collapsing header, which ships closed.
///
/// It defaults closed deliberately — the section is a diagnostic, not the
/// panel's main job — so a check that assumed it open would report the whole
/// feature missing on a correct build.
fn open_tab_order(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    if let Some(header) = declared(&trace, ui_rect, "forms.tab_order.header") {
        driver.click_at(session.frame()?.declared_center(header))?;
        session.settle(20);
    }
    Ok(())
}
