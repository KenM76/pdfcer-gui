//! Output Preview's print-production answers, driven on two fixtures.
//!
//! `output_preview_overprint` — the panel says whether the page uses
//! overprint. `output-preview.pdf` sets it on a patch whose four inks are all
//! non-zero, so it is set and changes nothing (`set=1 effect=0`);
//! `print-shop.pdf` overprints magenta onto cyan, so it takes effect
//! (`set=1 effect=1`). Each fixture is the other's control: a panel that
//! always says one answer fails on one of them.
//!
//! `output_preview_soft_proof` — the panel's *Simulate* list offers the
//! document's output intent, choosing it proofs the canvas (a new render, not
//! only a new menu label), and *Simulate paper colour* turns on black ink too
//! and renders again.
//!
//! `output_preview_coverage_warnings` — *Total area coverage* flags only the
//! 400% patch of `output-preview.pdf`; *Rich black* adds the rich black patch
//! beside it and not the plain black under it; the panel says both found
//! something and the overlay is drawn.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/output_preview_press.md`.

use crate::checks::driving::declared_in;
use crate::checks::print_shop::{
    OUTPUT_ITEM, OUTPUT_SWITCH, VIEW_TAB, await_line, launch_on, press,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: (&str, &str) = (
    "output-preview.pdf",
    "Rebuild it with `python fixtures/output-preview.PROVENANCE.py`.",
);
const PRINT_SHOP: (&str, &str) = (
    "print-shop.pdf",
    "Rebuild it with `python fixtures/print-shop.PROVENANCE.py`.",
);
const OVERPRINT: &str = "output-preview-overprint"; // ui-text-exempt: a trace event name, never displayed
const OVERPRINT_LINE: &str = "output-preview.overprint";
const UI_RECT: &str = "ui-rect";
const PROOF: &str = "output-preview-proof"; // ui-text-exempt: a trace event name, never displayed
const PROOF_LIST: &str = "output-preview.proof";
const PROOF_INTENT: &str = "output-preview.proof:intent";
const PAPER: &str = "output-preview.paper";
const BLACK_INK: &str = "output-preview.black-ink";
const RENDER: &str = "render-spawn"; // ui-text-exempt: a trace event name, never displayed
const TAC: &str = "output-preview.tac";
const RICH: &str = "output-preview.rich-black";
const WARN: &str = "output-preview-warn"; // ui-text-exempt: a trace event name, never displayed
const MASK: &str = "coverage-mask"; // ui-text-exempt: a trace event name, never displayed
const OVERLAY: &str = "coverage-overlay";
const OPACITY: &str = "output-preview.warn-opacity";

/// See the module documentation.
pub struct OutputPreviewSaysWhetherThePageOverprints;

impl Check for OutputPreviewSaysWhetherThePageOverprints {
    fn name(&self) -> &'static str {
        "output_preview_overprint"
    }

    fn defect(&self) -> &'static str {
        "Output Preview does not say whether the page uses overprint, or gives the same answer \
         for a page where overprint changes the print and one where it changes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut outcome = Ok(None);
        for (fixture, effect) in [(FIXTURE, "0"), (PRINT_SHOP, "1")] {
            outcome = drive_on(ctx, &mut report, fixture, effect);
            if !matches!(outcome, Ok(None)) {
                break;
            }
        }
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// [`overprint`] on `fixture`.
fn drive_on(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: (&str, &str),
    effect: &str,
) -> Result<Option<String>> {
    let name = format!("output_preview_overprint.{}", fixture.0);
    with_preview(ctx, report, &name, fixture, |report, session, _, mark| {
        overprint(report, session, fixture.0, effect, mark)
    })
}

/// Launch on `fixture`, switch the preview on, run `body`, and park the pointer.
fn with_preview(
    ctx: &CheckContext,
    report: &mut CheckReport,
    name: &str,
    fixture: (&str, &str),
    body: impl FnOnce(&mut CheckReport, &Session, &ScriptedPointer, usize) -> Result<Option<String>>,
) -> Result<Option<String>> {
    let (session, pointer) = launch_on(ctx, report, name, None, fixture)?;
    let outcome =
        switch_on(&session, &pointer).and_then(|mark| body(report, &session, &pointer, mark));
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Open the panel unless it is up, press the switch, and return the trace
/// mark taken before the press.
fn switch_on(session: &Session, pointer: &ScriptedPointer) -> Result<usize> {
    // Sessions of one check share a profile, so a later one opens with the
    // panel an earlier one left up; pressing the item again would close it.
    if declared_in(&session.trace()?, UI_RECT, OUTPUT_SWITCH).is_none() {
        press(session, pointer, VIEW_TAB)?;
        press(session, pointer, OUTPUT_ITEM)?;
    }
    let mark = session.trace()?.mark();
    press(session, pointer, OUTPUT_SWITCH)?;
    Ok(mark)
}

/// See the module documentation.
pub struct OutputPreviewSimulatesThePress;

impl Check for OutputPreviewSimulatesThePress {
    fn name(&self) -> &'static str {
        "output_preview_soft_proof"
    }

    fn defect(&self) -> &'static str {
        "Output Preview cannot simulate the document's output intent, or choosing a simulation or paper colour changes the panel without redrawing the canvas"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match with_preview(ctx, &mut report, self.name(), FIXTURE, soft_proof) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Choose the output intent, then paper colour; each owes its proof line and
/// a render started after the press.
fn soft_proof(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    mark: usize,
) -> Result<Option<String>> {
    if await_line(session, PROOF, mark, |l| l.get("source") == Some("none"))?.is_none() {
        return Ok(Some(format!(
            "the preview was switched on and no `{PROOF} source=none` line followed."
        )));
    }
    let mark = session.trace()?.mark();
    press(session, pointer, PROOF_LIST)?;
    if declared_in(&session.trace()?, UI_RECT, PROOF_INTENT).is_none() {
        return Ok(Some(format!(
            "the simulation list was opened and `{PROOF_INTENT}` was not in it: the document's output intent is not offered."
        )));
    }
    press(session, pointer, PROOF_INTENT)?;
    let owed = [("source", "intent"), ("paper", "0"), ("black", "0")];
    if let Some(failure) = proofed(report, session, mark, &owed)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(session, pointer, PAPER)?;
    let owed = [("source", "intent"), ("paper", "1"), ("black", "1")];
    if let Some(failure) = proofed(report, session, mark, &owed)? {
        return Ok(Some(failure));
    }
    if declared_in(&session.trace()?, UI_RECT, BLACK_INK).is_none() {
        return Ok(Some(format!(
            "a profile is simulated and `{BLACK_INK}` was never drawn."
        )));
    }
    Ok(None)
}

/// The proof line `owed` after `mark`, and a render spawned after it.
fn proofed(
    report: &mut CheckReport,
    session: &Session,
    mark: usize,
    owed: &[(&str, &str)],
) -> Result<Option<String>> {
    let matches = |l: &crate::trace::TraceLine| owed.iter().all(|(k, v)| l.get(k) == Some(v));
    let Some(line) = await_line(session, PROOF, mark, matches)? else {
        let last = session.trace()?.last(PROOF).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "no `{PROOF}` line with {owed:?} followed; last was {last:?}."
        )));
    };
    report.note(format!("proof: `{}`", line.raw));
    if await_line(session, RENDER, mark, |_| true)?.is_none() {
        return Ok(Some(format!(
            "`{}` was traced and no `{RENDER}` followed: the canvas kept the picture drawn before the simulation changed.",
            line.raw
        )));
    }
    Ok(None)
}

/// The overprint line `fixture` owes after the switch at `mark`.
fn overprint(
    report: &mut CheckReport,
    session: &Session,
    fixture: &str,
    effect: &str,
    mark: usize,
) -> Result<Option<String>> {
    let Some(line) = await_line(session, OVERPRINT, mark, |_| true)? else {
        return Ok(Some(format!(
            "on {fixture} the preview was switched on and no `{OVERPRINT}` line followed."
        )));
    };
    report.note(format!("{fixture}: `{}`", line.raw));
    let owed = [("set", "1"), ("effect", effect), ("whole", "1")];
    if let Some((key, want)) = owed.iter().find(|(k, v)| line.get(k) != Some(v)) {
        return Ok(Some(format!(
            "on {fixture} `{OVERPRINT}` owes {key}={want}; it traced `{}`.",
            line.raw
        )));
    }
    if declared_in(&session.trace()?, UI_RECT, OVERPRINT_LINE).is_none() {
        return Ok(Some(format!(
            "on {fixture} the overprint answer was traced but `{OVERPRINT_LINE}` was never drawn."
        )));
    }
    Ok(None)
}

/// See the module documentation.
pub struct OutputPreviewWarnsOfCoverage;

impl Check for OutputPreviewWarnsOfCoverage {
    fn name(&self) -> &'static str {
        "output_preview_coverage_warnings"
    }

    fn defect(&self) -> &'static str {
        "Output Preview's total area coverage or rich black warning flags nothing, flags a patch under its threshold, or is never painted over the page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let body = |r: &mut CheckReport, s: &Session, p: &ScriptedPointer, m: usize| {
            coverage(ctx, r, s, p, m)
        };
        match with_preview(ctx, &mut report, self.name(), FIXTURE, body) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The 400% patch, in canvas points (y down): the only area over 300%.
const FOUR_HUNDRED: [f64; 4] = [72.0, 142.0, 272.0, 292.0];
/// The 400% patch and the rich black patch beside it: the only rich blacks.
const RICH_BLACKS: [f64; 4] = [72.0, 142.0, 540.0, 292.0];
/// How far a flagged box may stand off a patch edge, canvas points.
const SLACK: f64 = 2.0;

/// Switch on total area coverage, then rich black; each owes a mask whose
/// flagged box is its patches and nothing else, and the overlay drawn.
fn coverage(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    _: usize,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press(session, pointer, TAC)?;
    if let Some(failure) = masked(report, session, mark, ("tac_box", FOUR_HUNDRED), "rich")? {
        return Ok(Some(failure));
    }
    // Total area coverage alone: green on the 400% patch, nowhere on the
    // rich black beside it.
    let owed = [(PATCH_400, TAC_GREEN, true), (PATCH_RICH, TAC_GREEN, false)];
    if let Some(failure) = painted(ctx, report, session, pointer, "tac", &owed)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(session, pointer, RICH)?;
    if let Some(failure) = masked(report, session, mark, ("rich_box", RICH_BLACKS), "")? {
        return Ok(Some(failure));
    }
    let both = |l: &crate::trace::TraceLine| {
        l.get("found_tac") == Some("1") && l.get("found_rich") == Some("1")
    };
    let Some(said) = await_line(session, WARN, mark, both)? else {
        let last = session.trace()?.last(WARN).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "both rules flagged a patch and the panel never said so; last `{WARN}` was {last:?}."
        )));
    };
    report.note(format!("panel: `{}`", said.raw));
    if declared_in(&session.trace()?, UI_RECT, OVERLAY).is_none() {
        return Ok(Some(format!(
            "masks were traced and `{OVERLAY}` was never drawn."
        )));
    }
    // Rich black painted over coverage: orange on both top patches, none on
    // plain black, which has no other ink in it.
    let owed = [
        (PATCH_400, RICH_ORANGE, true),
        (PATCH_RICH, RICH_ORANGE, true),
        (PATCH_K, RICH_ORANGE, false),
    ];
    if let Some(failure) = painted(ctx, report, session, pointer, "both", &owed)? {
        return Ok(Some(failure));
    }
    half_opaque(ctx, report, session, pointer)
}

/// Type 50 into the opacity field: the panel owes `opacity=50`, and the 400%
/// patch owes orange mixed with the page under it: no pixel the solid
/// overlay colour, and nearly all of them red over green over blue.
fn half_opaque(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press(session, pointer, OPACITY)?;
    pointer.key(session, None, "A", Some("ctrl"))?;
    pointer.type_text(session, None, "50")?;
    pointer.key(session, None, "Enter", None)?;
    if await_line(session, WARN, mark, |l| l.get("opacity") == Some("50"))?.is_none() {
        let last = session.trace()?.last(WARN).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "50 was typed into `{OPACITY}`; last `{WARN}` was {last:?}."
        )));
    }
    session.settle(10);
    let owed = [(PATCH_400, RICH_ORANGE, false)];
    if let Some(failure) = painted(ctx, report, session, pointer, "half", &owed)? {
        return Ok(Some(failure));
    }
    let path = ctx.out("output_preview_coverage.half.png");
    let image = Image::load_png(&path)?;
    let pixels: Vec<_> = image
        .pixels_in(patch_square(session, ctx, PATCH_400)?)
        .collect();
    let orange = |p: &&crate::image::Rgb| p.r > 80 && p.r < 240 && p.r > p.g && p.g > p.b;
    let mixed = pixels.iter().filter(orange).count() as f64 / pixels.len().max(1) as f64;
    report.note(format!(
        "half: (172,575) is {mixed:.3} orange over the page"
    ));
    if mixed < COVERED {
        return Ok(Some(format!(
            "at 50% opacity the 400% patch owes orange over the page; {mixed:.3} of it is. Screenshot: {}",
            path.display()
        )));
    }
    Ok(None)
}

/// The capture pixels of the square sampled at `centre`, PDF points.
fn patch_square(
    session: &Session,
    ctx: &CheckContext,
    (x, y): (f64, f64),
) -> Result<crate::geom::PixRect> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, x - HALF, y + HALF))?;
    let z = mapping.doc_to_window(DocPoint::new(0, x + HALF, y - HALF))?;
    let square = LRect::new(Pt::new(a.x(), a.y()), Pt::new(z.x(), z.y()));
    Ok(session.frame()?.logical_to_capture_pixels(square))
}

/// `output-preview.pdf`'s one page, US Letter.
const PAGE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
/// Patch centres on `output-preview.pdf`, PDF points (y up).
const PATCH_400: (f64, f64) = (172.0, 575.0);
const PATCH_RICH: (f64, f64) = (440.0, 575.0);
const PATCH_K: (f64, f64) = (440.0, 375.0);
/// The overlay colours `canvas::coverage` paints, opaque.
const TAC_GREEN: (u8, u8, u8) = (0, 220, 0);
const RICH_ORANGE: (u8, u8, u8) = (255, 140, 0);
/// Half the side of the square sampled at a patch centre, PDF points.
const HALF: f64 = 20.0;
/// The share of a sampled square that must be (or must not be) the colour.
const COVERED: f64 = 0.9;

/// A patch centre, PDF points; an overlay colour; whether it is owed there.
type Owed = ((f64, f64), (u8, u8, u8), bool);

/// Screenshot the window and require each `(centre, colour, on)`: the square
/// at `centre` is `colour` (at least [`COVERED`] of it) when `on`, and holds
/// none of it when not.
fn painted(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    tag: &str,
    owed: &[Owed],
) -> Result<Option<String>> {
    let path = ctx.out(&format!("output_preview_coverage.{tag}.png"));
    pointer.screenshot(session, &path)?;
    let image = Image::load_png(&path)?;
    for &((x, y), (r, g, b), on) in owed {
        let pixels: Vec<_> = image
            .pixels_in(patch_square(session, ctx, (x, y))?)
            .collect();
        let near = |p: &&crate::image::Rgb| {
            p.r.abs_diff(r) <= 12 && p.g.abs_diff(g) <= 12 && p.b.abs_diff(b) <= 12
        };
        let share = pixels.iter().filter(near).count() as f64 / pixels.len().max(1) as f64;
        report.note(format!(
            "{tag}: ({x},{y}) is {share:.3} of ({r},{g},{b}) over {} px",
            pixels.len()
        ));
        let held = if on { share >= COVERED } else { share == 0.0 };
        if pixels.is_empty() || !held {
            let want = if on { "painted" } else { "clear" };
            return Ok(Some(format!(
                "with `{tag}` on, the patch at PDF ({x},{y}) owes {want} of ({r},{g},{b}); {share:.3} of {} pixels are. Screenshot: {}",
                pixels.len(),
                path.display()
            )));
        }
    }
    Ok(None)
}

/// The `coverage-mask` line after `mark` whose `box` field is `want` within
/// [`SLACK`]; `off`, when named, must be `-` on it.
fn masked(
    report: &mut CheckReport,
    session: &Session,
    mark: usize,
    (key, want): (&str, [f64; 4]),
    off: &str,
) -> Result<Option<String>> {
    let Some(line) = await_line(session, MASK, mark, |l| {
        l.get(key).is_some_and(|v| v != "-")
    })?
    else {
        let last = session.trace()?.last(MASK).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "no `{MASK}` line flagged anything for `{key}`; last was {last:?}."
        )));
    };
    report.note(format!("mask: `{}`", line.raw));
    let got = line.get_f64_list(key).unwrap_or_default();
    let near = got.len() == 4 && got.iter().zip(want).all(|(g, w)| (g - w).abs() <= SLACK);
    if !near {
        return Ok(Some(format!(
            "`{key}` owes the patches {want:?} (canvas points, within {SLACK}); it traced `{}`.",
            line.raw
        )));
    }
    if !off.is_empty() && line.get(off) != Some("-") {
        return Ok(Some(format!(
            "`{off}` is switched off and was checked: `{}`.",
            line.raw
        )));
    }
    Ok(None)
}
