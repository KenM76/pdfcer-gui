//! `button_looks` — **while filling, a button under the pointer is drawn from
//! its `/AP /R` look, and from its `/AP /D` look while pressed; moving off it
//! puts the normal look back**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/button_looks.md`.

use super::dimdrive::launch_path;
use super::print_shop::await_line;
use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::{ScriptedPointer, xy};
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review"; // ui-text-exempt: a command id
const FIXTURE: &str = "button-looks.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/button-looks.PROVENANCE.py`.";
const LINE: &str = "canvas-widget-look"; // ui-text-exempt: a trace event name

/// One button of the fixture: its name and `/Rect` in PDF user space.
struct Button {
    name: &'static str,
    rect: (f64, f64, f64, f64),
}

const BUTTONS: [Button; 2] = [
    Button {
        name: "Go (push button)",
        rect: (72.0, 600.0, 232.0, 660.0),
    },
    Button {
        name: "Tick (check box)",
        rect: (72.0, 450.0, 232.0, 510.0),
    },
];

/// Which of the fixture's three flat colours a sample is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Colour {
    Grey,
    Blue,
    Red,
    Other,
}

/// See the module documentation.
pub struct AButtonsRolloverAndDownLooksAreDrawn;

impl Check for AButtonsRolloverAndDownLooksAreDrawn {
    fn name(&self) -> &'static str {
        "a_buttons_rollover_and_down_looks_are_drawn"
    }

    fn defect(&self) -> &'static str {
        "a push button or check box carrying a rollover or down appearance keeps its normal look \
         while hovered or pressed, or keeps the hover look after the pointer leaves"
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
    let doc = ctx.out("button_looks.pdf");
    let source = driving::repo_fixture(FIXTURE, METHOD)?;
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("no page size in {}.", source.display())))?;
    let (session, pointer) = launch_path(ctx, report, doc, INVOKE, "button_looks", &[])?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(45);
    let mut outcome = Ok(None);
    for button in &BUTTONS {
        outcome = walk(ctx, report, (&session, &pointer), page, button);
        if !matches!(outcome, Ok(None)) {
            break;
        }
    }
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Away, over, pressed, released and away again: grey, blue, red, then grey.
fn walk(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
    button: &Button,
) -> Result<Option<String>> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let (x0, y0, x1, y1) = button.rect;
    let at = |x: f64, y: f64| mapping.doc_to_window(DocPoint::new(0, x, y));
    let centre = at(f64::midpoint(x0, x1), f64::midpoint(y0, y1))?;
    let away = at(x1 + 120.0, y0 - 40.0)?;
    let pixels = inner(session, (at(x0, y0)?, at(x1, y1)?))?;
    let steps: [(&str, &str, &str, Colour); 4] = [
        ("away", "move", "kind=none", Colour::Grey),
        ("hover", "move", "kind=rollover", Colour::Blue),
        ("press", "down", "kind=down", Colour::Red),
        ("leave", "up-away", "kind=none", Colour::Grey),
    ];
    for (stage, verb, kind, owed) in steps {
        let mark = session.trace()?.mark();
        match verb {
            "move" if stage == "away" => drop(pointer.hover(session, away)?),
            "move" => drop(pointer.hover(session, centre)?),
            "down" => drop(pointer.send(session, &format!("down {}", xy(centre)))?),
            _ => {
                pointer.send(session, &format!("up {}", xy(centre)))?;
                pointer.hover(session, away)?;
            }
        }
        let tag = kind.trim_start_matches("kind=");
        let seen = await_line(session, LINE, mark, |l| {
            l.raw.contains(kind) && l.get("drawn") == Some("1")
        })?;
        // The first stage only settles: the look starts as none, so no change
        // is traced for it.
        if seen.is_none() && stage != "away" {
            let last = session.trace()?.last(LINE).map(|l| l.raw.clone());
            return Ok(Some(format!(
                "★★★ {} at `{stage}`: no texture drawn with the {tag} look; last `{LINE}` line \
                 {last:?}.",
                button.name
            )));
        }
        session.settle(20);
        let png = ctx.out(&format!("button-looks-{}-{stage}.png", &button.name[..2]));
        pointer.screenshot(session, &png)?;
        let got = classify(&Image::load_png(&png)?, pixels);
        report.note(format!("{} {stage}: {got:?}", button.name));
        if got != owed {
            return Ok(Some(format!(
                "★★★ {} at `{stage}` drew {got:?} where its {tag} look is {owed:?}.",
                button.name
            )));
        }
    }
    Ok(None)
}

/// The button's box less a margin, in capture pixels: corners `a` and `z`.
fn inner(session: &Session, (a, z): (WindowPoint, WindowPoint)) -> Result<PixRect> {
    let rect = LRect::new(
        Pt::new(a.x().min(z.x()) + 6.0, a.y().min(z.y()) + 6.0),
        Pt::new(a.x().max(z.x()) - 6.0, a.y().max(z.y()) - 6.0),
    );
    Ok(session.frame()?.logical_to_capture_pixels(rect))
}

/// The colour most of the box's pixels are.
fn classify(image: &Image, px: PixRect) -> Colour {
    let mut counts = [0usize; 4];
    for p in image.pixels_in(px) {
        let (r, g, b) = (i32::from(p.r), i32::from(p.g), i32::from(p.b));
        let at = if r > 170 && g > 170 && b > 170 && (r - b).abs() < 20 && r < 235 {
            0
        } else if b > 170 && r < 80 {
            1
        } else if r > 170 && g < 80 && b < 80 {
            2
        } else {
            3
        };
        counts[at] += 1;
    }
    let total: usize = counts.iter().sum();
    let best = (0..4).max_by_key(|&i| counts[i]).unwrap_or(3);
    if total == 0 || counts[best] * 10 < total * 8 {
        return Colour::Other;
    }
    [Colour::Grey, Colour::Blue, Colour::Red, Colour::Other][best]
}
