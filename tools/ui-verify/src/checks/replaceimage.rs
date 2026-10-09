//! Format ▸ Replace image driven on one placed image, through all three of
//! its routes: the Format tab, the canvas object menu, and the Properties
//! panel. Each press must draw a new image XObject in the same object slot,
//! and the page model read back afterwards must name it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/replaceimage.md`.

use super::dimdrive::{Fixture, click_on, press, press_on_tab, run_on_with};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit";
const REPLACED: &str = "image-replaced"; // ui-text-exempt: a trace event name, never displayed
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const FORMAT_TAB: &str = "ribbon.tab.format";
const RIBBON_ITEM: &str = "ribbon.item.format.replace_image";
const MENU_ROW: &str = "menu.item.canvas.object.format.replace_image";
const PANEL_BUTTON: &str = "properties.stroke.replace-image";
/// The environment seam that answers the image picker.
const IMAGE_PATH_ENV: &str = "PDFCER_DIAG_IMAGE_PATH"; // ui-text-exempt: an environment variable name
const IMAGE: Fixture = Fixture {
    file: "image-box.pdf",
    method: "Rebuild it with `python fixtures/image-box.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// The image's centre, page space; the image is the page's object 0.
const ON_THE_IMAGE: (f64, f64) = (200.0, 150.0);
/// The replacement's pixel size: 3:1 against the box's 2:1, so the default
/// fit letterboxes it.
const PNG_W: u32 = 30;
/// See [`PNG_W`].
const PNG_H: u32 = 10;

/// See the module documentation.
pub struct ReplaceImageSwapsThePictureInPlace;

impl Check for ReplaceImageSwapsThePictureInPlace {
    fn name(&self) -> &'static str {
        "replace_image_swaps_the_picture_in_place"
    }

    fn defect(&self) -> &'static str {
        "with an image selected, Format ▸ Replace image is missing from a route, reaches no \
         engine call, or replaces a different object than the one selected"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let png = ctx.out("replace_image_source.png");
        if let Err(why) = write_png(&png) {
            return CheckReport::new(self.name(), self.defect()).from_error(&why);
        }
        let env = [(IMAGE_PATH_ENV, png.display().to_string())];
        run_on_with(self, ctx, &IMAGE, INVOKE, "replaceimage", &env, drive)
    }
}

/// A solid red PNG of [`PNG_W`] × [`PNG_H`].
fn write_png(path: &std::path::Path) -> Result<()> {
    let pixels: Vec<u8> = std::iter::repeat_n([220_u8, 30, 30], (PNG_W * PNG_H) as usize)
        .flatten()
        .collect();
    let png = crate::png::encode_rgb(PNG_W, PNG_H, &pixels)
        .ok_or_else(|| Error::new("the harness's PNG encoder refused its own fixture."))?;
    std::fs::write(path, png)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    select_image(ctx, session, pointer)?;
    let mut drawn = 0;

    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, RIBBON_ITEM, FORMAT_TAB)?;
    match replaced(session, report, mark, "the Format tab", drawn)? {
        Ok(new) => drawn = new,
        Err(failure) => return Ok(Some(failure)),
    }

    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, IMAGE.page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, ON_THE_IMAGE.0, ON_THE_IMAGE.1))?;
    pointer.right_click(session, at)?;
    session.settle(30);
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, MENU_ROW)?;
    match replaced(session, report, mark, "the canvas menu", drawn)? {
        Ok(new) => drawn = new,
        Err(failure) => return Ok(Some(failure)),
    }

    let mark = session.trace()?.mark();
    press(ctx, session, pointer, PANEL_BUTTON)?;
    if let Err(failure) = replaced(session, report, mark, "the Properties panel", drawn)? {
        return Ok(Some(failure));
    }
    Ok(None)
}

/// Click the image and require the Object rung.
fn select_image(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    click_on(ctx, session, pointer, &IMAGE, 0, ON_THE_IMAGE)?;
    let trace = session.trace()?;
    let selection = trace.last(SELECTION);
    if selection.is_some_and(|l| l.get("level") == Some("Object")) {
        return Ok(());
    }
    Err(Error::new(format!(
        "a click on the image did not select it at the Object rung (`{}`). Trace: {}.",
        selection.map_or("none", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}

/// The `image-replaced` line after `mark`: object 0, drawing a new XObject in
/// place of `previous` (0 for the fixture's own, whatever its number), and
/// the re-read model naming the new one. The inner `Err` is the failure.
fn replaced(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    route: &str,
    previous: usize,
) -> Result<std::result::Result<usize, String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(REPLACED, mark) else {
        return Ok(Err(format!(
            "Replace image through {route} reached no replace (no `{REPLACED}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("{route}: `{}`", line.raw));
    let old = line.get_usize("old").unwrap_or(0);
    let new = line.get_usize("new").unwrap_or(0);
    let right = line.get("page") == Some("0")
        && line.get("object") == Some("0")
        && old != 0
        && (previous == 0 || old == previous)
        && new != 0
        && new != old
        && line.get_usize("model") == Some(new);
    if right {
        return Ok(Ok(new));
    }
    let old_wanted = if previous == 0 {
        "an old image".to_owned()
    } else {
        format!("`old={previous}`, the image the last press drew")
    };
    Ok(Err(format!(
        "through {route} the replace must read `page=0 object=0`, {old_wanted}, a different \
         new one, and `model=` the new one; it read `{}`. Trace: {}.",
        line.raw,
        session.trace_path().display()
    )))
}
