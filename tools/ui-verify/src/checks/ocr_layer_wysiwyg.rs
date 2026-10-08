//! `the_ocr_layer_draws_a_word_as_its_edit_does` — on a layer the engine wrote
//! with Paddle (per-word size and horizontal scaling), the OCR text layer
//! draws a word pixel for pixel as the text editor previews it, before an edit
//! opens and after it commits. Run with the scripted pointer in a window
//! placed off the desktop, on a copy of `fixtures/ocr-paddle-layer.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_layer_wysiwyg.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
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
const FIXTURE: &str = "ocr-paddle-layer.pdf";
/// Edit mode, the OCR layer shown, the Edit Text tool armed.
const INVOKE: &str = "mode.edit,view.ocr_layer,edit.text"; // ui-text-exempt: command ids, never displayed
/// Inside the last letter of `dimensions`, whose glyphs run from x = 66.8 to
/// 117.1 on the baseline y = 336.5, in PDF points: the caret lands after it.
const CLICK: (f64, f64) = (115.5, 340.0);
/// Letters typed at the end of the word; none of them is in the fixture.
const TYPED: &str = "XQ";
/// Fewest runs the layer must lay out in their own fonts. The page has 96.
const MIN_LAID: usize = 90;
/// Fewest layer-coloured pixels that count as a drawn word.
const MIN_INK: usize = 20;
/// A pixel differs when a channel moves by more than this.
const CHANNEL: i32 = 48;
/// The largest share of a word's layer-coloured pixels that may differ.
const MAX_DIFFERENT: f64 = 0.15;
/// Logical points trimmed off the editor box: its accent edge on every side,
/// and the caret, at the right-hand end, on the right.
const TRIM: f32 = 3.0;
const TRIM_RIGHT: f32 = 7.0;
const BUILT: &str = "ocr-ink-built"; // ui-text-exempt: a trace event name, never displayed
const BOX: &str = "text-edit.box"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct TheOcrLayerDrawsAWordAsItsEditDoes;

impl Check for TheOcrLayerDrawsAWordAsItsEditDoes {
    fn name(&self) -> &'static str {
        "the_ocr_layer_draws_a_word_as_its_edit_does"
    }

    fn defect(&self) -> &'static str {
        "the OCR text layer draws a recognised word in another font, size, place or strength \
         than the text editor previews it, so the word jumps when an edit opens or commits"
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
    let source = repo_fixture(FIXTURE, "See fixtures/ocr-paddle-layer.PROVENANCE.md.")?;
    let doc = ctx.out("ocr-layer-wysiwyg.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr-layer-wysiwyg.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("ocr-layer-wysiwyg.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

/// Two pictures of one word, compared over one band.
struct Pair {
    what: &'static str,
    a: Image,
    b: Image,
    band: PixRect,
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let pairs = edit(ctx, report, &session, &pointer, &doc);
    let parked = pointer.gone(&session);
    let pairs = pairs?;
    parked?;
    let path = session.trace_path().display().to_string();
    let pairs = match pairs {
        Ok(pairs) => pairs,
        Err(failure) => return Ok(Some(format!("{failure} Trace: {path}."))),
    };
    for pair in &pairs {
        if let Some(failure) = judge(report, pair) {
            return Ok(Some(format!("{failure} Trace: {path}.")));
        }
    }
    Ok(None)
}

/// Wait for an `ocr-ink-built` line after `mark`; its `laid=`.
fn built_after(session: &Session, mark: usize) -> Result<Option<usize>> {
    for _ in 0..30 {
        if let Some(line) = session.trace()?.last_after(BUILT, mark) {
            return Ok(line.get_usize("laid"));
        }
        session.settle(10);
    }
    Ok(None)
}

fn shot(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    name: &str,
) -> Result<Image> {
    let png = ctx.out(&format!("ocr-layer-wysiwyg-{name}.png"));
    pointer.screenshot(session, &png)?;
    let image = Image::load_png(&png)?;
    report.artifact(png);
    Ok(image)
}

/// The editor box's last declared rectangle, trimmed to the glyphs.
fn edited_band(session: &Session) -> Result<Option<PixRect>> {
    let Some(rect) = declared(&session.trace()?, "ui-rect", BOX) else {
        return Ok(None);
    };
    let inner = LRect::new(
        Pt::new(rect.min.x + TRIM, rect.min.y + TRIM),
        Pt::new(rect.max.x - TRIM_RIGHT, rect.max.y - TRIM),
    );
    Ok(Some(session.frame()?.logical_to_capture_pixels(inner)))
}

/// The drive: the layer, the edit opened on it, the edit typed into, the edit
/// committed. `Ok(Err(..))` is a failure found on the way.
fn edit(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    doc: &std::path::Path,
) -> Result<std::result::Result<Vec<Pair>, String>> {
    let Some(laid) = built_after(session, 0)? else {
        return Ok(Err(format!("★ the layer traced no `{BUILT}` line.")));
    };
    report.note(format!("★ {laid} runs laid out in their own fonts"));
    if laid < MIN_LAID {
        return Ok(Err(format!(
            "★ the layer laid out {laid} runs in their own fonts, fewer than {MIN_LAID}: the \
             rest are drawn in a stand-in."
        )));
    }
    let layer = shot(ctx, report, session, pointer, "layer")?;
    let page = crate::fixture::page_geometry(doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, CLICK.0, CLICK.1))?,
    )?;
    session.settle(30);
    let open = shot(ctx, report, session, pointer, "open")?;
    let Some(opened) = edited_band(session)? else {
        return Ok(Err(format!("★★ the click opened no editor box (`{BOX}`).")));
    };
    pointer.type_text(session, None, TYPED)?;
    session.settle(30);
    let typed = shot(ctx, report, session, pointer, "typed")?;
    let Some(typed_band) = edited_band(session)? else {
        return Ok(Err(format!("★★★ typing closed the editor box (`{BOX}`).")));
    };
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Escape", None)?;
    if built_after(session, mark)?.is_none() {
        return Ok(Err(format!(
            "★★★ the layer traced no `{BUILT}` line after the edit committed."
        )));
    }
    session.settle(10);
    let committed = shot(ctx, report, session, pointer, "committed")?;
    Ok(Ok(vec![
        Pair {
            what: "the layer, and the edit opened on it",
            a: layer,
            b: open,
            band: opened,
        },
        Pair {
            what: "the edit typed into, and the layer once it committed",
            a: typed,
            b: committed,
            band: typed_band,
        },
    ]))
}

/// A pixel of the layer's default colour, `[204, 0, 153]`, at any coverage
/// over the page: red and blue well above green.
fn layer_hued(p: crate::image::Rgb) -> bool {
    let (r, g, b) = (i32::from(p.r), i32::from(p.g), i32::from(p.b));
    r - g > 60 && b - g > 40 && r > b
}

fn judge(report: &mut CheckReport, pair: &Pair) -> Option<String> {
    let (mut ink_a, mut ink_b, mut different) = (0_usize, 0_usize, 0_usize);
    for (pa, pb) in pair.a.pixels_in(pair.band).zip(pair.b.pixels_in(pair.band)) {
        ink_a += usize::from(layer_hued(pa));
        ink_b += usize::from(layer_hued(pb));
        let moved = [(pa.r, pb.r), (pa.g, pb.g), (pa.b, pb.b)]
            .iter()
            .any(|(x, y)| (i32::from(*x) - i32::from(*y)).abs() > CHANNEL);
        different += usize::from(moved);
    }
    let share = different as f64 / ink_a.max(ink_b).max(1) as f64;
    report.note(format!(
        "{}: {ink_a} and {ink_b} layer-coloured pixels, {different} different ({:.0}%)",
        pair.what,
        share * 100.0
    ));
    if ink_a.min(ink_b) < MIN_INK {
        return Some(format!(
            "★★★★ {}: the word's band holds {ink_a} and {ink_b} layer-coloured pixels; one \
             picture has no word in it.",
            pair.what
        ));
    }
    (share > MAX_DIFFERENT).then(|| {
        format!(
            "★★★★ {}: {different} of the word's pixels differ ({:.0}%, more than {:.0}%). The \
             layer does not draw the word as its edit does.",
            pair.what,
            share * 100.0,
            MAX_DIFFERENT * 100.0
        )
    })
}
