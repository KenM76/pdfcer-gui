//! # `canvas::markup` — what a markup annotation IS, and the pen it is drawn with
//!
//! ## ★ The defect this module exists so that we never ship again
//!
//! The old shell's `canvas.rs` records it in the doc comment of the tool
//! variant this one is modelled on, and it is worth carrying across verbatim
//! because it is the reason a markup *substrate* exists at all rather than
//! eight commands that each insert a shape:
//!
//! > Until this variant existed, markup annotations did not go through the
//! > canvas at all: `Action::AddMarkupShape` called a function that derived a
//! > rectangle from the PAGE's own media box centre plus a per-author jitter,
//! > and inserted it. The shape therefore appeared in the middle of the page
//! > no matter where the operator had been pointing, and — because it never
//! > touched `active_tool` — it was invisible to every rule the other seven
//! > tools obey: Escape did not cancel it, it did not suppress the
//! > `ScrollArea`'s pan-by-drag, and it took no place in `TOOL_PRECEDENCE`.
//! > **The operator's report was exact: "they just drop things into the center
//! > of the pdf window."**
//!
//! Two things in that paragraph are the whole design brief. First, a markup
//! command must **arm a tool**, not perform an insertion — so that the shape
//! lands where the pointer is and so that every rule the canvas already has
//! about tools (Escape, cursor, pan suppression, the gesture machine's
//! press/drag/release) applies to it for free rather than being re-implemented
//! badly. Second, a markup that appears *somewhere the operator did not point*
//! is not a cosmetic complaint: it is the feature not working, and it passed
//! whatever tests it had because a shape really was added to the document.
//!
//! ---
//!
//! ## ★ THE SHAPE OF THIS MODULE CHANGED ON 2026-08-14, AND THIS IS WHY
//!
//! Until Ink, PolyLine and Polygon landed, this file held **one** family of
//! markup and its gesture together: the two-point rubber band, its preview, its
//! commit. That was right while there was one gesture. There are now **four**
//! families, and they differ in the only thing that matters here — *what the
//! operator does with the pointer*:
//!
//! | family | gesture | module |
//! |---|---|---|
//! | Rectangle · Ellipse · Arrow · Highlight | press, drag out a band, release | [`band`] |
//! | PolyLine · Polygon | click, click, click, then **say when** | [`vertex`] |
//! | Ink | press, follow the pointer, release | [`ink`] |
//! | Underline · StrikeOut · Squiggly | no pointer at all — the operand is a text selection | [`text`] |
//!
//! So the split is **by subject and not by line count**, and the subject is the
//! one this file is now left holding: *what a markup **is***. The kinds
//! ([`MarkupKind`]), the geometry an authored markup carries ([`Geometry`]), the
//! one place a gesture becomes a `MarkupSpec` ([`spec`]), the one place a
//! completed gesture becomes an `Action` ([`action`]), the refusals, and the
//! **pen** every family draws with. Each submodule answers *"how is this family
//! gestured?"* and none of them decides what it authors.
//!
//! That division is what keeps [`spec`] the single place a gesture becomes a
//! `MarkupSpec` — the property the whole equivalence argument rests on (§5
//! below) — while three genuinely different gestures feed it.
//!
//! ---
//!
//! ## The four obligations, in the shape [`crate::canvas::moving`] states them
//!
//! 1. **The geometry is PDF page space, never screen pixels.**
//!    [`band::endpoints`] and [`vertex::page_point`] are the only two places in
//!    this module tree that cross the boundary, and both do it through
//!    [`crate::viewer::canvas_to_pdf_space`] — the renderer's own transform —
//!    rather than by writing the Y-flip out again. A drag measured on screen and
//!    handed to `add_markup` compiles, runs, and merely scales with
//!    magnification: the same silent class as the hit-tolerance defect
//!    [`crate::canvas::mapping`] was built to make unavailable.
//! 2. **The preview must describe what the release will actually commit.**
//!    `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 welcomes a
//!    pre-commit affordance — *"a snap indicator, a hover highlight, a
//!    rubber-band, a selection handle — these are the cursor; they describe
//!    what is about to happen"* — and forbids marking content that has already
//!    been applied. A rubber band is squarely in the first category. But it is
//!    only honest if it is drawn **in the shape being authored**: an ellipse
//!    previewed as its bounding box, or an arrow previewed as a plain segment
//!    with no head, misdescribes the thing the operator is about to commit. So
//!    [`band::draw_preview`] draws an ellipse as an ellipse and an arrow with
//!    its head on; [`vertex::preview`] draws a polygon's **closing segment**,
//!    because that segment is in the file and a polyline's is not; and
//!    [`ink::draw_preview`] draws the **simplified** trail rather than the raw
//!    one, because the simplified trail is what lands. All three draw in the
//!    **pen's own colour** rather than in a chrome tint, because the pen colour
//!    is what will land in the file.
//! 3. **Escape abandons the gesture, and abandons exactly that.** A band drag
//!    and an ink drag are both a `DragKind` in [`crate::canvas::gesture`], so
//!    both are already Escape's claimant 1 — the *drag in flight* row of
//!    [`crate::canvas::keys`]'s precedence table — with no new mechanism and no
//!    second rule. A **vertex run** is a sequence of clicks and therefore has no
//!    drag for that claimant to cancel, exactly as a measure pick has none, so
//!    it takes its own rung beside [`crate::canvas::measure::abandon`]; see
//!    [`crate::canvas::keys`]'s header. Retiring the armed **tool** is a
//!    different act again and takes its own row.
//! 4. **An arrow keeps its RAW endpoints.** See [`spec`]. This is the one
//!    decision in the module that a reader will be tempted to "tidy up", and
//!    tidying it up silently reverses half of all arrows the operator draws.
//!
//! ## ★ A click with no drag places NOTHING for the band kinds, and that is a
//! ## decision
//!
//! The old shell answered the other way: `default_markup_at` (`main.rs:19770`)
//! turned a bare click into a 120 × 60 point box centred on the pointer, with
//! a `MIN_DRAG` of 4 **PDF points** below which a real drag was also treated as
//! a click. Neither half is carried across, and the reasons are specific rather
//! than a matter of taste:
//!
//! * **Its stated justification does not hold in this shell.** The old comment
//!   is explicit — the default box is *"obviously a placeholder the operator
//!   will resize (which slice 2 makes possible)"*. There is no slice 2 here.
//!   `EditSession` has the whole `move_*` family and **no scale or resize verb
//!   of any kind** ([`crate::canvas::handles`] consumes a grip drag and commits
//!   nothing), and an annotation is not even in the family those verbs address.
//!   So a default-sized box could not be resized, could not be moved, and could
//!   only be corrected by undoing it — which makes it not a placeholder but a
//!   wrong answer with a confident size.
//! * **The 4-point threshold is zoom-dependent in the wrong direction.**
//!   Measured in page space, 4 points is a 64 px screen drag at 16× — so a
//!   deliberate small mark on a title block would be silently replaced by a
//!   120 × 60 box, which is the same failure mode as the original centre-of-page
//!   defect wearing a smaller number. egui already applies the only threshold
//!   this gesture needs: a press-and-release that does not exceed **its** drag
//!   threshold is reported as `clicked` and never reaches a `DragKind` at all
//!   (see [`crate::canvas::gesture`]'s header). One threshold, in screen space,
//!   owned by the toolkit — exactly the argument
//!   [`crate::canvas::moving::PageDelta::is_travel`] makes for refusing a
//!   second one.
//!
//! What a click does instead is **nothing, out loud**: [`band::drag`] is never
//! reached, and the tool stays armed with its crosshair, so the operator's next
//! gesture — a drag — does what they asked. The cost is that a click is a
//! no-op; the alternative is authoring a shape nobody chose the size of and
//! cannot change.
//!
//! ★ **The two vertex kinds are the exception, and it is not an inconsistency**:
//! for them a click is the *whole* gesture, so of course it does something. The
//! rule above is about a gesture that has a drag and did not get one. See
//! [`vertex`]'s header, and [`crate::canvas::gesture::press_kind`], which gives
//! the vertex kinds a live click and **no drag at all** — the same shape it
//! gives the measure tools, and for the same reason.
//!
//! The same rule guards the degenerate *drag*: a press, a wander, and a release
//! back on the origin has zero extent on both axes, and
//! `pdfcer-core`'s `positive_rect` would quietly expand it to the 1-point
//! minimum — an invisible annotation holding a slot on the undo stack. That is
//! refused here as [`Refusal::NoExtent`] rather than committed, which is also
//! how the list-driven kinds' `EditError::EmptyGeometry` is kept off the
//! operator's screen: the shell never sends the engine geometry that draws
//! nothing, so the engine never has to refuse one. **Our guard is upstream of
//! theirs and is strictly the stricter of the two** — `validate_geometry`
//! accepts a `/Polygon` with two vertices, and [`action`] does not, because a
//! two-vertex closed polygon is a line drawn there and back and is not a shape
//! any operator meant to place.
//!
//! ## Which kinds are here, and which are deliberately not
//!
//! [`MarkupKind`] carries **seven**, in three gesture families, and the
//! remaining Phase 6 kinds are absent for reasons that are each different rather
//! than one blanket "later":
//!
//! | kind | where it is |
//! |---|---|
//! | Rectangle · Ellipse · Arrow · Highlight | here, gestured by [`band`] |
//! | ~~Polygon · PolyLine · Ink~~ | **Built 2026-08-14**, gestured by [`vertex`] and [`ink`] |
//! | ~~Underline · StrikeOut · Squiggly~~ | **Built 2026-08-14**, in [`text`], and they are still not variants of [`MarkupKind`] — see below |
//! | ~~Revision cloud~~ | **Built 2026-08-19.** Was *"blocked on the engine — `/BE` is never written"*, which stopped being true when `MarkupSpec::Cloud` shipped and nothing in this shell noticed for weeks. See [`MarkupKind::Cloud`]. |
//! | Plain line | The engine has `MarkupSpec::Line` and this shell spends it on Arrow. A second command differing only in its `/LE` is a Style question, not a kind. |
//! | Note · text box · sticky · stamp | Text-bearing, not geometric. A different gesture (place, then type) and a different spec type (`TextAnnotSpec`). |
//!
//! ### ★ The boundary this enum draws was RESTATED when the three new kinds
//! ### arrived, and the restatement is the useful part
//!
//! It used to read: *"a variant belongs in this enum when this rubber band can
//! draw it"*, and on that boundary Polygon, PolyLine and Ink were excluded in
//! terms — *"adding the variants now would put states into the type that no
//! `GestureOutcome` can reach."* That was exactly right **while the band was the
//! only gesture**, and it is the wrong boundary now, because the thing it was
//! really protecting was never the band: it was the pair of properties
//! `shell::commands::mapping` and `app::conditions` actually assert, namely that
//! **every variant has a command that arms this tool and a `selected:` condition
//! that lights while it is armed.**
//!
//! So the boundary is now stated as the property that is tested:
//!
//! > A variant belongs in [`MarkupKind`] when a `markup.*` command **arms the
//! > canvas tool with it**.
//!
//! All three new kinds clear that: each has a command, each arms
//! [`crate::canvas::tool::CanvasTool::Markup`], each renders pressed, and each
//! has a `GestureOutcome` that reaches it — a `DragKind::Markup` for Ink, and a
//! `GestureOutcome::Click` for the two vertex kinds, which is the same outcome
//! the measure tools have been reached by since Phase 7. Nothing about the old
//! sentence's *caution* is abandoned; only its proxy. The old wording is kept
//! above rather than deleted, because the mistake it guards against — variants
//! nothing can reach — is real, and the next reader adding a kind should be made
//! to show which control arms it.
//!
//! ## ★ The three text-markup kinds live in [`text`], and the boundary holds
//!
//! Underline, strikeout and squiggly shipped on 2026-08-14, and they are in a
//! submodule with an enum of their own rather than three variants here — which
//! is the boundary above being *applied* rather than being made an exception to,
//! under the new wording as much as the old. Their commands act at once and
//! **arm no tool at all**, so a `MarkupKind` variant would be a tool nothing can
//! arm, a pressed state that never lights, and a
//! [`crate::canvas::tool::CanvasTool`] state no
//! [`GestureOutcome`](crate::canvas::gesture::GestureOutcome) can reach.
//!
//! [`text`]'s own header carries the interaction decision — *select first, then
//! mark*, which is Acrobat's — and the mode intersection it produces.
//!
//! ## The names are the operator's, not the PDF specification's
//!
//! `Rectangle`/`Ellipse`/`Arrow` rather than `/Square`/`/Circle`/`/Line`. The
//! commands are `markup.rectangle`, `markup.ellipse` and `markup.arrow`, and
//! `text/commands.rs` calls them Rectangle, Ellipse and Arrow to the operator.
//! A type that spelled them the specification's way would make the ribbon, the
//! trace and the code disagree about the name of the same thing for no benefit;
//! the mapping to the subtype lives in exactly one place, [`spec`], where the
//! dictionary is built.
//!
//! ★ **`PolyLine` and `Polygon` are the exception, and they are the exception
//! because the operator's word and the specification's word are the same word.**
//! Bluebeam, Acrobat and every drafting office say "polyline" and "polygon";
//! there is no plainer name to prefer, so the rule above simply does not bite.
//! `Ink` is the specification's name for what the operator calls *freehand*, and
//! the **label** says Freehand (`text/commands.rs`) while the type says `Ink` —
//! which is the same split `Rectangle`/`/Square` makes, in the other direction.
//!
//! ## §5 — The split between the pure rules and the wiring
//!
//! [`spec`] and [`action`] are pure functions of plain data, so every rule above
//! is testable with no window and no document — the same discipline that makes
//! [`crate::canvas::moving::eligible`] and
//! [`crate::canvas::selection::SelectionState::click`] pure. The submodules'
//! entry points are the ones that touch the frame, and they do nothing except
//! gather inputs, call the pure functions in order, and trace what happened.
//!
//! **Nothing here builds an appearance stream.** [`spec`] hands `pdfcer-core` a
//! `MarkupSpec` and `EditSession::add_markup` does the rest, which is the same
//! route `pdfcer`'s `markup-add` takes with the same value — the equivalence
//! the measure salvage's tests exist to protect, and the reason a canvas-authored
//! annotation is byte-identical to a CLI-authored one.

use egui::{Color32, Pos2};
use pdfcer_core::annot_author::{Color, LineEnding, MarkupSpec, Quad, TextMarkupKind};
use pdfcer_core::page_tree::Rect as PageRect;

use crate::app::actions::Action;
use crate::canvas::mapping::PageMapping;

/// The two-point rubber band: Rectangle, Ellipse, Arrow, Highlight.
pub mod band;
/// Freehand: press, follow the pointer, release. `/Ink`.
pub mod ink;

/// ★★★ **Solid or dashed** — `/BS` `/S` and `/D` (§12.5.4, Table 166), as the
/// four choices this shell offers and the one reading it can only report.
///
/// `RIBBON_IA.md` §5.8's *Line style*, the eighth of that row's eight controls
/// and the only one that had **no engine verb at all** until the afternoon of
/// 2026-09-06. Three surfaces read it — the pen that authors, the Format ▸
/// Markup band and the Properties panel that restyle — and the module header
/// says why it is one list rather than three, why there is no phase control, and
/// why the shell refuses an unusable pattern by *offering only valid ones*
/// rather than by validating an entry it never takes.
pub mod linestyle;
/// Which markup gesture one drag reaches — band, freehand trail, or the
/// line-grouped quads of a highlight that found text. Split out of
/// `canvas::interact` under R2; its header carries the fallback ordering.
pub mod route;

/// ★★★ **Acrobat's own markup colours, measured** — the ten values Adobe
/// authors comments in, and the grid the Style swatch offers them from.
///
/// The data half of the operator's ask of 2026-09-06: *"make sure you've used
/// the same default colours and style look for these things as Adobe."* Every
/// number in it was read out of Acrobat DC's own tool-defaults registry rather
/// than chosen here; the module header carries the reading and the evidence.
pub mod palette;

/// ★ The colour and width the next markup is authored with — the **Style**
/// group `RIBBON_IA.md` §5.5 specifies and this shell shipped without.
///
/// §5.5 named the consequence in advance: *"Both must exist; today only the
/// first does, which is why a placed markup feels final."* This is the first.
pub mod pen;
/// Underline, strikeout and squiggly — the kinds whose operand is a text
/// selection rather than a pointer gesture. See this module's header for why
/// they are not [`MarkupKind`] variants.
/// ★ The Markup ▸ Style group's control — the `colour_swatch` custom item the
/// manifest declared at S2 and nothing ever drew, so the group rendered a
/// caption over an empty band.
pub mod swatch;

pub mod text;
/// The click-shaped kinds: PolyLine and Polygon, and the two endings that
/// finish them.
pub mod vertex;

/// Which markup annotation the markup tool is currently drawing.
///
/// Carried **by** [`crate::canvas::tool::CanvasTool::Markup`] rather than
/// becoming one tool variant per shape. See that variant's own docs for the
/// argument; the short form is that these are mutually exclusive states of one
/// mode, and a type that can express "the markup tool and the ellipse tool at
/// once" is the wrong shape for a thing that is exactly one of them.
///
/// **Seven variants in three gesture families** — see the module header's table
/// and the boundary that admits them. [`Self::is_band`], [`Self::is_vertex`] and
/// [`Self::is_freehand`] are the three predicates that name the families, and
/// they partition the enum: [`tests::the_three_families_partition_every_kind`]
/// is what says so, because a kind belonging to two families would be reached by
/// two gestures and a kind belonging to none would be armed by a control that
/// then does nothing at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupKind {
    /// `/Square` — a rectangle bounded by the drag. *"Drag from one corner to
    /// the other."*
    Rectangle,
    /// `/Circle` — an ellipse inscribed in the drag rectangle. *"Drag out the
    /// box it fits inside."*
    Ellipse,
    /// `/Line` with a head at the far end. *"Drag from the tail to the head."*
    Arrow,
    /// `/PolyLine` — an **open** run of segments. *"Click each corner; double-click
    /// the last."*
    ///
    /// See [`vertex`] for the gesture, its two endings, and what "open" costs the
    /// preview.
    PolyLine,
    /// `/Polygon` — the same run of clicks, **closed** back to the first vertex
    /// by the specification rather than by the operator.
    ///
    /// The difference from [`Self::PolyLine`] is *one segment in the file*, which
    /// is why they share a gesture, a state and a commit path and differ only in
    /// [`spec`] and in whether [`vertex::preview`] draws the closing segment.
    Polygon,
    /// A **revision cloud** — the same run of clicks as [`Self::Polygon`], with
    /// the cloudy border effect on it. *"Click each corner; double-click the
    /// last."*
    ///
    /// # ★ It is a `/Polygon` in the file, and that is the specification's doing
    ///
    /// There is no `/Cloud` subtype in ISO 32000. A revision cloud **is** a
    /// polygon whose border is drawn cloudy — Table 181 declares `/BE` on
    /// Polygon and PolyLine with the qualifier *"meaningful only for polygon
    /// annotations"* — so `pdfcer_core::annot_author::MarkupSpec::Cloud` writes
    /// `/Subtype /Polygon` and differs from `MarkupSpec::Polygon` only by
    /// `/BE << /S /C /I n >>` and by the baked appearance.
    ///
    /// Which is why this is a **seventh kind rather than a Style property of
    /// the sixth**, and the decision is worth stating because the file format
    /// argues the other way. `RIBBON_IA.md` §5.5 gives the revision cloud its
    /// own row in Markup ▸ Shapes, and it is right: an operator drawing a
    /// revision cloud is not drawing a polygon and then styling it, they are
    /// reaching for the one tool this audience names first. A control an
    /// AEC reviewer has to *discover* by styling something else is a control
    /// they will conclude is missing — which is exactly what happened, three
    /// times, in the operator's own words: *"still no revision cloud tool."*
    ///
    /// # What it shares with Polygon, and what it does not
    ///
    /// Everything except [`spec`]: the same [`vertex`] gesture, the same
    /// three-vertex floor, the same two endings, the same closing segment in
    /// the preview. [`Self::is_vertex`] answers `true` for it and every reader
    /// of that predicate needed no change, which is the evidence that it is the
    /// same gesture rather than a similar one.
    Cloud,
    /// `/Ink` — a freehand stroke that follows the pointer. *"Press and draw."*
    ///
    /// Drag-shaped like the band kinds, and **not** describable by two points:
    /// see [`ink`], which owns the trail, the simplification and the preview.
    Ink,
    /// `/Highlight` — a translucent band over the drag rectangle. *"Drag across
    /// what you want marked."*
    Highlight,
}

impl MarkupKind {
    /// Every variant, in the order the Markup ribbon tab lists them.
    ///
    /// Exists for the reason [`crate::app::actions::ViewChrome::ALL`] does, and
    /// is the same shape deliberately: it is what lets the *registry side* map
    /// a command id to a kind and back through one pair of total functions
    /// (`chrome_command` / `chrome_for_command` is the precedent), so an eighth
    /// kind added here fails a both-directions test rather than silently
    /// arriving with no command — or, worse, with a command that arms nothing.
    ///
    /// The order is the **ribbon's**, which is why Highlight is last: it sits in
    /// the Text markup band and the other six sit in Shapes.
    ///
    /// The mapping itself deliberately does **not** live here: command ids are
    /// `shell::commands`' vocabulary, and `shell/` is a single-writer resource.
    pub const ALL: &'static [MarkupKind] = &[
        MarkupKind::Rectangle,
        MarkupKind::Ellipse,
        MarkupKind::Arrow,
        MarkupKind::PolyLine,
        MarkupKind::Polygon,
        MarkupKind::Cloud,
        MarkupKind::Ink,
        MarkupKind::Highlight,
    ];

    /// Whether this kind is drawn by dragging a bounding **rectangle**, as
    /// opposed to a pair of endpoints.
    ///
    /// Salvaged from the old shell's `MarkupKind::is_rect`, and it earns its
    /// place for the same reason it did there: two separate decisions ask this
    /// one question — what shape the preview is, and whether the drag is
    /// normalised into a rect before it becomes a spec — and asking it as
    /// *"is this a rect kind?"* rather than *"is this Arrow?"* is what keeps
    /// both correct when a fifth kind arrives.
    #[must_use]
    pub fn is_rect(self) -> bool {
        matches!(self, Self::Rectangle | Self::Ellipse | Self::Highlight)
    }

    /// Whether this kind is gestured by the **two-point rubber band**.
    ///
    /// The predicate `canvas::interact` branches on to decide which of the three
    /// gesture modules takes a `GestureOutcome::Markup`, and the one [`band`]
    /// guards its own entry point with. Written as a question about the family
    /// rather than as `matches!(kind, Rectangle | Ellipse | Arrow | Highlight)`
    /// spelled at three call sites, for the reason [`Self::is_rect`] gives.
    #[must_use]
    pub fn is_band(self) -> bool {
        matches!(
            self,
            Self::Rectangle | Self::Ellipse | Self::Arrow | Self::Highlight
        )
    }

    /// Whether this kind is gestured by a **run of clicks** — PolyLine,
    /// Polygon and Cloud.
    ///
    /// Read by [`crate::canvas::gesture::press_kind`], which gives these two a
    /// live click and **no drag at all**, and by `canvas::interact`, which routes
    /// that click to [`vertex::click`] instead of to the selection. The two
    /// readers are the reason this is a method rather than a `matches!` in each:
    /// a press whose *meaning* and whose *routing* disagreed would be a click
    /// that placed a vertex and replaced the selection.
    #[must_use]
    pub fn is_vertex(self) -> bool {
        // ★ Cloud joins here and NOWHERE ELSE in this impl, which is the
        // property that made it a two-line change rather than a feature: every
        // reader of this predicate — `gesture::press_kind`'s live click,
        // `canvas::interact`'s routing away from the selection, `vertex`'s
        // whole state machine — is asking "is this a run of clicks", and the
        // cloud is one. The difference lives entirely in `spec`.
        matches!(self, Self::PolyLine | Self::Polygon | Self::Cloud)
    }

    /// Whether this kind follows the pointer freehand — Ink, and only Ink.
    ///
    /// A `bool` rather than a one-variant `Option` because there is nothing to
    /// carry: [`ink`] handles exactly one kind, and a second freehand kind would
    /// be a second `/InkList` subtype, of which the specification has none.
    #[must_use]
    pub fn is_freehand(self) -> bool {
        matches!(self, Self::Ink)
    }
}

/// The default border/stroke width, in PDF points, every geometric markup is
/// authored with.
///
/// ★ **No longer what a markup is authored at** — 2026-08-17. The pen control
/// landed and [`pen::Pen::width_pts`] is the value [`spec`] writes; this
/// constant survives as the **nominal** width, and it has exactly one consumer
/// left: [`ink::SIMPLIFY_TOLERANCE_PTS`], which derives a simplification
/// tolerance from a quarter of it.
///
/// That consumer is deliberately NOT re-pointed at the live pen, and the reason
/// is worth stating because the opposite looks obviously right. The tolerance
/// decides how much of a freehand trail is *thrown away*, so tying it to the
/// pen would mean the same gesture produced a different number of points
/// depending on a colour-and-width control the operator set for appearance —
/// and a 12 pt pen would discard six times as much of what they drew. The
/// simplification is about the fidelity of the recorded path; the pen is about
/// how that path is painted. Coupling them would make an appearance choice
/// silently destructive.
///
/// 2 points is the width a comment shape reads at on a dense CAD export
/// without dominating it — a hairline vanishes among the drawing's own 0.25 pt
/// linework, which is the specific failure a markup on an engineering drawing
/// has to avoid. It is [`pen::Pen::default`]'s width for the same reason.
pub const PEN_WIDTH_PTS: f64 = 2.0;

/// **The geometry one completed markup gesture produced**, in PDF user space.
///
/// # ★ Why one enum rather than three `Action` variants
///
/// Because [`spec`] is *"the single place a gesture becomes a `MarkupSpec`"*,
/// and that claim is what the whole equivalence argument rests on: a
/// canvas-authored annotation has to be byte-identical to the one
/// `pdfcer markup-add` writes, and the cheapest way to keep two things
/// identical is for there to be one of them. Three actions would be three apply
/// arms, each free to build its own spec, and the day one of them acquired a
/// normalisation the others did not is the day the claim quietly stopped being
/// true — with nothing to notice it, because every variant would still author a
/// perfectly valid annotation.
///
/// So the *kind* travels on [`Action::CommitMarkup`] exactly as it always did,
/// and what changed is that its geometry is no longer assumed to be two points.
///
/// Contrast [`Action::CommitTextMarkup`], which **is** a separate action and
/// stays one: its operand is not a gesture at all — it is a text selection that
/// already exists on the document — so it shares no rule with anything here. The
/// line this enum draws is *"produced by the pointer, on the canvas, now"*.
///
/// # Every variant is in PDF user space, and that is not a convention
///
/// It is the only frame in which an annotation has a place. Canvas-space
/// geometry stored here would be silently zoom-dependent — the class of defect
/// [`crate::canvas::mapping`]'s header exists to make unavailable — and the
/// conversion happens at exactly two places in this module tree, both named in
/// obligation 1 of the module header.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    /// The two **raw** endpoints of a rubber-band drag, in drag order.
    ///
    /// Un-normalised on purpose: [`spec`] normalises per kind, at the last point
    /// at which the raw pair is still available, because an arrow's head is at
    /// `end` and a normalised rect cannot say which corner the operator started
    /// at. See [`spec`]'s own ★ section.
    Band {
        /// Where the press landed. For [`MarkupKind::Arrow`] this is the **tail**.
        start: (f64, f64),
        /// Where the release landed. For [`MarkupKind::Arrow`] this is the **head**.
        end: (f64, f64),
    },
    /// A run of clicked vertices, in click order — PolyLine and Polygon.
    ///
    /// **Never carries the closing vertex for a polygon.** `/Polygon` closes
    /// back to the first entry of `/Vertices` by §12.5.6.13, so appending the
    /// first point again would author a duplicate vertex and a zero-length
    /// closing segment — visible on a rounded join as a blob, and invisible
    /// everywhere else, which is the worst of both.
    Vertices(Vec<(f64, f64)>),
    /// One or more freehand strokes — Ink, and only Ink.
    ///
    /// A list of lists because `/InkList` is one, even though the shipped
    /// gesture always produces exactly one stroke: [`ink`]'s header records that
    /// **one drag is one annotation**, and the outer list is the engine's shape
    /// rather than a promise about a gesture that does not exist yet.
    Strokes(Vec<Vec<(f64, f64)>>),
}

impl Geometry {
    /// Every coordinate this geometry carries, in no particular order.
    ///
    /// One iterator so the finiteness check in [`action`] is written once rather
    /// than three times — which matters more than it looks, because the failure
    /// of a *missed* variant is a NaN reaching an annotation's `/Rect` and the
    /// symptom is a document some readers refuse to open.
    fn coordinates(&self) -> impl Iterator<Item = f64> + '_ {
        // A boxed iterator rather than three branches at the call site: the arms
        // have three different concrete types and the alternative is repeating
        // the predicate per arm, which is the thing this exists to avoid.
        let it: Box<dyn Iterator<Item = f64> + '_> = match self {
            Self::Band { start, end } => Box::new([start.0, start.1, end.0, end.1].into_iter()),
            Self::Vertices(points) => Box::new(points.iter().flat_map(|&(x, y)| [x, y])),
            Self::Strokes(strokes) => Box::new(
                strokes
                    .iter()
                    .flat_map(|s| s.iter().flat_map(|&(x, y)| [x, y])),
            ),
        };
        it
    }
}

/// Why a markup gesture committed nothing.
///
/// Reported rather than silently absorbed, and reported with enough detail to
/// act on, because *"nothing happened"* has several causes with opposite
/// responses — the same argument [`crate::canvas::moving::Refusal`] makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The gesture ended where it began: no extent on either axis, or a vertex
    /// run every point of which is the same point. See the module docs on
    /// degenerate input.
    NoExtent,
    /// A coordinate was not finite. Refused rather than authored, because the
    /// alternative is a NaN in an annotation's `/Rect`.
    NotFinite,
    /// The page's device transform is not invertible, so there is no
    /// well-defined page-space position for the gesture. Declining is the only
    /// honest answer; authoring fabricated geometry is not.
    DegeneratePage,
    /// The frame has no page to author onto — a strip whose visible window fell
    /// outside every page, or a document whose pages have not loaded.
    NoPage,
    /// A vertex run too short for its kind: fewer than two for a `/PolyLine`,
    /// fewer than **three** for a `/Polygon`.
    ///
    /// ★ **This is the one place the shell is deliberately stricter than the
    /// engine.** `pdfcer-core`'s `validate_geometry` refuses `vertices.len() < 2`
    /// for both, so a two-vertex `/Polygon` would be accepted and authored: a
    /// closed shape drawn from A to B and back to A, which renders as a single
    /// line and is not a polygon anybody meant. The engine is right to accept it
    /// — it is legal PDF and a CLI caller may have reason — and the shell is
    /// right to refuse it, because a *gesture* that produced it is an operator
    /// who double-clicked one click early.
    TooFewVertices,
    /// The geometry does not describe the kind — a `Band` for `/Ink`, a
    /// `Vertices` for a `/Square`.
    ///
    /// **Structurally unreachable from any gesture**: each family builds its own
    /// [`Geometry`] variant beside the kind it belongs to, and the two are
    /// written on adjacent lines. Refused explicitly anyway, for the reason
    /// [`text::Refusal::NoQuads`] is: an [`Action`] is plain data, it can be
    /// constructed by a test or replayed by a future undo/redo surface, and the
    /// alternative to a named refusal is a panic or a silently wrong annotation.
    Mismatched,
}

/// The `/BE /I` intensity every revision cloud this shell authors carries.
///
/// `pdfcer-core` accepts any finite value in `0.0..=2.0` and refuses the rest by
/// name (`EditError::BorderEffectIntensityOutOfRange`), so this is a *choice*
/// inside a legal range rather than the only value that works.
///
/// **1.0 because that is Acrobat's default cloud.** The standing tie-breaker
/// for anything an operator will compare against the program they are replacing
/// is to make it behave the way that program does; a reviewer drawing the same
/// cloud in both must not be able to tell them apart by the size of the
/// scallop. It is a `const` rather than a literal in [`spec`] so the day a
/// Style control for it lands, the search for "what does this replace" finds
/// one name and one paragraph.
const CLOUD_INTENSITY: f64 = 1.0;

/// Build the `pdfcer-core` spec one markup gesture authors.
///
/// **The single place a gesture becomes a `MarkupSpec`** — the property the
/// equivalence with `pdfcer markup-add` rests on, and the reason [`Geometry`]
/// is one enum rather than three actions. Pure, and unit-tested, which is the
/// reason it is here rather than inline in the apply arm: the arm is a routing
/// line, and the decisions below are rules that deserve a test each.
///
/// Returns `None` for a kind/geometry pair no gesture constructs — see
/// [`Refusal::Mismatched`], which is where the apply arm's refusal is named.
///
/// # ★ An arrow keeps its RAW endpoints; a rectangle kind is normalised
///
/// Carried across from the old shell's `commit_markup` (`main.rs:5624-5627`),
/// which states it in one sentence: *"the direction the operator dragged is the
/// direction the line points, and its arrowheads make that visible.
/// Normalising here would silently flip half of all drawn arrows."*
///
/// It is sharper in this shell than it was there, because this shell's arrow
/// has **one** head rather than two. The old shell authored
/// `(OpenArrow, OpenArrow)` — a double-headed line, for which a reversal is
/// invisible. `text/commands.rs` already promises the operator *"drag from the
/// tail to the head"*, so the head belongs at the **end** of the drag, and with
/// a single head a normalised rect would put it on the wrong end of half of all
/// arrows drawn — up-and-left and up-and-right ones — with nothing in the
/// document to say the shell had reversed them.
///
/// The rectangle kinds go the other way and *must* be normalised: `Rect` with
/// `llx > urx` is not a rectangle any reader will draw, and the operator may
/// drag in any of the four directions.
///
/// # ★ A vertex run is never re-ordered, and neither is an ink stroke
///
/// The same rule as the arrow's, one dimension up. `/Vertices` and `/InkList`
/// are **sequences**, and their order is the order the operator drew: a
/// polyline's segments join consecutive entries, so sorting or normalising the
/// list would author a different figure from the one that was previewed. There
/// is nothing here that could tempt a tidy-up in the way `Rect::from_corners`
/// does, which is precisely why it is written down.
///
/// # Neither vertex kind is filled
///
/// `/Polygon` accepts an `/IC` interior and this authors `None`, for the reason
/// the Square and Circle arms already give: a filled comment shape hides the
/// drawing it is a comment about, which on a CAD sheet is the whole content
/// under it. A fill is a Style property (`markup.fill`, still in `PLANNED`) and
/// belongs to the surface that will set the pen colour too.
#[must_use]
pub fn spec(kind: MarkupKind, geometry: &Geometry, pen: pen::Pen) -> Option<MarkupSpec> {
    // ★ The pen is a PARAMETER as of 2026-08-17, and this is the seam
    // `MarkupKind::rgb`'s own doc comment named in advance: *"give it a colour
    // and a width from the document's markup state and nothing else in the
    // module changes."* Nothing else in this module did.
    let (r, g, b) = pen.colour_for(kind);
    let color = Color::Rgb(r, g, b);
    let width = pen.width_pts;
    match (kind, geometry) {
        (
            MarkupKind::Rectangle | MarkupKind::Ellipse | MarkupKind::Arrow | MarkupKind::Highlight,
            Geometry::Band { start, end },
        ) => {
            let rect = PageRect::from_corners(
                start.0.min(end.0),
                start.1.min(end.1),
                start.0.max(end.0),
                start.1.max(end.1),
            );
            Some(match kind {
                MarkupKind::Rectangle => MarkupSpec::Square {
                    rect,
                    border: Some(color),
                    // No fill. A filled comment shape hides the drawing it is a
                    // comment about, which on a CAD sheet is the whole content
                    // under it.
                    interior: None,
                    border_width: width,
                    // ★ `/BE` — the CLOUDY border, which `pdfcer-core` gained on
                    // 2026-08-18 (Pass 82.0). `None` is a plain rectangle,
                    // which is what this tool has always drawn and what the
                    // Markup ▸ Shapes ▸ Rectangle control promises.
                    //
                    // Named explicitly rather than absorbed by a struct update,
                    // for the reason `to_engine_settings` states about the same
                    // situation in the print adapter: a `..Default::default()`
                    // would have taken this field silently and would take the
                    // NEXT one too, which is how a shell comes to ignore a
                    // capability the engine grew for it. The compile error this
                    // replaced is the whole value of naming every field.
                    //
                    // The cloud is a SEPARATE control — `RIBBON_IA.md` §5.5's
                    // revision-cloud row — and giving Rectangle a cloudy border
                    // would change what a shipped control draws without asking.
                    border_effect: None,
                },
                MarkupKind::Ellipse => MarkupSpec::Circle {
                    rect,
                    border: Some(color),
                    interior: None,
                    border_width: width,
                },
                // ★ RAW `start` and `end` — see this function's docs.
                MarkupKind::Arrow => MarkupSpec::Line {
                    start: *start,
                    end: *end,
                    color,
                    width,
                    // Tail then head, in the operator's own words. `None` at the
                    // start is what makes the raw-endpoint rule above
                    // load-bearing rather than decorative.
                    endings: (LineEnding::None, LineEnding::OpenArrow),
                },
                // Exactly one quad, always, so `validate_geometry`'s empty-quad
                // refusal is structurally unreachable from this path.
                _ => MarkupSpec::TextMarkup {
                    kind: TextMarkupKind::Highlight,
                    quads: vec![Quad::from_rect(rect)],
                    color,
                },
            })
        }
        (MarkupKind::PolyLine, Geometry::Vertices(vertices)) => Some(MarkupSpec::PolyLine {
            vertices: vertices.clone(),
            color,
            width,
        }),
        (MarkupKind::Polygon, Geometry::Vertices(vertices)) => Some(MarkupSpec::Polygon {
            vertices: vertices.clone(),
            border: Some(color),
            interior: None,
            width,
        }),
        // ★ **The revision cloud**, and the ONLY line in this module that
        // distinguishes it from the arm above.
        //
        // `intensity` is the whole of the difference in the file. Table 167's
        // `I` row is typed `number` and constrained *"in the range 0 to 2"* —
        // a CONTINUOUS range, not the enumeration `{0, 1, 2}` it is routinely
        // mis-read as; `pdfcer-core`'s `MarkupSpec::Cloud` carries the evidence
        // (the sibling `S` row in the same four-line table uses the standard's
        // enumeration idiom, and `I` does not). So a shell may pick any value
        // in the range and 1.0 is a choice rather than the only legal one.
        //
        // **1.0 is Acrobat's own default cloud**, which is the whole argument.
        // The standing tie-breaker for anything an operator will compare
        // against the program they are replacing is *make it work the way the
        // other program does*, and a reviewer who draws a cloud in pdfcer and a
        // cloud in Acrobat over the same drawing must not be able to tell which
        // is which by the size of the scallop.
        //
        // It is deliberately **not** exposed as a control. `markup.line_width`,
        // `markup.fill` and `markup.opacity` are all in
        // `crate::shell::manifest::PLANNED` for the same reason — Style sets the
        // NEXT markup's properties and only colour has a control today — and a
        // cloud-intensity slider arriving before a line-width one would be this
        // shell offering the ninth-most-wanted property first.
        (MarkupKind::Cloud, Geometry::Vertices(vertices)) => Some(MarkupSpec::Cloud {
            vertices: vertices.clone(),
            border: Some(color),
            // No fill, for the reason every other arm here gives: a filled
            // comment shape hides the drawing it is a comment about, and on a
            // revision cloud that is the *revision* — the thing the cloud was
            // drawn to draw attention to.
            interior: None,
            width,
            intensity: CLOUD_INTENSITY,
        }),
        (MarkupKind::Ink, Geometry::Strokes(strokes)) => Some(MarkupSpec::Ink {
            strokes: strokes.clone(),
            color,
            width,
        }),
        // Every remaining pair is a kind holding another family's geometry. See
        // `Refusal::Mismatched`: unreachable from a gesture, refused rather than
        // guessed.
        _ => None,
    }
}

/// [`spec`] with the shipped pen — **for tests and for `apply`'s own
/// falsifier only.**
///
/// Not a convenience. It exists so a test that is about *geometry* — a
/// placement, a normalisation, a refusal — does not have to state a colour it
/// does not care about, and so the one place that reads it in non-test code
/// (`apply`'s D9 falsifier, which re-authors a known rectangle to compare
/// against) is visibly not the operator's pen.
///
/// Production paths take the pen from the action they are applying. That is
/// checked by there being no other caller.
#[must_use]
pub fn spec_default_pen(kind: MarkupKind, geometry: &Geometry) -> Option<MarkupSpec> {
    spec(kind, geometry, pen::Pen::default())
}

/// The ONE action a completed markup gesture becomes.
///
/// Pure, and the only place the degenerate-input rules are applied. Deliberately
/// says nothing about *which* page is current or what the pen is: those are the
/// caller's and [`spec`]'s respectively, so this function is a statement about
/// the gesture alone and can be tested as one.
///
/// # The three degeneracy rules, and why each has to be here rather than in
/// # `pdfcer-core`
///
/// | rule | refused as | what it prevents |
/// |---|---|---|
/// | any coordinate non-finite | [`Refusal::NotFinite`] | a NaN in an annotation's `/Rect` |
/// | a band with identical endpoints, or a vertex/ink run with no extent at all | [`Refusal::NoExtent`] | a 1-point mark nobody can see, holding a slot on the undo stack |
/// | a run too short for its kind | [`Refusal::TooFewVertices`] | a two-vertex "polygon" that renders as a line |
///
/// The engine refuses the *empty* cases and only those. Everything above is
/// about a gesture the operator could actually produce, and refusing it here is
/// what keeps `EditError::EmptyGeometry` off their screen: the shell never
/// sends geometry that draws nothing, so the engine never has to explain one.
///
/// # Why the geometry travels un-normalised
///
/// Because normalising here would destroy the arrow's direction before anything
/// downstream could ask about it, and would re-order a vertex run into a figure
/// nobody drew. Normalisation happens in [`spec`], per kind, at the moment the
/// `Rect` is built — the last point at which the raw data is still available.
pub fn action(
    kind: MarkupKind,
    page: usize,
    geometry: Geometry,
    pen: pen::Pen,
) -> Result<Action, Refusal> {
    if !geometry.coordinates().all(f64::is_finite) {
        return Err(Refusal::NotFinite);
    }
    match (&geometry, kind) {
        (Geometry::Band { start, end }, k) if k.is_band() => {
            // ★ No second threshold, and none in page space. egui's own drag
            // threshold has already separated a click from a drag in SCREEN
            // space; all that is refused here is a drag that ended exactly where
            // it began, which would author a 1-point mark nobody can see. See
            // the module docs.
            if start == end {
                return Err(Refusal::NoExtent);
            }
        }
        (Geometry::Vertices(points), MarkupKind::PolyLine) => {
            if points.len() < 2 {
                return Err(Refusal::TooFewVertices);
            }
            if all_the_same(points) {
                return Err(Refusal::NoExtent);
            }
        }
        // ★ Polygon and Cloud share this arm, and sharing it is the assertion.
        //
        // A cloud IS a polygon in the file — `MarkupSpec::Cloud` writes
        // `/Subtype /Polygon` and differs only by `/BE` — so a vertex run that
        // is too short for one is too short for the other, by exactly the same
        // argument. Two arms with the same body would be two places for that to
        // stop being true.
        //
        // The engine agrees on the floor for the cloud and not for the polygon:
        // `validate_geometry` refuses `vertices.len() < 3` for `Cloud`
        // (Pass 82.1's *"a two-vertex cloud is a line pretending to be an
        // area"*) and `< 2` for `Polygon`. So this arm is redundant for one
        // kind and load-bearing for the other, and it is written once because
        // the SHELL's reason — the module header's "stricter than the engine"
        // note — is the same for both: a *gesture* that produced two vertices
        // is an operator who double-clicked one click early.
        (Geometry::Vertices(points), MarkupKind::Polygon | MarkupKind::Cloud) => {
            // Three, not two — the module header's "stricter than the engine"
            // note, and `Refusal::TooFewVertices`' own docs.
            if points.len() < 3 {
                return Err(Refusal::TooFewVertices);
            }
            if all_the_same(points) {
                return Err(Refusal::NoExtent);
            }
        }
        (Geometry::Strokes(strokes), MarkupKind::Ink) => {
            // A stroke of one point draws nothing at all: `ink`'s builder emits
            // a `move_to` and then paints, which strokes zero length. The engine
            // would accept it (its guard is `strokes.iter().all(Vec::is_empty)`)
            // and the operator would get an invisible annotation and an undo
            // step. Refused as NoExtent, which is the same fact the band kinds'
            // zero-length drag reports.
            if strokes.iter().all(|s| s.len() < 2) {
                return Err(Refusal::NoExtent);
            }
            if strokes.iter().all(|s| all_the_same(s)) {
                return Err(Refusal::NoExtent);
            }
        }
        // A kind holding another family's geometry — see `Refusal::Mismatched`.
        _ => return Err(Refusal::Mismatched),
    }
    Ok(Action::CommitMarkup {
        page,
        kind,
        geometry,
        pen,
    })
}

/// Whether every point in a run is the same point.
///
/// The vertex and ink form of *"the drag ended where it began"*. A run of forty
/// identical points is what a press-and-hold with no movement produces once the
/// duplicate filter is off, and it authors an annotation with a zero-area
/// `/Rect` that `pdfcer-core`'s `bounds_of` then pads to the pen's half-width —
/// a 1-point blob nobody chose.
fn all_the_same(points: &[(f64, f64)]) -> bool {
    points.first().is_none_or(|first| {
        points
            .iter()
            .all(|p| (p.0 - first.0).abs() < f64::EPSILON && (p.1 - first.1).abs() < f64::EPSILON)
    })
}

/// Report a markup gesture that committed nothing, with the reason.
///
/// One trace shape for every refusal, so a harness reads `markup-declined` and
/// finds the cause on the same line rather than inferring it from an absence —
/// the contract `canvas-move-declined` and `canvas-delete-declined` already
/// honour. Shared by all three gesture modules, so the channel carries one line
/// shape whichever family declined.
pub(crate) fn decline(kind: MarkupKind, page: usize, reason: Refusal) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("markup-declined kind={kind:?} page={page} reason={reason:?}")
    });
}

/// Report a markup that is about to be authored, with its geometry.
///
/// ★ **Traced with numbers, not a success flag.** The old shell's own note says
/// why, and it is the sharpest sentence in that file: *"the whole defect this
/// Pass fixes was a shape landing somewhere the operator did not choose, and a
/// trace saying only 'committed' would have been equally true before and after
/// the fix."*
///
/// What each family puts on the line is what can be *wrong* about it: the band
/// kinds print their raw endpoints in drag order, so a harness can prove the
/// arrow's head is at the end the operator dragged to; the vertex kinds print
/// the vertex count and the first and last vertex, so a run that lost its ends
/// or gained a duplicate closing point is visible; ink prints the raw and kept
/// point counts, which is the only place the simplification's effect is
/// observable from outside the process.
pub(crate) fn trace_commit(kind: MarkupKind, page: usize, detail: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("markup-commit kind={kind:?} page={page} {detail}")
    });
}

/// The pen's width in **screen** points at this frame's magnification.
///
/// Derived by measuring the mapping rather than by asking it for a zoom:
/// [`PageMapping`] has no `zoom()` accessor, deliberately, because everything
/// that divided by one was a place a second division could hide (see that
/// module's header). Projecting a one-unit page-space step and measuring what
/// arrives is the same answer with no number to divide by, and it keeps the
/// preview's thickness equal to the stroke that will actually land.
///
/// Floored at one point so the band is never invisible at low zoom — a preview
/// the operator cannot see is a preview they cannot aim.
///
/// Shared by all three gesture modules: the pen is a property of the *markup*,
/// not of the gesture that draws it, so a second copy here would be a second
/// place the preview could stop matching the annotation.
pub(crate) fn pen_px(mapping: &PageMapping, pen: pen::Pen) -> f32 {
    #[allow(clippy::cast_possible_truncation)]
    let width = pen.width_pts as f32;
    let scale = mapping.to_screen(Pos2::new(1.0, 0.0)).x - mapping.to_screen(Pos2::ZERO).x;
    if scale.is_finite() && scale > 0.0 {
        (width * scale).max(1.0)
    } else {
        width.max(1.0)
    }
}

/// The pen colour, as egui sees it.
pub(crate) fn pen_color(kind: MarkupKind, pen: pen::Pen) -> Color32 {
    let (r, g, b) = pen.colour_for(kind);
    let byte = |v: f64| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let out = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        out
    };
    // DOCUMENT COLOUR: the operator's pen, converted for the preview from the
    // exact components `spec` writes into `/C`. Deriving it from one source
    // rather than naming a second is what keeps the band the colour of the
    // thing it is previewing.
    Color32::from_rgb(byte(r), byte(g), byte(b))
}

#[cfg(test)]
mod tests;
