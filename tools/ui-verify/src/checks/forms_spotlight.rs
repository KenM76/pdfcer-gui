//! `clicking_a_form_row_lights_the_field_on_the_page` — focusing a row in the
//! Forms panel outlines that field's box on the canvas.
//!
//! # The operator's ask, `OPERATOR_REQUESTS.md` O98
//!
//! > *"when we have the fill form panel visible and I click on fields in it
//! > instead it should highlight the field on the canvas that is being
//! > filled."*
//!
//! On a drawing with a dozen fields you fill one in the panel and cannot see
//! where it went. Note the direction — **panel → canvas**. The other direction
//! shipped as O53.
//!
//! # ★★★ Why this needs driving, and why it needed two new instruments first
//!
//! The feature is a **handshake across two surfaces inside one frame**: the
//! panel writes a field name into `egui`'s temp store when a row has focus, and
//! the canvas reads it while painting. Both halves are a few lines and both are
//! individually trivial. What can break is the *join*:
//!
//! * the panel writes a name the canvas cannot match — the identity bug the
//!   channel was built on fully-qualified names rather than indices to avoid;
//! * the panel writes and the canvas never reads, because the canvas painted
//!   before the panel drew;
//! * the row never takes focus at all, so nothing is ever written.
//!
//! **No unit test can reach any of the three**, because all three require a
//! real frame with both surfaces in it and a real pointer press that moves
//! keyboard focus.
//!
//! ★★ Two instruments had to be built before this check could exist, and that
//! is worth naming because it keeps happening:
//!
//! 1. **The spotlight published no trace.** Drawing an outline is invisible to
//!    the harness. It now traces `canvas-form-spotlight field= drawn=
//!    candidates=` — and it traces `field=none` too, so "pointing at nothing"
//!    and "pointing at something the canvas cannot place" are distinguishable
//!    rather than both being silence.
//! 2. **The fill rows published no region.** There was nothing to aim a pointer
//!    at. They now publish `forms.fill.row.<index>`.
//!
//! A feature that cannot be observed cannot be verified, and the fix is to give
//! it an oracle rather than to weaken the assertion.
//!
//! # ★ The assertion that distinguishes working from plausible
//!
//! `drawn >= 1` **with the field named**. A build where the panel writes and
//! the canvas silently fails to match would trace `field=Subscribe drawn=0`,
//! which is a specific, diagnosable state — and it looks exactly like a working
//! build to anyone assert­ing only that the trace line appeared.
//!
//! `candidates=` is carried alongside so the two reasons for `drawn=0` can be
//! told apart: zero candidates means the canvas has no box for that name at all
//! (an identity mismatch), and candidates with `drawn=0` means the field exists
//! but sits on a page that is not on screen (not a defect).

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the Forms panel is reachable from.
const MODE: &str = "review";
/// The ribbon item that opens the Forms panel, and the tab it lives on.
const PANEL_ITEM: &str = "ribbon.item.view.panel_forms";
/// The prefix of the fill list's per-row regions; the 0-based index follows.
const ROW: &str = "forms.fill.row.";
/// The Forms panel's dock body — on screen by construction.
const PANEL_BODY: &str = "dock.body.view.panel_forms";
/// The Select tool, which is the only tool the canvas offers form boxes under.
const TOOL_SELECT: &str = "ribbon.item.view.tool_select";
/// The canvas's spotlight trace.
const SPOTLIGHT: &str = "canvas-form-spotlight";
/// The fixture: two widgets on one page, both visible at once.
/// The fixture — **this project's own**, not the engine's, and that is a
/// finding rather than a preference.
///
/// ★★★ Not one form fixture in `D:\Dev\pdfcerixtures\syntheticorms\` carries a
/// plain text field with an `/AP` `/N` appearance stream. Measured 2026-09-02
/// across all eighteen: `demo-form` and `radio-choice-form` have text fields
/// with **no** appearance, `rich-field-form`'s one paint-ready text field is
/// **rich text** (which the canvas declines by design), and every other
/// paint-ready widget is a 12 x 12 check box or radio.
///
/// That matters because the canvas census refuses a widget with no appearance
/// (`NotOnCanvas::NoAppearance`) — the page draws nothing there, so there is
/// nothing to outline. On every engine fixture the spotlight is therefore
/// *unable* to light the one row the panel offers, and the check could only
/// ever have failed. It is the check being unable to reach the feature, not
/// the feature being broken: exactly the shape this whole afternoon has been
/// about.
///
/// So this is 1,129 hand-written bytes: one page, one text field, one real
/// appearance stream. `fixtures/off-page-object.pdf` is the precedent.
const FIXTURE: &str = "text-field-with-appearance.pdf";

/// See the module documentation.
pub struct ClickingAFormRowLightsTheFieldOnThePage;

impl Check for ClickingAFormRowLightsTheFieldOnThePage {
    fn name(&self) -> &'static str {
        "clicking_a_form_row_lights_the_field_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "clicking a row in the Forms panel does not show which box on the page it fills, so on \
         a drawing with a dozen fields the operator types a value and has no way to see where \
         it went"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
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
            "input is disabled (--no-input). This check clicks a panel row. Reported as \
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

    // Its own fixture rather than `--pdf`, for the reason the tab-order drag
    // check gives at length: the harness's usual fixture is a CAD drawing with
    // no `/AcroForm`, so a fallback would SKIP forever and a SKIP is not red.
    let fixture = form_fixture().ok_or_else(|| {
        Error::new(format!(
            "the engine fixture `{FIXTURE}` is not on disk, so there is no document with form \
             fields. This check does NOT fall back to `--pdf`: the usual fixture has no form, \
             on which an empty panel is correct, and falling back would turn a real failure \
             into a permanent skip."
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("forms_spotlight.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    // ★ ONLY IF IT IS NOT ALREADY THERE — a panel toggle that is already on
    // CLOSES the thing this check needs. `crate::checks::pages_drag` carries the
    // same guard; `tab_order_drag` shipped without it and spent an afternoon
    // reporting a working feature as missing.
    if declared(&session.trace()?, ui_rect, PANEL_BODY).is_none() {
        open_from_tab(&session, &driver, ui_rect, "view", PANEL_ITEM)?;
        session.settle(24);
    }

    // ★★★ THE SELECT TOOL, EXPLICITLY — and this is a finding about the
    // feature, not merely a step.
    //
    // The canvas's whole form overlay is gated on `offered_in(tool)`, which is
    // `CanvasTool::Select` and nothing else. Edit mode does not start there, so
    // on entering it the overlay stops drawing entirely: no wash, no spotlight,
    // no form boxes. Measured 2026-09-02 — the canvas traced its last spotlight
    // line at the exact frame `mode-changed to=edit` appeared, 260 lines before
    // the panel first wrote a field into the channel.
    //
    // ⇒ The check selects the tool so it measures the feature in the state the
    // feature is designed for. **The gap this leaves is recorded rather than
    // papered over**: an operator who opens the Forms panel with any other tool
    // active gets no highlight when they click a row, and nothing says why.
    // Whether the spotlight — a read-only cue driven entirely by the panel —
    // should depend on the canvas tool at all is an operator question, filed in
    // `OPERATOR_REQUESTS.md` O98 rather than decided here.
    select_tool(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    let rows = declared_names(&trace, ui_rect, ROW);
    if rows.is_empty() {
        return Err(Error::new(format!(
            "the Forms panel declares no `{ROW}*` region, so there is no row to click. Either \
             the panel did not open or `{FIXTURE}` no longer carries a fillable text field. \
             Regions beginning `forms.`: {}.",
            list(&declared_names(&trace, ui_rect, "forms."))
        )));
    }
    report.note(format!("{} fillable row(s) on screen", rows.len()));

    // ★ Assert the STARTING state, so the check cannot pass on a build that
    // spotlights something unconditionally. A run that begins where the defect
    // lands proves nothing — a lesson this project has paid for.
    let before = trace.last(SPOTLIGHT).map(|e| e.raw.clone());
    if let Some(line) = &before
        && !line.contains("field=none")
    {
        return Ok(Some(format!(
            "before any row was clicked, the canvas was already spotlighting a field: `{line}`. \
             The spotlight is meant to follow the operator's attention, so a build that lights \
             one up on open would make this check pass while telling the operator nothing."
        )));
    }
    report.note("nothing is spotlighted before the click");

    // --- click a row's value box -------------------------------------------
    //
    // ★★ THE FIRST ROW THAT IS ACTUALLY DECLARED, not `{ROW}0`, and resolved
    // from the SAME trace snapshot the names came from.
    //
    // Hard-coding index 0 made this SKIP with the self-contradictory pair
    // *"1 fillable row(s) on screen"* immediately followed by *"no
    // `forms.fill.row.0` region"*. Both were true: the panel had scrolled
    // between the two reads, row 0 had left the viewport and a later row had
    // entered it. A region name carries an index into the form, not a promise
    // about what is on screen, and two `session.trace()` calls are two moments.
    let name = rows
        .first()
        .cloned()
        .ok_or_else(|| Error::new("no fillable row region on screen".to_owned()))?;
    let first = declared(&trace, ui_rect, &name).ok_or_else(|| {
        Error::new(format!(
            "`{name}` was named in this trace and could not then be resolved \
             from it. The two reads are the same snapshot, so this should be \
             impossible. Declared: {}.",
            list(&rows)
        ))
    })?;
    let at = session.frame()?.declared_center(first);
    report.note(format!("clicking `{name}` at ({}, {})", at.x(), at.y()));
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(spot) = trace.last(SPOTLIGHT) else {
        return Ok(Some(format!(
            "the first row was clicked and the canvas traced no `{SPOTLIGHT}` line at all, so \
             the spotlight code did not run. Either the canvas is not drawing form boxes on \
             this document, or the panel→canvas channel was never read."
        )));
    };
    report.note(format!("canvas: `{}`", spot.raw));

    let field = spot.get("field").unwrap_or("none");
    if field == "none" {
        return Ok(Some(format!(
            "the row was clicked and the canvas reports `field=none`, so the panel wrote \
             nothing into the channel. The row did not take keyboard focus — the write is \
             gated on `Response::has_focus()`, deliberately, so that arriving by Tab and \
             still being there three keystrokes later both keep the light on. A click that \
             lands on a label rather than the value box focuses nothing. Line: `{}`.",
            spot.raw
        )));
    }

    // ★★ THE ASSERTION THAT SEPARATES WORKING FROM PLAUSIBLE.
    let drawn: usize = spot.get("drawn").and_then(|v| v.parse().ok()).unwrap_or(0);
    if drawn == 0 {
        let candidates = spot.get("candidates").unwrap_or("?");
        return Ok(Some(format!(
            "the panel is pointing at `{field}` and the canvas drew NOTHING ({drawn} outlines, \
             {candidates} candidate box(es)). With zero candidates the two surfaces disagree \
             about the field's fully-qualified name, which is the identity bug this channel \
             was built on names rather than indices to avoid. With candidates present the \
             field is on a page that is not on screen — not a defect, but this fixture is one \
             page, so it would mean the page view and the box list disagree. Line: `{}`.",
            spot.raw
        )));
    }
    report.note(format!("the canvas outlined `{field}` — {drawn} box(es)"));

    Ok(None)
}

/// Make the Select tool active, because the canvas's form overlay is gated on
/// it — see the call site.
///
/// Silent when the item is not on screen: the caller's own assertions report
/// the consequence, and a missing ribbon item is a different finding from a
/// spotlight that does not light.
fn select_tool(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    // ★ The View tab has to be brought forward first. The panel guard above
    // deliberately does NOT click it when the panel is already open — pressing
    // a panel toggle that is on closes it — and a ribbon item on a tab that is
    // not showing publishes no region at all. The first version of this helper
    // looked for the tool without doing that, found nothing, and silently did
    // nothing, so the check went on measuring the wrong state.
    let trace = session.trace()?;
    if let Some(tab) = declared(&trace, ui_rect, "ribbon.tab.view") {
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(14);
    }
    let trace = session.trace()?;
    if let Some(item) = declared(&trace, ui_rect, TOOL_SELECT) {
        driver.click_at(session.frame()?.declared_center(item))?;
        session.settle(16);
    }
    Ok(())
}

/// Resolve [`FIXTURE`] under this project's own `fixtures/`.
///
/// Read-only, as everything under `D:\Dev\pdfcer` is until fold-in day.
fn form_fixture() -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("fixtures").join(FIXTURE);
    path.is_file().then_some(path)
}

/// Click a ribbon tab, then the item on it, following it into the overflow if
/// the group has collapsed.
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
