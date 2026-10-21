//! Format ▸ Make pictures smaller driven on two pictures gathered with
//! Ctrl+click, through the Format tab and through a right-click on one of
//! them. Each route must open the window on both pictures, and Apply must
//! rewrite both as one edit.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/shrink_pictures.md`.

use super::dimdrive::{Fixture, click_on, press, press_on_tab, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit";
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const OPENED: &str = "shrink-images-opened"; // ui-text-exempt: a trace event name, never displayed
const SHRUNK: &str = "images-shrunk"; // ui-text-exempt: a trace event name, never displayed
const FORMAT_TAB: &str = "ribbon.tab.format";
const RIBBON_ITEM: &str = "ribbon.item.format.shrink_images";
const MENU_ROW: &str = "menu.item.canvas.object.format.shrink_images";
const COMMIT: &str = "shrink-images.commit";
const PHOTOS: Fixture = Fixture {
    file: "two-photos.pdf",
    method: "Rebuild it with `python fixtures/two-photos.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 200.0,
    },
};
/// The first picture's centre, page space.
const PICTURE_A: (f64, f64) = (100.0, 100.0);
/// The second picture's centre, page space.
const PICTURE_B: (f64, f64) = (300.0, 100.0);

/// See the module documentation.
pub struct ShrinkPicturesReachesEverySelectedPicture;

impl Check for ShrinkPicturesReachesEverySelectedPicture {
    fn name(&self) -> &'static str {
        "shrink_pictures_reaches_every_selected_picture"
    }

    fn defect(&self) -> &'static str {
        "with two pictures gathered by Ctrl+click, Format ▸ Make pictures smaller is missing \
         from the Format tab or the right-click menu, opens on fewer than both, or Apply \
         rewrites fewer than both"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &PHOTOS, INVOKE, "shrinkpictures", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    gather_both(ctx, session, pointer)?;
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, RIBBON_ITEM, FORMAT_TAB)?;
    if let Some(failure) = opened_on_both(session, report, mark, "the Format tab")? {
        return Ok(Some(failure));
    }
    press(ctx, session, pointer, COMMIT)?;
    session.settle(60);
    if let Some(failure) = shrunk(session, report, mark, true)? {
        return Ok(Some(failure));
    }

    gather_both(ctx, session, pointer)?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PHOTOS.page, 0)?;
    pointer.right_click(
        session,
        mapping.doc_to_window(DocPoint::new(0, PICTURE_B.0, PICTURE_B.1))?,
    )?;
    session.settle(30);
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, MENU_ROW)?;
    if let Some(failure) = opened_on_both(session, report, mark, "the right-click menu")? {
        return Ok(Some(failure));
    }
    press(ctx, session, pointer, COMMIT)?;
    session.settle(60);
    shrunk(session, report, mark, false)
}

/// Click picture A, Ctrl+click picture B, and require both selected at the
/// Object rung.
fn gather_both(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    click_on(ctx, session, pointer, &PHOTOS, 0, PICTURE_A)?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PHOTOS.page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, PICTURE_B.0, PICTURE_B.1))?;
    pointer.hover(session, at)?;
    session.settle(10);
    pointer.click_with(session, at, "ctrl")?;
    session.settle(20);
    let trace = session.trace()?;
    let selection = trace.last(SELECTION);
    if selection.is_some_and(|l| l.get("level") == Some("Object") && l.get("sel") == Some("2")) {
        return Ok(());
    }
    Err(Error::new(format!(
        "a click on one picture and a Ctrl+click on the other did not select both at the \
         Object rung (`{}`). Trace: {}.",
        selection.map_or("none", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}

/// The window opened after `mark`, on both pictures.
fn opened_on_both(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    route: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(OPENED, mark) else {
        return Ok(Some(format!(
            "{route} opened no picture window (no `{OPENED}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("{route}: `{}`", line.raw));
    if line.get("pictures") == Some("2") {
        return Ok(None);
    }
    Ok(Some(format!(
        "{route} opened the window on `{}`; both selected pictures must be in it \
         (`pictures=2`). Trace: {}.",
        line.raw,
        session.trace_path().display()
    )))
}

/// The edit after `mark` asked for both pictures; when `first`, it also
/// rewrote and resized both (a second pass finds them already at the
/// resolution and may leave them).
fn shrunk(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    first: bool,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(SHRUNK, mark) else {
        return Ok(Some(format!(
            "Apply reached no edit (no `{SHRUNK}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("Apply: `{}`", line.raw));
    let asked = line.get("asked") == Some("2");
    let both = line.get("changed") == Some("2") && line.get("resized") == Some("2");
    if asked && (!first || both) {
        return Ok(None);
    }
    let wanted = if first {
        "`asked=2 changed=2 resized=2`: both 300 ppi pictures brought down to 150"
    } else {
        "`asked=2`: both pictures handed to the engine"
    };
    Ok(Some(format!(
        "Apply must read {wanted}; it read `{}`. Trace: {}.",
        line.raw,
        session.trace_path().display()
    )))
}
