//! `settings_navigate_by_page` — the Settings window shows one page at a time
//! from a grouped page list, its search keeps only the pages holding a match,
//! and Tools ▸ Font folders opens it on the Fonts page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/settings_nav.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const OPEN_SETTINGS: &str = "file.settings";
const OPEN_FONT_FOLDERS: &str = "tools.font_folders";
const DIALOG: &str = "dialog:settings";
const SEARCH: &str = "settings.search";
const HEADING: &str = "settings.heading.";
/// The trace event naming the page on show.
const PAGE_EVENT: &str = "settings-page"; // ui-text-exempt: a trace event name, never displayed
/// Every page, in the order the list must show them.
const PAGES: [&str; 17] = [
    "general",
    "presets",
    "appearance",
    "display",
    "acrobat",
    "remote",
    "colour",
    "fonts",
    "images",
    "text",
    "pages",
    "signatures",
    "measuring",
    "comments",
    "forms",
    "saving",
    "redaction",
];
/// Typed into the search; the Fonts page must survive it and some page must not.
const NEEDLE: &str = "font";

/// See the module documentation.
pub struct SettingsNavigateByPage;

impl Check for SettingsNavigateByPage {
    fn name(&self) -> &'static str {
        "settings_navigate_by_page"
    }

    fn defect(&self) -> &'static str {
        "the Settings window is one long column again, or its page list does not change the \
         page shown, or its search does not narrow the list, or Tools ▸ Font folders does not \
         open on the Fonts page"
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

/// The page keys whose list entries are on screen now.
fn live_pages(trace: &Trace, ui_rect: &str) -> Vec<String> {
    declared_names(trace, ui_rect, HEADING)
        .into_iter()
        .filter(|n| declared(trace, ui_rect, n).is_some())
        .map(|n| n.trim_start_matches(HEADING).to_owned())
        .collect()
}

/// The page the window last said it shows.
fn page_on_show(trace: &Trace) -> Option<String> {
    trace
        .last(PAGE_EVENT)
        .and_then(|l| l.get("key"))
        .map(str::to_owned)
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    invoke: &str,
    stem: &str,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), invoke.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("{invoke}: launched as pid {}", session.pid()));
    session.settle(40);
    Ok((session, pointer))
}

/// Click a declared region in whichever viewport declared it.
fn click(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, name: &str) -> Result<bool> {
    let Some((rect, viewport)) = declared_in(&session.trace()?, ui_rect, name) else {
        return Ok(false);
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    Ok(true)
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // --- The list, in order, and the first page on show ---------------------
    let (session, pointer) = launch(ctx, report, OPEN_SETTINGS, "settings-nav")?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, DIALOG).is_none() {
        return Err(Error::new(format!(
            "`{OPEN_SETTINGS}` was invoked and no `{DIALOG}` appeared."
        )));
    }
    let live = live_pages(&trace, ui_rect);
    let missing: Vec<&str> = PAGES
        .iter()
        .copied()
        .filter(|p| !live.iter().any(|l| l == p))
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!(
            "the page list does not show {}. Entries on screen: {}.",
            list(&missing.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>()),
            list(&live)
        )));
    }
    let tops: Vec<f32> = PAGES
        .iter()
        .filter_map(|p| declared(&trace, ui_rect, &format!("{HEADING}{p}")).map(|r| r.min.y))
        .collect();
    if tops.windows(2).any(|w| w[1] <= w[0]) {
        return Ok(Some(format!(
            "the page list is not in its contract order ({}); entry tops are {tops:?}.",
            PAGES.join(", ")
        )));
    }
    if page_on_show(&trace).as_deref() != Some(PAGES[0]) {
        return Ok(Some(format!(
            "the window opened on {:?}, not the first page {:?}.",
            page_on_show(&trace),
            PAGES[0]
        )));
    }
    report.note(format!(
        "{} pages listed in order; opened on general",
        PAGES.len()
    ));

    // --- A click in the list changes the page on show ------------------------
    click(&session, &pointer, ui_rect, &format!("{HEADING}signatures"))?;
    if page_on_show(&session.trace()?).as_deref() != Some("signatures") {
        return Ok(Some(format!(
            "Signatures was clicked in the page list and the window shows {:?}.",
            page_on_show(&session.trace()?)
        )));
    }
    report.note("clicking Signatures in the list showed the Signatures page");

    // --- The search narrows the list -----------------------------------------
    // The window is its own viewport; keys go to the one holding the box.
    let Some((_, dialog_vp)) = declared_in(&session.trace()?, ui_rect, SEARCH) else {
        return Ok(Some(format!("the window declares no `{SEARCH}` box.")));
    };
    click(&session, &pointer, ui_rect, SEARCH)?;
    let vp = dialog_vp.as_deref();
    pointer.type_text(&session, vp, NEEDLE)?;
    session.settle(20);
    let trace = session.trace()?;
    let kept = live_pages(&trace, ui_rect);
    if !kept.iter().any(|p| p == "fonts") || kept.len() >= PAGES.len() {
        return Ok(Some(format!(
            "searching {NEEDLE:?} should keep the Fonts page and drop some others; the list \
             shows {}.",
            list(&kept)
        )));
    }
    let shown = page_on_show(&trace);
    if !shown.as_ref().is_some_and(|s| kept.contains(s)) {
        return Ok(Some(format!(
            "the search kept {} and the window shows {shown:?}, a page the list does not.",
            list(&kept)
        )));
    }
    report.note(format!(
        "searching {NEEDLE:?} kept {} and shows {shown:?}",
        list(&kept)
    ));

    // --- Clearing it brings every page back ----------------------------------
    pointer.key(&session, vp, "A", Some("ctrl"))?;
    pointer.key(&session, vp, "Backspace", None)?;
    session.settle(20);
    let back = live_pages(&session.trace()?, ui_rect);
    if back.len() != PAGES.len() {
        return Ok(Some(format!(
            "the search was cleared and the list shows {} of {} pages: {}.",
            back.len(),
            PAGES.len(),
            list(&back)
        )));
    }
    report.note("clearing the search restored every page");
    drop(session);

    // --- Tools ▸ Font folders opens on Fonts ---------------------------------
    let (session, _pointer) = launch(ctx, report, OPEN_FONT_FOLDERS, "settings-nav-fonts")?;
    let shown = page_on_show(&session.trace()?);
    if shown.as_deref() != Some("fonts") {
        return Ok(Some(format!(
            "`{OPEN_FONT_FOLDERS}` opened the Settings window on {shown:?}, not the Fonts page."
        )));
    }
    report.note("Tools ▸ Font folders opened on the Fonts page");
    Ok(None)
}
