//! `checks::ctxdrive` — off-screen driving of the run's own `--pdf`: launch a
//! copy of it with the scripted pointer, and the steps checks on that document
//! share (a ribbon tab, a page point, a picture of the window's own frame).
//!
//! The copy, not the file itself, is opened: `--pdf` is usually a checked-in
//! fixture and a session may save.

use crate::checks::CheckContext;
use crate::checks::driving::{TAB_EVENT, declared_in, declared_names, list, shell_trace};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The run's `--pdf`, or an error asking for it.
pub fn pdf(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    ctx.pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))
}

/// Page size of the run's document: `--page-size` when given, else read from it.
pub fn page(ctx: &CheckContext) -> Result<PageGeometry> {
    if let Some((w, h)) = ctx.page_size {
        return Ok(PageGeometry {
            width_pt: w,
            height_pt: h,
        });
    }
    let pdf = pdf(ctx)?;
    crate::fixture::page_geometry(&pdf).ok_or_else(|| {
        Error::new(format!(
            "cannot read a page size from {}. Pass --page-size WxH.",
            pdf.display()
        ))
    })
}

/// Copy `--pdf` to `<stem>.pdf` in the scratch folder and launch it off the
/// desktop with the scripted pointer, the diagnostic trace, `invoke` and `env`.
pub fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    invoke: &str,
    env: &[(&str, String)],
) -> Result<(Session, ScriptedPointer)> {
    let src = pdf(ctx)?;
    let doc = ctx.out(&format!("{stem}.pdf"));
    std::fs::copy(&src, &doc).map_err(|e| Error::new(format!("copying {}: {e}", src.display())))?;
    super::dimdrive::launch_path(ctx, report, doc, invoke, stem, env)
}

/// The profile's `ui-rect` event name.
pub fn ui_rect(ctx: &CheckContext) -> Result<&'static str> {
    super::dimdrive::ui_rect_event(ctx)
}

/// Click ribbon tab `tab` (`ribbon.tab.<tab>`) and confirm the shell selected it.
pub fn open_tab(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    tab: &str,
) -> Result<()> {
    let ui_rect = ui_rect(ctx)?;
    let region = format!("ribbon.tab.{tab}");
    let trace = session.trace()?;
    let (rect, vp) = declared_in(&trace, ui_rect, &region).ok_or_else(|| {
        Error::new(format!(
            "no `{region}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(14);
    if !shell_trace(session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(tab))
    {
        return Err(Error::new(format!(
            "the click on `{region}` produced no `{TAB_EVENT} tab={tab}` line, so nothing \
             below would mean anything."
        )));
    }
    Ok(())
}

/// The canvas mapping for page `page` of the run's document, as last traced.
pub fn mapping(ctx: &CheckContext, session: &Session, page: usize) -> Result<CanvasMapping> {
    CanvasMapping::from_trace(
        &session.trace()?,
        &ctx.profile.vocab,
        self::page(ctx)?,
        page,
    )
}

/// Hover then click `at` on the page, through the current canvas mapping.
pub fn click_doc(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: DocPoint,
) -> Result<WindowPoint> {
    let point = mapping(ctx, session, at.page)?.doc_to_window(at)?;
    pointer.hover(session, point)?;
    session.settle(6);
    pointer.click(session, point)?;
    session.settle(14);
    Ok(point)
}

/// The window's own frame written to `name` in the scratch folder and added
/// to the report; a note, not a failure, when the seam cannot take it.
pub fn picture(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    name: &str,
) -> Option<std::path::PathBuf> {
    let shot = ctx.out(name);
    match pointer.screenshot(session, &shot) {
        Ok(()) => {
            report.artifact(shot.clone());
            Some(shot)
        }
        Err(why) => {
            report.note(format!("no picture of the window for {name}: {why}"));
            None
        }
    }
}
