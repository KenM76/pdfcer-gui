//! `the_format_tab_restyles_a_selected_mark` — the Format ▸ Markup band, driven
//! from a drawn shape all the way to a thicker line on the page.
//!
//! # The surface
//!
//! `RIBBON_IA.md` §5.8 specified a **Markup** group on the contextual Format tab
//! on 2026-08-12 — line colour, fill, line width, opacity, arrowheads — and the
//! tab carried nothing for it until 2026-09-06. Before that commit a placed
//! mark's colour, width and opacity could be changed **in the Properties panel
//! only**, and `/IC` and `/LE` could not be changed anywhere: two `MarkupStyle`
//! fields the engine had shipped for weeks with zero GUI callers.
//!
//! Six controls now sit there. This check drives **one** of them end to end and
//! asserts the presence and the *absence* of the rest.
//!
//! # ★★★ Seven links, and no test in the workspace observes two of them joined
//!
//! | # | link | why a unit test cannot see it |
//! |---|---|---|
//! | 1 | a drawn `/Square` **selects** on a click | `selection::annot` hit-tests a `/Rect` through two coordinate spaces, against a canvas only the running program has laid out |
//! | 2 | the selection publishes `selection.markup_restylable` | a `ConditionSet` is recomputed per frame from live state; `manifest::format`'s test asserts the item **carries** the condition, never that anything satisfies it |
//! | 3 | the contextual **Format tab appears** | the shell decides tab visibility from the condition set; the application never asks |
//! | 4 | the six items **draw**, and the ones that do not apply draw *nothing* | `markupband::draw` returns `None` for an unknown kind and `endings` returns `false` for a subtype with no `/LE` — the difference between a control that is absent and one that is greyed is **pixels**, and R9 says which it must be |
//! | 5 | a drag on the width field commits **on release** | `DragValue::drag_stopped`, a property of a real pointer gesture across a real widget |
//! | 6 | the commit reaches `EditSession::set_markup_style` | `app::actions::apply`'s routing, over a parked operand the renderer put down |
//! | 7 | the regenerated `/AP` is **repainted** | the page raster's invalidation, then `pdfcer-render`, then the compositor |
//!
//! ★★ **Link 4 is the one with no other oracle at all.** `visible_when` in a
//! *menu* did nothing for the whole of this project's life until 2026-09-06 —
//! `menu::plan::resolve` never read `Item::visible_condition()`, so every row
//! meant to vanish was **greyed** instead, R9 inverted, with prose at each site
//! describing behaviour that was not happening. The commit that found it says
//! why no test could: *"every one asked the model rather than the resolution."*
//! The arrowhead control here is the same shape one surface over — the manifest
//! deliberately gives it **no condition** and lets `markupband` decide its own
//! absence from the value it read — and the only way to tell an absent control
//! from a greyed one is to ask how much space it took.
//!
//! # What it does, and the two oracles it ends on
//!
//! 1. Arm Review and the Rectangle tool through `PDFCER_DIAG_INVOKE`; draw a
//!    shape with one drag.
//! 2. Photograph the strip along its top edge — **the thin line**.
//! 3. Put the pen down, click the shape, confirm `annot-select`.
//! 4. Click the Format tab. Assert the four controls a `/Square` has are drawn
//!    and **substantial**, and that the arrowhead chooser is **not**.
//! 5. Drag the width field to its ceiling.
//! 6. Assert `set-markup-style` reached the engine — *and* photograph the same
//!    strip again with nothing selected: **the line is thicker**.
//!
//! ★★★ **Step 6 is two assertions because they fail separately.** A build whose
//! parked operand never reaches `apply` traces nothing and paints nothing. A
//! build that restyles the dictionary and never re-bakes the appearance — or
//! bakes it and never invalidates the page raster — traces `set-markup-style`
//! **perfectly** and paints the old line. That second failure is this project's
//! signature shape, and `markup_node_edit` records the same pair for the same
//! reason: *"the engine's own note is that a shell writing some of the three
//! looks right in every renderer."*
//!
//! # Calibration
//!
//! ```text
//! --pdf fixtures/a1-titleblock.pdf --doc-point 0,300,500
//! ```
//!
//! ⚠ `--doc-point` is **0-based** and this check does not aim with it — it
//! places its shape in page fractions. Any single-page fixture with blank paper
//! across the middle third serves; the check asserts that emptiness and SKIPs
//! rather than measuring a fixture's own linework.
//!
//! # Every way this reports SKIP
//!
//! * no binary, no `--pdf`, `--no-input`;
//! * the canvas is not showing page 1;
//! * the fixture has its own content under the strip, so a thickness reading
//!   could not be attributed to this check's mark;
//! * the rectangle tool authored nothing, or the shape could not be selected —
//!   both are `dragging_a_markup_moves_it`'s subject and both are steps *before*
//!   the one under test;
//! * the width field was drawn but the drag did not change its value, so there
//!   was no restyle to observe.

use crate::checks::driving::{
    SHELL_DIAG_ENV, arm_select_from_ribbon, declared, declared_names, list,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::pixels::InkReport;
use crate::report::CheckReport;

/// Review mode, then the rectangle tool. See `markup_palette::INVOKE`.
const INVOKE: &str = "mode.review,markup.rectangle";

/// The contextual tab the band lives on.
const FORMAT_TAB: &str = "ribbon.tab.format";

/// The line the canvas writes when a shape is authored.
const COMMIT_EVENT: &str = "markup-commit";
/// The line the apply arm writes when the engine has authored it.
const APPLY_EVENT: &str = "add-markup";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// `set-markup-style …` — `vector_edit`'s line for the restyle.
///
/// ★ The bare name and not `-applied`: `apply::vector_edit` names the edit
/// itself, and a refusal traces `set-markup-style-refused`, which the failure
/// message points at.
const RESTYLE_EVENT: &str = "set-markup-style";
/// What a refused `vector_edit` writes instead.
const REFUSED_SUFFIX: &str = "-refused";
/// `ribbon-command-invoked id=… handler=…` — the shell reporting that a band
/// control handed its token to the application.
const INVOKE_EVENT: &str = "ribbon-command-invoked";

/// The four controls a `/Square` must be given.
///
/// ★ A `/Square` and not "a markup", because which controls apply is a property
/// of the subtype: a highlight has no border to widen, an arrow has no interior
/// to fill. This check draws a rectangle, so this is the list for a rectangle.
const EXPECTED: [(&str, &str); 4] = [
    ("ribbon.item.format.colour", "the line-colour swatch (`/C`)"),
    ("ribbon.item.format.fill", "the fill swatch (`/IC`)"),
    (
        "ribbon.item.format.line_width",
        "the line-width field (`/BS` `/W`)",
    ),
    ("ribbon.item.format.opacity", "the opacity field (`/CA`)"),
];

/// The control that must be drawn as **nothing** for a `/Square`.
///
/// `/LE` is meaningful for a `/Line` alone. `manifest::format` deliberately gives
/// this item no condition and lets `markupband::endings` decide its own absence
/// from the value it read — *"a control that decides its own absence from the
/// value it reads, in the one place that has read it"* — so the only way to
/// check that decision is to look at how much room it took.
const ABSENT_FOR_A_SQUARE: &str = "ribbon.item.format.arrowheads";

/// The namespace a failure lists when it cannot find one of [`EXPECTED`].
const ITEM_PREFIX: &str = "ribbon.item.format.";

/// The smallest logical width **and** height a control must occupy to count as
/// drawn.
///
/// # ★ Why "declared" is not enough, in both directions
///
/// `markupband::draw` publishes its `ui-rect` from the response of an
/// `add_enabled_ui` **whether or not the closure drew anything**, so an absent
/// control still declares a rect — a degenerate one. Presence and absence are
/// therefore both statements about *area*, not about the name appearing in the
/// trace, and a check that only asked "is it declared" would pass on a build
/// where every one of the six drew nothing.
///
/// The number is a floor on the smaller of the two dimensions rather than on the
/// area, because the failure this guards against is a control laid out with **no
/// usable extent in one axis** — which is the redaction panel's apply button
/// shipped below the bottom of its own pane, a defect this project has already
/// had once. 10 logical px is under half the height of the smallest control in
/// the band and an order of magnitude over the ~0 an empty `add_enabled_ui`
/// allocates.
const MIN_CONTROL_EXTENT: f32 = 10.0;

/// How far the pointer drags the width field, in logical pixels.
///
/// `DragValue::speed(0.1)` means 0.1 pt per pixel, so 200 px is +20 pt against a
/// field whose range is 0.25–12 pt. **Deliberately past the ceiling**: the
/// commit is compared against the range's own maximum rather than against an
/// arithmetic prediction, so this check does not have to track the speed
/// constant, and a clamp that stopped working would show up as a value over 12
/// rather than as a check that had to be re-tuned.
const WIDTH_DRAG_PX: f32 = 200.0;

/// What is typed into the width field when a drag on it commits nothing.
///
/// Inside the control's own 0.25–12 pt range, and six times the 2 pt a shape is
/// drawn with, so the thickening assertion downstream has something to see.
const TYPED_WIDTH: &str = "12";

/// How much of a band item's published rect is certainly the widget.
///
/// A third. `markupband::FIELD_WIDTH` is 46 logical px and a two-digit
/// `DragValue` with a ` pt` suffix is comfortably wider than 15, so the left
/// third is inside the spinner on any theme this shell has had. See the drag's
/// note for what the centre cost.
const GRAB_FRACTION: f32 = 0.33;

/// The width the field must land on: `markupband::MAX_WIDTH_PT`.
///
/// Spelled here rather than imported, like every other constant in this crate:
/// the harness must be able to fail against a binary built from a different
/// tree, and an import would make the check assert that the application agrees
/// with itself.
const MAX_WIDTH_PT: f64 = 12.0;

/// The width a freshly drawn shape starts at — `canvas::markup::pen`'s 2 pt.
///
/// Used only to make the failure messages arithmetic rather than vague, and to
/// justify [`MIN_THICKENING`]: the drag asks for six times the starting width.
const START_WIDTH_PT: f64 = 2.0;

/// How much more ink the strip must hold after the restyle, as a multiple.
///
/// # ★★ Why 1.5 and not 6
///
/// The width goes from 2 pt to 12 pt, which is **six times** the line — and the
/// ink count does not go up sixfold, because the strip is a fixed box and a
/// thicker line fills more of its height but no more of its length. Measured on
/// this fixture at fit-page zoom (20 %), a 2 pt line covers about one row of the
/// strip and a 12 pt line about two: the honest expectation is a doubling, not a
/// sextupling.
///
/// ⇒ **A floor derived from what the geometry can actually produce**, not from
/// the ratio of the numbers that were typed. A check demanding six times would
/// go red on a working build, and the session that met it would go looking for a
/// rendering defect. This project has already spent a morning on exactly that
/// mistake in the other direction (`markup_rectangle`'s note on three candidate
/// palettes).
///
/// 1.5 is comfortably under the measured doubling and comfortably over the ±2
/// pixels an antialiased edge moves by, on a strip that starts with tens of ink
/// pixels.
const MIN_THICKENING: f64 = 1.5;

/// Where the shape is drawn, as fractions of the page.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));

/// Half-height of the strip laid along the top edge, as a fraction of the page
/// height.
///
/// ★ Four times `markup_palette`'s, and the difference is the subject: that
/// check reads a *colour* and wants as little paper in the box as possible, this
/// one reads a *thickness* and needs room above and below for the line to grow
/// into. A strip only as tall as the thin line would saturate at the first
/// widening and report nothing about the rest.
const STRIP_HALF: f64 = 0.02;

/// How much of the top edge's length the strip covers, as a fraction of the
/// shape's width, centred. Corners excluded: a join lays down twice the ink.
const STRIP_SPAN: f64 = 0.6;

/// Where the pointer is parked before a capture, as fractions of the page.
///
/// ★★ Blank paper in a corner, and `markup_palette`'s first run is why: a
/// pointer left on the shape pops the *"No note has been written on this
/// markup."* tooltip, a floating dark panel that lands on the very box being
/// measured and reads as ink. A driven check photographs the pointer as well as
/// the program.
const PARK: (f64, f64) = (0.10, 0.90);

/// Where the deselecting click goes, as fractions of the page.
///
/// Far from the shape and far from the strip, so it can neither re-select the
/// mark nor add ink to what is about to be measured. Distinct from [`PARK`] on
/// purpose: the click lands here and the pointer then moves on, so the tooltip
/// question and the deselection question do not share an answer.
const ELSEWHERE: (f64, f64) = (0.80, 0.15);

/// The smallest ink count that counts as *something is drawn here*, and the
/// ceiling the baseline must stay under. See `markup_palette::INK_FLOOR`.
const INK_FLOOR: usize = 4;

/// See the module documentation.
pub struct TheFormatTabRestylesASelectedMark;

impl Check for TheFormatTabRestylesASelectedMark {
    fn name(&self) -> &'static str {
        "the_format_tab_restyles_a_selected_mark"
    }

    fn defect(&self) -> &'static str {
        "selecting a comment shape does not raise the Format ▸ Markup band, or raises it with \
         controls that took no space, or with the arrowhead chooser GREYED where R9 says it must \
         be absent — or the width field commits to the trace and the line on the page never \
         changes, which is the failure a trace assertion cannot see"
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

/// Capture the window and count the ink lying along the shape's top edge.
///
/// The mapping is re-derived by the caller for every reading rather than cached:
/// a cached mapping is a stale coordinate, and a stale coordinate is
/// symptom-identical to a broken conversion — the confusion behind one
/// filed-then-retracted defect in this codebase.
fn edge_ink(
    session: &Session,
    mapping: &CanvasMapping,
    page: PageGeometry,
    out: &std::path::Path,
) -> Result<(InkReport, std::path::PathBuf)> {
    let image = crate::capture::window_to_png(session, out)?;
    let frame = session.frame()?;
    let (lo, hi) = strip_box(page);
    let a = mapping.doc_to_window(lo)?;
    let b = mapping.doc_to_window(hi)?;
    let rect = LRect::new(
        Pt::new(a.x().min(b.x()), a.y().min(b.y())),
        Pt::new(a.x().max(b.x()), a.y().max(b.y())),
    );
    Ok((
        crate::pixels::ink_run_into(&image, frame.logical_to_capture_pixels(rect)),
        out.to_path_buf(),
    ))
}

/// Whether a declared rect is big enough to be a control an operator can hit.
fn substantial(rect: LRect) -> bool {
    rect.width() >= MIN_CONTROL_EXTENT && rect.height() >= MIN_CONTROL_EXTENT
}

/// The leftmost `frac` of a rect, full height.
///
/// See the drag's own note: a band item's published rect is the widget **plus**
/// whatever padding the renderer allocated to make the row line up, and the
/// padding is always on the right.
fn left_part(rect: LRect, frac: f32) -> LRect {
    LRect::new(
        rect.min,
        Pt::new(rect.min.x + (rect.max.x - rect.min.x) * frac, rect.max.y),
    )
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape with a drag, clicks it, \
             clicks a ribbon tab and drags a spinner. Every one is a real pointer gesture. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. This check needs a page with blank paper to draw a shape on.")
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
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("markup-band.trace.txt"));
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

    let trace = session.trace()?;
    let shown = trace
        .last(ctx.profile.vocab.canvas_event)
        .and_then(|l| l.get_usize("page"));
    if shown != Some(0) {
        return Err(Error::new(format!(
            "the canvas is showing page {shown:?}, not page 1, so this check's page-1 fractions \
             describe something other than what is on screen."
        )));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;

    // --- the baseline -------------------------------------------------------
    let (blank, blank_shot) =
        edge_ink(&session, &mapping, page, &ctx.out("markup_band_blank.png"))?;
    report.artifact(blank_shot.clone());
    if blank.ink >= INK_FLOOR {
        return Err(Error::new(format!(
            "★ THIS FIXTURE HAS ITS OWN LINEWORK WHERE THE CHECK MEASURES: the strip along the \
             shape's future top edge already holds {}, against a ceiling of {INK_FLOOR}. A \
             thickness reading taken there could not be attributed to this check's mark. SKIPPED \
             rather than failed — that is a fact about {} and not about the program. Capture: {}.",
            blank.summary(),
            pdf.display(),
            blank_shot.display()
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
             `{COMMIT_EVENT}`. That is `dragging_a_markup_moves_it`'s subject and it is the step \
             before this one — there is no mark here to restyle. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if trace.events(APPLY_EVENT).count() == 0 {
        return Ok(Some(format!(
            "THE ENGINE NEVER AUTHORED THE SHAPE: the canvas decided to — `{}` — and no \
             `{APPLY_EVENT}` followed. Look for `add-markup-refused` first. Trace: {}.",
            commit.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★ a rectangle was authored: `{}`", commit.raw));

    // --- the THIN line, photographed ---------------------------------------
    let parked = aim(ctx, &session, page, corner(PARK))?;
    driver.move_to(parked)?;
    session.settle(24);
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let (thin, thin_shot) = edge_ink(&session, &mapping, page, &ctx.out("markup_band_thin.png"))?;
    report.artifact(thin_shot.clone());
    report.note(format!(
        "the {START_WIDTH_PT} pt line as drawn: {}",
        thin.summary()
    ));
    if thin.ink < INK_FLOOR {
        return Ok(Some(format!(
            "★★ THE SHAPE REACHED THE DOCUMENT AND NOT THE PAGE: `{APPLY_EVENT}` is in the trace \
             and the strip along its top edge holds {}, against a floor of {INK_FLOOR}. There is \
             nothing on screen for a restyle to change, so this check cannot reach its subject — \
             and `a_new_markup_is_drawn_in_acrobats_red` is the check whose whole job is that \
             step. Capture: {}.",
            thin.summary(),
            thin_shot.display()
        )));
    }

    // --- select it ----------------------------------------------------------
    //
    // ★ Put the pen down first. With the Rectangle tool still armed a click on
    // the page draws a second shape rather than selecting the first —
    // `sys::vk::V`'s own doc comment records a check that did not, and reported
    // "the shape could not be selected" about a build whose selection is fine.
    let centre = corner((
        f64::midpoint(SHAPE.0.0, SHAPE.1.0),
        f64::midpoint(SHAPE.0.1, SHAPE.1.1),
    ));
    let centre_screen = aim(ctx, &session, page, centre)?;
    if !arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        report.note(
            "the ribbon route to the select tool was unavailable, so the pen was put down with \
             the `V` chord instead",
        );
        driver.press(crate::sys::vk::V)?;
        session.settle(12);
    }
    driver.click_at(centre_screen)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(selected) = trace.events(SELECT_EVENT).last() else {
        return Ok(Some(format!(
            "THE SHAPE COULD NOT BE SELECTED: a click at its centre produced no \
             `{SELECT_EVENT}`. Selecting is the step before the one under test. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the shape was selected: `{}`", selected.raw));
    if selected.get("locked") == Some("true") {
        return Err(Error::new(
            "the annotation reports itself LOCKED (§12.5.3 bit 8). `markupband` deliberately \
             draws the controls live for a locked mark and refuses the commit with a sentence, \
             which is a different subject from this one. SKIPPED rather than failed.",
        ));
    }

    // --- the contextual tab -------------------------------------------------
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, FORMAT_TAB) else {
        return Ok(Some(format!(
            "★★ SELECTING A MARK DID NOT RAISE THE FORMAT TAB: nothing published \
             `{FORMAT_TAB}` after `{SELECT_EVENT}`. The Markup group's items all carry \
             `shown_when(\"selection.markup_restylable\")`, so either that condition is not \
             published for a `/Square`, or the shell is not making the tab visible from it. \
             Tabs declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab.")),
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(20);
    // The band, photographed. Not asserted on — the assertions below are on the
    // declared rects — but a failure about a control that "took no space" is a
    // failure a reader will want to look at, and this project's standing rule is
    // to read the capture before believing a pixel assertion.
    let band_shot = crate::capture::window_to_png(&session, &ctx.out("markup_band_ribbon.png"))
        .map(|_| ctx.out("markup_band_ribbon.png"));
    if let Ok(path) = band_shot {
        report.artifact(path);
    }

    // --- link 4: what is drawn, and what is deliberately not -----------------
    let trace = session.trace()?;
    let frame = session.frame()?;
    let mut missing: Vec<String> = Vec::new();
    for (region, what) in EXPECTED {
        match declared(&trace, ui_rect, region) {
            Some(rect) if substantial(rect) => {}
            Some(rect) => missing.push(format!(
                "{what} declared `{region}` at {:.0}x{:.0} logical px, under the \
                 {MIN_CONTROL_EXTENT:.0} px floor — it was laid out with no usable extent",
                rect.width(),
                rect.height()
            )),
            None => missing.push(format!("{what} declared no `{region}` at all")),
        }
    }
    if !missing.is_empty() {
        return Ok(Some(format!(
            "★★★ THE FORMAT ▸ MARKUP BAND IS NOT DRAWN FOR A SELECTED RECTANGLE. {}. Items \
             declared under `{ITEM_PREFIX}`: {}. This is `markup_style`'s recorded defect one \
             surface over — an item the manifest declares and no renderer matches reserves its \
             space and draws nothing, and every test of the manifest passes. Trace: {}.",
            missing.join("; "),
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX)),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ all four controls a `/Square` has are drawn and substantial: {}",
        list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
    ));

    // ★★★ R9, and it is the half with no other oracle. Paired with the four
    // positives above **from the same trace**, which is what stops it being the
    // vacuous negative this project has already shipped once: a "did not
    // happen" assertion that passed with the whole feature switched off. Here
    // the control is that four sibling items in the same group, drawn by the
    // same function on the same frame, DID take space.
    if let Some(rect) = declared(&trace, ui_rect, ABSENT_FOR_A_SQUARE)
        && substantial(rect)
    {
        return Ok(Some(format!(
            "★★ THE ARROWHEAD CHOOSER IS DRAWN FOR A RECTANGLE: `{ABSENT_FOR_A_SQUARE}` took \
             {:.0}x{:.0} logical px, past the {MIN_CONTROL_EXTENT:.0} px floor, on a `/Square` \
             that has no `/LE` to choose. R9 says an unavailable capability renders **nothing**, \
             not a greyed control — a greyed one claims the mark could have arrowheads if only \
             something were different, and nothing is. `markupband::endings` is where that \
             decision is made, from the value it read; `manifest::format` deliberately gives the \
             item no condition so that this is the only place it can be made. ⚠ This is the same \
             shape as `menu::plan::resolve` ignoring `visible_condition` for the whole of this \
             project's life, which greyed every row that was meant to vanish and which no test \
             could see. The four sibling controls in the same group DID draw on this frame, so \
             this is not a band that failed to render.",
            rect.width(),
            rect.height()
        )));
    }
    report.note(
        "★★ and the arrowhead chooser is ABSENT, not greyed — `/LE` means nothing to a `/Square` \
         and R9 says an unavailable capability renders nothing",
    );

    // --- link 5 and 6: drag the width field ---------------------------------
    let Some(width_rect) = declared(&trace, ui_rect, EXPECTED[2].0) else {
        return Err(Error::new(
            "the width field vanished between the presence check and the drag, which can only \
             mean the selection was lost. SKIPPED: nothing was driven.",
        ));
    };
    // ★★★ **The LEFT of the published rect, not its centre — and the first run
    // of this check is why.**
    //
    // `markupband::width` draws a `DragValue` and then calls
    // `ui.allocate_space(FIELD_WIDTH - response.rect.width())` to pad the item
    // out to a fixed 46 px, so the rect the application publishes is **the
    // spinner plus a strip of empty ribbon to its right**. A press at that
    // rect's centre can land in the padding, on nothing at all — which is what
    // happened: the drag produced no `ribbon-command-invoked` and no restyle,
    // and the check reported the dispatch broken about a build whose dispatch is
    // fine.
    //
    // ⇒ **A published rect is where a control was ALLOCATED, not where it is
    // hittable.** That is a general fact about this shell's band items, and the
    // fix is to aim where the widget certainly is rather than where the average
    // of its bounds falls. `markup_rectangle` never met it because a `Button`
    // fills its allocation.
    //
    // The release point is the same sub-rect translated — never a screen
    // coordinate assembled by hand. This crate's rule is that a check aims only
    // at rectangles the application published, and a translated rect keeps that
    // property while a hand-built desktop point loses it: at a different DPI
    // scale the two disagree and the check drags the wrong distance without
    // saying so.
    let width_rect = left_part(width_rect, GRAB_FRACTION);
    let grab = frame.declared_center(width_rect);
    let release = frame.declared_center(LRect::new(
        Pt::new(width_rect.min.x + WIDTH_DRAG_PX, width_rect.min.y),
        Pt::new(width_rect.max.x + WIDTH_DRAG_PX, width_rect.max.y),
    ));
    driver.drag(grab, release)?;
    session.settle(40);
    let dragged = session.trace()?.events(RESTYLE_EVENT).count();
    report.note(if dragged > 0 {
        "★ the width field committed on a DRAG".to_owned()
    } else {
        format!(
            "⚠ the drag across the width field committed nothing — no `{RESTYLE_EVENT}`. The \
             check now types a value into the same field, which is what tells a broken control \
             apart from a broken dispatch; see the verdict below."
        )
    });

    // ★★★ **The typed route, and it is a DIAGNOSTIC and not a fallback.**
    //
    // A drag and a typed entry reach `markupband::width`'s commit through two
    // different gates — `drag_stopped()` and `lost_focus()` — and everything
    // after the commit is shared. So the pair separates the two failures that a
    // single route reports identically:
    //
    // | drag | typed | what it means |
    // |---|---|---|
    // | commits | — | the control works; the check goes on to the pixels |
    // | nothing | commits | **the DRAG gate is broken and nothing downstream is** — a defect in one line of one function, and the operator's spinner is inert |
    // | nothing | nothing | the parked edit never reaches `apply`, or the engine refuses — a dispatch defect |
    //
    // Without the second route the middle row is indistinguishable from the
    // bottom one, and a session reading the report would go looking through
    // `app::actions::apply` for a defect that is not there. That is the same
    // discipline as pairing a negative assertion with a control: **a check that
    // can only say "it did not happen" cannot say what did.**
    if dragged == 0 {
        // A click puts an `egui::DragValue` into keyboard-edit mode; Ctrl+A
        // selects what is in it, so the typed digits replace rather than append
        // to the value already there.
        driver.click_at(grab)?;
        session.settle(16);
        driver.press_held(&[crate::sys::vk::CONTROL], crate::sys::vk::A, 1)?;
        session.settle(8);
        driver.type_ascii(TYPED_WIDTH)?;
        session.settle(8);
        driver.press(crate::sys::vk::ENTER)?;
        session.settle(40);
    }

    let trace = session.trace()?;
    // ★★★ **HELD, not returned.** The drag defect is real and it is reported — but
    // returning here would stop the check before its most valuable assertion,
    // which is that the restyle reaches the **page**. Link 7 has no other oracle
    // in the workspace, and a check that stops short of it every run until
    // somebody fixes a different line is a check that is not verifying anything.
    //
    // ⇒ The typed route commits, so the subject is reachable; the pixel
    // assertion below runs on that, and the verdict at the end prefers the
    // *worse* finding. A page that never repaints is worse than a spinner that
    // cannot be dragged, and a reader must be told the worse one first.
    let drag_defect = if dragged == 0 && trace.events(RESTYLE_EVENT).count() > 0 {
        Some(format!(
            "★★★ THE LINE-WIDTH SPINNER CANNOT BE DRAGGED. A {WIDTH_DRAG_PX:.0} px drag across \
             `{}` produced no `{RESTYLE_EVENT}`; TYPING `{TYPED_WIDTH}` into the same field \
             produced one — `{}` — so everything downstream of the commit works and the drag \
             gate alone does not.\n\
             ★ The shape of it, from `markupband::width`: the control is drawn as \
             `DragValue::new(&mut value)` over `let mut value = was`, where `was` is read fresh \
             from the annotation's dictionary **on every frame**, and the commit is gated on \
             `response.drag_stopped() && (value - was).abs() > f64::EPSILON`. `drag_stopped()` \
             is true on the frame the button comes UP, and on that frame the pointer has not \
             moved, so `DragValue` adds nothing to a `value` that was just re-seeded from the \
             document: the difference is exactly zero and the commit is skipped. A spinner an \
             operator drags is a spinner that does nothing.\n\
             ⚠ Reported, not fixed — this file owns the harness. The same shape is worth \
             checking on the opacity field beside it, which `markupband` says takes \
             the same shape as `width`, and on any other `DragValue` in the shell whose \
             backing value is re-read per frame rather than held as a draft. Trace: {}.",
            EXPECTED[2].0,
            trace
                .events(RESTYLE_EVENT)
                .last()
                .map_or_else(String::new, |l| l.raw.clone()),
            session.trace_path().display()
        ))
    } else {
        None
    };
    if trace.events(RESTYLE_EVENT).count() == 0 {
        let invoked = trace
            .events(INVOKE_EVENT)
            .filter(|l| l.get("id") == Some("format.line_width"))
            .count();
        let refused = trace
            .events(&format!("{RESTYLE_EVENT}{REFUSED_SUFFIX}"))
            .count();
        return Ok(Some(format!(
            "★★★ DRAGGING THE WIDTH FIELD DID NOT RESTYLE THE MARK: no `{RESTYLE_EVENT}` line. \
             The shell reported `{INVOKE_EVENT} id=format.line_width` {invoked} time(s) and \
             `{RESTYLE_EVENT}{REFUSED_SUFFIX}` {refused} time(s), which tells them apart: \
             **invoked and not restyled** means the parked `(AnnotTarget, MarkupEdit)` never \
             reached `app::actions::apply`'s `SetMarkupStyle` arm, or reached it and \
             `EditSession::set_markup_style` refused; **neither** means the drag never changed \
             the field's value at all, which is `DragValue::drag_stopped` and not the \
             application's dispatch — `markupband::width` commits on release and on lost focus, \
             never on change, so a drag that ended outside the widget commits nothing. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    let restyle = trace
        .events(RESTYLE_EVENT)
        .last()
        .map_or_else(String::new, |l| l.raw.clone());
    report.note(format!(
        "★★ the width field reached the engine: `{restyle}`"
    ));

    // --- link 7: and the LINE ON THE PAGE is thicker ------------------------
    //
    // Deselect first, so what is counted is the annotation's own appearance and
    // not the selection outline drawn over it — which follows the shape's box
    // and would grow with it, satisfying this assertion for a reason that has
    // nothing to do with `/BS` `/W`.
    let elsewhere = aim(ctx, &session, page, corner(ELSEWHERE))?;
    driver.click_at(elsewhere)?;
    session.settle(24);
    let parked = aim(ctx, &session, page, corner(PARK))?;
    driver.move_to(parked)?;
    session.settle(40);

    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let (thick, thick_shot) =
        edge_ink(&session, &mapping, page, &ctx.out("markup_band_thick.png"))?;
    report.artifact(thick_shot.clone());
    report.note(format!("the line after the restyle: {}", thick.summary()));

    let grew = (thick.ink as f64) / (thin.ink.max(1) as f64);
    if grew < MIN_THICKENING {
        return Ok(Some(format!(
            "★★★ THE MARK WAS RESTYLED AND THE PAGE DID NOT CHANGE. `{RESTYLE_EVENT}` is in the \
             trace — `{restyle}` — so the engine accepted a width of up to {MAX_WIDTH_PT} pt \
             against the {START_WIDTH_PT} pt the shape was drawn with, and the strip along its \
             top edge went from {} to {}: {grew:.2}x, against a floor of {MIN_THICKENING}x.\n\
             **This is precisely the failure no trace assertion can see.** Three candidates, in \
             the order to check them: the `/AP` stream was not re-baked from the new `/BS`; the \
             page raster was not invalidated (`vector_edit`'s `page_epochs` bump); or the \
             appearance was rebaked and the `/Rect` not grown with it, so §12.5.5 is clipping \
             the wider stroke to the old box. Captures: {} and {}.",
            thin.summary(),
            thick.summary(),
            thin_shot.display(),
            thick_shot.display()
        )));
    }
    report.note(format!(
        "★★★ and the LINE ON THE PAGE is thicker: the strip went from {} ink px to {} \
         ({grew:.2}x, floor {MIN_THICKENING}x) with nothing selected, so what grew is the \
         annotation's own appearance and not a selection outline",
        thin.ink, thick.ink
    ));

    // Everything about the restyle works. What is left is the gate the drag
    // could not get through, which the diagnostic above has already localised.
    Ok(drag_defect)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A degenerate rect is not a control.
    ///
    /// ★ The property the whole of link 4 rests on: `markupband::draw` publishes
    /// a `ui-rect` for an item whose closure drew nothing, so *declared* and
    /// *drawn* are different questions and this is what tells them apart.
    #[test]
    fn an_empty_control_is_not_substantial() {
        let empty = LRect::new(Pt::new(100.0, 40.0), Pt::new(100.0, 40.0));
        assert!(!substantial(empty), "a zero-area rect is not a control");

        let sliver = LRect::new(Pt::new(100.0, 40.0), Pt::new(146.0, 42.0));
        assert!(
            !substantial(sliver),
            "a control with no usable height is not a control — that is the redaction panel's \
             apply button below the bottom of its own pane"
        );

        let real = LRect::new(Pt::new(100.0, 40.0), Pt::new(146.0, 62.0));
        assert!(substantial(real), "a 46x22 field is a control");
    }

    /// The strip is centred on the shape's top edge and does not reach its
    /// sides.
    ///
    /// ★ A check aimed at the wrong box produces an articulate failure message
    /// about nothing, which is this project's commonest wasted afternoon. The
    /// arithmetic that decides where this one looks is therefore asserted rather
    /// than eyeballed.
    #[test]
    fn the_strip_lies_on_the_top_edge_and_clear_of_the_corners() {
        let page = PageGeometry {
            width_pt: 1000.0,
            height_pt: 800.0,
        };
        let (lo, hi) = strip_box(page);
        let top = SHAPE.1.1 * page.height_pt;
        assert!(lo.y < top && hi.y > top, "the strip straddles the top edge");

        let left = SHAPE.0.0 * page.width_pt;
        let right = SHAPE.1.0 * page.width_pt;
        assert!(lo.x > left, "clear of the left corner");
        assert!(hi.x < right, "clear of the right corner");

        let bottom = SHAPE.0.1 * page.height_pt;
        assert!(
            lo.y > bottom,
            "and nowhere near the bottom edge, whose ink would be counted as the top edge's"
        );
    }

    /// The park and the deselecting click are both outside the strip.
    #[test]
    fn nothing_this_check_points_at_lands_in_the_box_it_measures() {
        let page = PageGeometry {
            width_pt: 1000.0,
            height_pt: 800.0,
        };
        let (lo, hi) = strip_box(page);
        for (name, (fx, fy)) in [("PARK", PARK), ("ELSEWHERE", ELSEWHERE)] {
            let (x, y) = (fx * page.width_pt, fy * page.height_pt);
            let inside = x >= lo.x && x <= hi.x && y >= lo.y && y <= hi.y;
            assert!(!inside, "{name} lands inside the measured strip");
        }
    }
}
