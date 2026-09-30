//! `page_tabs_chooser` — **a page's tab order is chosen in the Tab-order
//! section, written to the page, refused where the file's version cannot hold
//! it, and removable again.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_tabs_chooser.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, declared_since, list};
use crate::checks::tab_order_drag::{
    MODE, PANEL_BODY, PANEL_ITEM, enlarge_forms_pane, form_fixture, open_from_tab, open_tab_order,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Page 0's chooser; its options are `{CHOOSER}.{absent|R|C|S|A|W}`.
const CHOOSER: &str = "forms.tab_order.tabs.0";
/// The funnel's success and refusal lines for the verb.
const APPLIED: &str = "page-tabs-set"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "set-page-tabs-refused"; // ui-text-exempt: a trace event name, never displayed
/// The tab-order listing's per-page line, read back from the document.
const PAGE_LINE: &str = "forms-tab-page"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct PageTabsChooser;

impl Check for PageTabsChooser {
    fn name(&self) -> &'static str {
        "page_tabs_chooser"
    }

    fn defect(&self) -> &'static str {
        "a page's tab order cannot be chosen, or the choice never reaches the page, or a \
         PDF 2.0-only order is written into an older file"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input); this check clicks a combo box.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    // demo-form.pdf: PDF 1.7, one page, two widgets, no `/Tabs`. Not `--pdf`,
    // which has no form and so no Tab-order section to choose in.
    let fixture = form_fixture()
        .ok_or_else(|| Error::new("the engine's forms/demo-form.pdf fixture is not on disk."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("page_tabs_chooser.trace.txt"));
    spec.pdf = Some(fixture);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PANEL_BODY).is_none() {
        open_from_tab(&session, &driver, ui_rect, "view", PANEL_ITEM)?;
        session.settle(24);
    }
    enlarge_forms_pane(&session, &driver, ui_rect)?;
    open_tab_order(&session, &driver, ui_rect)?;
    if declared(&session.trace()?, ui_rect, CHOOSER).is_none() {
        return Err(Error::new(format!(
            "the Tab-order section is open and `{CHOOSER}` is not declared visible. \
             Tab-order regions: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "forms.tab_order."
            ))
        )));
    }

    // 1: Columns — written, and the listing reads it back from the page.
    if let Some(f) = choose(&session, &driver, ui_rect, "C")? {
        return Ok(Some(f));
    }
    let trace = session.trace()?;
    let Some(set) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "Columns was chosen and no `{APPLIED}` line followed{}.",
            trace
                .last(REFUSED)
                .map(|l| format!("; the engine refused: `{}`", l.raw))
                .unwrap_or_default()
        )));
    };
    report.note(format!("columns: `{}`", set.raw));
    if set.get("before") != Some("absent") || set.get("after") != Some("C") {
        return Ok(Some(format!(
            "the fixture states no /Tabs and Columns was chosen; the edit reads `{}`.",
            set.raw
        )));
    }
    if let Some(f) = page_reads(&session, "page:C")? {
        return Ok(Some(f));
    }

    // 2: "This list (PDF 2.0)" on a PDF 1.7 file — refused, page unchanged.
    let mark = session.trace()?.mark();
    if let Some(f) = choose(&session, &driver, ui_rect, "A")? {
        return Ok(Some(f));
    }
    let trace = session.trace()?;
    if let Some(wrote) = trace.last_after(APPLIED, mark) {
        return Ok(Some(format!(
            "the fixture is PDF 1.7 and /Tabs /A is a PDF 2.0 value, and it was written: `{}`.",
            wrote.raw
        )));
    }
    let Some(refused) = trace.last_after(REFUSED, mark) else {
        return Ok(Some(format!(
            "\"This list (PDF 2.0)\" was chosen on a PDF 1.7 file and neither `{APPLIED}` \
             nor `{REFUSED}` followed: the choice never reached the engine."
        )));
    };
    report.note(format!("array order refused: `{}`", refused.raw));
    if let Some(f) = page_reads(&session, "page:C")? {
        return Ok(Some(f));
    }

    // 3: Not stated — the key comes off again.
    if let Some(f) = choose(&session, &driver, ui_rect, "absent")? {
        return Ok(Some(f));
    }
    let shot = ctx.out("page_tabs_chooser.png");
    crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot);
    page_reads(&session, "absent")
}

/// Open page 0's combo and click the option named `value`.
fn choose(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    value: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(combo) = declared(&trace, ui_rect, CHOOSER) else {
        return Ok(Some(format!("`{CHOOSER}` stopped being declared visible.")));
    };
    let mark = trace.mark();
    driver.click_at(session.frame()?.declared_center(combo))?;
    session.settle(12);
    let option = format!("{CHOOSER}.{value}");
    let trace = session.trace()?;
    let Some(row) = declared_since(&trace, ui_rect, &option, mark) else {
        return Ok(Some(format!(
            "the chooser was clicked and its popup declared no `{option}`. Options declared: {}.",
            list(&declared_names(&trace, ui_rect, &format!("{CHOOSER}.")))
        )));
    };
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(24);
    Ok(None)
}

/// Require the listing's latest line for page 0 to read `tabs=want`.
fn page_reads(session: &Session, want: &str) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace
        .events(PAGE_LINE)
        .filter(|l| l.get("page") == Some("0"))
        .last();
    Ok(match line {
        Some(l) if l.get("tabs") == Some(want) => None,
        Some(l) => Some(format!(
            "page 0's tab-order listing should read tabs={want} and reads `{}`.",
            l.raw
        )),
        None => Some(format!("no `{PAGE_LINE} page=0` line was traced.")),
    })
}
