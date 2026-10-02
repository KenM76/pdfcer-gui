//! `a_key_typed_mid_line_previews_where_it_commits` — a letter typed into the
//! middle of a line a word processor wrote in pieces is previewed with the rest
//! of its words moved over to make room, exactly where the commit puts them.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/mid_line_preview.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// First line `Date Premises Required____`, one `TJ` at y=700 from x=72.
const FIXTURE: &str = "word-fragmented-lines.pdf";
const INVOKE: &str = "mode.edit,edit.text";
const PAGE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
/// Inside `Date`, PDF points.
const AIM: (f64, f64) = (100.0, 704.0);
/// The line's ink band, PDF points `(x0, y_top, x1, y_bottom)`: inside the
/// draft's outline vertically, past the line's end horizontally.
const BAND: (f64, f64, f64, f64) = (66.0, 709.0, 260.0, 698.0);
/// Caret steps from Home to the end of `Date`.
const STEPS: usize = 4;
const TYPED: &str = "X";
/// The line's characters once `X` is in.
const CHARS: &str = "chars=28";
const SHAPED: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
/// Below this share of differing ink columns the preview and the commit are
/// one picture; the two rasterisers' antialiasing alone stays well under it.
const SAME: f64 = 0.2;
/// Above this the band can see a moved word: the untyped line against the
/// commit, the control.
const SEEN: f64 = 0.3;

/// See the module documentation.
pub struct AKeyTypedMidLinePreviewsWhereItCommits;

impl Check for AKeyTypedMidLinePreviewsWhereItCommits {
    fn name(&self) -> &'static str {
        "a_key_typed_mid_line_previews_where_it_commits"
    }

    fn defect(&self) -> &'static str {
        "a letter typed into the middle of a line is previewed on top of the next one, and \
         the words after it jump right only when the edit is committed"
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

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(FIXTURE, "Run fixtures/word-fragmented-lines.PROVENANCE.py.")?;
    let doc = ctx.out("mid-line-preview.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("mid-line-preview.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("mid-line-preview.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

/// Shoot the window to `name` and read it back.
fn shot(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    name: &str,
) -> Result<Image> {
    let png = ctx.out(name);
    pointer.screenshot(session, &png)?;
    let image = Image::load_png(&png)?;
    report.artifact(png);
    Ok(image)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
    let corner = |x, y| mapping.doc_to_window(DocPoint::new(0, x, y));
    let (a, b) = (corner(BAND.0, BAND.1)?, corner(BAND.2, BAND.3)?);
    let band = LRect::new(Pt::new(a.x(), a.y()), Pt::new(b.x(), b.y()));
    pointer.click(&session, corner(AIM.0, AIM.1)?)?;
    session.settle(20);
    pointer.key(&session, None, "Home", None)?;
    for _ in 0..STEPS {
        pointer.key(&session, None, "ArrowRight", None)?;
    }
    session.settle(15);
    let untyped = shot(ctx, &session, &pointer, report, "mid-line-untyped.png")?;
    pointer.type_text(&session, None, TYPED)?;
    session.settle(30);
    let preview = shot(ctx, &session, &pointer, report, "mid-line-preview.png")?;
    let shaped = session
        .trace()?
        .events(SHAPED)
        .last()
        .map(|l| l.raw.clone());
    pointer.key(&session, None, "Escape", None)?;
    session.settle(40);
    pointer.gone(&session)?;
    session.settle(10);
    let committed = shot(ctx, &session, &pointer, report, "mid-line-committed.png")?;
    let px = session.frame()?.logical_to_capture_pixels(band);
    judge(
        report,
        shaped.as_deref(),
        [&untyped, &preview, &committed],
        px,
    )
}

fn judge(
    report: &mut CheckReport,
    shaped: Option<&str>,
    [untyped, preview, committed]: [&Image; 3],
    band: PixRect,
) -> Result<Option<String>> {
    if !shaped.is_some_and(|l| l.contains(CHARS) && l.contains("shaped=1")) {
        return Err(Error::new(format!(
            "the draft was not drawn in the line's own face with `{TYPED}` in it (`{shaped:?}`), \
             so there is no preview to compare."
        )));
    }
    let control = differing_columns(untyped, committed, band);
    let measured = differing_columns(preview, committed, band);
    report.note(format!(
        "differing ink columns against the commit: untyped line {control:.2}, preview \
         {measured:.2}, in {band:?}"
    ));
    if control < SEEN {
        return Err(Error::new(format!(
            "the untyped line and the committed one differ in {control:.2} of their ink \
             columns, so this band cannot see a moved word."
        )));
    }
    if measured >= SAME {
        return Ok(Some(format!(
            "★ the preview differs from the committed line in {measured:.2} of its ink columns: \
             the words after `{TYPED}` are not drawn where the commit puts them."
        )));
    }
    Ok(None)
}

/// The share of columns of `band` holding dark ink in one image and not the
/// other. Accent-blue pixels (the draft's outline and caret) are not ink.
fn differing_columns(a: &Image, b: &Image, band: PixRect) -> f64 {
    let inked = |img: &Image, x: u32| {
        (band.y..band.y + band.h).any(|y| {
            img.pixel(x, y).is_some_and(|p| {
                let sum = u16::from(p.r) + u16::from(p.g) + u16::from(p.b);
                sum < 3 * 128 && u16::from(p.b) <= u16::from(p.r) + 40
            })
        })
    };
    let (mut either, mut one) = (0u32, 0u32);
    for x in band.x..band.x + band.w {
        let (i, j) = (inked(a, x), inked(b, x));
        either += u32::from(i || j);
        one += u32::from(i != j);
    }
    if either == 0 {
        return 0.0;
    }
    f64::from(one) / f64::from(either)
}
