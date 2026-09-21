//! `the_raster_wall_stops_the_zoom_instead_of_painting_an_error` — **O186**,
//! the two halves that can be driven.
//!
//! # The report
//!
//!
//! > *"At deeper zooms I am still experiencing the cursor jumping, and at some
//! > point it sometimes repositions to where the object area I was zooming into
//! > is no longer on screen. […] I think this sometimes results in similar
//! > error to 'This page could not be drawn. requested raster size
//! > 50411508x32619210 is empty or exceeds MAX_PIXMAP_EDGE'. perhaps the zoom
//! > is fine, but the cursor has jumped somewhere unsuported. If this error is
//! > caused by some other limitation that will always happen, zoom should stop
//! > at the limit and not end up showing an error - the canvas will just stop
//! > zooming in and can still function. the error can still be shown on the
//! > bottom bar so the user has some idea as to why zooming stopped short of 1
//! > trillion percent."*
//!
//! One paragraph, and it contains two findings that turned out to have nothing
//! to do with each other:
//!
//!
//! ★★★ **The sheet that could not be drawn was never the sheet he was looking
//! at.** `50411508 × 32619210` is `1224 × 792` pt at scale `41185.87`, and
//! `SW41177.pdf` holds thirty-four sheets at `1584 × 1224` against exactly two
//! at `1224 × 792`. He was right that *"perhaps the zoom is fine"*: the zoom
//! was fine, the acting page rendered perfectly through the region tier, and
//! the error was a **neighbour** in the continuous strip.
//!
//! # ★★ Why part A asserts on the REFUSAL and not on the sentence he saw
//!
//! This is the single most important property of this check, and getting it
//! wrong would have produced a check that passes on a build with the defect
//! still in it.
//!
//! `render::settle::absorb`'s `absorb_render` now has a `BeyondRaster` arm that learns
//! a raster ceiling from the refusal and **returns early** — without setting
//! `render_error` and without clearing the page texture. That arm is part of
//! O186's own fix (clause two), and it is upstream of the sentence. So on a
//! build where `fill_strip` still places the impossible order:
//!
//! * the engine still refuses it — `bad-raster-size px=… page=…` still appears;
//! * a ceiling is still learned from a **neighbour's** geometry, which silently
//!   caps the zoom of a page that could have gone much further;
//! * and **no `canvas-message` is published at all**, because the refusal was
//!   absorbed.
//!
//! A check asserting *"the operator is never shown the MAX_PIXMAP_EDGE
//! sentence"* would therefore pass on the broken build, and would have been
//! read as proof that the defect was fixed. The assertion is made one layer
//! down, at the engine's refusal, where the condition is unambiguous — and the
//! refusal line carries `page=`, which is the whole finding: it names the sheet
//! and the sheet is not the one being looked at.
//!
//! ★ Generalised, and worth carrying away: **a net that swallows a symptom
//! invalidates every check that asserts on that symptom.** When a fix adds a
//! handler upstream of a user-visible complaint, the regression test has to
//! move upstream with it.
//!
//! # ★ The seam trick — how part A reaches the state at all
//!
//! Part A needs a state that sounds awkward to reach: a page **visible** at the
//! same time as the acting page, at a zoom high enough that the visible one
//! cannot be rastered whole. Two facts make it reachable, and a third — added
//! after the check turned out flaky — decides how wide the window is.
//!
//! 1. **Ctrl+wheel is zoom-to-cursor.** The document point under the pointer is
//!    held where it is, give or take the drift measured below. So a pointer
//!    parked in the gap *between* two pages keeps **both** of those pages either
//!    side of it as the climb goes up, rather than scrolling one away.
//! 2. **`viewer::strip::ROW_GAP` is 12 points at zoom 1 and is scaled by the
//!    zoom**, like the rest of the strip's geometry. The gap therefore grows
//!    with the zoom, and the two pages separate at a known rate: aiming at the
//!    gap's midpoint puts each page's edge `6 × zoom` points from the pointer.
//! 3. **And that is also what closes the window.** The neighbour's top edge
//!    descends towards the bottom of the canvas as the gap grows, so there is a
//!    zoom above which it is off screen and the state cannot be measured at all.
//!
//! # ★★★ The window is NARROW, and getting that wrong made the check flaky
//!
//! This section used to read *"With `fixtures/four-pages.pdf` the window is
//! wide … both are far below the zoom at which the growing gap pushes the
//! neighbour off screen."* That was reasoned from the design, not measured, and
//! it is **false**. Two runs on 2026-09-12, minutes apart, one passed and one
//! SKIPPED with *"the strip never reported a visible page it could not order"* —
//! and the only difference was where a plain wheel notch happened to leave the
//! seam.
//!
//! The **lower** bound of part A's window is the engine's.  [`FIXTURE`]'s pages
//! are `2383.937 × 1683.78`, `612 × 792`, `612 × 792` and `306 × 396`, and
//! `render::strategy::whole_page_raster_fits` refuses a whole-page raster once
//! `longest × raster_scale` passes `MAX_PIXMAP_EDGE - 1`. A letter-size
//! neighbour therefore becomes unorderable at a raster scale of about **20.7**
//! (`16384 / 792`), and the E-size sheet at about 6.9.
//!
//! The **upper** bound is the one that was missed. It is not *"half the canvas"*
//! — it is the room between the parked seam and the bottom of the canvas, and
//! the seam is parked *below* the middle for a separate and unrelated reason
//! ([`SEAM_BAND`]). From the failing run's own trace:
//!
//! ```text
//! canvas-viewport   [288 174] - [1258 944]   770 pt tall
//! seam parked at    y = 700                  room below it: 244 pt
//! neighbour last counted visible at zoom 17.9, gone by 18.9
//! neighbour unorderable from zoom 20.7
//! ```
//!
//! The window was **empty**: the neighbour left the screen two notches before it
//! became unorderable, so the state part A exists to measure was unreachable and
//! the run reported an absence it had never been in a position to observe. The
//! repair is in two constants — [`VIEWPORT`] is now tall enough that half a
//! canvas is 575 points rather than 385, and [`SEAM_BAND`] is narrow and sits
//! just below the middle — plus a **pre-climb feasibility check** that SKIPs with
//! the arithmetic printed when the window is empty, instead of climbing ninety
//! notches and calling the result an absence.
//!
//! ★ Two details the measurement turned up, both of which the naive arithmetic
//! gets wrong:
//!
//! * the strip stops counting a page as visible about **50 points above** the
//!   bottom of the published canvas rect, not at it ([`BOTTOM_DEAD_BAND_PT`]);
//! * the pointer does **not** hold its document point exactly. Over one climb the
//!   parked seam slid down the screen at about 4.7 points per unit of zoom, so
//!   the neighbour's top edge approaches the bottom of the canvas at about
//!   `10.7 × zoom` rather than `12 × zoom`. The feasibility check divides by 12,
//!   which makes its prediction land *early* — the conservative direction for a
//!   gate whose job is to refuse to measure.
//!
//! **None of this arithmetic is used for a verdict.** The state is still
//! *detected* from the application's own `strip-beyond-raster pages=` line. What
//! it is used for is aim, budget, and the decision not to bother.
//!
//! # ★★ Why `strip-beyond-raster pages=0` is printed, and why that matters here
//!
//! `fill_strip` traces the count of visible-but-unorderable pages **before** it
//! scans for something to order, and prints `pages=0` deliberately. Its own
//! comment says why: *"or the check cannot tell 'never entered it' from 'still
//! in it'"*. This check is the caller that needed it. The whole of part A's
//! subject is an absence — no order, no refusal, no sentence — and an absence
//! measured in a state the run never entered is not a measurement. `pages=` is
//! what lets the run prove it stood where the defect fires.
//!
//! # ⚠ Why the beyond-raster state is a precondition and NOT a gate on the
//! failure
//!
//! Order matters here and it is not obvious. On a build with the defect, the
//! neighbour's refusal *teaches a ceiling* and the zoom is pulled back — so the
//! climb stalls at roughly the boundary and `strip-beyond-raster pages=` may
//! never reach 1 at all. A check that treated `pages >= 1` as a gate would then
//! SKIP on exactly the build it exists to catch.
//!
//! So the refusal is looked for **first**, on every notch, over the whole climb.
//! Only when the climb finishes with no refusal does `pages >= 1` decide
//! between *"the state was entered and nothing went wrong"* (pass) and *"the
//! state was never entered"* (skip).
//!
//! # What part B asserts, and the one thing it only NOTES
//!
//! Part B leaves the strip, enters Single, and climbs until the rasterizer's
//! own content-dependent wall is met — `RasterizerLimit`, which is a different
//! refusal from the pixmap-edge one and arrives at a scale that depends on how
//! much ink the page holds. `render::settle::absorb`'s `learn_raster_ceiling` turns it
//! into a learned ceiling and pulls the zoom back, tracing
//! `raster-ceiling-learned … moved=true`. From that point the operator's fourth
//! clause is the specification, and it is checked literally: the zoom does not
//! rise past the learned ceiling, no error sentence is painted anywhere, and
//! the status bar's `status-group:raster-stop` region **is** on screen, not
//! clipped, inside the window.
//!
//! ★★★ **Part B now DOES assert that the page is still drawn at saturation**,
//! and the history of that sentence is worth a paragraph because it is an
//! argument about when an exemption expires.
//!
//!
//!
//! ⚠ The failure message names O186 and points at `geometry::pasteboard`, so a
//! reader who finds *only* this check red is not sent hunting in the rasterizer
//! for a defect that lives in the layout.
//!
//! # What a passing run does NOT prove
//!
//! That the learned ceiling is the *highest* scale the page could have reached
//! (that is the rasterizer's property, not the shell's, and it varies 28× with
//! ink — an E-size sheet was measured giving out at 284,964 where a business
//! card reached 8,053,069). That the neighbour is ever drawn again on the way
//! back down. That anything holds on a facing-continuous layout, which lays out
//! two pages per row and is not exercised here.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, clipped_away, declared, declared_names, declared_since,
    list,
};
use crate::checks::page_display_recentres::enter_display;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::sys::vk;
use crate::trace::Trace;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The document, pinned rather than taken from `--pdf`.
///
/// Part A needs **more than one page**, pages of **different sizes** (so a
/// neighbour reaches its pixmap ceiling at a different zoom from the acting
/// page, which is what the operator's own set did), and a **known page count**
/// so the seam search can refuse to park after the last sheet. The operator's
/// drawings are frequently single-sheet, and a single-sheet document cannot be
/// in the state part A is about at all.
const FIXTURE: &str = "fixtures/four-pages.pdf";

/// The document the second check drives, and it is **not in the repository**.
///
/// # ★★★ Why a repository fixture cannot serve — measured, not assumed
///
///
/// That is not a defect, it is the region tier working. Above
/// `viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT` the canvas asks for the VISIBLE
/// REGION rather than the whole page, and the visible region shrinks as the zoom
/// rises — the last line of that trace asked for a region about 2×10⁻⁷ pt
/// across. The raster therefore stays viewport-sized for ever and there is no
/// zoom at which a pixmap ceiling can bind on it.
///
/// ★★ **So the refusal is a property of INK, not of size or of zoom.** What
/// gives out is the rasterizer's capacity to draw the content inside the
/// requested region at that scale, and a test document of four empty sheets has
/// no content to give out on. The operator met it on a 36-sheet SOLIDWORKS
/// drawing set; this check meets it on a 5.7 MB dense vector site plan, which is
/// the same kind of document and is already on the machine as the project's
/// standing render benchmark.
///
/// ⚠ Absent, this check is **SKIPPED and says which file is missing** — never
/// passed, and never failed. Committing a multi-megabyte CAD drawing to
/// `fixtures/` was considered and rejected: that directory is for documents
/// whose specific content is the point, and this one is wanted for the sheer
/// quantity of it.
const FIXTURE_DENSE: &str = "D:/Dev/pdfTests/ncored-benchmark-cad-drawing.pdf";

/// How many pages [`FIXTURE`] has.
///
/// Used only to refuse to look for a seam *after the last page*, where there is
/// no neighbour to be unorderable. A wrong value here makes the run SKIP, not
/// pass — the seam band is still asserted.
const PAGE_COUNT: usize = 4;

/// The mode whose ribbon carries the View tab.
const MODE: &str = "review";

/// Where and how large the window is placed, as `PDFCER_DIAG_VIEWPORT` takes
/// it: `x,y,w,h`.
///
/// Fixed rather than maximised so the seam band below means the same thing on
/// every machine.
///
///
/// The height is not cosmetic — it is the whole of part A's measuring window.
/// The run parks the pointer in the gap between two pages and climbs; the gap
/// is `12 pt × zoom`, so the room between the seam and the bottom of the canvas
/// is what decides the zoom at which the lower page leaves the screen:
///
/// ```text
/// neighbour leaves the screen at about   zoom = room_below / 12
/// neighbour stops being orderable at     zoom = 20.7   (16384 / 792 pt)
/// ```
///
/// At the original `1600,1000` the canvas measured `[288 174] - [1258 944]` —
/// 770 points tall — and a seam parked at the top of a generous band left only
/// **244 points** below it. The neighbour therefore left the screen at zoom
/// 20.3, *just under* the 20.7 at which it becomes unorderable, so the state
/// part A measures was unreachable and the run SKIPPED with "the strip never
/// reported a page it could not order". One run before it had squeezed inside
/// the same window and passed. ★★ **A check whose window is bounded above and
/// below by two different mechanisms is flaky until the arithmetic is done**,
/// and a check that skips half the time is a check that has stopped running
/// without anyone noticing.
///
/// 1380 is chosen against this machine's working area (3440 × 1392) and leaves
/// the canvas about 1150 points tall, so half of it is 575 and the neighbour
/// survives to zoom 47 — comfortably past 20.7 with the first notch above the
/// boundary landing near 25. The chrome above and below the canvas measured 230
/// points in total, which is the constant in `canvas_height = height - 230`.
const VIEWPORT: &str = "0,0,1600,1380";

/// The canvas viewport. Every aim point in this check is expressed as a
/// fraction of it, per the crate's coordinate contract.
const CANVAS_REGION: &str = "canvas-viewport";

/// The status-bar region that says **why zooming stopped** — O186 clause four.
const STATUS_REGION: &str = "status-group:raster-stop";

/// The region an error sentence drawn over the page occupies.
///
/// ★ It has exactly two publishers — `canvas::present`'s no-pages arm and its
/// **Single-mode** `render_error` arm — and the `nothing-visible` arm publishes
/// **no** region at all. So part B's dead end cannot be mistaken for the
/// painted error, which is the one confusion that would have made this region a
/// useless oracle.
const MESSAGE_REGION: &str = "canvas-message";

/// The canvas's "I drew nothing" verdict. Shares its trace slot with
/// [`CANVAS_EVENT`], so which of the two stands is decided by line number.
const UNAVAILABLE_EVENT: &str = "canvas-unavailable";

/// How many visible pages the strip found it could not order. Printed even when
/// zero — see the module header.
const BEYOND_EVENT: &str = "strip-beyond-raster";

/// ★★ `render::settle`'s record of whether the CURRENT page's raster order
/// could be filled at all — O186's third route, added 2026-09-12.
///
/// `fillable=false` means the frame declined to place an order because the page
/// had no region and its whole sheet is past the pixmap ceiling. Read here for
/// two different jobs: it is the transition pair that proves the new guard is
/// live, and when part A's window turns out to be empty it distinguishes "the
/// neighbour left the screen" from "the acting page itself went unorderable".
const UNFILLABLE_EVENT: &str = "current-order-unfillable";
/// One strip raster actually ordered, with the page it was ordered for.
const REQUESTED_EVENT: &str = "strip-raster-requested";

/// One finished render, with `outcome={done|cancelled|failed}`. Carries **no**
/// page, which is why it is the secondary oracle and not the primary one.
const RENDER_EVENT: &str = "render-async-done";

/// ★★★ The operator's error, at its source, **carrying the page index**. The
/// primary oracle of part A.
const BAD_RASTER_EVENT: &str = "bad-raster-size";

/// The rasterizer's own content-dependent wall, as distinct from the pixmap
/// edge. Read for the report's notes: it says which refusal taught the ceiling.
const RASTER_LIMIT_EVENT: &str = "raster-limit";

/// The shell's record of turning a refusal into a zoom ceiling. Part B's
/// trigger.
const LEARNED_EVENT: &str = "raster-ceiling-learned";

/// `viewer::strip::ROW_GAP`, in points at zoom 1.
///
/// # ★ Why a copy of another module's constant is tolerable here, and how a
/// drift would show up
///
/// This is used for **aim**, never for a verdict. The pointer is parked
/// `ROW_GAP_PT / 2 × zoom` below the acting page's bottom edge, i.e. at the
/// midpoint of the gap, because that is the position at which the two pages
/// separate most slowly as the zoom rises.
///
/// If the application's gap grew, this aim would land *nearer the upper page*
/// but still inside the gap. If it shrank below this value, the aim would land
/// a few points onto the **lower** page — which is still a point that holds
/// both pages either side of it, because zoom-to-cursor is linear in the strip.
/// Either way the run still works, and the two things that could actually
/// invalidate it are asserted rather than assumed: the seam must land inside
/// [`SEAM_BAND`] of the canvas, and the canvas must report `visible >= 2`.
const ROW_GAP_PT: f32 = 12.0;

/// Where in the canvas the seam must sit before the climb starts, as fractions
/// of the canvas height.
///
/// # ★★★ Just BELOW the middle, and the asymmetry is the whole reason
///
/// This was `(0.30, 0.70)` — centred and generous — on the argument that *"the
/// further it is from the middle the sooner one of the two pages reaches an
/// edge"*. That argument treats the two pages as interchangeable and they are
/// not. Measured 2026-09-12 against [`FIXTURE`]:
///
/// * The page **above** the seam is the E-size sheet, and by the time any of
///   this matters its top edge is tens of thousands of points off the top of the
///   canvas. It is visible for as long as the seam itself is, so the room above
///   the seam never binds.
/// * The page **below** the seam is letter-size and its top edge sits `6 pt ×
///   zoom` under the pointer. It leaves the screen when that exceeds the room
///   below the seam — and *that* is the bound a band reaching to 0.70 destroyed:
///   a seam parked at 0.68 of a 770-point canvas left 244 points, and the window
///   closed at zoom 18.9, below the 20.7 the measurement needs.
///
/// So the room below wants to be as large as possible, which argues for a seam
/// near the **top**. Against that stands one thing: `viewer::strip::page_at_view`
/// makes the page with the **greatest visible area** the current one, and part
/// A's whole subject is a NEIGHBOUR being ordered while the **acting** page is
/// fine. Both pages are wider than the canvas long before any of this, so their
/// visible areas are proportional to their visible heights, and the E-size sheet
/// stays current only while the seam is below the canvas mid-line.
///
/// ```text
/// seam fraction > 0.50    ->  the E-size sheet stays the acting page
/// seam fraction < ~0.60   ->  the room below lets the gap grow past zoom 31
/// ```
///
/// Hence a narrow band hugging the mid-line from below. Against the 1150-point
/// canvas [`VIEWPORT`] now gives, a seam at 0.51 leaves 513 points below it and
/// the window closes at about zoom 43; at 0.60 it leaves 410 and closes at about
/// 34. Both clear [`NEIGHBOUR_UNORDERABLE_ZOOM`] times [`WINDOW_MARGIN`], and at
/// 0.51 the acting page's area still leads by about 4 %, which is decisive for an
/// argmax.
///
/// ⚠ The alternative — park **above** the mid-line, let the letter page become
/// current and the E-size sheet be the unorderable neighbour (it gives out at
/// raster scale 6.9, three times lower, which would widen the window a lot) —
/// was considered and rejected. It inverts the fixture's roles, so the run would
/// no longer reproduce the operator's situation: a large CAD sheet being zoomed,
/// a small neighbour painting an error across it. Worse, the current page would
/// change mid-climb, which puts the neighbour into `acting` and quietly makes the
/// exact-mechanism clause below vacuous.
const SEAM_BAND: (f32, f32) = (0.51, 0.60);

/// How far above the bottom of the published canvas rect the strip stops
/// counting a page as visible, in window-logical points. **Measured, not
/// derived.**
///
/// On the failing run the neighbour was last counted at `visible=2` with its top
/// edge at y = 889 and was gone by y = 900, against a canvas whose published
/// bottom was 944. Whatever accounts for the band — a scroll bar, a clip inset,
/// the strip's own culling margin — the arithmetic that predicts when part A's
/// window closes has to allow for it, or it predicts a window that is about two
/// notches wider than the one that exists.
///
/// ★ Used only for the feasibility prediction, never for a verdict. An exact
/// value is not needed and is not claimed; what is needed is that the prediction
/// errs on the early side.
const BOTTOM_DEAD_BAND_PT: f32 = 50.0;

/// The zoom at which a letter-size neighbour stops being orderable as a whole
/// page: `MAX_PIXMAP_EDGE / 792 pt`, with `MAX_PIXMAP_EDGE` measured at 16,384.
///
/// ★ Used **only** to decide, before the climb, whether the window part A needs
/// exists at all — never as a verdict. The state itself is still detected from
/// the application's own `strip-beyond-raster pages=` line. Same discipline as
/// [`ROW_GAP_PT`]: a copy of someone else's number is tolerable for aim and for
/// feasibility and is not tolerable in an assertion.
///
/// If the engine's ceiling moved, this number would make the run SKIP with the
/// arithmetic printed — which is the failure mode wanted, a stated infeasibility
/// rather than a silent absence.
const NEIGHBOUR_UNORDERABLE_ZOOM: f32 = 20.7;

/// How much room beyond [`NEIGHBOUR_UNORDERABLE_ZOOM`] the window must have
/// before the climb is worth starting, as a multiple of that zoom.
///
/// A Ctrl+wheel notch multiplies the zoom by about 1.22, and the climb can only
/// observe the state on a notch boundary, so the first notch above 20.7 can land
/// as high as 25.9. 1.5 covers that with room to spare and still leaves the whole
/// of [`SEAM_BAND`] feasible.
const WINDOW_MARGIN: f32 = 1.5;

/// How many plain wheel notches the seam search may spend.
///
/// It moves one notch at a time and re-reads the canvas each time, because the
/// notch distance is egui's and the document's opening zoom is the
/// application's — neither is this check's business to know.
const SEAM_SEARCH_NOTCHES: usize = 60;

/// How close to the canvas's left and right edges the aim point may sit.
///
/// The acting page's horizontal centre is used when it is on screen, and
/// clamped into the canvas when it is not. A point on the very edge risks
/// `Driver::confirm_uncovered` finding a scroll bar, which is a harness failure
/// dressed as an application one.
const EDGE_MARGIN_PT: f32 = 24.0;

/// How many Ctrl+wheel notches part A may spend climbing.
///
/// At roughly 1.22× a notch this is about eleven orders of magnitude of zoom —
/// vastly more than the twenty or so needed to put a letter-size neighbour past
/// its pixmap ceiling. Overshooting is cheap and under-shooting would SKIP, so
/// the budget is set generously and the run breaks as soon as the state is
/// reached.
const CLIMB_NOTCHES_A: usize = 90;

/// How many Ctrl+wheel notches part B may spend reaching the rasterizer's wall.
///
/// The wall is content-dependent — the same build gave out at raster scale
/// 284,964 on an E-size sheet and 8,053,069 on a business card — so this cannot
/// be derived, only budgeted. If it is not met, part B is NOTED as unmeasured
/// with the zoom it reached, never quietly dropped.
const CLIMB_NOTCHES_B: usize = 160;

/// How many notches part B sends per round trip through the OS.
///
/// Batched, unlike part A's single notches, because part B has no state to
/// detect *during* the climb other than its end, and a round trip per notch
/// over a budget of 160 is most of a minute of wall clock for nothing.
const CLIMB_BATCH_B: usize = 4;

/// How many further notches part B pushes **after** the ceiling is learned.
///
/// This is the operator's *"the canvas will just stop zooming in"*: the test is
/// not that the clamp happened once, it is that it holds against continued
/// pressure.
const EXTRA_NOTCHES_B: usize = 12;

/// How far above the learned ceiling the final zoom may sit, as a fraction.
///
/// The ceiling is converted scale → zoom through a division by the display
/// density and back again, so an exact comparison would be asserting `f32`
/// rounding. A thousandth is four orders of magnitude below a single wheel
/// notch, so it cannot mask a zoom that kept climbing.
const CEILING_SLACK: f32 = 1.0e-3;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// [`FIXTURE_DENSE`] as a path. Absolute and outside the repository, so
/// unlike [`fixture_path`] there is nothing to resolve it against.
fn dense_path() -> PathBuf {
    PathBuf::from(FIXTURE_DENSE)
}

/// O186's first half — the neighbour sheet. See the module documentation.
pub struct TheStripNeverOrdersARasterItCannotFill;

impl Check for TheStripNeverOrdersARasterItCannotFill {
    fn name(&self) -> &'static str {
        "the_strip_never_orders_a_raster_it_cannot_fill"
    }

    fn defect(&self) -> &'static str {
        "the continuous strip orders a whole-page raster for a VISIBLE NEIGHBOUR at the CURRENT \
         page's deep raster scale, which above the pixmap ceiling is an order that cannot be \
         filled — so the operator reads `requested raster size 50411508x32619210 … exceeds \
         MAX_PIXMAP_EDGE` about a sheet he was not looking at"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_a(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// O186's fourth clause — the wall, and the sentence on the bottom bar.
///
/// # ★★★ Why this is a SEPARATE check, decided by driving rather than by taste
///
///
/// The first half needs several pages of differing sizes in one strip. This half
/// needs the rasterizer to REFUSE, and on a repository fixture it never does —
/// the measured run climbed to a zoom of ten billion, a trillion percent, with
/// zero refusals of any kind, because above `SUB_PIXEL_CONTENT_EXTENT` the
/// region tier asks only for the visible region and the visible region SHRINKS
/// as the zoom rises. There is no page size at which that order becomes too
/// large. What makes the rasterizer give out is the amount of INK inside the
/// region, which is why the operator met it on a 36-sheet SOLIDWORKS drawing
/// set and this check meets it on a dense CAD site plan. See [`FIXTURE_DENSE`].
///
/// Keeping them separate buys the thing the project keeps relearning: one check
/// is red for one reason. A merged check would have been SKIPPED on every
/// machine without the operator's drawings, taking the neighbour-sheet half —
/// which is measurable anywhere — down with it.
pub struct TheRasterWallStopsTheZoomInsteadOfPaintingAnError;

impl Check for TheRasterWallStopsTheZoomInsteadOfPaintingAnError {
    fn name(&self) -> &'static str {
        "the_raster_wall_stops_the_zoom_instead_of_painting_an_error"
    }

    fn defect(&self) -> &'static str {
        "when the rasterizer refuses a scale, the zoom must STOP there and the bottom bar must \
         say why, rather than leaving `This page could not be drawn` across the drawing — the \
         operator's *\"zoom should stop at the limit and not end up showing an error\"*"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_b(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

// ---------------------------------------------------------------------------
// Reading the trace, and getting onto the seam
// ---------------------------------------------------------------------------
//
//
// What stays in THIS file is the part whose content is its order: the two
// `part_*` sequences, the two `drive_*` wrappers, and the constants, which stay
// here because they are this check's parameters and the arithmetic that chose
// them belongs next to the `Check` impls that are judged by it.

mod park;
mod trace;

use park::{park_on_the_seam, seam_y, window_closes_at};
use trace::{
    Learned, beyond_after, failed_renders_after, first_bad_raster_after, last_learned_after,
    last_two_visible_zoom, latest_canvas, requested_pages_after, unavailable_now,
    unfillable_counts, went_blank_between,
};

/// ★★★ Part A. The operator's raster error, at its source.
///
/// Returns `Some(failure)` when the refusal happened, `None` when the state was
/// entered and nothing went wrong. A state that was never entered is an `Err`,
/// which the report turns into a SKIP.
fn part_a(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    canvas: LRect,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    enter_display(
        session,
        driver,
        ui_rect,
        "ribbon.item.view.page_continuous",
        "continuous",
    )?;
    let (seam, opening) = park_on_the_seam(session, driver, canvas, report)?;

    // Every page the canvas reported as ACTING during the climb. The "only the
    // operator's own sheet was ordered" clause is stated against this set
    // rather than against one page index, because the current-page tracker is
    // entitled to hand over as the view changes and a request for the page that
    // was acting two notches ago is not a defect.
    let mut acting: BTreeSet<usize> = BTreeSet::new();
    acting.insert(opening.page);

    let mark = session.trace()?.mark();
    let mut notches = 0usize;
    let mut beyond = 0usize;
    let mut wall_mark = mark;
    // The parked seam in window-logical points, kept because two reports below
    // need it after `opening` has been moved into `state`: the feasibility
    // arithmetic is about where the seam IS, not about where the canvas was
    // last drawn.
    let seam_logical = seam_y(&opening);
    let mut state = opening;
    while notches < CLIMB_NOTCHES_A {
        driver.scroll_at_held(seam, &[vk::CONTROL], 1, 1)?;
        session.settle(12);
        notches += 1;
        let trace = session.trace()?;

        // ★ The refusal is looked for FIRST, before the precondition, and on
        // every notch. See the module header: on a build with the defect the
        // refusal teaches a ceiling and stalls the climb, so a run that gated
        // on the precondition would SKIP on exactly the build it is for.
        if let Some((page, line, at)) = first_bad_raster_after(&trace, mark) {
            // ★★★ THREE ROUTES TO ONE SENTENCE, and the report has to say which.
            //
            //
            // So the attribution is made from the trace rather than assumed: the
            // blank frame is looked for FIRST, because it is upstream of both of
            // the others.
            let blank = went_blank_between(&trace, mark, at);
            let cause = if let Some(blank_at) = blank {
                format!(
                    "the canvas had ALREADY gone blank {} line(s) earlier — \
                     `{UNAVAILABLE_EVENT} reason=nothing-visible` — so this is O186's THIRD \
                     route and not the neighbour one. With no visible part of the page, \
                     `canvas::tier::decide`'s region tier has nothing to intersect and leaves \
                     `OpenDoc::raster_region` as `None`; a request with no region is a request \
                     for the WHOLE SHEET, which at this scale is the refusal above. \
                     `render::settle::settle_and_rasterize` must ask \
                     `OpenDoc::raster_order_fillable` before it spawns. ★★ And the damage is not \
                     the refusal: `absorb_render` learns a zoom CEILING from it, so a page that \
                     renders through the region tier at ten billion percent gets capped at a \
                     number that was an internal mistake. Note that the blank frame itself is a \
                     DIFFERENT defect again — O186, and ★ NOT `canvas::deep`'s anchor as this \
                     sentence said until 2026-09-13: the blank was measured at the SHALLOW tier, \
                     two orders of magnitude below the `f64` hand-over, and its cause is a \
                     pasteboard of exactly one viewport in `canvas::geometry::pasteboard`. Fixing \
                     this one does not fix that",
                    at.saturating_sub(blank_at)
                )
            } else {
                match page {
                    Some(p) if acting.contains(&p) => format!(
                        "it names page {p}, which IS a page the canvas was acting on, and the \
                         canvas never reported itself blank — so this is neither the neighbour \
                         defect nor the blank-frame one, but a failure of the acting page's own \
                         tier selection, which `render::strategy::for_page` exists to prevent"
                    ),
                    Some(p) => format!(
                        "it names page {p}, which the canvas was NEVER acting on (it acted on \
                         {acting:?}) — the operator's own finding, and the defect this check was \
                         written for: the sheet that could not be drawn is not the sheet he was \
                         looking at. `render::settle::fill_strip` must ask \
                         `OpenDoc::strip_page_orderable` before it places a strip order, because \
                         a strip page is handed `region: None` by construction and above the \
                         pixmap ceiling there is nothing to ask for"
                    ),
                    None => "the refusal did not name a page at all, which is itself a defect in \
                             the trace: `bad-raster-size` is the only line that can attribute one \
                             of these and O186 turned on exactly that attribution"
                        .to_owned(),
                }
            };
            return Ok(Some(format!(
                "the engine refused a raster at notch {notches} of the climb. The refusal, \
                 verbatim: `{line}`. This is the engine saying `requested raster size … is empty \
                 or exceeds MAX_PIXMAP_EDGE`, which is `OPERATOR_REQUESTS.md` O186. {cause}. Note \
                 that no `{MESSAGE_REGION}` is expected on any of these routes — `absorb_render` \
                 absorbs the refusal to learn a ceiling from it — which is exactly why this check \
                 asserts at the refusal and not at the sentence the operator saw."
            )));
        }

        if let Some(now) = latest_canvas(&trace) {
            acting.insert(now.page);
            state = now;
        }
        if let Some(n) = beyond_after(&trace, mark)
            && n >= 1
        {
            beyond = n;
            wall_mark = trace.mark();
            break;
        }
    }

    report.note(format!(
        "part A climbed {notches} Ctrl+wheel notches on the seam to zoom {:.1} ({:.0} %), acting \
         pages {acting:?}, visible {} of which {} had a raster; strip pages it could not order: \
         {beyond}",
        state.zoom,
        state.zoom * 100.0,
        state.visible,
        state.drawn
    ));

    if beyond == 0 {
        let trace = session.trace()?;
        let closes = window_closes_at(canvas, seam_logical);
        let lost = last_two_visible_zoom(&trace, mark);
        let (declined, placed) = unfillable_counts(&trace, mark);
        return Err(Error::new(format!(
            "the climb reached zoom {:.1} ({:.0} %) over {notches} notches and the strip never \
             reported a visible page it could not order (`{BEYOND_EVENT} pages=` stayed at 0), so \
             the state O186's first half lives in was NEVER ENTERED and its absence has not been \
             measured. SKIPPED rather than passed: an absence measured outside the state it is \
             about is not a measurement. ★ The cause is far more likely to be the WINDOW than the \
             application. The row gap grows with the zoom, so from this seam the neighbour is \
             expected off screen by about zoom {closes:.0}, while it only becomes unorderable at \
             about {NEIGHBOUR_UNORDERABLE_ZOOM:.1}; the canvas last drew two pages at zoom \
             {lost:?}. `VIEWPORT` and `SEAM_BAND` set that window between them and both carry the \
             arithmetic. ★★ For the record the climb also saw {declined} decline(s) and {placed} \
             placement(s) of the ACTING page's own raster order (`{UNFILLABLE_EVENT}`, transitions \
             and not frames). A decline there is O186's THIRD route, a different finding from this \
             one, and it would mean the page being looked at had itself gone unorderable.",
            state.zoom,
            state.zoom * 100.0
        )));
    }

    // The state IS entered. Everything below is a claim about it.
    if state.display != "continuous" {
        return Err(Error::new(format!(
            "the strip reported {beyond} unorderable visible page(s), but by then the canvas \
             said its display was `{}` and not `continuous`. `visible >= 2` below would be \
             counting the two halves of a SPREAD rather than a page and its neighbour down the \
             strip, which is a different state from the one O186 is about. SKIPPED.",
            state.display
        )));
    }
    if state.visible < 2 {
        return Err(Error::new(format!(
            "the strip says {beyond} visible page(s) cannot be ordered, but the canvas says it \
             drew only {} page this frame — so the two disagree about what is on screen and the \
             run cannot say which it measured. SKIPPED.",
            state.visible
        )));
    }
    if state.drawn < 1 {
        return Ok(Some(format!(
            "at zoom {:.1} ({:.0} %) the canvas drew {} pages and NONE of them had a raster. The \
             neighbour being unorderable is correct and expected; the operator's OWN sheet going \
             blank is not. `render::settle::fill_strip`'s orderability filter must exclude only \
             pages that are not the current one — `page != current` — or it skips the one page \
             the region tier could have drawn.",
            state.zoom,
            state.zoom * 100.0,
            state.visible
        )));
    }

    let trace = session.trace()?;
    let failed = failed_renders_after(&trace, mark);
    if !failed.is_empty() {
        return Ok(Some(format!(
            "{} render(s) FAILED during part A's climb, the first being `{}`. \
             `{RENDER_EVENT}` carries no page index, so this says less about the cause than \
             `{BAD_RASTER_EVENT}` would — but a failure at a zoom where one page is drawable and \
             another is not is the shape of O186's first half, and the secondary oracle is \
             asserted because the primary one can be bypassed by a refusal kind that never \
             reaches it.",
            failed.len(),
            failed[0]
        )));
    }

    // ★ The exact-mechanism clause. When the strip can order every visible page
    // but one, the only page it may order is the one the canvas was acting on.
    let requested = requested_pages_after(&trace, wall_mark);
    if requested.is_empty() {
        report.note(format!(
            "no strip raster was ordered at all after the wall was reached, so the \
             'only the acting page may be ordered' clause had nothing to measure — a strip \
             raster is ordered once per page per render key and the acting page's was already \
             cached. The decisive assertion is the absence of `{BAD_RASTER_EVENT}` above, which \
             IS non-vacuous: the run proved it stood in the state (`{BEYOND_EVENT} pages={beyond}`, \
             {} pages drawn) where the defect fires.",
            state.visible
        ));
    } else if let Some(stranger) = requested.iter().find(|p| !acting.contains(p)).copied() {
        return Ok(Some(format!(
            "after the strip reported {beyond} visible page(s) it could not order, it went on to \
             order a raster for page {stranger}, which the canvas was never acting on (it acted \
             on {acting:?}; it ordered {requested:?}). That order cannot be filled — \
             `OpenDoc::region_for` refuses a region for any page but the current one, so a strip \
             page is a whole-page order or nothing — and an unfillable order is precisely how the \
             `MAX_PIXMAP_EDGE` sentence reached the operator's screen for a sheet he was not \
             looking at."
        )));
    } else {
        report.note(format!(
            "after the wall, the strip ordered rasters only for {requested:?}, all of which the \
             canvas had been acting on"
        ));
    }

    let learned = last_learned_after(&trace, mark);
    if let Some(l) = learned {
        return Ok(Some(format!(
            "part A's climb taught the shell a raster ceiling of {:.1} from page {:?}, pulling \
             the zoom back to {:.2}. Nothing in part A should teach a ceiling: the acting page \
             has the region tier to fall back on, and a NEIGHBOUR's geometry must never cap the \
             zoom of the page being looked at. This is the quiet half of O186's first finding — \
             the fixture's pages differ 6× in size, so a ceiling learned from the wrong one \
             stops the zoom far short of where the operator's sheet could have gone.",
            l.scale, l.page, l.to
        )));
    }

    let limits = trace.events(RASTER_LIMIT_EVENT).count();
    if limits > 0 {
        report.note(format!(
            "{limits} `{RASTER_LIMIT_EVENT}` line(s) during part A — the rasterizer's \
             content-dependent wall, not the pixmap edge. Not a failure here: it is absorbed into \
             a ceiling and is part B's subject."
        ));
    }
    Ok(None)
}

/// Require that every thread panic in a capture was **converted** into the
/// engine's own refusal, rather than merely tolerated.
///
/// [`Session::expect_thread_panic`] silences the harness's panic detector for
/// the whole session, and on its own that is an assertion both outcomes satisfy:
/// a rasterizer that gave out and was caught and one that simply died look the
/// same afterwards. `pdfcer-render` catches the panic and hands back a
/// `RasterizerLimit`, which the canvas publishes as a `raster-limit` line
/// carrying the panic text in its `panic=` field — so the conversion has a
/// witness, and a check that declares a panic owes the reader that witness.
///
/// Measured 1:1 across three runs on 2026-09-15: one `raster-limit` per panic in
/// each of the two zoom climbs, two of each in this check's own part B.
///
/// Returns the failure sentence, or `None` when every panic was converted.
pub(crate) fn panic_was_converted(session: &Session) -> Result<Option<String>> {
    Ok(unconverted_panics(&session.trace()?))
}

/// [`panic_was_converted`]'s judgement, over a trace rather than a session, so
/// that both of its outcomes can be produced from a string.
fn unconverted_panics(trace: &Trace) -> Option<String> {
    let panics = trace
        .other
        .iter()
        .filter(|l| l.contains("panicked at"))
        .count();
    if panics == 0 {
        return None;
    }
    let converted = trace
        .events(RASTER_LIMIT_EVENT)
        .filter(|l| l.get("panic").is_some())
        .count();
    if converted >= panics {
        return None;
    }
    Some(format!(
        "{panics} thread panic(s) in the capture but only {converted} \
         `{RASTER_LIMIT_EVENT}` line(s) carrying a `panic=` field. This check drives past \
         the raster ceiling on purpose and declares the panic, and the engine is supposed \
         to CATCH it and hand back a refusal. A panic with no refusal behind it is a \
         worker that simply died, which is a defect nobody has filed."
    ))
}

/// ★★ Part B. The operator's fourth clause: the zoom stops, and the bottom bar
/// says why.
///
/// Returns `Some(failure)` for a breach of that clause. An unreachable wall is
/// `Ok(None)` **with a note** — never an `Err`, because part A has already
/// measured something real and turning the whole check into a SKIP would throw
/// that away.
fn part_b(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    canvas: LRect,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    enter_display(
        session,
        driver,
        ui_rect,
        "ribbon.item.view.page_single",
        "single",
    )?;

    // ★ The canvas centre, not the page-rect centre. At part A's exit zoom the
    // page is far larger than the window, so its rect's midpoint is a
    // window-logical coordinate a long way outside the window — and
    // `declared_at` does not clamp, deliberately. Aiming there would send a
    // Ctrl+wheel at whatever owns that pixel, which is the operator's desktop.
    let centre = session.frame()?.declared_at(canvas, 0.5, 0.5);
    let mark = session.trace()?.mark();

    let mut notches = 0usize;
    let mut learned: Option<Learned> = None;
    while notches < CLIMB_NOTCHES_B {
        driver.scroll_at_held(centre, &[vk::CONTROL], 1, CLIMB_BATCH_B)?;
        session.settle(24);
        notches += CLIMB_BATCH_B;
        let trace = session.trace()?;
        if let Some((reason, _)) = unavailable_now(&trace) {
            report.note(format!(
                "part B NOT MEASURED: at notch {notches} the canvas stopped drawing anything \
                 (`{UNAVAILABLE_EVENT} reason={reason}`). That is O186 STAGE 1 — the cursor jump \
                 — and it is a terminal state, because both wheel handlers in `canvas::present` \
                 sit below the early return that produces it. It is not this check's subject and \
                 is deliberately NOT reported as a failure here; the owner is the \
                 `deep_anchor` clamp in `canvas::deep`. The operator's fourth clause has \
                 therefore not been measured on this run."
            ));
            return Ok(None);
        }
        if let Some(l) = last_learned_after(&trace, mark) {
            learned = Some(l);
            break;
        }
    }

    let Some(first) = learned else {
        let zoom = latest_canvas(&session.trace()?).map_or(f32::NAN, |s| s.zoom);
        report.note(format!(
            "part B NOT MEASURED: {notches} Ctrl+wheel notches reached zoom {zoom:.0} ({:.0} %) \
             and the rasterizer never refused, so no ceiling was learned and the operator's \
             fourth clause — stop, and say why on the bottom bar — had nothing to fire on. The \
             wall is content-dependent (an E-size sheet was measured giving out at raster scale \
             284,964 where a business card reached 8,053,069), so it cannot be derived, only \
             budgeted. Raise `CLIMB_NOTCHES_B` if this note becomes the usual outcome.",
            zoom * 100.0
        ));
        return Ok(None);
    };
    report.note(format!(
        "part B learned a raster ceiling of {:.1} from page {:?} after {notches} notches, \
         pulling the zoom back to {:.2} ({:.0} %)",
        first.scale,
        first.page,
        first.to,
        first.to * 100.0
    ));

    // The operator's *"the canvas will just stop zooming in"*: the clamp has to
    // hold against continued pressure, not merely happen once.
    driver.scroll_at_held(centre, &[vk::CONTROL], 1, EXTRA_NOTCHES_B)?;
    session.settle(40);
    let trace = session.trace()?;

    if let Some((reason, _)) = unavailable_now(&trace) {
        report.note(format!(
            "part B's clamp was learned but the {EXTRA_NOTCHES_B} further notches ended with the \
             canvas drawing nothing (`{UNAVAILABLE_EVENT} reason={reason}`) — O186 stage 1 again, \
             noted rather than failed here. The clamp assertions below are not made."
        ));
        return Ok(None);
    }
    let Some(state) = latest_canvas(&trace) else {
        return Err(Error::new(
            "the canvas published no drawn frame after part B's clamp, and no reason for drawing \
             nothing either. SKIPPED: neither half of the fourth clause can be read."
                .to_owned(),
        ));
    };
    // The ceiling may be re-learned tighter on a later attempt, so the bound is
    // the LAST one the shell taught itself, not the first.
    let ceiling = last_learned_after(&trace, mark).unwrap_or(first);
    if state.zoom > ceiling.to * (1.0 + CEILING_SLACK) {
        return Ok(Some(format!(
            "the shell learned a raster ceiling and pulled the zoom back to {:.2}, and then \
             {EXTRA_NOTCHES_B} further Ctrl+wheel notches pushed it to {:.2} anyway ({:.0} % \
             versus {:.0} %). The operator's words are *\"the canvas will just stop zooming \
             in\"*: a clamp that is re-applied after each refusal rather than held as a ceiling \
             means every further notch costs another failed render, and the limit is not a limit. \
             `viewer::ceiling::zoom_ceiling`'s learned clause is the one place this is decided.",
            ceiling.to,
            state.zoom,
            state.zoom * 100.0,
            ceiling.to * 100.0
        )));
    }
    if state.zoom < 1.0 {
        return Ok(Some(format!(
            "the learned ceiling pulled the zoom back to {:.2} ({:.0} %), below 100 %. A limit \
             the operator meets while zooming IN must not throw him further out than he started; \
             `zoom_ceiling` floors its learned clause at `MIN_ZOOM` for exactly this, and a \
             result below 1.0 on a page this size means the floor is not being applied.",
            state.zoom,
            state.zoom * 100.0
        )));
    }

    // ★ Anchored at `mark` — the start of part B — rather than at the learning.
    // No refusal on this path should EVER paint a sentence: `BadRasterSize` and
    // `RasterizerLimit` both map to `RefusalKind::BeyondRaster`, which
    // `absorb_render` absorbs. So zero messages across the whole of part B is
    // the claim, and it is stronger than "none since the clamp" — which a
    // change-log channel could satisfy with a sentence that was already
    // standing.
    if let Some(rect) = declared_since(&trace, ui_rect, MESSAGE_REGION, mark) {
        return Ok(Some(format!(
            "an error sentence was drawn over the page during part B, at \
             {:.0},{:.0}..{:.0},{:.0}. The operator's fourth clause is explicit: *\"zoom should \
             stop at the limit and not end up showing an error … the error can still be shown on \
             the bottom bar\"*. A refusal of kind `BeyondRaster` must be absorbed into a ceiling \
             by `render::settle::absorb`'s `absorb_render`, which leaves `render_error` unset and the page \
             texture in place; a published `{MESSAGE_REGION}` means some refusal reached \
             `RefusalKind::Other` and was painted.",
            rect.min.x, rect.min.y, rect.max.x, rect.max.y
        )));
    }
    if let Some((reason, _)) = unavailable_now(&trace)
        && reason == "render-failed"
    {
        return Ok(Some(format!(
            "the canvas's standing verdict after part B is `{UNAVAILABLE_EVENT} \
             reason=render-failed`, i.e. the error arm. Same clause as above, caught from the \
             other side: this arm and `{MESSAGE_REGION}` are published together, and reading \
             both means a sentence standing from before part B cannot hide in the \
             change-log."
        )));
    }

    // And the half the operator asked for in as many words: say why.
    let Some(status) = declared(&trace, ui_rect, STATUS_REGION) else {
        let why = clipped_away(&trace, ui_rect, STATUS_REGION).unwrap_or_else(|| {
            format!(
                "it was never published. Status regions that were: {}.",
                list(&declared_names(&trace, ui_rect, "status-group:"))
            )
        });
        return Ok(Some(format!(
            "the zoom stopped at the learned ceiling {:.2} and the status bar says nothing about \
             it: no `{STATUS_REGION}` region — {why} The operator's fourth clause asks for this \
             in as many words: *\"the error can still be shown on the bottom bar so the user has \
             some idea as to why zooming stopped short of 1 trillion percent.\"* Without it the \
             `+` button and Ctrl+wheel simply stop responding with nothing anywhere saying why, \
             which is the silently-inert control this project has been corrected about twice.",
            ceiling.to
        )));
    };
    if declared_since(&trace, ui_rect, STATUS_REGION, mark).is_none() {
        return Ok(Some(format!(
            "a `{STATUS_REGION}` region is standing at \
             {:.0},{:.0}..{:.0},{:.0}, but it was last published BEFORE part B began — so it is a \
             fossil from an earlier state rather than a disclosure of the ceiling this run \
             reached. R9 reserves this region for the state it describes: the ceiling did not \
             exist before part B, so its explanation cannot have been drawn before part B either.",
            status.min.x, status.min.y, status.max.x, status.max.y
        )));
    }
    if !status.is_substantial() {
        return Ok(Some(format!(
            "the `{STATUS_REGION}` region is {:.1} x {:.1} pt — drawn, and too small to read. A \
             zero-area region is a widget that was laid out into no space, which on the status \
             bar means the row ran out of width; the sentence exists and the operator cannot see \
             it.",
            status.width(),
            status.height()
        )));
    }
    let client = session.frame()?.client_logical();
    if !client.contains_rect(status) {
        return Ok(Some(format!(
            "the `{STATUS_REGION}` region sits at {:.0},{:.0}..{:.0},{:.0} and the window's \
             client area is {:.0},{:.0}..{:.0},{:.0} — so the explanation of why zooming stopped \
             is off the edge of the window. Drawn is not seen: a region below or beside the fold \
             publishes a perfectly good rect.",
            status.min.x,
            status.min.y,
            status.max.x,
            status.max.y,
            client.min.x,
            client.min.y,
            client.max.x,
            client.max.y
        )));
    }
    report.note(format!(
        "the bottom bar explains the stop: `{STATUS_REGION}` at \
         {:.0},{:.0}..{:.0},{:.0}, inside the window, at zoom {:.2} ({:.0} %) against a learned \
         ceiling of {:.2}",
        status.min.x,
        status.min.y,
        status.max.x,
        status.max.y,
        state.zoom,
        state.zoom * 100.0,
        ceiling.to
    ));
    if state.drawn == 0 {
        return Ok(Some(format!(
            "part B climbed to the learned ceiling {:.2} ({:.0} %) and the saturated frame drew \
             {} of {} visible page(s) — no page was rastered at all. ★ Read this as O186, NOT as \
             a rasterizer fault: the guarantee is `canvas::geometry::MIN_SHEET_ON_SCREEN`, which \
             narrows the pasteboard so that the extremes of `visible_origin_range` still leave a \
             sliver of sheet on screen. Before that constant the pasteboard was exactly one \
             viewport, which made both ends of the range the zero-overlap placement at every \
             zoom. If this is the only red check in the sweep, start at `canvas::geometry`, not \
             here — and `a_view_carried_off_the_sheet_comes_back` is the check that isolates it.",
            ceiling.to,
            state.zoom * 100.0,
            state.drawn,
            state.visible
        )));
    }
    report.note(format!(
        "part B's saturated frame drew {} of {} page(s) with a raster — ★ asserted, not merely \
         noted, since O186's fix made an empty frame here a regression rather than a second known \
         defect",
        state.drawn, state.visible
    ));
    Ok(None)
}

fn drive_a(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses ribbon commands and rolls the \
             wheel, plain and with Ctrl held, for a hundred notches or more. Reported as SKIPPED \
             rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    let fixture = fixture_path();
    if !fixture.exists() {
        return Err(Error::new(format!(
            "{FIXTURE} is missing from the repository, so this check has no multi-page document \
             with pages of differing sizes. SKIPPED."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was IGNORED; this check pins {FIXTURE} because part A needs a VISIBLE \
             NEIGHBOUR page whose pixmap ceiling is reached at a different zoom from the acting \
             page's, and because the page COUNT is part of the seam search"
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("raster-wall.trace.txt"));
    spec.pdf = Some(fixture);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    } else {
        return Err(Error::new(format!(
            "the `{}` profile declares no viewport override, so this check cannot fix the window \
             size — and the seam band below is quoted as a fraction of a canvas of known height. \
             SKIPPED rather than measured at whatever size the window happened to open at.",
            ctx.profile.name
        )));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(16);

    let canvas = declared(&session.trace()?, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is the document open?")))?;
    if !canvas.is_substantial() {
        return Err(Error::new(format!(
            "the `{CANVAS_REGION}` region is {:.1} x {:.1} pt, which is not a laid-out canvas. \
             SKIPPED.",
            canvas.width(),
            canvas.height()
        )));
    }

    part_a(&session, &driver, ui_rect, canvas, report)
}

/// Launch on the dense drawing and run [`part_b`].
///
/// ⚠ Deliberately a second launch of its own rather than a second stage of
/// [`drive_a`]'s session. The fixtures differ, and a check that opened a second
/// document into the first's window would be measuring the multi-document tab
/// machinery as well as the raster wall — two subjects, one verdict.
fn drive_b(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check rolls the wheel with Ctrl held, a hundred \
             notches and more. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    let dense = dense_path();
    if !dense.exists() {
        return Err(Error::new(format!(
            "{FIXTURE_DENSE} is not on this machine, so this check has no document with enough \
             ink on it to make the rasterizer refuse a scale — and a refusal is the whole \
             precondition of the operator's fourth clause. Measured on 2026-09-12: against the \
             repository's own {FIXTURE} the zoom climbs to ten billion (a trillion percent) with \
             zero refusals, because the region tier's request SHRINKS as the zoom rises. SKIPPED, \
             naming the file: a pass here would read as 'the zoom stops at the wall' when nothing \
             was ever asked to stop."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was IGNORED; this check pins {FIXTURE_DENSE} because the rasterizer only \
             refuses on dense content and a sparse page cannot produce the state the operator's \
             fourth clause is about"
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("raster-wall-b.trace.txt"));
    spec.pdf = Some(dense);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    } else {
        return Err(Error::new(format!(
            "the `{}` profile declares no viewport override, so this check cannot fix the window \
             size. SKIPPED rather than measured at whatever size the window happened to open at.",
            ctx.profile.name
        )));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    //
    // Without this declaration the harness reports A THREAD PANICKED and fails
    // the check on a run whose own trace shows the feature working perfectly —
    // which is exactly how the first driven run came out. The declaration is
    // per-session and deliberately not a profile flag: every other check keeps
    // the full detection. See `Session::expect_thread_panic`.
    session.expect_thread_panic();
    report.note(format!(
        "launched {} as pid {} on the dense drawing",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    // ★ A longer settle than part A's. This is a 5.7 MB vector drawing and the
    // first frame has to raster a whole sheet of it; a short settle reads the
    // opening zoom before the fit has been applied, which would put the climb's
    // first notch somewhere this check did not choose.
    session.settle(80);

    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(24);

    let canvas = declared(&session.trace()?, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is the document open?")))?;
    if !canvas.is_substantial() {
        return Err(Error::new(format!(
            "the `{CANVAS_REGION}` region is {:.1} x {:.1} pt, which is not a laid-out canvas. \
             SKIPPED.",
            canvas.width(),
            canvas.height()
        )));
    }
    part_b(&session, &driver, ui_rect, canvas, report)
}

#[cfg(test)]
mod panic_conversion_tests {
    use super::unconverted_panics;
    use crate::trace::Trace;

    const PREFIX: &str = "pdfcer-diag";

    /// The panic line as the capture actually carries it: not a diag line at
    /// all, so it lands in `Trace::other`.
    const PANIC: &str = "thread '<unnamed>' panicked at crates/tiny-skia/src/pipeline.rs:1: slice";

    /// The witness the canvas publishes when the engine caught it.
    const REFUSAL: &str = "pdfcer-diag raster-limit scale=64 panic=slice";

    #[test]
    fn a_panic_with_a_refusal_behind_it_is_the_conversion_this_check_declares() {
        let trace = Trace::parse(&format!("{PANIC}\n{REFUSAL}"), PREFIX);
        assert_eq!(unconverted_panics(&trace), None);
    }

    #[test]
    fn a_panic_with_no_refusal_behind_it_is_a_worker_that_died() {
        let trace = Trace::parse(PANIC, PREFIX);
        let complaint = unconverted_panics(&trace).expect("an unconverted panic must be reported");
        assert!(complaint.contains("1 thread panic(s)"), "{complaint}");
        assert!(complaint.contains("only 0"), "{complaint}");
    }

    /// A `raster-limit` line with no `panic=` field is the ceiling being
    /// reported for some other reason, and cannot discharge a panic.
    #[test]
    fn a_refusal_carrying_no_panic_field_does_not_discharge_one() {
        let trace = Trace::parse(
            &format!("{PANIC}\npdfcer-diag raster-limit scale=64"),
            PREFIX,
        );
        assert!(unconverted_panics(&trace).is_some());
    }

    /// The common case, and the one that must stay cheap: no panic at all.
    #[test]
    fn a_capture_with_no_panic_is_not_asked_for_a_refusal() {
        let trace = Trace::parse("pdfcer-diag canvas-zoom scale=2", PREFIX);
        assert_eq!(unconverted_panics(&trace), None);
    }
}
