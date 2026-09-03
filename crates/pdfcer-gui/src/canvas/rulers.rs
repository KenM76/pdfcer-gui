//! # `canvas::rulers` — the ruler gutters, the tick ladder, and the drawing grid
//!
//! `RIBBON_IA.md` §5.2's View ▸ Display row *"Rulers · Grid · Guides"*, and
//! `FEATURES.md`'s last unbuilt Phase 3 line. This module owns two of the
//! three; [`super::guides`] owns the third and is built on the arithmetic
//! here.
//!
//! ---
//!
//! ## ★ 1. What unit the ruler reads in — the question, and the answer
//!
//! `RIBBON_IA.md` says *"rulers along the canvas edges, **in the document's
//! units**"*, and that phrase hides a real decision rather than a formatting
//! detail. pdfcer is a CAD-drawing tool: `pdfcer-core` carries a whole
//! dimensioning subsystem in which a **group** owns a scale, a display unit
//! and a number format, and in which the displayed value of a length is
//! *derived* — `value = measured_points × scale`.
//!
//! The rule this module implements, in one sentence:
//!
//! > **The ruler reads in exactly the same units, at exactly the same scale,
//! > and through exactly the same formatter as a dimension placed on the same
//! > sheet would.**
//!
//! Concretely, three states:
//!
//! | the document's default dimension group | what the ruler shows |
//! |---|---|
//! | no scale ever set (`ScaleState::NeverSet`) | **PDF points** — `100.00 pt`, `200.00 pt` … |
//! | explicitly 1:1 (`ScaleState::OneToOne`) | the group's unit at true size — a 72 pt span reads `25.40 mm` |
//! | calibrated (`ScaleState::Calibrated`) | the group's unit at the operator's scale — on a sheet drawn 1:50 in metres, a 72 pt span reads `1.270 m` |
//!
//! ### Why points is the *default*, and why that is not an arbitrary pick
//!
//! Because it is the only unit the document itself states. Every other number
//! this application shows about geometry — the `canvas-pointer` trace's
//! `page=` reading, the Properties panel's extents, an annotation `/Rect` — is
//! in PDF user-space units, and a ruler that invented millimetres while every
//! neighbouring readout said points would be a second measuring system with no
//! way to tell which one a number came from. Points is what the file says;
//! anything else is an interpretation, and an interpretation needs the
//! operator to have supplied it.
//!
//! ### Why a set scale changes the ruler, and why the numbers come from *core*
//!
//! Because an operator who has calibrated the sheet has said what the drawing
//! *means*, and a ruler that ignored that would be measuring the paper while
//! the drawing is about a building. The two readings must also **agree to the
//! digit**: someone who runs a linear dimension along a wall and then checks
//! it against the ruler is entitled to the same number, in the same unit, with
//! the same decimal marker and the same number of places.
//!
//! The only way to guarantee that is to call the same function, so [`Scale`]
//! calls [`pdfcer_core::dimension::format_measurement`] — the function that
//! renders every dimension label and every live measurement readout — and this
//! module never formats a length itself. That has a second, quieter payoff:
//! **the ruler contributes no operator-visible string to `crate::text` at
//! all.** Its labels are core's spelling of a measurement, unit abbreviation
//! and ISO comma included. The dimensioning subsystem's own rule is
//! *"disclosures rendered by core, never invented at the GUI layer"*; a ruler
//! tick is a measurement rendered by core, for the same reason.
//!
//! ### Where the scale is read from, and the honest limit of that today
//!
//! [`Scale::of`] reads [`pdfcer_core::edit::EditSession::dimension_model`] and
//! takes the **default group** (`DEFAULT_GROUP_ID`), which `pdfcer-core`
//! guarantees always exists. That is the document's own
//! `/PieceInfo /pdfcer /Private` sidecar, so a sheet calibrated in a previous
//! session — by this application once the scale surface lands, or by
//! `pdfcer` today — is honoured on open with nothing else wired.
//!
//! **The limit, stated rather than left to be discovered:** this build has no
//! way to *set* a scale. `measure.set_scale` is registered and has no dispatch
//! arm, and `EditSession`'s scale verbs have no caller in the GUI. So today
//! the ruler reads points on every document that has not been through the CLI,
//! which is every test document in `D:\Dev\temp\pdfcer`. That is why this is
//! written as *reading a document property* rather than as *reading a pdfcer
//! setting*: when the scale surface lands it needs no change here, and until
//! it does the ruler is not pretending to a calibration nobody supplied.
//!
//! **There is no cache, and the number is measured rather than assumed.**
//! `dimension_model()` walks catalog → `/PieceInfo` → `/pdfcer` → `/Private`
//! and deserialises; on a document with no sidecar — every ordinary PDF — it
//! stops at the first `None` after four dictionary lookups and one catalog
//! clone. **Measured on `ncored-benchmark-cad-drawing.pdf` (A3, 129,758
//! objects): 3.65 µs per call, called twice a frame** — once by the rulers and
//! once by the grid. The same sheet's *raster* is 1,244 ms
//! (`render-async-done ms=1244`), and a 60 Hz frame is 16,667 µs.
//!
//! Caching it would need a key on `edit_epoch`, a `RefCell` on `OpenDoc` and a
//! staleness question, bought for 7 µs against a frame that has just uploaded
//! a page texture; the same argument [`crate::viewer::strip`]'s header makes
//! about not caching the strip.
//!
//! ## Measured cost of the whole overlay
//!
//! On the same sheet, at its fit zoom of 1.3634 in a 2,200 × 1,300 window:
//!
//! | | |
//! |---|---|
//! | ruler ticks | 223, of which 22 are labelled |
//! | grid lines | 205 |
//! | tick walk + label formatting | **8.9 µs per frame** |
//! | `Scale::of`, twice | **7.3 µs per frame** |
//! | **total** | **≈16 µs, or 0.1 % of a 60 Hz frame** |
//!
//! Most of the 8.9 µs is the 22 `format_measurement` calls, each of which
//! allocates a `String`. That is the thing to attack if this ever needs to be
//! faster, and it does not: the page it is drawn over costs four orders of
//! magnitude more.
//!
//! ---
//!
//! ## ★ 2. Which space the grid is drawn in — page space, per page
//!
//! Under a continuous mode several pages are on screen at once, so "where is
//! the grid" has two candidate answers and only one survives contact with a
//! drafter.
//!
//! **A viewport-space grid** is drawn once, over the whole scrolling area, and
//! is anchored to the window. Scroll, and it slides across the paper: a line
//! that sat on an intersection comes off it, and the same feature on two
//! sheets falls at two different places in the grid. It is wallpaper, not a
//! reference. It is also cheaper and easier to write, which is why it is the
//! one to be careful about.
//!
//! **A page-space grid** is drawn per page, anchored to that page's own
//! top-left corner, and clipped to that page's rectangle. It scrolls *with*
//! the sheet, so an intersection is a fixed place on the drawing; every sheet
//! in a set gets the same grid in the same place relative to its own border;
//! and the gaps between pages carry no grid, which is truthful — there is no
//! paper there to be ruled.
//!
//! pdfcer draws the second, and the reason is what a grid is *for*. A drafter
//! uses one to judge alignment and spacing **on the drawing**, and every
//! answer read off it is a statement about the sheet. A grid not attached to
//! the sheet cannot make such a statement. The same argument settles the
//! guides, which is why [`super::guides`] stores a guide against a **page**.
//!
//! **Every numbered ruler tick has a grid line under it**, because both come
//! from the same 1-2-5 [`Ladder`] — which is the whole reason to ship a ruler
//! and a grid rather than two independent ornaments: a feature sitting on a
//! grid line can be read off the ruler without counting. Not the stronger
//! "every *heavy* grid line is numbered", which was the first claim and is not
//! true: both steps are 1-2-5 numbers and a 1-2-5 number is not always
//! divisible by a smaller one (500 over 200 is 2.5). See
//! [`tests::every_ruler_label_has_a_grid_line_under_it`].
//!
//! The two ladders are resolved by **different** constructors, and the
//! difference is a defect this feature shipped once and had measured out of
//! it: [`Ladder::for_labels`] bounds the *labelled* step, because that is what
//! must not overlap; [`Ladder::for_lines`] bounds the *drawn* step, because
//! every grid line is drawn. Using the first for the grid put a line every 1.4
//! screen pixels on the benchmark sheet — a tint rather than a grid — and no
//! screenshot and no test caught it.
//!
//! ---
//!
//! ## ★ 3. Rulers reserve layout space, and the reservation is a CONSTANT
//!
//! The gutters sit along the canvas edges, so switching them on shrinks the
//! viewport the strip is laid out into — and the viewport is what
//! [`crate::viewer::ViewState::apply_fit`] divides by, **on every frame a fit
//! mode is active**. That is rule **R128** (`app::status`'s header carries the
//! measured case): *a panel whose size feeds a fit-to-viewport computation has
//! a fixed size.* A content-driven gutter — one that grew to fit its widest
//! label — would be a measured feedback loop: a wider label on frame N is a
//! smaller fit scale on frame N+1 is a different label on frame N+2. pdfcer has
//! already watched a page shrink across three frames from exactly this shape.
//!
//! So [`THICKNESS_PTS`] is a constant, it is the *only* thing [`reserve`]
//! subtracts, and nothing here measures a string before deciding how much room
//! to take. Labels are laid out **inside** a gutter whose size was already
//! settled, and one that does not fit is clipped — the same posture the dock
//! takes with a panel body, for the same reason.
//!
//! Switching rulers on therefore costs exactly one re-fit, once, which is what
//! the operator asked for.
//! [`tests::the_gutters_are_a_constant_bite_out_of_the_viewport`] asserts the
//! constancy rather than trusting this paragraph.
//!
//! ---
//!
//! ## ★ 4. Rule 4 — why chrome the operator switched on is allowed
//!
//! `panels`' one-line test: *would a screenshot of the editing canvas differ
//! from a screenshot of the same document saved and reopened?* For a ruler and
//! a grid the answer is **yes, and the operator is the reason**. Rule 4
//! forbids pdfcer drawing *its own inferences* onto the page — a badge saying
//! "this text was OCRed", a dashed outline meaning "this bound is
//! approximate". Nothing here is keyed on any property of the content: the
//! grid's spacing comes from the zoom and the document's stated scale, never
//! from what is on the sheet; the ruler's numbers come from the page geometry,
//! never from an analysis of it; and both vanish the instant the operator
//! switches them off, which is the second half of what makes them chrome
//! rather than marking.
//!
//! The version that would fail the test is a grid that **snapped to something
//! pdfcer found** — a detected drawing frame, an inferred module size. There is
//! no such code here and there must not be: that is an inference, and an
//! inference owes an off-canvas report.
//!
//! ---
//!
//! ## What is in this file
//!
//! | item | subject |
//! |---|---|
//! | [`THICKNESS_PTS`], [`reserve`], [`Gutters`] | the constant bite out of the viewport, and the child `Ui` the canvas is drawn into |
//! | [`CanvasGeometry`] | what the frame learned about where its pages are, handed back so the gutters can be drawn against it |
//! | [`Scale`] | what unit the ruler reads in, read from the document |
//! | [`Ladder`], [`nice_step`] | the 1-2-5 tick ladder, chosen in display units and returned in points |
//! | [`draw`] | the gutters: ticks, labels, the page's own edges, the pointer |
//! | [`draw_grids`] | one grid per visible page, in that page's own space |

use egui::{Align, Layout, Pos2, Rect, Stroke, Ui, UiBuilder, pos2, vec2};
use pdfcer_core::dimension::{
    DEFAULT_GROUP_ID, NumberFormat, ScaleState, Unit, format_measurement,
};

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::strip::PageView;

/// The outer thickness of each ruler gutter, in egui logical points.
///
/// **A constant, and R128 is why** — see this module's header, §3. It is never
/// derived from a label's width, from the zoom, or from anything else that
/// varies per frame, because the viewport it is subtracted from feeds
/// [`crate::viewer::ViewState::apply_fit`].
///
/// 22 points holds the small text style with a point of padding either side,
/// plus the 6-point major tick that runs up from the inner edge. Chosen by
/// measuring the drawn result rather than by arithmetic: at 18 the labels
/// touched the ticks, and a number sitting on a line is a number the eye has
/// to disentangle from it.
pub(super) const THICKNESS_PTS: f32 = 22.0;

/// The shortest on-screen distance, in logical points, between two
/// **labelled** ticks.
///
/// The one number that decides the ladder: [`Ladder::for_labels`] walks the
/// 1-2-5 sequence upward until a step is at least this far apart on screen.
///
/// 76 rather than something tighter because core's formatter is *fixed-place*
/// — `format_measurement` renders `100.00 pt`, not `100 pt` — so a label is
/// routinely ten characters, which is ≈46 logical points at the small text
/// style. 76 leaves a 30-point gap between one label's end and the next
/// label's start. Labels that run together are worse than a coarser ladder:
/// the operator can always read an exact value off the pointer indicator, and
/// can read nothing at all off two overlapping numbers.
pub(super) const MIN_MAJOR_PITCH_PTS: f32 = 76.0;

/// A hard ceiling on the ticks or grid lines drawn along one axis.
///
/// [`MIN_GRID_PITCH_PTS`] and [`MIN_MAJOR_PITCH_PTS`] already bound the count
/// by the viewport, so this is unreachable in ordinary use. It exists because
/// the ladder divides by a zoom and by a scale, both of which arrive from
/// outside this module, and a degenerate value there would otherwise be a
/// frame that never finishes rather than a frame that draws slightly wrong. A
/// visible mistake beats a hang.
pub(super) const MAX_LINES: usize = 4_000;

/// How far a major tick runs in from the gutter's inner edge, in points.
const MAJOR_TICK_PTS: f32 = 6.0;

/// How far a minor tick runs in from the gutter's inner edge, in points.
///
/// Deliberately well under half [`MAJOR_TICK_PTS`] rather than a little under:
/// the two lengths are the *only* thing distinguishing a labelled tick from an
/// unlabelled one wherever the label has been clipped by the gutter's end, so
/// the difference has to survive a glance.
const MINOR_TICK_PTS: f32 = 2.5;

/// The alpha, out of 255, of the tint marking the page's own span on a ruler.
///
/// Low, because it is a *band the ticks and labels are drawn on top of* — see
/// [`draw`] on why it is a tint rather than the line it started as. High enough
/// that the paper's edge is findable at a glance on both shipped themes, which
/// is the whole job: at a fit zoom that edge is a one-pixel difference in fill
/// and the ruler is the only place it can be stated plainly.
const PAGE_SPAN_ALPHA: u8 = 40;

/// Named region: the horizontal ruler gutter, in window logical points.
///
/// Published so a screenshot oracle can crop the ruler out of a window capture
/// and ask the question a trace cannot answer — *is this legible, and is it
/// aligned with the page it is measuring?* See [`crate::diag::ui_rect`] on
/// naming: names are matched literally by checks, so renaming one silently
/// un-aims whatever was measuring it.
const REGION_RULER_TOP: &str = "ruler-top"; // ui-text-exempt: trace region name, never displayed

/// Named region: the vertical ruler gutter.
const REGION_RULER_LEFT: &str = "ruler-left"; // ui-text-exempt: trace region name, never displayed

// ---------------------------------------------------------------------------
// The reservation
// ---------------------------------------------------------------------------

/// The rectangles a ruler-bearing canvas is divided into.
///
/// `Copy` for the same reason [`PageMapping`] is: it is a fact about one
/// frame's layout, and one that outlived the frame would describe a canvas
/// that has since been resized.
///
/// With rulers off, `top`, `left` and `corner` are all `None` and `content`
/// **is** `outer` — so every downstream expression is exactly the one it was
/// before this feature. That is the same "the default path is unchanged, and
/// it is asserted rather than intended" discipline [`crate::viewer::strip`]
/// applies to single-page display.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Gutters {
    /// The whole region the canvas was given, gutters included.
    pub(super) outer: Rect,
    /// The region the strip is laid out into.
    pub(super) content: Rect,
    /// The horizontal ruler, along the top of the content.
    pub(super) top: Option<Rect>,
    /// The vertical ruler, down the left of the content.
    pub(super) left: Option<Rect>,
    /// The square where the two meet, above and left of the content.
    ///
    /// Held as a **rect** rather than left implicit because the two rulers
    /// must not overlap in it: a tick drawn into the corner by both would be
    /// drawn twice, at two different alphas, and would read as a defect at
    /// exactly the place the eye starts.
    pub(super) corner: Option<Rect>,
}

impl Gutters {
    /// A child [`Ui`] covering [`Self::content`], which the whole of the
    /// canvas is then drawn into.
    ///
    /// A child rather than a `scope_builder` closure so that
    /// [`super::show_in`] keeps its early `return`s meaning what they say.
    /// Inside a closure a `return` leaves the closure rather than the
    /// function — the kind of silent semantic change a re-indentation hides
    /// perfectly.
    ///
    /// The clip is **intersected** with the parent's rather than replacing it,
    /// so a canvas inside an already-clipped dock compartment stays clipped by
    /// both.
    pub(super) fn content_ui(self, ui: &mut Ui) -> Ui {
        let mut child = ui.new_child(
            UiBuilder::new()
                // ui-text-exempt: internal widget id, never displayed
                .id_salt("canvas-content")
                .max_rect(self.content)
                .layout(Layout::top_down(Align::Min)),
        );
        child.set_clip_rect(self.content.intersect(ui.clip_rect()));
        child
    }
}

/// Take the ruler gutters out of `ui`'s available space, and claim the whole
/// region in the parent's layout.
///
/// # Why the space is claimed here rather than at the end
///
/// [`super::show_in`] has four exits and only one reaches the bottom of the
/// function. Advancing the parent's cursor at each would be four places to
/// forget; advancing it once, up front, from a rect that is already known,
/// cannot be forgotten. Nothing else uses the parent `Ui` for layout
/// afterwards — only for painting, which does not consult the cursor.
///
/// # Why a degenerate canvas turns the rulers off rather than clamping them
///
/// A dock compartment dragged down to nothing, or a window mid-resize, can
/// leave less than two gutters' worth of room. Clamping would produce two
/// rulers and no canvas, a picture that says the application is broken.
/// Returning the no-ruler shape is the honest answer to *"there is not enough
/// room to draw this"*, it recovers by itself on the next frame that has room,
/// and — because the toggle is untouched — it does not silently countermand
/// the operator's choice.
pub(super) fn reserve(ui: &mut Ui, show: bool) -> Gutters {
    let outer = ui.available_rect_before_wrap();
    ui.advance_cursor_after_rect(outer);

    let t = THICKNESS_PTS;
    // Three gutters' worth on each axis: two for the rulers and at least one
    // more for the canvas between them. Below that the canvas is not a canvas.
    let room = outer.width() > t * 3.0 && outer.height() > t * 3.0;
    if !show || !room {
        return Gutters {
            outer,
            content: outer,
            top: None,
            left: None,
            corner: None,
        };
    }

    let content = Rect::from_min_max(outer.min + vec2(t, t), outer.max);
    Gutters {
        outer,
        content,
        top: Some(Rect::from_min_max(
            pos2(content.min.x, outer.min.y),
            pos2(content.max.x, content.min.y),
        )),
        left: Some(Rect::from_min_max(
            pos2(outer.min.x, content.min.y),
            pos2(content.min.x, content.max.y),
        )),
        corner: Some(Rect::from_min_max(outer.min, content.min)),
    }
}

// ---------------------------------------------------------------------------
// What the frame learned
// ---------------------------------------------------------------------------

/// The geometry the rulers and the guides are drawn against, produced by the
/// canvas once its scroll area has settled.
///
/// # Why this is handed back rather than read again
///
/// Because it is only knowable *inside* [`super::show_in`], after the scroll
/// area has laid out — the same reason `last_scroll_offset` is stored and the
/// same reason `strip_visible` is published during layout. Re-deriving it
/// outside would be a second answer to "where is the page", which is the
/// failure `canvas::mapping`'s whole header exists to prevent.
///
/// `None` from a frame that drew no page at all (no pages, a whole-canvas
/// render refusal, a strip whose visible window fell outside every page). The
/// gutters are still drawn in that state, empty: the chrome the operator asked
/// for is there and has nothing to measure. Hiding it instead would make the
/// canvas jump by 22 points on a page that failed to draw.
pub(super) struct CanvasGeometry {
    /// Every page the frame drew, with its screen ⟷ canvas map.
    pub(super) pages: Vec<PageView>,
    /// The page the frame's input was about — the one the ruler's zero is
    /// pinned to.
    pub(super) current: usize,
    /// The scroll viewport, in screen coordinates. Both the region a pointer
    /// must be inside to count as "over the canvas" and the extent a guide
    /// preview is drawn across.
    pub(super) viewport: Rect,
}

impl CanvasGeometry {
    /// The map for `page`, if the frame drew it.
    pub(super) fn map_of(&self, page: usize) -> Option<PageMapping> {
        self.pages.iter().find(|p| p.page == page).map(|p| p.map)
    }

    /// The map for the page the ruler's zero is pinned to, falling back to
    /// whatever was drawn first.
    ///
    /// The fallback is reachable for one frame after a mode change, when the
    /// scroll area has moved the strip but `view.page_index` has not caught up
    /// yet. A ruler measuring the wrong page for one frame is invisible; a
    /// ruler that vanishes for one frame is a flicker.
    pub(super) fn anchor(&self) -> Option<PageMapping> {
        self.map_of(self.current)
            .or_else(|| self.pages.first().map(|p| p.map))
    }

    /// The page under a screen point, if the pointer is over one.
    ///
    /// The truthful answer is `None` in the gaps between rows and in the
    /// centring margin either side of a narrow page, and [`super::guides`]
    /// relies on it: a guide dropped into the grey belongs to no page and is
    /// therefore not created.
    pub(super) fn page_at(&self, screen: Pos2) -> Option<(usize, PageMapping)> {
        self.pages
            .iter()
            .find(|p| p.map.image_rect().contains(screen))
            .map(|p| (p.page, p.map))
    }
}

// ---------------------------------------------------------------------------
// The unit
// ---------------------------------------------------------------------------

/// What the ruler reads in: the document's own measurement scale and number
/// format.
///
/// See this module's header, §1. `Copy`, and both fields are `pdfcer-core`
/// types — this struct adds no model of its own, it *selects* one the engine
/// already owns, which is what keeps the ruler and a dimension in agreement by
/// construction rather than by care.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Scale {
    state: ScaleState,
    format: NumberFormat,
}

impl Default for Scale {
    /// Raw PDF points — what a document that has never been calibrated reads
    /// in, and what every failure below falls back to.
    ///
    /// The unit carried alongside `NeverSet` is millimetres, not "points":
    /// `pdfcer-core` has no `Unit::Point`, because points are what a
    /// measurement is *before* a unit is chosen. `format_measurement` ignores
    /// the unit entirely in the never-set branch and renders ` pt`, so the
    /// value here is unobservable — it is `Millimeter` only because
    /// `DimensionModel::new` seeds its default group that way and agreeing
    /// with core costs nothing.
    fn default() -> Self {
        Self {
            state: ScaleState::NeverSet,
            format: NumberFormat::decimal(Unit::Millimeter, 2),
        }
    }
}

impl Scale {
    /// Read the document's scale from its dimensioning sidecar.
    ///
    /// The **default group**, which `pdfcer-core` guarantees always exists
    /// (`ui-spec` §5.3: *"a dimension always has a home and the group panel is
    /// never empty"*).
    ///
    /// ★ **This used to say "not an 'active' group, because the GUI has no
    /// group picker yet … when that surface lands, this is the one line that
    /// changes."* The surface landed on 2026-08-18 —
    /// `crate::dialogs::dimension_groups`' *Draw into* column, written through
    /// `crate::canvas::measure::set_active_group` — and **the line has
    /// deliberately not changed.** The prediction assumed the answer was
    /// obvious once the picker existed. It is not, and the two readings are
    /// both defensible:
    ///
    /// | follow the **default** group (today) | follow the **active** group |
    /// |---|---|
    /// | the ruler is page furniture, read while panning and reading, and a tool state left over from ten minutes ago is an arbitrary thing for it to depend on | the ruler and the ce dimension the operator is about to draw would **agree**, which is the ruler's stated purpose — *"a ruler and a dimension across one span agree to the digit"* |
    /// | a scale that changes because a radio moved in another window, with nothing on screen saying why, is a bug report | on a sheet with a 1:50 plan and a 1:5 detail, one fixed scale is wrong for half the sheet whatever it is |
    ///
    /// It is a **behaviour question for the operator**, not a gap, so it is
    /// recorded here rather than decided. If it is answered *active*, this is
    /// still the one line that changes — the function would take an
    /// `&egui::Context` and ask `measure::active_group`, and everything
    /// downstream is already scale-agnostic.
    ///
    /// A document whose sidecar is missing, unreadable or written by a newer
    /// build answers [`Scale::default`] — raw points. Every one of those means
    /// the same thing to a ruler (*nobody has told me what this drawing is
    /// scaled to*), and a preference is not worth an error path; the same
    /// posture `viewer::remembered::recall` takes.
    pub(super) fn of(doc: &OpenDoc) -> Self {
        doc.session
            .dimension_model()
            .group(DEFAULT_GROUP_ID)
            .map_or_else(Self::default, |g| Self {
                state: g.scale,
                format: g.format,
            })
    }

    /// Display units per PDF point.
    ///
    /// `1.0` when no scale is set, which is what makes the raw-points path the
    /// *same* arithmetic as every other path rather than a special case: the
    /// ladder is chosen in display units and converted back to points by
    /// dividing by this, and dividing by one is the identity.
    ///
    /// A non-finite or non-positive factor — reachable from a sidecar carrying
    /// a nonsense calibration — degrades to `1.0` rather than producing NaN
    /// tick positions. A ruler in the wrong unit is a legible mistake; a ruler
    /// whose ticks are all at NaN paints nothing and says nothing.
    pub(super) fn units_per_point(self) -> f64 {
        match self.state.effective_scale(self.format.unit) {
            Some(s) if s.is_finite() && s > 0.0 => s,
            _ => 1.0,
        }
    }

    /// Render a canvas-space distance the way a dimension of that length would
    /// be rendered.
    ///
    /// Straight through [`pdfcer_core::dimension::format_measurement`] — see
    /// the header, §1, on why this module formats nothing itself.
    pub(super) fn label(self, points: f64) -> String {
        format_measurement(points, self.state, self.format).text
    }
}

// ---------------------------------------------------------------------------
// The ladder
// ---------------------------------------------------------------------------

/// The tick spacing for one view: how far apart the labelled ticks are, and
/// how far apart the unlabelled ones are.
///
/// Both in **PDF points**, because that is the space every position downstream
/// is computed in. The 1-2-5 choice is made in *display* units — the numbers
/// the operator reads have to be round, and 100 mm at 1:50 is not a round
/// number of points — and converted back here, once.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Ladder {
    /// Distance between labelled ticks, in PDF points.
    pub(super) major: f64,
    /// Distance between unlabelled ticks, in PDF points.
    pub(super) minor: f64,
}

/// The smallest number of the form `1×10ⁿ`, `2×10ⁿ` or `5×10ⁿ` that is at
/// least `minimum`.
///
/// The universal ruler and axis ladder. 1-2-5 rather than 1-2-2.5-5 (Excel's)
/// or anything containing a 3, because those are the multiples an operator can
/// subdivide mentally: halves, fifths and tenths of the labelled step.
///
/// Returns `1.0` for a non-finite or non-positive input rather than
/// propagating it. Every caller then produces a ruler with the wrong spacing
/// instead of one with no ticks at all, and a wrong ruler is visible where an
/// empty one looks exactly like the feature being switched off.
#[must_use]
pub(super) fn nice_step(minimum: f64) -> f64 {
    if !minimum.is_finite() || minimum <= 0.0 {
        return 1.0;
    }
    let decade = 10f64.powf(minimum.log10().floor());
    // Guard the decade as well: `log10` of a subnormal underflows, and the
    // division below would then be an infinity no later comparison catches.
    if !decade.is_finite() || decade <= 0.0 {
        return 1.0;
    }
    let mantissa = minimum / decade;
    let chosen = if mantissa <= 1.0 {
        1.0
    } else if mantissa <= 2.0 {
        2.0
    } else if mantissa <= 5.0 {
        5.0
    } else {
        10.0
    };
    chosen * decade
}

/// `Some(v)` when `v` is finite and positive, `None` otherwise.
///
/// The one guard both [`Ladder`] constructors take against a degenerate zoom.
/// Written once because the two answer it identically and a version that
/// forgot would produce a ladder of NaN — which paints nothing at all, and is
/// therefore indistinguishable from the feature being switched off.
fn positive(v: f64) -> Option<f64> {
    (v.is_finite() && v > 0.0).then_some(v)
}

/// How many minor ticks one major step is divided into, given that step's
/// 1-2-5 mantissa.
///
/// Chosen so a minor tick always lands on a number the operator can name: 100
/// divides into ten tens, which keeps the halfway point on a tick; 200 divides
/// into four fifties, because 200/10 = 20 would put ticks at 20, 40, 60 …
/// under a label reading 200, and counting those is harder than not having
/// them at all.
fn minor_divisions(major: f64) -> f64 {
    let decade = 10f64.powf(major.log10().floor());
    if !decade.is_finite() || decade <= 0.0 {
        return 5.0;
    }
    match (major / decade).round() as i64 {
        2 => 4.0,
        5 => 5.0,
        // 1, and the 10 `nice_step` can return from a rounding edge.
        _ => 10.0,
    }
}

impl Ladder {
    /// **The ruler's ladder**: labelled ticks at least `min_pitch_pts` apart on
    /// screen.
    ///
    /// The derivation, in one line: a tick every `s` display units is
    /// `s / units_per_point` points is `s × zoom / units_per_point` logical
    /// points on screen, so the smallest acceptable display-unit step is
    /// `min_pitch × units_per_point / zoom`, and [`nice_step`] rounds that up
    /// to something an operator can read.
    ///
    /// The **minor** ticks then fall where [`minor_divisions`] puts them, and
    /// they may be as close as a point or two on screen — which is right for a
    /// ruler, where a fine comb between the numbers is exactly what you want
    /// to count against, and wrong for a grid. See [`Self::for_lines`].
    ///
    /// A degenerate zoom yields a one-point ladder, which the caller's own
    /// line-count bound then refuses to draw — rather than an infinity, which
    /// it would happily try to.
    #[must_use]
    pub(super) fn for_labels(scale: Scale, zoom: f32, min_pitch_pts: f32) -> Self {
        let upp = scale.units_per_point();
        let Some(zoom) = positive(f64::from(zoom)) else {
            return Self {
                major: 1.0,
                minor: 1.0,
            };
        };
        let major_units = nice_step(f64::from(min_pitch_pts) * upp / zoom);
        Self::from_major(major_units, upp)
    }

    /// ★ **The grid's ladder**: every *drawn line* at least `min_pitch_pts`
    /// apart on screen — the **minor** step, not the major.
    ///
    /// # Why this is a second constructor, and the defect that produced it
    ///
    /// The first version called [`Self::for_labels`] with
    /// [`MIN_GRID_PITCH_PTS`], which bounds the **labelled** step. On the
    /// benchmark A3 sheet at its fit zoom of 1.3634 that chose a 10-point major
    /// and therefore a **1-point minor** — a grid line every 1.4 screen pixels.
    ///
    /// That is not a grid, it is a tint, which is the exact failure
    /// [`MIN_GRID_PITCH_PTS`]'s own docs say the constant exists to prevent.
    /// It also drew about 2,450 lines a frame instead of about 250.
    ///
    /// **Measured, not spotted.** It survived a screenshot — a 1.4-pixel mesh
    /// over a drawing reads as a plausible fine grid — and it survived the
    /// suite, because `the_grid_is_finer_than_the_ruler_and_its_heavy_lines_line_up`
    /// asserted the grid was *finer*, which it emphatically was. What found it
    /// was printing the ladder the running application had actually chosen.
    ///
    /// # How it climbs
    ///
    /// The 1-2-5 rung whose minor step clears the pitch is not `nice_step` of
    /// anything simple, because the divisor changes with the mantissa: 100
    /// divides into tens, 200 into fifties, 500 into hundreds. So it climbs the
    /// sequence one rung at a time and stops at the first that clears — at most
    /// three iterations, because three consecutive rungs span a factor of ten
    /// and the divisor never exceeds ten.
    #[must_use]
    pub(super) fn for_lines(scale: Scale, zoom: f32, min_pitch_pts: f32) -> Self {
        let upp = scale.units_per_point();
        let Some(zoom) = positive(f64::from(zoom)) else {
            return Self {
                major: 1.0,
                minor: 1.0,
            };
        };
        let min_units = f64::from(min_pitch_pts) * upp / zoom;
        let mut major_units = nice_step(min_units);
        // Bounded rather than `loop`: the arithmetic terminates in at most
        // three steps, and a bound means a NaN that slipped past `positive`
        // costs a wrong grid rather than a hung frame.
        for _ in 0..8 {
            if major_units / minor_divisions(major_units) >= min_units {
                break;
            }
            major_units = nice_step(major_units * 1.5);
        }
        Self::from_major(major_units, upp)
    }

    /// A ladder from its major step in **display units**, converted to points.
    ///
    /// The one place the display-unit → point conversion happens, so the two
    /// constructors cannot disagree about which side of the division the scale
    /// goes on — a mistake that is invisible at 1:1, where `upp` is 1.
    fn from_major(major_units: f64, upp: f64) -> Self {
        let minor_units = major_units / minor_divisions(major_units);
        Self {
            major: major_units / upp,
            minor: minor_units / upp,
        }
    }

    /// The **index** of the first multiple of `step` at or after `from`.
    ///
    /// An index rather than the value, because [`Self::steps`] multiplies an
    /// integer index for the exactness reason that function documents. It is
    /// still split out because it is the one place a `ceil` could be a `floor`
    /// and nobody would notice: the ticks would simply start one step outside
    /// the view, which is invisible on a ruler and is a missing first line on a
    /// grid.
    fn first_index(step: f64, from: f64) -> f64 {
        (from / step).ceil()
    }

    /// ★ **Every minor tick between `from` and `to`, as `index × minor`.**
    ///
    /// The one walk the rulers and both grid axes share, and it multiplies an
    /// **integer index** rather than accumulating `value += minor`. That is a
    /// correction made from a screenshot of this running, and it fixed two
    /// things at once:
    ///
    /// 1. **The label at the page's top edge read `-0.00 pt`.** Repeated
    ///    addition from a negative start lands on `-1.8e-15` instead of zero,
    ///    which `format_measurement` renders with two decimals *and its sign*.
    ///    A ruler whose origin is labelled "minus zero" is a ruler the operator
    ///    has to stop and think about. Every test was green: the tick was in
    ///    the right place to well under a pixel, and the number was wrong.
    /// 2. **[`Self::is_major`] drifts.** Its tolerance exists because the
    ///    accumulated error grows without bound; from an exact multiple the
    ///    comparison is exact for any tick count a screen can hold.
    ///
    /// The residual `-0.0` — `(-0.15f64).ceil()` is negative zero, and
    /// `-0.0 * 10.0` is still negative zero — is normalised by the `+ 0.0`
    /// below, which is the one arithmetic identity that is *not* a no-op in
    /// IEEE 754: `-0.0 + 0.0 == 0.0`.
    ///
    /// Bounded by [`MAX_LINES`] so a degenerate ladder is a frame that draws
    /// slightly wrong rather than a frame that never finishes.
    pub(super) fn steps(self, from: f64, to: f64) -> impl Iterator<Item = f64> {
        let minor = self.minor;
        let first = Self::first_index(minor, from);
        let count = if minor > 0.0 && (to - from).is_finite() {
            (((to - from) / minor).floor() as i64 + 2).clamp(0, MAX_LINES as i64) as usize
        } else {
            0
        };
        (0..count)
            .map(move |i| (first + i as f64) * minor + 0.0)
            .take_while(move |v| *v <= to)
    }

    /// Whether `value` is a whole number of major steps from zero.
    ///
    /// Compared against a tenth of a minor step rather than exactly, because
    /// the tick walk accumulates `value` by repeated addition and an exact
    /// remainder test starts missing majors after a few hundred ticks —
    /// visible as a ruler that stops labelling halfway along.
    pub(super) fn is_major(self, value: f64) -> bool {
        if self.major <= 0.0 || self.minor <= 0.0 {
            return false;
        }
        let steps = value / self.major;
        (steps - steps.round()).abs() * self.major < self.minor * 0.1
    }
}

// ---------------------------------------------------------------------------
// Painting
// ---------------------------------------------------------------------------

/// Which quantity a ruler measures, or which way a grid line runs.
///
/// `pub(super)` because [`super::grid`] speaks the second reading; see below.
///
/// One enum for both, and the two readings are stated where each is used:
/// [`ticks`] takes the axis of the **quantity being measured** (the top ruler
/// measures canvas *x*), while [`grid_axis`] takes the axis the lines are
/// **spaced along** (vertical lines are spaced along *x*). They coincide,
/// which is why one enum serves; naming it for one of the two would make the
/// other read backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Axis {
    /// Canvas **x** — the top ruler, and the vertical grid lines.
    X,
    /// Canvas **y** — the left ruler, and the horizontal grid lines.
    Y,
}

impl Axis {
    /// A point whose component on this axis is `value` and whose other
    /// component is zero.
    ///
    /// Legal in **either** space, because the screen ⟷ canvas map is
    /// separable: `to_page` and `to_screen` compute x from x and y from y, so
    /// the component this axis does not care about cannot affect the one it
    /// does. That is what lets every position in this module be produced by
    /// handing a point to [`PageMapping`] rather than by a hand-rolled
    /// `origin + value * zoom` — which would compile, run, and drift the
    /// instant a page's drawn rect and its nominal zoom disagreed by a
    /// rounding, as they do on the frame a fit is settling. `canvas::mapping`
    /// exists precisely so there is no second place that divides by the zoom.
    pub(super) fn point(self, value: f32) -> Pos2 {
        match self {
            Axis::X => pos2(value, 0.0),
            Axis::Y => pos2(0.0, value),
        }
    }

    /// The component of a screen point that this axis controls.
    pub(super) fn of(self, p: Pos2) -> f32 {
        match self {
            Axis::X => p.x,
            Axis::Y => p.y,
        }
    }
}

/// Draw both ruler gutters.
///
/// # ★ Where the zero is, and why it is the page's top-left
///
/// The origin is the **current page's own top-left corner in canvas space** —
/// the same origin `canvas::mapping` calls canvas space and the same one the
/// `canvas-pointer` trace reports as `page=`. Three consequences, all wanted:
///
/// 1. **The ruler and the pointer readout agree.** They are the same number in
///    the same frame; a ruler with its own origin would be a second coordinate
///    system with nothing to say which one a value came from.
/// 2. **Y increases downward**, which is what Acrobat, InDesign, Illustrator
///    and every layout tool do, and which needs no flip — so the classic
///    silent Y-up/Y-down defect `mapping`'s header warns about cannot occur
///    here, because there is no conversion to get backwards.
/// 3. **Under a continuous mode the numbers restart at each sheet**, because
///    the zero follows `view.page_index`. That is right for the same reason
///    the grid is per page: each sheet is its own drawing with its own
///    coordinate system, and a ruler that kept counting through a 36-sheet set
///    would be measuring the scroll rather than the drawing.
///
/// The alternative — PDF user space, Y-up from the un-rotated CropBox corner —
/// is what an annotation `/Rect` is written in, and is deliberately *not* what
/// is shown: it disagrees with the pointer trace, it flips under `/Rotate`,
/// and it is a frame the operator never sees anywhere in this application.
pub(super) fn draw(ui: &Ui, doc: &OpenDoc, gutters: Gutters, geometry: Option<&CanvasGeometry>) {
    let (Some(top), Some(left), Some(corner)) = (gutters.top, gutters.left, gutters.corner) else {
        return;
    };
    let visuals = ui.visuals();
    let painter = ui.painter().with_clip_rect(gutters.outer);

    // The chrome itself, painted before anything is measured — so a frame that
    // drew no page shows two rulers rather than two holes.
    for rect in [top, left, corner] {
        painter.rect_filled(rect, 0.0, visuals.panel_fill);
    }
    let edge = visuals.widgets.noninteractive.bg_stroke;
    painter.hline(top.x_range(), top.max.y, edge);
    painter.vline(left.max.x, left.y_range(), edge);

    crate::diag::ui_rect(REGION_RULER_TOP, top);
    crate::diag::ui_rect(REGION_RULER_LEFT, left);

    let Some(map) = geometry.and_then(CanvasGeometry::anchor) else {
        return;
    };
    let scale = Scale::of(doc);
    let ladder = Ladder::for_labels(scale, doc.view.zoom, MIN_MAJOR_PITCH_PTS);
    let accent = visuals.selection.stroke.color;

    // ★ **The page's own span, as a TINT across the gutter** — and it is a tint
    // rather than the 2-point line it was in the first draft, which is a
    // correction made from a screenshot of this running.
    //
    // The single most useful thing a ruler can say about a drawing sheet is
    // where its borders are: at a fit zoom the paper's edge against the grey
    // surround is a one-pixel difference in fill, legible on a white sheet and
    // very nearly invisible on a dark theme. The first version said it with a
    // heavy line along the gutter's inner edge — and *that line sat exactly on
    // top of the ticks*, which run 2.5 points in from the same edge. Every
    // minor tick over the page, which is every tick that matters, was drowned
    // by the thing marking the page.
    //
    // A tint over the whole gutter says the same thing in a place nothing else
    // occupies, and it is the convention InDesign and Illustrator use for the
    // same purpose. Painted **before** the ticks so the ticks and labels sit on
    // top of it rather than under it.
    let page = map.image_rect();
    let tint = super::overlay::at_alpha(accent, PAGE_SPAN_ALPHA);
    painter.rect_filled(
        Rect::from_min_max(pos2(page.min.x, top.min.y), pos2(page.max.x, top.max.y)).intersect(top),
        0.0,
        tint,
    );
    painter.rect_filled(
        Rect::from_min_max(pos2(left.min.x, page.min.y), pos2(left.max.x, page.max.y))
            .intersect(left),
        0.0,
        tint,
    );

    ticks(ui, &painter, Axis::X, top, map, scale, ladder);
    ticks(ui, &painter, Axis::Y, left, map, scale, ladder);

    // Where the pointer is, on both rulers at once. A crosshair in the gutter
    // is how a ruler answers "how far across is this" without the operator
    // having to place a dimension to find out.
    if let Some(p) = ui.ctx().pointer_latest_pos()
        && geometry.is_some_and(|g| g.viewport.contains(p))
    {
        let stroke = Stroke::new(1.0, accent);
        painter.vline(p.x, top.y_range(), stroke);
        painter.hline(left.x_range(), p.y, stroke);
    }
}

/// Paint one ruler's ticks and labels.
///
/// Walks the **minor** ladder once and promotes a tick to major when
/// [`Ladder::is_major`] says so, rather than walking two ladders separately.
/// One walk means a major tick and the minor tick that would have coincided
/// with it can never land at two different positions through a rounding
/// difference — which is exactly the sort of half-point disagreement that
/// reads as a blurry ruler.
fn ticks(
    ui: &Ui,
    painter: &egui::Painter,
    axis: Axis,
    gutter: Rect,
    map: PageMapping,
    scale: Scale,
    ladder: Ladder,
) {
    let span = match axis {
        Axis::X => gutter.x_range(),
        Axis::Y => gutter.y_range(),
    };
    // The gutter's two ends, in canvas units. Through the mapping, so the only
    // arithmetic here is on values the one screen ⟷ canvas conversion
    // produced.
    let from = f64::from(axis.of(map.to_page(axis.point(span.min))));
    let to = f64::from(axis.of(map.to_page(axis.point(span.max))));
    if !from.is_finite() || !to.is_finite() || to <= from || ladder.minor <= 0.0 {
        return;
    }

    let visuals = ui.visuals();
    let minor_stroke = Stroke::new(1.0, visuals.weak_text_color());
    let major_stroke = Stroke::new(1.0, visuals.text_color());
    let font = egui::TextStyle::Small.resolve(ui.style());

    for step in ladder.steps(from, to) {
        let value = step;
        let at = axis.of(map.to_screen(axis.point(value as f32)));
        let major = ladder.is_major(value);
        let (length, stroke) = if major {
            (MAJOR_TICK_PTS, major_stroke)
        } else {
            (MINOR_TICK_PTS, minor_stroke)
        };
        match axis {
            Axis::X => painter.vline(at, gutter.max.y - length..=gutter.max.y, stroke),
            Axis::Y => painter.hline(gutter.max.x - length..=gutter.max.x, at, stroke),
        };
        if major {
            let galley =
                painter.layout_no_wrap(scale.label(value), font.clone(), major_stroke.color);
            match axis {
                // Two points clear of the tick, and the label starts AT the
                // tick rather than being centred on it. Centring reads better
                // in isolation and is wrong here: at the gutter's left end
                // half of a centred label is clipped away, and half a number
                // is worse than a number sitting slightly to the right of the
                // line it names.
                Axis::X => painter.galley(
                    pos2(at + 2.0, gutter.min.y + 1.0),
                    galley,
                    major_stroke.color,
                ),
                // A quarter turn anticlockwise, which is what every vertical
                // ruler does and what lets a 22-point gutter hold a
                // ten-character label at all. `with_angle` rotates about the
                // shape's own position, so the anchor is offset by the
                // galley's length to make the text run *up* from the tick.
                Axis::Y => {
                    painter.add(
                        egui::epaint::TextShape::new(
                            pos2(gutter.min.x + 1.0, at + galley.size().x + 2.0),
                            galley,
                            major_stroke.color,
                        )
                        .with_angle(-std::f32::consts::FRAC_PI_2),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The gutters take a CONSTANT bite out of the viewport** — R128.
    ///
    /// The property the whole of §3 is about, asserted rather than argued: the
    /// content rect's size depends only on the outer rect and
    /// [`THICKNESS_PTS`], never on anything that could vary with what is
    /// drawn. The test drives it across a range of outer sizes because the
    /// failure mode is a reservation that is *nearly* constant — one that
    /// scales with the canvas, say — which a single-size check would pass.
    #[test]
    fn the_gutters_are_a_constant_bite_out_of_the_viewport() {
        for (w, h) in [(400.0_f32, 300.0_f32), (1936.0, 1100.0), (91.0, 91.0)] {
            let outer = Rect::from_min_size(pos2(17.0, 29.0), egui::vec2(w, h));
            let g = gutters_for(outer, true);
            if g.top.is_none() {
                // Too small to hold rulers at all — the documented degenerate
                // answer, and it must leave the canvas whole.
                assert_eq!(g.content, outer);
                continue;
            }
            assert!(
                (g.content.width() - (w - THICKNESS_PTS)).abs() < f32::EPSILON,
                "the horizontal bite moved at {w}×{h}"
            );
            assert!(
                (g.content.height() - (h - THICKNESS_PTS)).abs() < f32::EPSILON,
                "the vertical bite moved at {w}×{h}"
            );
        }
    }

    /// With rulers off the canvas is byte-for-byte the rect it was handed.
    ///
    /// The mechanical form of "this feature does not change the default path".
    #[test]
    fn rulers_off_leaves_the_canvas_exactly_as_it_was() {
        let outer = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(1000.0, 800.0));
        let g = gutters_for(outer, false);
        assert_eq!(g.content, outer);
        assert_eq!(g.top, None);
        assert_eq!(g.left, None);
        assert_eq!(g.corner, None);
    }

    /// The three gutters tile the region they take, without overlapping.
    ///
    /// The corner exists precisely so the two rulers do not both draw into it
    /// — see [`Gutters::corner`] — and an overlap would be two ticks at two
    /// alphas at the place the eye starts reading.
    #[test]
    fn the_two_rulers_do_not_overlap_each_other_or_the_canvas() {
        let outer = Rect::from_min_size(pos2(5.0, 7.0), egui::vec2(900.0, 700.0));
        let g = gutters_for(outer, true);
        let (top, left, corner) = (
            g.top.expect("a top ruler"),
            g.left.expect("a left ruler"),
            g.corner.expect("a corner"),
        );
        for (a, b) in [(top, left), (top, corner), (left, corner)] {
            let hit = a.intersect(b);
            assert!(
                hit.width() <= 0.0 || hit.height() <= 0.0,
                "gutters overlap: {a:?} vs {b:?}"
            );
        }
        for r in [top, left, corner] {
            let hit = r.intersect(g.content);
            assert!(
                hit.width() <= 0.0 || hit.height() <= 0.0,
                "a gutter overlaps the canvas: {r:?}"
            );
        }
    }

    /// A degenerate canvas turns the rulers off rather than clamping them into
    /// a picture with no room left for a page.
    #[test]
    fn a_canvas_too_small_for_rulers_draws_none() {
        let tiny = Rect::from_min_size(Pos2::ZERO, egui::vec2(THICKNESS_PTS * 2.0, 400.0));
        assert_eq!(gutters_for(tiny, true).content, tiny);
        let flat = Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, THICKNESS_PTS));
        assert_eq!(gutters_for(flat, true).content, flat);
    }

    /// [`reserve`]'s geometry, without a `Ui`.
    ///
    /// The rects are a pure function of the outer rect and the toggle, and
    /// `reserve`'s only other effect is advancing the parent's cursor — so a
    /// headless twin can assert the whole of the geometry. Written as a
    /// re-derivation rather than by calling `reserve` because constructing an
    /// `egui::Ui` in a unit test needs a `Context` and a full frame, and a
    /// geometry test that needs a window is a geometry test nobody runs.
    fn gutters_for(outer: Rect, show: bool) -> Gutters {
        let t = THICKNESS_PTS;
        let room = outer.width() > t * 3.0 && outer.height() > t * 3.0;
        if !show || !room {
            return Gutters {
                outer,
                content: outer,
                top: None,
                left: None,
                corner: None,
            };
        }
        let content = Rect::from_min_max(outer.min + vec2(t, t), outer.max);
        Gutters {
            outer,
            content,
            top: Some(Rect::from_min_max(
                pos2(content.min.x, outer.min.y),
                pos2(content.max.x, content.min.y),
            )),
            left: Some(Rect::from_min_max(
                pos2(outer.min.x, content.min.y),
                pos2(content.min.x, content.max.y),
            )),
            corner: Some(Rect::from_min_max(outer.min, content.min)),
        }
    }

    /// ★ **The 1-2-5 ladder, exhaustively over one decade and across five.**
    ///
    /// The property: the answer is always of the form 1, 2 or 5 times a power
    /// of ten, and it is always at least the minimum asked for. Both halves
    /// matter — a ladder that rounded *down* would put labels closer together
    /// than [`MIN_MAJOR_PITCH_PTS`] allows, which is the overlapping-labels
    /// failure that constant exists to prevent.
    #[test]
    fn the_tick_ladder_is_one_two_or_five_times_a_power_of_ten() {
        for exp in -3..=3 {
            let decade = 10f64.powi(exp);
            for k in 1..=99 {
                let want_at_least = decade * f64::from(k) / 10.0;
                let step = nice_step(want_at_least);
                assert!(
                    step >= want_at_least - 1e-9,
                    "nice_step({want_at_least}) = {step} is finer than asked"
                );
                let m = step / 10f64.powf(step.log10().floor());
                assert!(
                    (m - 1.0).abs() < 1e-9 || (m - 2.0).abs() < 1e-9 || (m - 5.0).abs() < 1e-9,
                    "nice_step({want_at_least}) = {step} has mantissa {m}"
                );
            }
        }
    }

    /// Degenerate inputs produce a usable ladder rather than a NaN one.
    ///
    /// A ruler with the wrong spacing is a visible mistake; a ruler whose
    /// ticks are all at NaN paints nothing, which is indistinguishable from
    /// the feature being switched off.
    #[test]
    fn a_degenerate_ladder_still_produces_finite_ticks() {
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!((nice_step(bad) - 1.0).abs() < f64::EPSILON);
        }
        let scale = Scale::default();
        for zoom in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
            let l = Ladder::for_labels(scale, zoom, MIN_MAJOR_PITCH_PTS);
            assert!(l.major.is_finite() && l.major > 0.0, "zoom {zoom}");
            assert!(l.minor.is_finite() && l.minor > 0.0, "zoom {zoom}");
        }
    }

    /// ★ **The on-screen pitch of the labelled ticks stays inside its band at
    /// every zoom on the ladder.**
    ///
    /// The law the whole feature rests on, and the one a screenshot cannot
    /// check across fourteen zoom levels: labels never come closer than
    /// [`MIN_MAJOR_PITCH_PTS`], and — because the 1-2-5 sequence never jumps
    /// by more than 2.5× — never spread further than 2.5 times that either. A
    /// ruler whose labels drift apart until only two are visible is as useless
    /// as one whose labels overlap.
    #[test]
    fn labelled_ticks_keep_a_readable_pitch_at_every_zoom() {
        let scale = Scale::default();
        for &zoom in crate::viewer::ZOOM_LADDER {
            let l = Ladder::for_labels(scale, zoom, MIN_MAJOR_PITCH_PTS);
            let pitch = l.major * f64::from(zoom);
            assert!(
                pitch >= f64::from(MIN_MAJOR_PITCH_PTS) - 1e-6,
                "at {zoom}× the labels are {pitch} pt apart, closer than the minimum"
            );
            assert!(
                pitch <= f64::from(MIN_MAJOR_PITCH_PTS) * 2.5 + 1e-6,
                "at {zoom}× the labels are {pitch} pt apart, further than one ladder step"
            );
        }
    }

    /// The minor ticks divide the major step into a whole number of parts, so
    /// every major tick is also a minor one and none is drawn twice.
    #[test]
    fn every_major_tick_lands_on_a_minor_tick() {
        let scale = Scale::default();
        for &zoom in crate::viewer::ZOOM_LADDER {
            let l = Ladder::for_labels(scale, zoom, MIN_MAJOR_PITCH_PTS);
            let parts = l.major / l.minor;
            assert!(
                (parts - parts.round()).abs() < 1e-6,
                "at {zoom}× a major step is {parts} minor steps"
            );
            assert!(l.is_major(0.0), "zero is always a labelled tick");
            assert!(l.is_major(l.major * 7.0), "the seventh major step");
            assert!(!l.is_major(l.minor), "one minor step is not a label");
        }
    }

    /// ★ **A ruler with no scale set reads in points, and a calibrated one
    /// reads in its group's unit** — the header's §1 table, as a test.
    ///
    /// Asserted through [`Scale::label`] rather than through
    /// `units_per_point`, because the operator's question is what the *label
    /// says*, and the label is core's. A change in core's spelling should fail
    /// here rather than surprise someone reading a drawing.
    #[test]
    fn the_ruler_reads_points_until_the_document_says_otherwise() {
        let raw = Scale::default();
        assert!((raw.units_per_point() - 1.0).abs() < f64::EPSILON);
        assert_eq!(raw.label(144.0), "144.00 pt");
        assert_eq!(raw.label(-72.0), "-72.00 pt");

        // A sheet calibrated so one point is a quarter of a foot — the
        // worked example in `format_measurement`'s own documentation.
        let feet = Scale {
            state: ScaleState::Calibrated { scale: 0.25 },
            format: NumberFormat::decimal(Unit::DecimalFeet, 2),
        };
        assert_eq!(feet.label(144.0), "36.00 ft");
        assert!((feet.units_per_point() - 0.25).abs() < f64::EPSILON);

        // …and an explicit 1:1 metric group reads true size, which is a
        // different state from "never set" even though both are unscaled.
        let mm = Scale {
            state: ScaleState::OneToOne,
            format: NumberFormat::decimal(Unit::Millimeter, 2),
        };
        assert_eq!(mm.label(72.0), "25.40 mm");
    }

    /// ★ **A set scale moves the ladder into the operator's units**, so the
    /// numbers on the ruler are round in *their* system rather than in points.
    ///
    /// This is the whole point of §1 and it is the part a reader is most
    /// likely to doubt: at a 1:50 metric scale the ruler must label metres,
    /// not the awkward point values that happen to correspond to them.
    #[test]
    fn a_calibrated_sheet_gets_round_numbers_in_its_own_unit() {
        // One point = 0.0176 m, i.e. a sheet at roughly 1:50 in metres.
        let scale = Scale {
            state: ScaleState::Calibrated { scale: 0.0176 },
            format: NumberFormat::decimal(Unit::Meter, 3),
        };
        let l = Ladder::for_labels(scale, 1.0, MIN_MAJOR_PITCH_PTS);
        // The major step, converted back into the display unit, is round.
        let in_metres = l.major * scale.units_per_point();
        let m = in_metres / 10f64.powf(in_metres.log10().floor());
        assert!(
            (m - 1.0).abs() < 1e-6 || (m - 2.0).abs() < 1e-6 || (m - 5.0).abs() < 1e-6,
            "the ladder labelled {in_metres} m, which is not a round number"
        );
        // …and the same step in points is not round, which is exactly why the
        // ladder is chosen in display units rather than in points.
        assert!(
            (l.major - l.major.round()).abs() > 1e-9,
            "the point-space step happened to be round, so this asserts nothing"
        );
    }

    /// The tick walk starts inside the view rather than one step outside it.
    ///
    /// The index, not the value — see [`Ladder::steps`] on why the walk
    /// multiplies an integer.
    #[test]
    fn the_first_tick_index_lands_inside_the_view() {
        assert!((Ladder::first_index(50.0, -120.0) - -2.0).abs() < f64::EPSILON);
        assert!((Ladder::first_index(50.0, 100.0) - 2.0).abs() < f64::EPSILON);
        assert!((Ladder::first_index(50.0, 101.0) - 3.0).abs() < f64::EPSILON);
    }

    /// ★ **The origin is labelled `0.00 pt`, never `-0.00 pt`** — the defect a
    /// screenshot of the running binary found and no test had.
    ///
    /// The ruler's zero is the page's top-left corner, and a view scrolled so
    /// that the paper starts a little way into the gutter walks the ticks up
    /// from a negative value. Accumulating `value += minor` from there lands on
    /// `-1.8e-15` rather than on zero; core's formatter is fixed-place and
    /// prints the sign, so the operator reads *minus zero* at the origin of
    /// their drawing.
    ///
    /// Both halves are asserted, because the second is the one that surprises:
    /// an exact `0.0` is not enough — `(-0.15f64).ceil()` is **negative zero**,
    /// which multiplies to negative zero and formats with the sign — so
    /// [`Ladder::steps`] normalises it, and this catches a future refactor that
    /// drops the normalisation.
    #[test]
    fn the_origin_is_never_labelled_minus_zero() {
        let scale = Scale::default();
        let ladder = Ladder::for_labels(scale, 1.36, MIN_MAJOR_PITCH_PTS);
        // A gutter running from a little before the page's corner to well past
        // it — exactly the geometry the screenshot was taken in.
        for from in [-1.5_f64, -18.4, -0.001, -999.0] {
            let zeroes: Vec<f64> = ladder.steps(from, 900.0).filter(|v| *v == 0.0).collect();
            assert_eq!(zeroes.len(), 1, "from {from}: the origin must be a tick");
            assert!(
                !zeroes[0].is_sign_negative(),
                "from {from}: the origin came out as negative zero"
            );
            assert_eq!(scale.label(zeroes[0]), "0.00 pt");
        }
    }

    /// ★ **A long walk does not drift**, which is what makes the last tick on a
    /// wide sheet land where the first one promised.
    ///
    /// `index × step` rather than repeated addition. Asserted over a thousand
    /// steps because the error accumulates linearly and a dozen would not show
    /// it: at 1,000 minor ticks a naive walk is already off by enough for
    /// [`Ladder::is_major`]'s tolerance to start missing labels.
    #[test]
    fn a_long_tick_walk_stays_exact() {
        let ladder = Ladder {
            major: 100.0,
            minor: 10.0,
        };
        let ticks: Vec<f64> = ladder.steps(0.0, 10_000.0).collect();
        assert_eq!(ticks.len(), 1001);
        for (i, v) in ticks.iter().enumerate() {
            assert!(
                (v - (i as f64) * 10.0).abs() < 1e-9,
                "tick {i} drifted to {v}"
            );
        }
        // …and every hundredth is still recognised as a label, which is the
        // property the drift would have broken first.
        assert_eq!(ticks.iter().filter(|v| ladder.is_major(**v)).count(), 101);
    }

    /// The walk is bounded even when the ladder is degenerate, so a bad zoom is
    /// a frame that draws slightly wrong rather than one that never finishes.
    #[test]
    fn the_tick_walk_is_bounded() {
        let ladder = Ladder {
            major: 1.0,
            minor: 0.000_001,
        };
        assert!(ladder.steps(0.0, 1e9).count() <= MAX_LINES);
        let broken = Ladder {
            major: 0.0,
            minor: 0.0,
        };
        assert_eq!(broken.steps(0.0, 100.0).count(), 0);
        assert_eq!(ladder.steps(f64::NAN, 10.0).count(), 0);
    }
}
