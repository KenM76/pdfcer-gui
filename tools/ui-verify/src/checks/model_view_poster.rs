//! `checks::model_view_poster` — **the view chosen in the 3D viewer becomes
//! the model's picture on the page**
//!
//! Places the engine corpus's `assembly.prc` as `checks::model_view_window`
//! does, opens it with *View…*, turns to the Top view and presses *Use this
//! view on the page*, then closes the viewer.
//!
//! Oracles: a `set-3d-poster` edit line, and the page's pixels between the
//! frame after the insert and the frame after the press differing in at least
//! `MIN_CHANGED` places: the engine's default view looks from the side, so a
//! picture from above that reached the page cannot leave it as it was.
//!
//! [`APictureFileBecomesTheModelsPagePicture`] drives the other route: the
//! Attachments row's *Picture…* with a solid red PNG as the picker's answer.
//! Its oracle is the page gaining at least `MIN_CHANGED` red pixels, a colour
//! the engine's grey-and-white default poster of `assembly.prc` never draws.

use crate::checks::driving::{declared, declared_in};
use crate::checks::model_view_window::{
    MODEL, VIEW_REGION, launch_with_model, launch_with_model_env, press_insert, ui_rect,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::PixRect;
use crate::image::{Image, Rgb};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The viewer's Top view button: `model3d::REGION_VIEW_PREFIX` and index 3.
const TOP_REGION: &str = "model3d.view.3";
/// The viewer's *Use this view on the page* button.
const USE_REGION: &str = "model3d.use_on_page";
/// The viewer's Close button: closed, it cannot be what the frame shows.
const CLOSE_REGION: &str = "model3d.close";
/// The first page's region on the canvas.
const PAGE_REGION: &str = "page";
/// The Attachments row's *Picture…* button.
const PICTURE_REGION: &str = "models.poster";
/// The image picker's diagnostic answer, read by `app::files::pick_image_source`.
const IMAGE_ENV: &str = "PDFCER_DIAG_IMAGE_PATH";
/// The edit's own trace line, named by `vector_edit`.
const EDIT: &str = "set-3d-poster";
/// A channel difference below this is antialiasing or rounding.
const TOLERANCE: u8 = 24;
/// Fewer changed pixels than this is no new picture on the page.
const MIN_CHANGED: usize = 2_000;

/// See the module documentation.
pub struct TheViewersViewBecomesThePagePicture;

impl Check for TheViewersViewBecomesThePagePicture {
    fn name(&self) -> &'static str {
        "the_3d_viewers_view_becomes_the_page_picture"
    }

    fn defect(&self) -> &'static str {
        "Use this view on the page in the 3D viewer leaves the page's picture of the model as \
         it was"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model(ctx, &mut report, "model-view-poster")
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn shot(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    name: &str,
) -> Result<Image> {
    let path = ctx.out(name);
    pointer.screenshot(session, &path)?;
    report.artifact(path.clone());
    Image::load_png(&path)
}

/// Click the region `name`, wherever it is declared; `false` when it is not.
fn click(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, name: &str) -> Result<bool> {
    let trace = session.trace()?;
    let Some((rect, vp)) = declared_in(&trace, ui_rect, name) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(true)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let placed = shot(
        ctx,
        report,
        session,
        pointer,
        "model-view-poster-placed.png",
    )?;
    let ui_rect = ui_rect(ctx)?;
    for (region, what) in [
        (VIEW_REGION, "View… beside the placed model"),
        (TOP_REGION, "the viewer's Top button"),
        (USE_REGION, "the viewer's Use this view on the page button"),
        (CLOSE_REGION, "the viewer's Close button"),
    ] {
        if !click(session, pointer, ui_rect, region)? {
            return Ok(Some(format!("no `{region}` region: {what} was not drawn.")));
        }
    }
    pointer.gone(session)?;
    session.settle(30);
    let chosen = shot(
        ctx,
        report,
        session,
        pointer,
        "model-view-poster-chosen.png",
    )?;
    let trace = session.trace()?;
    for key in [
        "model-view-poster",
        "model-poster-requested",
        "model-poster-declined",
    ] {
        let line = trace.events(key).last().map(|l| l.raw.clone());
        report.note(format!("{key}: {line:?}"));
    }
    if trace.events(EDIT).count() == 0 {
        return Ok(Some(format!(
            "the press made no `{EDIT}` edit: the view never reached the page."
        )));
    }
    let Some(page) = declared(&trace, ui_rect, PAGE_REGION) else {
        return Ok(Some(format!("no `{PAGE_REGION}` region on the canvas.")));
    };
    let page = session.frame()?.logical_to_capture_pixels(page);
    let changed = changed(&placed, &chosen, page);
    report.note(format!("page pixels changed by the press: {changed}"));
    Ok((changed < MIN_CHANGED).then(|| {
        format!(
            "the press changed {changed} page pixels, under {MIN_CHANGED}: the page still shows \
             the picture it had."
        )
    }))
}

fn differs(a: Rgb, b: Rgb) -> bool {
    a.r.abs_diff(b.r) > TOLERANCE || a.g.abs_diff(b.g) > TOLERANCE || a.b.abs_diff(b.b) > TOLERANCE
}

/// How many pixels inside `page` differ between `a` and `b`.
fn changed(a: &Image, b: &Image, page: PixRect) -> usize {
    let w = a.width().min(b.width()).min(page.x + page.w);
    let h = a.height().min(b.height()).min(page.y + page.h);
    (page.y..h)
        .flat_map(|y| (page.x..w).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            a.pixel(x, y)
                .zip(b.pixel(x, y))
                .is_some_and(|(p, q)| differs(p, q))
        })
        .count()
}

/// See the module documentation.
pub struct APictureFileBecomesTheModelsPagePicture;

impl Check for APictureFileBecomesTheModelsPagePicture {
    fn name(&self) -> &'static str {
        "a_picture_file_becomes_a_3d_models_page_picture"
    }

    fn defect(&self) -> &'static str {
        "Picture… on a 3D model's Attachments row leaves the page's picture of the model as it \
         was"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = red_png(ctx)
            .and_then(|png| {
                let extra = [(IMAGE_ENV, png.to_string_lossy().into_owned())];
                launch_with_model_env(ctx, &mut report, "model-picture", MODEL, &extra)
            })
            .and_then(|(session, pointer)| drive_picture(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A 400 x 300 solid red PNG, the picker's answer.
fn red_png(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    let (w, h) = (400u32, 300u32);
    let bgra = [0u8, 0, 255, 255].repeat((w * h) as usize);
    let path = ctx.out("model-picture-red.png");
    Image::from_bgra(w, h, bgra)?.save_png(&path)?;
    Ok(path)
}

fn drive_picture(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let placed = shot(ctx, report, session, pointer, "model-picture-placed.png")?;
    let ui_rect = ui_rect(ctx)?;
    if !click(session, pointer, ui_rect, PICTURE_REGION)? {
        return Ok(Some(format!(
            "no `{PICTURE_REGION}` region: Picture… was not drawn beside the placed model."
        )));
    }
    pointer.gone(session)?;
    session.settle(30);
    let picked = shot(ctx, report, session, pointer, "model-picture-picked.png")?;
    let trace = session.trace()?;
    for key in [
        "model-poster-requested",
        "model-poster-declined",
        "model-poster-cancelled",
    ] {
        let line = trace.events(key).last().map(|l| l.raw.clone());
        report.note(format!("{key}: {line:?}"));
    }
    if trace.events(EDIT).count() == 0 {
        return Ok(Some(format!(
            "the press made no `{EDIT}` edit: the picture never reached the page."
        )));
    }
    let Some(page) = declared(&trace, ui_rect, PAGE_REGION) else {
        return Ok(Some(format!("no `{PAGE_REGION}` region on the canvas.")));
    };
    let page = session.frame()?.logical_to_capture_pixels(page);
    let before = red(&placed, page);
    let after = red(&picked, page);
    report.note(format!("red page pixels: {before} before, {after} after"));
    Ok((after.saturating_sub(before) < MIN_CHANGED).then(|| {
        format!(
            "the page gained {} red pixels, under {MIN_CHANGED}: it does not show the red picture.",
            after.saturating_sub(before)
        )
    }))
}

/// How many pixels inside `page` are plainly red.
fn red(image: &Image, page: PixRect) -> usize {
    let w = image.width().min(page.x + page.w);
    let h = image.height().min(page.y + page.h);
    (page.y..h)
        .flat_map(|y| (page.x..w).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            image
                .pixel(x, y)
                .is_some_and(|p| p.r > 200 && p.g < 60 && p.b < 60)
        })
        .count()
}
