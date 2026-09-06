//! `a_new_markup_is_drawn_in_acrobats_red` — the colour an operator's first
//! comment shape actually comes out of the program, measured off the glass.
//!
//! # The defect
//!
//! `canvas::markup::palette` was written on 2026-09-06 from Acrobat DC's own
//! registry — `HKCU\…\DC\Annots\cAnnots\<subtype>\cstrokeColor`, read twice,
//! minutes apart, agreeing to six decimals. Shapes are **`#DB3425`**. The value
//! it replaced had been written from memory, and the same commit found the
//! highlighter's default is **orange `#FF6200`, not yellow** — *"what everybody
//! knows was wrong"*.
//!
//! A palette written from memory is a defect that ships silently: nothing
//! crashes, every test is green, and the only symptom is that a drawing marked
//! up in pdfcer does not look like a drawing marked up in Acrobat when the two
//! are put side by side — which is the comparison the operator's own request
//! ("*make sure you've used the same default colours and style look for these
//! things as Adobe*") is about.
//!
//! # ★★★ Why the sixteen unit tests beside it cannot see this
//!
//! `palette::tests` is thorough and it is thorough about **the table**:
//! `each_constant_is_the_registry_value_it_claims_to_be` divides the byte by 255
//! and compares against the float from the registry;
//! `every_shipped_default_is_one_click_away_in_the_grid` checks the swatch grid
//! contains every default; `pen::tests` asserts `PenSlot::Shape` maps to
//! `MARKUP_RED`. **Every one of them passes on a build where a drawn rectangle
//! comes out black**, because none of them is downstream of the drawing.
//!
//! Between `MARKUP_RED` and a red line on the glass there are six links, and no
//! test in the workspace observes more than one of them at a time:
//!
//! | # | link | why a unit test cannot see it |
//! |---|---|---|
//! | 1 | the Rectangle tool arms in Review | `Capabilities::for_mode` reads the real manifest; the mode is entered by a segment click or by `PDFCER_DIAG_INVOKE` |
//! | 2 | a drag becomes a `/Square` with the **pen's** colour | `canvas::markup::spec` builds the `MarkupSpec` from the live `Pen`; the drag's two corners come from the canvas transform, and nothing in-process performs one |
//! | 3 | `add_markup` writes `/C` | the engine's, and it is exercised in-process against a session that never paints |
//! | 4 | the appearance stream is **baked** with that colour | `/AP` generation is the engine's; nothing in the GUI's tests renders one |
//! | 5 | `pdfcer-render` paints the `/AP` onto the page raster | a separate crate, driven by the canvas |
//! | 6 | the composited window shows it | the compositor, the theme, and whatever the canvas draws over the top |
//!
//! ## ★ What this check measures is the DEFAULT, and that is a deliberate scope
//!
//! `canvas::markup::pen`'s header is explicit — the pen *"is deliberately **not
//! persisted** to the settings file"*, because a pen colour is a preference
//! rather than one of the ambiguities `pdfcer_core::settings` exists for, and
//! persisting it *"belongs with the ribbon layout and the keymap ... in their own
//! file"*, which is not built. So every launch starts from
//! `PenSlot::Shape`'s constant, and what this check measures is exactly what an
//! operator's **first** comment shape comes out — the reading that the operator's
//! own request is about.
//!
//! ⚠ A first draft of this header claimed the opposite, that the pen was loaded
//! from `userdata/` and that a private profile was therefore load-bearing here.
//! It was written from the shape of the neighbouring modules rather than from
//! `pen`'s own words, and it is corrected rather than quietly deleted because
//! **a premise nobody rechecks is how a check ends up asserting the wrong
//! thing**. The private profile ([`crate::sandbox`]) is still what this check
//! runs under and still matters — the stored **mode** reaches the ribbon, and a
//! check that inherited Edit would arm its tool in a different mode — but it is
//! not the pen that makes it matter.
//!
//! ⇒ A colour an operator has *changed* is a different subject, reached through
//! the Format ▸ Markup band's swatch, and its popup publishes no regions for a
//! harness to aim at. `markup_style`'s header records that limit for the same
//! control; it is named here rather than implied.
//!
//! # What it measures, and the baseline that makes it mean anything
//!
//! Three readings from two captures:
//!
//! | | before the drag | after it |
//! |---|---|---|
//! | **the edge strip** — a thin box lying along where the rectangle's top edge will be | must be **blank paper** | must hold ink, and that ink must be `#DB3425` |
//! | **the interior box** — well inside the shape | blank | still blank |
//!
//! ★★★ **The blank baseline is not politeness, it is what stops the check
//! passing for the wrong reason.** A "the ink here is red" assertion is
//! satisfied by any red thing: a red title block, a red revision cloud already
//! on the sheet, a red selection outline. This project has paid for exactly that
//! once — `markup_node_edit`'s first draft sampled a corner of `four-pages.pdf`
//! that carries a **coloured title block**, and its assertion passed on a delta
//! of 28 pixels against a floor of 423. So the strip is asserted **empty first**,
//! and a fixture whose own content lies under it makes this check SKIP — a
//! statement that it could not measure — rather than pass on the fixture's ink.
//!
//! The interior box is the differential: it says the shape is an **outline**,
//! not a filled blob. `canvas::markup::spec` authors every shape with
//! `interior: None` for a stated reason — *"a filled comment shape hides the
//! drawing it is a comment about, which on a CAD sheet is the whole content
//! under it"* — and that is a claim about the picture, so it is asserted from
//! the picture.
//!
//! # ★★★ How the stroke's colour is extracted — and why the obvious way fails
//!
//! **There is no core pixel to sample.** `a1-titleblock.pdf` is a 2384 × 1684 pt
//! A1 sheet displayed fit-page in an 1100 × 800 window, which is **20 % zoom**:
//! a 2 pt stroke is **0.36 px wide**. Every pixel it lays down is a blend of the
//! ink with the paper behind it, and no threshold, no percentile and no amount
//! of taking-the-darkest-quarter recovers a pure sample from a line thinner than
//! a pixel.
//!
//! ★ Measured on the first driven run of this check, 2026-09-06, with the
//! darkest quarter of the strip's ink averaged: **`#EB9D96`** — a pale pink, 105
//! away from `#DB3425` in blue. A tolerance wide enough to call that Acrobat's
//! red would be wide enough to call almost anything Acrobat's red. The capture is
//! in `evidence/`: the rectangle in it is plainly the right colour to a human eye
//! and plainly not `#DB3425` to a byte comparison.
//!
//! ## The measure that survives it: the direction away from the paper
//!
//! Compositing `α` of an ink over paper gives, per channel,
//!
//! ```text
//! c' = α·c + (1 − α)·paper
//! so   paper − c' = α·(paper − c)
//! ```
//!
//! ⇒ **The vector from the paper to the measured colour is the vector from the
//! paper to the ink, scaled by `α`.** Its *direction* does not depend on `α` at
//! all. So [`hue_from_paper`] normalises that difference by its largest
//! component and compares directions, and the dilution — the thing that defeats
//! every absolute comparison — divides out exactly.
//!
//! Worked, on that same first run:
//!
//! | | `paper − c`, per channel | normalised |
//! |---|---|---|
//! | Acrobat's `#DB3425` | 36, 203, 218 | **0.165, 0.931, 1.000** |
//! | measured `#EB9D96` | 20, 98, 105 | **0.190, 0.933, 1.000** |
//! | the highlighter's `#FF6200` | 0, 157, 255 | 0.000, 0.616, 1.000 |
//! | the old yellow `#FFFF00` | 0, 0, 255 | 0.000, 0.000, 1.000 |
//!
//! The measurement lands **0.025** from the right answer and **0.315** from the
//! nearest wrong one — a factor of twelve, on a reading taken through a
//! sub-pixel stroke. [`HUE_TOLERANCE`] sits between them, and
//! [`tests::the_tolerance_cannot_swallow_a_near_miss`] refuses to let it be
//! widened past any of them.
//!
//! ⚠ **This measures hue and not strength**, deliberately and with a cost: a
//! rectangle drawn in Acrobat's red at 10 % opacity passes. That is the right
//! trade here — `/CA` is a different control with a check of its own to be
//! written — but it is a limit rather than an oversight, so it is written down.
//! What it does still require is that the ink be *far enough* from the paper to
//! have a direction at all; see [`MIN_PAPER_GAP`].
//!
//! ## Why not [`crate::pixels::contrast_at`]
//!
//! That oracle answers *"is this legible"*: the **dominant** bucket is the
//! background and the bucket with the largest luminance gap is the foreground,
//! both quantised and then averaged over every pixel in the bucket. For a
//! sub-pixel stroke the largest-gap bucket is the least-diluted skirt averaged
//! with the rest of the skirt. It is the right tool for a caption on a plate and
//! the wrong one for *what colour is this line*.
//!
//! # Calibration
//!
//! ```text
//! --pdf fixtures/a1-titleblock.pdf --doc-point 0,300,500
//! ```
//!
//! ⚠ `--doc-point` is **0-based** and is not what this check aims with — it
//! places its own shape in page fractions, so it works on any single-page
//! fixture with blank paper in the middle third. The point is still required by
//! the harness's convention and is passed through to nothing here.
//!
//! The fixture must have **blank paper across the middle of page 1**. Both
//! `fixtures/a1-titleblock.pdf` (a real CAD sheet, blank inside the frame) and
//! `fixtures/four-pages.pdf` satisfy that; a fixture that does not makes this
//! check SKIP with the strip's ink count in the reason.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the fixture's own content lies under the strip or inside the shape, so
//!   neither reading could be attributed to the mark this check draws;
//! * the canvas is not showing page 1, so the page fractions describe something
//!   other than what is on screen;
//! * the rectangle tool authored nothing — that is `markup_move`'s subject and
//!   there is no mark here to have a colour.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::{Image, Rgb};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review mode, then arm the rectangle tool — both through the harness seam
/// rather than through ribbon clicks.
///
/// ★ Review because markup is authored there, and naming the mode makes the run
/// reproducible rather than dependent on whatever mode was last stored. With
/// [`crate::sandbox`] there is no stored mode to inherit, and the invoke is kept
/// anyway: a check should say which mode it drives rather than rely on the
/// absence of state.
const INVOKE: &str = "mode.review,markup.rectangle";

/// The line the canvas writes when a shape is authored.
const COMMIT_EVENT: &str = "markup-commit";

/// The line the apply arm writes when the engine has authored it.
const APPLY_EVENT: &str = "add-markup";

/// **Acrobat's markup red**, `#DB3425` — `canvas::markup::palette::MARKUP_RED`.
///
/// Spelled here as bytes rather than imported, deliberately and for the same
/// reason every constant in this crate is: the harness must be able to fail
/// against a binary built from a *different* source tree. An import would make
/// the check assert that the application agrees with itself.
const ACROBAT_RED: Rgb = Rgb {
    r: 219,
    g: 52,
    b: 37,
};

/// Two colours this shell has actually drawn shapes in, named so a failure can
/// say *which* wrong answer it got.
///
/// ★ The yellow is not hypothetical: `(1.0, 1.0, 0.0)` is what the highlighter
/// carried in this shell until 2026-09-06, *"written from memory"*. The orange
/// is Acrobat's real highlighter and is the near miss a careless fix would
/// produce — 46 apart from the red in the green channel, which is inside a
/// sloppy tolerance and outside this one.
const NEAR_MISSES: [(&str, Rgb); 3] = [
    (
        "Acrobat's HIGHLIGHTER orange #FF6200 — the right table, the wrong row",
        Rgb {
            r: 255,
            g: 98,
            b: 0,
        },
    ),
    (
        "the classic yellow #FFFF00 this shell used until 2026-09-06, written from memory",
        Rgb {
            r: 255,
            g: 255,
            b: 0,
        },
    ),
    (
        "black — the value a lost `/C` falls back to",
        Rgb { r: 0, g: 0, b: 0 },
    ),
];

/// How far the measured **direction away from the paper** may sit from
/// [`ACROBAT_RED`]'s, as a maximum absolute per-component difference on the
/// 0–1 normalised vector [`hue_from_paper`] returns.
///
/// # ★★ Where the number comes from
///
/// Measured, 2026-09-06, first driven run, `a1-titleblock.pdf` at 20 % zoom: the
/// strip read `#EB9D96`, whose normalised direction is `0.190, 0.933, 1.000`
/// against Acrobat's red at `0.165, 0.931, 1.000` — **0.025 apart**, through a
/// stroke thinner than a pixel.
///
/// The nearest wrong answer this shell could plausibly produce is the
/// highlighter's orange at **0.315**. `0.10` sits four times above the observed
/// noise and three times below the nearest miss, which is the same shape of
/// margin `markup_rectangle::MIN_PRESSED_DELTA` argues for and for the same
/// reason: a threshold derived from one measured pair moves every time anything
/// about the rendering changes, and one with a stated gap on both sides does
/// not.
///
/// ★ [`tests::the_tolerance_cannot_swallow_a_near_miss`] fails if this is ever
/// widened to admit anything in [`NEAR_MISSES`]. That is what stops a future
/// session making a red run green by moving a constant at four in the afternoon.
const HUE_TOLERANCE: f64 = 0.10;

/// The smallest `paper − ink` a channel may reach before the direction is
/// treated as unmeasurable.
///
/// ★ Normalising a vector near the origin amplifies noise without bound: two
/// pixels of capture noise on a nearly-white box would produce a confident
/// direction pointing anywhere. Below this the check reports that it could not
/// read a colour — a SKIP — rather than reporting a wrong one.
///
/// 40 of 255, against a measured 105 on the first run. Well under what a real
/// stroke produces at the worst zoom this suite drives, and well over anything a
/// blank box can.
const MIN_PAPER_GAP: f64 = 40.0;

/// A pixel counts as ink when the sum of its channels is at least this far below
/// the strip's paper, in the 0–765 space [`crate::pixels::ink_run_into`] uses.
///
/// The same 180 that oracle uses (`INK_CONTRAST` 60, times three channels), and
/// for its stated reason: it separates a stroke from a border column without
/// having to know what colour the theme is painting.
const INK_BELOW_PAPER: i32 = 180;

/// The fraction of the ink pixels, darkest first, that are averaged into the
/// answer.
///
/// A quarter. Enough pixels for the mean to be stable — the strip is a hundred
/// or more pixels long, so a quarter of its ink is tens of samples — and few
/// enough that the skirt does not dominate. See [`TOLERANCE`] on why even this
/// quarter is a composite at fit-page zoom.
const CORE_FRACTION: f64 = 0.25;

/// Where the shape is drawn, as fractions of the page: `((x0, y0), (x1, y1))` in
/// PDF user space, origin bottom-left.
///
/// ★ The middle third of the sheet, which on a CAD drawing is inside the frame
/// and clear of the title block. `markup_move` places its shape in the same
/// region for the same reason, and this check asserts the emptiness rather than
/// assuming it.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));

/// Half-height of the strip laid along the top edge, as a fraction of the page
/// **height**.
///
/// ★ A fraction of the page and not a constant in points, because
/// `markup_node_edit` measured what a points constant costs: 22 pt on its
/// fixture at fit-page zoom is an 8 × 9 pixel window, too few pixels for any
/// oracle to speak. This is roughly 1 % of the sheet, which is 8 px on an 800 px
/// window — enough rows to contain the stroke wherever antialiasing puts it, and
/// far too few to reach the shape's other edges.
const STRIP_HALF: f64 = 0.005;

/// How much of the top edge's length the strip covers, as a fraction of the
/// shape's width, centred.
///
/// Not the whole edge: the corners are where two strokes meet and where a join
/// puts twice the ink, which would bias a colour reading with no benefit.
const STRIP_SPAN: f64 = 0.6;

/// Where the pointer is parked before the capture, as fractions of the page.
///
/// Blank paper near a corner of the sheet: nowhere near either sampled box, and
/// carrying no annotation to hover. See the park's own note in [`drive`] for the
/// tooltip that made it necessary.
const PARK: (f64, f64) = (0.10, 0.90);

/// The interior box, as fractions of the **shape**, `((x0, y0), (x1, y1))`.
///
/// Middle half of the rectangle, so it is nowhere near the stroke on any side.
const INTERIOR: ((f64, f64), (f64, f64)) = ((0.25, 0.25), (0.75, 0.75));

/// The smallest ink count that counts as *something is drawn here*.
///
/// ★ Four, and the reasoning is `InkReport::is_text`'s: one or two pixels either
/// way is antialiasing on an edge that did not move. Used in both directions —
/// as the floor the strip must clear *after* the drag, and as the ceiling the
/// strip and the interior must stay under *before* it — because a baseline
/// asserted with a different threshold from the measurement is two claims about
/// two different things.
const INK_FLOOR: usize = 4;

/// See the module documentation.
pub struct ANewMarkupIsDrawnInAcrobatsRed;

impl Check for ANewMarkupIsDrawnInAcrobatsRed {
    fn name(&self) -> &'static str {
        "a_new_markup_is_drawn_in_acrobats_red"
    }

    fn defect(&self) -> &'static str {
        "a comment shape drawn in pdfcer comes out a colour Acrobat never uses — the palette was \
         written from memory rather than read, or the pen's colour never reaches the authored \
         `/C`, or it reaches it and the baked appearance paints something else. Every one of \
         those is invisible to the sixteen unit tests beside the palette, which assert the table \
         and never the drawing"
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

/// What [`core_ink`] found in one box.
#[derive(Clone, Copy, Debug)]
struct Core {
    /// The mean of the darkest [`CORE_FRACTION`] of the ink pixels.
    colour: Rgb,
    /// The box's own paper — the mean of its brightest quarter.
    ///
    /// ★ Measured per box rather than assumed to be white, because the paper a
    /// stroke is composited over is whatever the renderer put there: a sheet
    /// with a tint, a theme that dims the page, a canvas that draws a shadow. The
    /// direction the ink pulls away from the paper is only meaningful against
    /// **that** paper.
    paper: Rgb,
    /// How many pixels in the box counted as ink at all.
    ink: usize,
    /// How many pixels were looked at.
    sampled: usize,
}

impl Core {
    fn summary(&self) -> String {
        format!(
            "{} on paper {} from {} ink px of {} sampled",
            self.colour, self.paper, self.ink, self.sampled
        )
    }

    /// The normalised direction from this box's paper to its ink.
    fn hue(&self) -> Option<[f64; 3]> {
        hue_from_paper(self.colour, self.paper)
    }
}

/// The direction from `paper` to `ink`, normalised by its largest component.
///
/// See the module header: compositing scales that vector and does not turn it,
/// so this is the one property of a sub-pixel stroke's pixels that still names
/// the ink that made them.
///
/// `None` when no channel reaches [`MIN_PAPER_GAP`] — there is nothing here far
/// enough from the paper to have a direction, and normalising it would amplify
/// capture noise into a confident wrong answer.
fn hue_from_paper(ink: Rgb, paper: Rgb) -> Option<[f64; 3]> {
    let d = [
        f64::from(paper.r) - f64::from(ink.r),
        f64::from(paper.g) - f64::from(ink.g),
        f64::from(paper.b) - f64::from(ink.b),
    ];
    let max = d[0].max(d[1]).max(d[2]);
    if max < MIN_PAPER_GAP {
        return None;
    }
    Some([
        (d[0] / max).clamp(0.0, 1.0),
        (d[1] / max).clamp(0.0, 1.0),
        (d[2] / max).clamp(0.0, 1.0),
    ])
}

/// The direction a pure ink pulls away from white paper — what a swatch in the
/// palette table *would* measure if it could be sampled undiluted.
fn hue_of(ink: Rgb) -> [f64; 3] {
    hue_from_paper(ink, Rgb::new(255, 255, 255))
        .expect("every colour in this module's tables is far enough from white to have a direction")
}

/// Largest per-component difference between two directions.
fn hue_delta(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0])
        .abs()
        .max((a[1] - b[1]).abs())
        .max((a[2] - b[2]).abs())
}

/// A direction, for a report.
fn hue_text(h: [f64; 3]) -> String {
    format!("{:.3}, {:.3}, {:.3}", h[0], h[1], h[2])
}

/// The colour of whatever is drawn on the paper inside `region`.
///
/// The paper is the box's own **90th-percentile luminance** rather than its
/// dominant bucket: a box lying along a stroke can be a third ink, and a mode is
/// a fragile way to find the plate when the second population is that large. A
/// percentile is not.
///
/// Returns `None` for a box with no pixels, which means the region resolved
/// outside the captured window — a finding rather than a measurement, and the
/// caller reports it as one.
fn core_ink(img: &Image, region: PixRect) -> Option<Core> {
    let mut lumas: Vec<(i32, Rgb)> = img
        .pixels_in(region)
        .map(|p| (i32::from(p.r) + i32::from(p.g) + i32::from(p.b), p))
        .collect();
    if lumas.is_empty() {
        return None;
    }
    let sampled = lumas.len();
    lumas.sort_by_key(|(l, _)| *l);
    let paper = lumas[(sampled * 9 / 10).min(sampled - 1)].0;

    let ink: Vec<Rgb> = lumas
        .iter()
        .take_while(|(l, _)| paper - l >= INK_BELOW_PAPER)
        .map(|(_, p)| *p)
        .collect();
    // The paper: the mean of the brightest quarter, so a tint or a page shadow
    // is carried rather than assumed away.
    let bright = (sampled / 4).max(1);
    let (mut pr, mut pg, mut pb) = (0u64, 0u64, 0u64);
    for (_, px) in lumas.iter().rev().take(bright) {
        pr += u64::from(px.r);
        pg += u64::from(px.g);
        pb += u64::from(px.b);
    }
    let bn = bright as u64;
    let paper_colour = Rgb::new((pr / bn) as u8, (pg / bn) as u8, (pb / bn) as u8);

    if ink.is_empty() {
        return Some(Core {
            colour: paper_colour,
            paper: paper_colour,
            ink: 0,
            sampled,
        });
    }
    // Darkest first — `lumas` is sorted ascending and `take_while` preserved it.
    let core = (((ink.len() as f64) * CORE_FRACTION).ceil() as usize).max(1);
    let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
    for p in ink.iter().take(core) {
        r += u64::from(p.r);
        g += u64::from(p.g);
        b += u64::from(p.b);
    }
    let n = core as u64;
    Some(Core {
        colour: Rgb::new((r / n) as u8, (g / n) as u8, (b / n) as u8),
        paper: paper_colour,
        ink: ink.len(),
        sampled,
    })
}

/// The document-space box for the strip along the shape's top edge.
fn strip_box(page: PageGeometry) -> (DocPoint, DocPoint) {
    let (x0, x1) = (SHAPE.0.0 * page.width_pt, SHAPE.1.0 * page.width_pt);
    let mid = f64::midpoint(x0, x1);
    let half = (x1 - x0).abs() * STRIP_SPAN / 2.0;
    let y = SHAPE.1.1 * page.height_pt;
    let dy = STRIP_HALF * page.height_pt;
    (
        DocPoint::new(0, mid - half, y - dy),
        DocPoint::new(0, mid + half, y + dy),
    )
}

/// The document-space box well inside the shape.
fn interior_box(page: PageGeometry) -> (DocPoint, DocPoint) {
    let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
    let x = |t: f64| lerp(SHAPE.0.0, SHAPE.1.0, t) * page.width_pt;
    let y = |t: f64| lerp(SHAPE.0.1, SHAPE.1.1, t) * page.height_pt;
    (
        DocPoint::new(0, x(INTERIOR.0.0), y(INTERIOR.0.1)),
        DocPoint::new(0, x(INTERIOR.1.0), y(INTERIOR.1.1)),
    )
}

/// One capture, both boxes.
///
/// ★★ Both from the **same** capture, which is what makes the interior reading a
/// control on the strip reading rather than a second experiment: nothing that
/// happened between two frames can satisfy one and not the other.
fn read_both(
    session: &Session,
    mapping: &CanvasMapping,
    page: PageGeometry,
    out: &std::path::Path,
) -> Result<(Core, Core, std::path::PathBuf)> {
    let image = crate::capture::window_to_png(session, out)?;
    let frame = session.frame()?;
    let measure = |(lo, hi): (DocPoint, DocPoint)| -> Result<Core> {
        let a = mapping.doc_to_window(lo)?;
        let b = mapping.doc_to_window(hi)?;
        let rect = LRect::new(
            Pt::new(a.x().min(b.x()), a.y().min(b.y())),
            Pt::new(a.x().max(b.x()), a.y().max(b.y())),
        );
        core_ink(&image, frame.logical_to_capture_pixels(rect)).ok_or_else(|| {
            Error::new(format!(
                "the sampled box resolved to no pixels ({rect:?}), so it lies outside the \
                 window's client area. The shape is placed in page fractions; either the canvas \
                 is scrolled away from it or the window is smaller than this check assumes."
            ))
        })
    };
    let strip = measure(strip_box(page))?;
    let interior = measure(interior_box(page))?;
    Ok((strip, interior, out.to_path_buf()))
}

/// Which known-wrong colour the measurement is nearest to, if any is nearer than
/// [`ACROBAT_RED`].
///
/// Used only to sharpen a failure message. *"Measured `#FF6200`"* sends its
/// reader to a hex table; *"measured the HIGHLIGHTER's orange"* sends them to
/// `pen::PenSlot`, which is where the fix is.
fn nearest_miss(measured: [f64; 3]) -> Option<&'static str> {
    let mine = hue_delta(measured, hue_of(ACROBAT_RED));
    NEAR_MISSES
        .iter()
        .map(|(name, colour)| (hue_delta(measured, hue_of(*colour)), *name))
        .filter(|(d, _)| *d < mine)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, name)| name)
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape with a real drag across a \
             real canvas and then photographs it. Reported as SKIPPED rather than passed: a \
             check that did not run has learned nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a page with blank paper across its middle third to draw \
             a shape on and photograph.",
        )
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "could not read a page size from {}, and this check places its shape in page \
                 fractions. Pass --page-size.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("markup-palette.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // ★ The mapping is re-derived after every settle in this check rather than
    // cached, for `text_selection::aim`'s stated reason: the same `DocPoint` is
    // a different screen pixel in different modes and at different scroll
    // positions, and a cached mapping is a stale coordinate — which this project
    // has already filed and retracted one defect over.
    let trace = session.trace()?;
    let shown = trace
        .last(ctx.profile.vocab.canvas_event)
        .and_then(|l| l.get_usize("page"));
    if shown != Some(0) {
        return Err(Error::new(format!(
            "the canvas is showing page {shown:?}, not page 1. This check's boxes are page-1 \
             fractions, and converting them against another page's rect would sample somewhere \
             plausible and wrong."
        )));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;

    // --- the baseline: the paper this check is about to draw on -------------
    let (blank_strip, blank_interior, before_shot) = read_both(
        &session,
        &mapping,
        page,
        &ctx.out("markup_palette_before.png"),
    )?;
    report.artifact(before_shot.clone());
    report.note(format!(
        "before the drag: edge strip {} · interior {}",
        blank_strip.summary(),
        blank_interior.summary()
    ));
    if blank_strip.ink >= INK_FLOOR || blank_interior.ink >= INK_FLOOR {
        return Err(Error::new(format!(
            "★ THIS FIXTURE HAS ITS OWN CONTENT WHERE THE CHECK SAMPLES, so a colour read after \
             the drag could not be attributed to the mark this check draws — it could be the \
             sheet's. Edge strip {} and interior {}, against a ceiling of {INK_FLOOR} ink px \
             each. SKIPPED rather than failed: that is a fact about {} and not about the \
             program. Point --pdf at a fixture with blank paper across the middle third of page \
             1. Capture: {}.",
            blank_strip.summary(),
            blank_interior.summary(),
            pdf.display(),
            before_shot.display()
        )));
    }

    // --- draw the rectangle -------------------------------------------------
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    let from = aim(ctx, &session, page, corner(SHAPE.0))?;
    let to = aim(ctx, &session, page, corner(SHAPE.1))?;
    driver.drag(from, to)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).last() else {
        return Ok(Some(format!(
            "THE RECTANGLE TOOL AUTHORED NOTHING: a drag across the page produced no \
             `{COMMIT_EVENT}` line, so `markup.rectangle` did not arm or the drag was not seen \
             as one. That is `dragging_a_markup_moves_it`'s subject; there is no mark here to \
             have a colour. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if trace.events(APPLY_EVENT).count() == 0 {
        return Ok(Some(format!(
            "THE ENGINE NEVER AUTHORED THE SHAPE: the canvas decided to — `{}` — and no \
             `{APPLY_EVENT}` followed. A refused `vector_edit` traces `add-markup-refused`; look \
             for that first. Trace: {}.",
            commit.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★ a rectangle was authored: `{}`", commit.raw));

    // --- and photograph it --------------------------------------------------
    //
    // ★ Nothing is selected: the rectangle tool authors and does not select, so
    // there is no selection outline and no resize grip anywhere in either box.
    // A check that photographed a selected shape would be measuring the
    // **shell's** accent colour along the same edge, which is drawn over the
    // annotation and would satisfy an "ink is here" reading with a colour that
    // has nothing to do with the palette.
    //
    // ★★★ **PARK THE POINTER FIRST, and the first run of this check is why.**
    //
    // A drag ends with the pointer on the shape's own corner, and hovering a
    // markup pops the note tooltip — *"No note has been written on this
    // markup."* — a rounded dark panel that landed **inside the interior box**
    // and read as 14 ink pixels of near-black. The check duly reported
    // *"THE SHAPE IS FILLED, NOT OUTLINED"* about a build that had drawn a
    // perfectly ordinary outline, and the capture in `evidence/` shows the
    // tooltip sitting exactly where the reading was taken.
    //
    // ⇒ **A driven check photographs the pointer as well as the program.** That
    // is `markup_rectangle::PARK`'s rule for a hovered ribbon control, one layer
    // out: there it is a hover *fill*, here it is a whole floating panel. The
    // park goes to blank paper in a corner of the sheet, which is nowhere near
    // either box and carries no annotation to hover.
    let parked = aim(ctx, &session, page, corner(PARK))?;
    driver.move_to(parked)?;
    session.settle(24);

    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let (strip, interior, after_shot) = read_both(
        &session,
        &mapping,
        page,
        &ctx.out("markup_palette_after.png"),
    )?;
    report.artifact(after_shot.clone());
    report.note(format!(
        "after the drag: edge strip {} · interior {}",
        strip.summary(),
        interior.summary()
    ));

    if strip.ink < INK_FLOOR {
        return Ok(Some(format!(
            "★★★ THE SHAPE REACHED THE DOCUMENT AND NOT THE PAGE. `{APPLY_EVENT}` is in the \
             trace, so the engine authored a `/Square` — and the strip lying along its top edge \
             holds {} ink pixels of {} sampled, against a floor of {INK_FLOOR}. So nothing was \
             painted where the mark is. Three candidates, in the order to check them: the `/AP` \
             stream was never baked; the page raster was not invalidated (`vector_edit`'s \
             `page_epochs` bump); or the `/Rect` is not where the drag said. Captures: {} and \
             {}.",
            strip.ink,
            strip.sampled,
            before_shot.display(),
            after_shot.display()
        )));
    }

    // ★★ The control. A filled shape would satisfy every colour assertion below
    // and would be a different program from the one `canvas::markup::spec`
    // describes — *"a filled comment shape hides the drawing it is a comment
    // about"*. Asserted here, from the same capture the colour comes from.
    if interior.ink >= INK_FLOOR {
        return Ok(Some(format!(
            "★★ THE SHAPE IS FILLED, NOT OUTLINED: the box well inside it holds {}, against a \
             ceiling of {INK_FLOOR} ink px, and it was blank before the drag. \
             `canvas::markup::spec` authors every shape with `interior: None` — *\"a filled \
             comment shape hides the drawing it is a comment about, which on a CAD sheet is the \
             whole content under it\"* — and Acrobat's default is the same. Capture: {}.",
            interior.summary(),
            after_shot.display()
        )));
    }

    let Some(measured) = strip.hue() else {
        return Err(Error::new(format!(
            "the strip holds ink but nothing in it is {MIN_PAPER_GAP:.0} of 255 away from the \
             paper in any channel ({}), so there is no direction to read and normalising it \
             would turn capture noise into a confident wrong answer. SKIPPED rather than \
             failed. Capture: {}.",
            strip.summary(),
            after_shot.display()
        )));
    };
    let expected = hue_of(ACROBAT_RED);
    let off_by = hue_delta(measured, expected);
    if off_by > HUE_TOLERANCE {
        let named = nearest_miss(measured)
            .map_or_else(String::new, |name| format!(" It is nearest to {name}."));
        return Ok(Some(format!(
            "★★★ A NEW COMMENT SHAPE IS NOT ACROBAT'S RED. The stroke reads {} on paper {}, \
             which pulls away from that paper in the direction [{}] where Acrobat's #DB3425 \
             would pull [{}] — {off_by:.3} apart, against a tolerance of {HUE_TOLERANCE}.{named}\n\
             ★ The comparison is of DIRECTION, not of colour, so a diluted stroke does not \
             cause this: at this fixture's zoom the line is thinner than a pixel and a correct \
             build still lands within 0.03. Acrobat DC's own registry holds `0.858826, \
             0.203918, 0.145096` for `cSquare` under `HKCU\\…\\DC\\Annots\\cAnnots`, read \
             twice on 2026-09-06 and agreeing to six decimals; `ACROBAT_DEFAULTS.md` carries the \
             table and the command to re-run it. Four places to look, in order: \
             `canvas::markup::palette::MARKUP_RED`; `canvas::markup::pen`'s `PenSlot::Shape` \
             mapping; `canvas::markup::spec`, which is what turns a live pen into the \
             `MarkupSpec` the engine authors; and whether the baked `/AP` carries `/C` at \
             all. Captures: {} and {}.",
            strip.colour,
            strip.paper,
            hue_text(measured),
            hue_text(expected),
            before_shot.display(),
            after_shot.display()
        )));
    }
    report.note(format!(
        "★★★ a freshly drawn rectangle paints in Acrobat's red: the stroke reads {} on paper \
         {}, a direction of [{}] against #DB3425's [{}], {off_by:.3} apart (tolerance \
         {HUE_TOLERANCE}) — and the box inside it is still blank paper, so the shape is an \
         outline and not a fill",
        strip.colour,
        strip.paper,
        hue_text(measured),
        hue_text(expected),
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An [`Image`] from a row-major list of colours.
    ///
    /// `Image` is BGRA because that is what the Windows capture hands over, so
    /// a test that wants to state its subject in colours has to lay the bytes
    /// out in that order. Written here rather than in `image` because it exists
    /// for these three tests and a constructor on the type would be a second,
    /// unused way to build one.
    fn image_of(w: u32, h: u32, px: &[Rgb]) -> Image {
        let mut bgra = Vec::with_capacity(px.len() * 4);
        for p in px {
            bgra.extend_from_slice(&[p.b, p.g, p.r, 255]);
        }
        Image::from_bgra(w, h, bgra).expect("a well-formed buffer")
    }

    /// **The check must be able to fail.**
    ///
    /// ★★★ A tolerance wide enough to admit the highlighter's orange would make
    /// every assertion in this file decoration — and widening a constant is
    /// exactly what a future session does when a run goes red at four in the
    /// afternoon. This test is what makes that widening cost something: it goes
    /// red the moment [`TOLERANCE`] reaches the nearest wrong answer.
    #[test]
    fn the_tolerance_cannot_swallow_a_near_miss() {
        let red = hue_of(ACROBAT_RED);
        for (name, wrong) in NEAR_MISSES {
            let gap = hue_delta(red, hue_of(wrong));
            assert!(
                gap > HUE_TOLERANCE,
                "HUE_TOLERANCE {HUE_TOLERANCE} would accept {name} ({gap:.3} away from Acrobat's \
                 red in direction), so the check could not fail on it"
            );
        }
    }

    /// ★★★ **The measure is unaffected by dilution** — the property the whole
    /// oracle rests on, asserted directly.
    ///
    /// Acrobat's red composited over white at every strength from 10 % to 100 %
    /// must read as the same direction. If this ever fails, the check has
    /// silently become sensitive to zoom, and a run at a different fit-page
    /// scale would start reporting a palette defect that is not there.
    #[test]
    fn a_diluted_stroke_still_names_its_ink() {
        let red = hue_of(ACROBAT_RED);
        for step in 1..=10 {
            let alpha = f64::from(step) / 10.0;
            let mix = |c: u8| (f64::from(c).mul_add(alpha, 255.0 * (1.0 - alpha))).round() as u8;
            let diluted = Rgb::new(mix(ACROBAT_RED.r), mix(ACROBAT_RED.g), mix(ACROBAT_RED.b));
            let Some(hue) = hue_from_paper(diluted, Rgb::new(255, 255, 255)) else {
                assert!(
                    alpha < 0.25,
                    "a stroke at {alpha:.1} strength should still have a readable direction"
                );
                continue;
            };
            let gap = hue_delta(hue, red);
            assert!(
                gap <= HUE_TOLERANCE,
                "at {alpha:.1} strength the direction moved {gap:.3}, past the tolerance"
            );
        }
    }

    /// And a diluted WRONG colour is still wrong.
    ///
    /// ★ The pair with the test above is what makes either mean anything: a
    /// measure invariant to dilution is worthless if it is invariant to
    /// everything. The highlighter's orange at 20 % strength — a pale peach,
    /// closer to Acrobat's red in raw bytes than the red is to itself undiluted
    /// — must still be refused.
    #[test]
    fn a_diluted_wrong_colour_is_still_refused() {
        let red = hue_of(ACROBAT_RED);
        let orange = NEAR_MISSES[0].1;
        let alpha = 0.2;
        let mix = |c: u8| (f64::from(c).mul_add(alpha, 255.0 * (1.0 - alpha))).round() as u8;
        let pale = Rgb::new(mix(orange.r), mix(orange.g), mix(orange.b));
        let hue = hue_from_paper(pale, Rgb::new(255, 255, 255)).expect("a direction");
        assert!(
            hue_delta(hue, red) > HUE_TOLERANCE,
            "a pale orange must not pass as Acrobat's red"
        );
    }

    /// The core reading takes the ink and not the paper.
    ///
    /// A synthetic strip: mostly white, a few pure-red pixels, and a band of
    /// half-diluted red between them. The answer must be the red, not the mean
    /// of the strip — which would be nearly white — and not the dilution.
    #[test]
    fn the_core_reading_finds_the_stroke_and_not_the_paper() {
        let (w, h) = (40u32, 10u32);
        let mut px = vec![Rgb::new(255, 255, 255); (w * h) as usize];
        for x in 0..w {
            // one core row and one diluted row, the rest paper
            px[(4 * w + x) as usize] = ACROBAT_RED;
            px[(5 * w + x) as usize] = Rgb::new(237, 153, 146);
        }
        let img = image_of(w, h, &px);
        let core = core_ink(&img, PixRect::new(0, 0, w, h)).expect("a reading");
        let hue = core.hue().expect("a direction");
        assert!(
            hue_delta(hue, hue_of(ACROBAT_RED)) <= HUE_TOLERANCE,
            "expected the stroke's direction, measured {} ({})",
            core.colour,
            hue_text(hue)
        );
        assert_eq!(
            core.paper,
            Rgb::new(255, 255, 255),
            "the plate is the paper"
        );
    }

    /// Blank paper reads as no ink, which is what the baseline gate depends on.
    #[test]
    fn blank_paper_reads_as_no_ink() {
        let (w, h) = (20u32, 8u32);
        let img = image_of(w, h, &vec![Rgb::new(255, 255, 255); (w * h) as usize]);
        let core = core_ink(&img, PixRect::new(0, 0, w, h)).expect("a reading");
        assert_eq!(core.ink, 0, "paper is not ink");
    }

    /// A wrong colour is named rather than merely reported as a hex triple.
    #[test]
    fn a_yellow_stroke_is_named_as_the_colour_it_is() {
        let hue = hue_from_paper(Rgb::new(250, 250, 10), Rgb::new(255, 255, 255)).expect("a hue");
        let named = nearest_miss(hue).expect("a name");
        assert!(named.contains("yellow"), "got `{named}`");
    }
}
