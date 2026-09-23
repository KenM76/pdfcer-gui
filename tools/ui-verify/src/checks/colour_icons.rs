//! `icons_are_coloured_only_when_asked` — O232, the optional coloured icon set.
//!
//! Two launches on the same document, `colour_icons = false` then `true`, and
//! the band above the canvas (ribbon, tab strip, quick-access bar) photographed
//! in each. Pixels are sorted into three hue classes the plain chrome does not
//! use — green, red, amber. Blue is not counted, because the theme's own
//! accent is blue and would count in both launches.
//!
//! # What fails it
//!
//! - **Off is not plain:** more than [`OFF_TOLERANCE`] such pixels with the
//!   option off. The option's contract is that off looks as it always has.
//! - **On does nothing:** fewer than [`ON_FLOOR`] with it on — the preference
//!   was read and never reached the painter, the join no unit test sees.

use std::path::Path;

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Rgb;
use crate::launch::{LaunchSpec, Session};

const CANVAS_REGION: &str = "canvas-viewport";

/// Accent-hued pixels allowed with the option off: antialiasing at a
/// document thumbnail's edge, never an icon.
const OFF_TOLERANCE: usize = 20;

/// Accent-hued pixels required with it on. The accents are single strokes
/// and blue ones are not counted, so the File tab at the harness's default
/// window carries on the order of a hundred; forty keeps a clear margin over
/// both that and [`OFF_TOLERANCE`].
const ON_FLOOR: usize = 40;

pub struct IconsAreColouredOnlyWhenAsked;

impl Check for IconsAreColouredOnlyWhenAsked {
    fn name(&self) -> &'static str {
        "icons_are_coloured_only_when_asked"
    }

    fn defect(&self) -> &'static str {
        "the coloured-icons option either leaks colour into the default look or never reaches \
         the icons (O232)"
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

/// Counts of the three hue classes.
#[derive(Default, Debug)]
struct Hues {
    green: usize,
    red: usize,
    amber: usize,
}

impl Hues {
    fn total(&self) -> usize {
        self.green + self.red + self.amber
    }

    fn add(&mut self, p: Rgb) {
        let (r, g, b) = (i32::from(p.r), i32::from(p.g), i32::from(p.b));
        if g >= r + 40 && g >= b + 30 {
            self.green += 1;
        } else if r >= g + 70 && r >= b + 60 && g < 150 {
            self.red += 1;
        } else if r >= b + 80 && g >= b + 40 && r > g {
            self.amber += 1;
        }
    }
}

fn write_preference(exe: &Path, on: bool) -> Result<()> {
    let dir = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::new(format!("could not create {}: {e}", dir.display())))?;
    // Through `sandbox::write_prefs`, never `fs::write`: see `ui_scale`.
    crate::sandbox::write_prefs(&dir, &format!("colour_icons = {on}\n"))
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))
}

fn measure(ctx: &CheckContext, report: &mut CheckReport, on: bool) -> Result<Hues> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf: the full ribbon draws only with a document open."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    write_preference(&exe, on)?;
    let tag = if on { "on" } else { "off" };

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("colour_icons.{tag}.trace.txt")));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);

    let trace = session.trace()?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;
    let png = ctx.out(&format!("colour_icons.{tag}.png"));
    let image = crate::capture::window_to_png(&session, &png)?;
    report.artifact(png);

    let band = LRect::new(Pt::new(0.0, 0.0), Pt::new(canvas.max.x, canvas.min.y));
    let px = frame.logical_to_capture_pixels(band);
    let mut hues = Hues::default();
    for p in image.pixels_in(px) {
        hues.add(p);
    }
    report.note(format!(
        "colour_icons = {on}: over the band above the canvas, {} green, {} red, {} amber",
        hues.green, hues.red, hues.amber
    ));
    Ok(hues)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // Leave the profile as a first run left it, on every path out.
    struct Restore<'a>(Option<&'a Path>);
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            if let Some(exe) = self.0
                && let Some(dir) = exe.parent()
            {
                let _ = crate::sandbox::reset_prefs(&dir.join("userdata"));
            }
        }
    }
    let exe = ctx.resolve_exe();
    let _restore = Restore(exe.as_deref());

    let off = measure(ctx, report, false)?;
    let on = measure(ctx, report, true)?;
    let mut failures = Vec::new();
    if off.total() > OFF_TOLERANCE {
        failures.push(format!(
            "with coloured icons OFF the chrome carries {} accent-hued pixels ({off:?}); off \
             must look as it always has",
            off.total()
        ));
    }
    if on.total() < ON_FLOOR {
        failures.push(format!(
            "with coloured icons ON the chrome carries only {} accent-hued pixels ({on:?}), \
             under {ON_FLOOR}: the preference did not reach the icons",
            on.total()
        ));
    }
    Ok((!failures.is_empty()).then(|| failures.join("\n")))
}
