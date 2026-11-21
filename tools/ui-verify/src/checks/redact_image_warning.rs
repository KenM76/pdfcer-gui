//! `marking_over_an_image_says_so_before_apply` — a redaction mark that covers a
//! raster image discloses it when the rectangle is drawn, not when Apply is
//! pressed.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/redact_image_warning.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::scripteddrive::{OFFSCREEN_WIDE, offscreen_at};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the Redact panel is reachable from.
const MODE: &str = "edit";
/// The command that opens the Redact panel, and the tab it lives on.
const PANEL_ITEM: &str = "ribbon.item.edit.redact";
/// The panel's own region, so the toggle is only pressed when it is closed.
const PANEL: &str = "redact-panel";
/// The **Mark whole page** button.
const WHOLE_PAGE: &str = "redact-whole-page";
/// The funnel's line for a whole-page mark; it carries `disclosures=`.
const MARKED: &str = "redact-mark-page";
/// A fixture that is nothing but a raster image.
const WITH_IMAGE: &str = "fixtures/synthetic-image-only.pdf";
/// A real CAD sheet with no raster image on it at all.
const WITHOUT_IMAGE: &str = "fixtures/a1-titleblock.pdf";
/// The words the disclosure must carry when it fires.
const CONSEQUENCE: &str = "destroyed, not hidden";

/// See the module documentation.
pub struct MarkingOverAnImageSaysSoBeforeApply;

impl Check for MarkingOverAnImageSaysSoBeforeApply {
    fn name(&self) -> &'static str {
        "marking_over_an_image_says_so_before_apply"
    }

    fn defect(&self) -> &'static str {
        "a redaction mark that covers a raster image is authored in silence, so the operator \
         does not learn until he opens the saved file that those image samples were destroyed \
         and overwritten rather than hidden — a raster redaction is irreversible in a way a \
         text one is not"
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
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // --- 1: a document that HAS an image ----------------------------------
    let with = mark_whole_page(ctx, &exe, ui_rect, WITH_IMAGE, report)?;
    let Some(line) = with else {
        return Ok(Some(format!(
            "no `{MARKED}` line after pressing Mark whole page on `{WITH_IMAGE}`, so nothing \
             was marked at all and this check never reached its subject."
        )));
    };
    report.note(format!("with an image: `{line}`"));
    if !line.contains(CONSEQUENCE) {
        return Ok(Some(format!(
            "a whole-page mark on `{WITH_IMAGE}` — which is nothing BUT a raster image — was \
             authored without saying that applying it will be refused. The operator finds out \
             at Apply, for the whole document, with no indication of which region caused it. \
             Line: `{line}`."
        )));
    }

    // --- 2: and one that does NOT ------------------------------------
    let without = mark_whole_page(ctx, &exe, ui_rect, WITHOUT_IMAGE, report)?;
    let Some(line) = without else {
        return Ok(Some(format!(
            "no `{MARKED}` line after pressing Mark whole page on `{WITHOUT_IMAGE}`."
        )));
    };
    report.note(format!("without an image: `{line}`"));
    if line.contains(CONSEQUENCE) {
        return Ok(Some(format!(
            "a whole-page mark on `{WITHOUT_IMAGE}` — a CAD sheet with NO raster image on it — \
             was warned about anyway. A caveat attached to every mark is one an operator learns \
             to scroll past, and the day it is load-bearing they will scroll past it too. The \
             warning must track the fact, not the feature being switched on. Line: `{line}`."
        )));
    }
    report.note("…and a sheet with no image is marked in silence, which is the half that makes the warning worth reading");

    Ok(None)
}

/// Launch on `fixture`, open the Redact panel, press **Mark whole page**, and
/// return the funnel's line for it.
fn mark_whole_page(
    ctx: &CheckContext,
    exe: &std::path::Path,
    ui_rect: &str,
    fixture: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let path = std::path::Path::new(fixture);
    if !path.is_file() {
        return Err(Error::new(format!(
            "the fixture `{fixture}` is not on disk, so this check cannot establish the \
             document it needs."
        )));
    }
    let mut spec = LaunchSpec::new(exe, ctx.out("redact-image-warning.trace.txt"));
    spec.pdf = Some(path.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    // Desktop-sized, because the Edit tab's later groups fold away at the
    // harness's default width.
    let stem = path
        .file_stem()
        .map_or_else(|| "fixture".into(), |s| s.to_string_lossy());
    let pointer = offscreen_at(
        ctx,
        &mut spec,
        &format!("redact-image-warning-{stem}"),
        OFFSCREEN_WIDE,
    )?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);

    crate::checks::driving::click_mode_segment(&session, &pointer, ui_rect, MODE)?;
    session.settle(14);

    open_panel(&session, &pointer, ui_rect)?;

    let trace = session.trace()?;
    let button = declared(&trace, ui_rect, WHOLE_PAGE).ok_or_else(|| {
        Error::new(format!(
            "no `{WHOLE_PAGE}` region, so the Redact panel is not on screen or has no Mark \
             whole page control. Regions beginning `redact-`: {}.",
            list(&declared_names(&trace, ui_rect, "redact-"))
        ))
    })?;
    pointer.click(&session, WindowPoint::centre_of(button))?;
    session.settle(30);
    pointer.gone(&session)?;

    Ok(session.trace()?.last(MARKED).map(|e| e.raw.clone()))
}

/// Open the Redact panel from the Security tab, unless it is already showing.
fn open_panel(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<()> {
    // Only if it is not already there: a panel toggle that is already on
    // CLOSES the thing this check needs.
    if declared(&session.trace()?, ui_rect, PANEL).is_none() {
        let trace = session.trace()?;
        let tab = declared(&trace, ui_rect, "ribbon.tab.security").ok_or_else(|| {
            Error::new(format!(
                "no `ribbon.tab.security` region. Tabs declared: {}.",
                list(&declared_names(&trace, ui_rect, "ribbon.tab."))
            ))
        })?;
        pointer.click(session, WindowPoint::centre_of(tab))?;
        session.settle(14);
        let item = crate::checks::driving::declared_or_in_overflow(
            session, pointer, ui_rect, PANEL_ITEM,
        )?
        .ok_or_else(|| {
            Error::new(format!(
                "no `{PANEL_ITEM}` region on the Security tab or in its overflow. Items declared: \
                 {}.",
                list(&declared_names(
                    &session.trace().unwrap_or_default(),
                    ui_rect,
                    "ribbon.item.edit."
                ))
            ))
        })?;
        pointer.click(session, WindowPoint::centre_of(item))?;
        session.settle(24);
    }
    Ok(())
}
