//! `an_open_paragraph_shows_each_words_look` — a paragraph opened for editing
//! is drawn in place in its own fonts, each word in its own colour: the bold
//! red word of `mixed-paragraph.pdf` stays red while the draft is open.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/draft_mixed_ink.md`.

use super::dimdrive::run_on;
use super::draft_mixed::{FIXTURE, ON_FIRST_LINE};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::Result;
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const SHAPED: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
/// Line 2, baseline 684, which holds the red word `notes`; PDF points
/// (x0, y0, x1, y1).
const RED_LINE: (f64, f64, f64, f64) = (72.0, 680.0, 330.0, 696.0);
/// Line 1, baseline 700, all black.
const BLACK_LINE: (f64, f64, f64, f64) = (72.0, 698.0, 400.0, 712.0);
/// Fewest red pixels that count as the word drawn at the default zoom.
const MIN_RED: usize = 20;
/// Fewest dark pixels that count as a line drawn.
const MIN_DARK: usize = 40;

/// An open paragraph shows each word's look.
pub struct AnOpenParagraphShowsEachWordsLook;

impl Check for AnOpenParagraphShowsEachWordsLook {
    fn name(&self) -> &'static str {
        "an_open_paragraph_shows_each_words_look"
    }

    fn defect(&self) -> &'static str {
        "a paragraph opened for editing is drawn in an editor box in the program's own font, or \
         every word in the first word's colour, so the operator cannot see what the edit will \
         look like until it is committed (OPERATOR_REQUESTS.md O295)"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "draft-mixed-ink",
            drive,
        )
    }
}

/// Red ink, at any coverage over white paper: not black, not the accent
/// blue of the caret and outline.
fn red(image: &Image, region: PixRect) -> usize {
    image
        .pixels_in(region)
        .filter(|p| {
            let (r, g, b) = (i32::from(p.r), i32::from(p.g), i32::from(p.b));
            r - g > 60 && r - b > 60
        })
        .count()
}

fn dark(image: &Image, region: PixRect) -> usize {
    image
        .pixels_in(region)
        .filter(|p| u16::from(p.r) + u16::from(p.g) + u16::from(p.b) < 240)
        .count()
}

/// Red and dark pixel counts on the two lines, as `[red line red, red line
/// dark, black line red, black line dark]`.
fn counts(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    png: &str,
    bands: (LRect, LRect),
) -> Result<[usize; 4]> {
    let path = ctx.out(png);
    pointer.screenshot(session, &path)?;
    let image = Image::load_png(&path)?;
    let frame = session.frame()?;
    let (r, b) = (
        frame.logical_to_capture_pixels(bands.0),
        frame.logical_to_capture_pixels(bands.1),
    );
    Ok([
        red(&image, r),
        dark(&image, r),
        red(&image, b),
        dark(&image, b),
    ])
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let band = |(x0, y0, x1, y1): (f64, f64, f64, f64)| -> Result<LRect> {
        let a = mapping.doc_to_window(DocPoint::new(0, x0, y1))?;
        let b = mapping.doc_to_window(DocPoint::new(0, x1, y0))?;
        Ok(LRect::new(Pt::new(a.x(), a.y()), Pt::new(b.x(), b.y())))
    };
    let bands = (band(RED_LINE)?, band(BLACK_LINE)?);
    let page = counts(ctx, session, pointer, "draft-mixed-ink.page.png", bands)?;
    report.note(format!("page, before the click: {page:?}"));
    // Calibration: the bands must see the page's own red word and black line.
    if page[0] < MIN_RED || page[3] < MIN_DARK || page[2] > 0 {
        return Err(crate::error::Error::new(format!(
            "the page itself reads {page:?} (red on line 2, dark on line 2, red on line 1, dark \
             on line 1): the bands do not sit on the fixture's lines at this zoom."
        )));
    }
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, ON_FIRST_LINE.0, ON_FIRST_LINE.1))?,
    )?;
    session.settle(30);
    let shaped = session
        .trace()?
        .events(SHAPED)
        .last()
        .map(|l| l.raw.clone());
    let open = counts(ctx, session, pointer, "draft-mixed-ink.open.png", bands)?;
    report.note(format!("★ {shaped:?}"));
    report.note(format!("★★ draft open: {open:?}"));
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    let in_font = shaped
        .as_deref()
        .is_some_and(|s| s.contains("block=1") && s.contains("shaped=1"));
    if !in_font {
        return Ok(Some(format!(
            "the open paragraph was not laid out in its own fonts: last `{SHAPED}` {shaped:?}. \
             Trace: {path}."
        )));
    }
    if open[0] < MIN_RED || open[1] < MIN_DARK || open[2] > 0 || open[3] < MIN_DARK {
        return Ok(Some(format!(
            "with the paragraph open, line 2 has {} red and {} dark pixels and line 1 {} red \
             and {} dark; want the red word still red ({MIN_RED}+), both lines inked \
             ({MIN_DARK}+) and no red on line 1. Compare draft-mixed-ink.page.png with \
             draft-mixed-ink.open.png. Trace: {path}.",
            open[0], open[1], open[2], open[3]
        )));
    }
    Ok(None)
}
