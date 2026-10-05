//! `an_ocr_word_is_previewed_in_its_own_font_in_the_layer_colour` — with the
//! OCR text layer shown, editing a recognised word previews it in the word's
//! own font, in the layer's colour, in place of the layer's drawing of it, and
//! the saved word is still invisible. Run with the scripted pointer in a
//! window placed off the desktop, on a copy of `fixtures/ocr-layer.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_edit_preview.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "ocr-layer.pdf";
/// Edit mode, the OCR layer shown, the Edit Text tool armed.
const INVOKE: &str = "mode.edit,view.ocr_layer,edit.text"; // ui-text-exempt: command ids, never displayed
/// Just inside the end of the recognised word `SITE`, 14 pt Helvetica from
/// (72, 700), whose advance ends at x = 103.1, in PDF points. A click there
/// puts the caret after the `E`; End would not, since it goes to the end of
/// the line, which in a scan's layer is another word.
const CLICK: (f64, f64) = (102.0, 704.0);
/// Letters typed at the end of the word; none of them is in the fixture.
const TYPED: &str = "XQ";
/// `SITE` and the room the typed letters take, in PDF points (x0, y0, x1, y1).
const EDITED: (f64, f64, f64, f64) = (70.0, 697.0, 128.0, 712.0);
/// The neighbouring recognised word `DRAWING`, on the line below, which the
/// layer must still draw.
const NEIGHBOUR: (f64, f64, f64, f64) = (143.0, 673.0, 210.0, 688.0);
/// Fewest layer-coloured pixels that count as a drawn word at the default zoom.
const MIN_INK: usize = 20;
const SHAPED: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
const FALLBACK: &str = "text-edit-preview-fallback"; // ui-text-exempt: a trace event name, never displayed
const HELD: &str = "ocr-layer-held"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AnOcrWordIsPreviewedInItsOwnFont;

impl Check for AnOcrWordIsPreviewedInItsOwnFont {
    fn name(&self) -> &'static str {
        "an_ocr_word_is_previewed_in_its_own_font_in_the_layer_colour"
    }

    fn defect(&self) -> &'static str {
        "editing a recognised word with the text layer shown previews it in a stand-in font, in \
         a colour the layer does not use, drawn twice, or hides the words around it — or the \
         saved word is no longer invisible"
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

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer, std::path::PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(FIXTURE, "Run tools/gen-ocr-layer-fixture.py.")?;
    let doc = ctx.out("ocr-edit-preview.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr-edit-preview.trace.txt"));
    spec.pdf = Some(doc.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("ocr-edit-preview.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

/// What the drive saw while the edit was open.
struct Seen {
    shaped: Option<String>,
    fallback: Option<String>,
    held: Option<String>,
    image: Image,
    edited: PixRect,
    neighbour: PixRect,
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let seen = edit(ctx, report, &session, &pointer, &doc);
    let parked = pointer.gone(&session);
    let seen = seen?;
    parked?;
    let path = session.trace_path().display().to_string();
    if let Some(failure) = judge_preview(report, &seen, &path) {
        return Ok(Some(failure));
    }
    judge_save(report, &doc)
}

fn edit(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    doc: &std::path::Path,
) -> Result<Seen> {
    let page = crate::fixture::page_geometry(doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let band = |(x0, y0, x1, y1): (f64, f64, f64, f64)| -> Result<LRect> {
        let a = mapping.doc_to_window(DocPoint::new(0, x0, y1))?;
        let b = mapping.doc_to_window(DocPoint::new(0, x1, y0))?;
        Ok(LRect::new(Pt::new(a.x(), a.y()), Pt::new(b.x(), b.y())))
    };
    let (edited, neighbour) = (band(EDITED)?, band(NEIGHBOUR)?);
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, CLICK.0, CLICK.1))?,
    )?;
    session.settle(20);
    pointer.type_text(session, None, TYPED)?;
    session.settle(30);
    let png = ctx.out("ocr-edit-preview.png");
    pointer.screenshot(session, &png)?;
    let image = Image::load_png(&png)?;
    report.artifact(png);
    let trace = session.trace()?;
    let last = |event: &str| trace.events(event).last().map(|l| l.raw.clone());
    let frame = session.frame()?;
    let seen = Seen {
        shaped: last(SHAPED),
        fallback: last(FALLBACK),
        held: last(HELD),
        image,
        edited: frame.logical_to_capture_pixels(edited),
        neighbour: frame.logical_to_capture_pixels(neighbour),
    };
    // Escape commits the edit; Ctrl+S writes the file.
    pointer.key(session, None, "Escape", None)?;
    session.settle(30);
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    if !session
        .trace()?
        .events("save-in-place")
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Err(Error::new(format!(
            "Ctrl+S traced no `save-in-place outcome=ok`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok(seen)
}

/// A pixel of the layer's default colour, `[204, 0, 153]`, at any coverage
/// over white paper: red and blue well above green. Not black stand-in ink,
/// and not the accent-blue caret or outline.
fn layer_hued(image: &Image, region: PixRect) -> usize {
    image
        .pixels_in(region)
        .filter(|p| {
            let (r, g, b) = (i32::from(p.r), i32::from(p.g), i32::from(p.b));
            r - g > 60 && b - g > 40 && r > b
        })
        .count()
}

/// A pixel dark enough to be ink in a non-layer colour: what a stand-in or a
/// preview in the run's own (black) fill would leave.
fn dark(image: &Image, region: PixRect) -> usize {
    image
        .pixels_in(region)
        .filter(|p| u16::from(p.r) + u16::from(p.g) + u16::from(p.b) < 180)
        .count()
}

fn judge_preview(report: &mut CheckReport, seen: &Seen, path: &str) -> Option<String> {
    let shaped = seen.shaped.clone().unwrap_or_default();
    if !(shaped.contains("shaped=1") && shaped.contains("chars=6")) {
        return Some(format!(
            "★ the edited word was not laid out in its own font with `{TYPED}` in it: last \
             `{SHAPED}` {:?}, last `{FALLBACK}` {:?}. Trace: {path}.",
            seen.shaped, seen.fallback
        ));
    }
    report.note(format!("★ {shaped}"));
    if seen
        .fallback
        .as_deref()
        .is_some_and(|l| !l.contains("reason=none"))
    {
        return Some(format!(
            "★ the preview recorded a fallback: {:?}. Trace: {path}.",
            seen.fallback
        ));
    }
    if seen.held.as_deref().is_none_or(|l| !l.contains("runs=1")) {
        return Some(format!(
            "★★ the layer did not leave exactly the edited word to the preview: last `{HELD}` \
             {:?}. The word is drawn twice, or a neighbour is hidden. Trace: {path}.",
            seen.held
        ));
    }
    report.note(format!("★★ {}", seen.held.clone().unwrap_or_default()));
    let (ink, dark_ink) = (
        layer_hued(&seen.image, seen.edited),
        dark(&seen.image, seen.edited),
    );
    let neighbour = layer_hued(&seen.image, seen.neighbour);
    report.note(format!(
        "★★★ edited band: {ink} layer-coloured, {dark_ink} dark pixels; neighbour: {neighbour} \
         layer-coloured"
    ));
    if ink < MIN_INK || dark_ink > ink / 4 {
        return Some(format!(
            "★★★ the edited word is not drawn in the layer's colour: {ink} layer-coloured and \
             {dark_ink} dark pixels in its band. Trace: {path}."
        ));
    }
    (neighbour < MIN_INK).then(|| {
        format!(
            "★★★ the neighbouring word `DRAWING` lost its layer drawing while `SITE` was \
             edited: {neighbour} layer-coloured pixels. Trace: {path}."
        )
    })
}

/// The saved word must still be shown in rendering mode 3.
fn judge_save(report: &mut CheckReport, pdf: &std::path::Path) -> Result<Option<String>> {
    let modes = crate::checks::invisible_text_scripted::modes_showing(pdf, "SITEXQ")?;
    report.note(format!("`SITEXQ` saved under rendering modes {modes:?}"));
    if modes.is_empty() {
        return Ok(Some(
            "★★★★ the saved file shows `SITEXQ` in no content stream: the edit was not written."
                .to_owned(),
        ));
    }
    Ok((!modes.iter().all(|m| m == "3")).then(|| {
        format!(
            "★★★★ the edited word is saved under rendering modes {modes:?}, not 3: it is now \
             drawn on the scan."
        )
    }))
}
