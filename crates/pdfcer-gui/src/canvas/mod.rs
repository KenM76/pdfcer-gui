//! # canvas — the page on screen, what is selected on it, and the gestures that move both
//!
//! The one place a rasterized page is drawn and the one place canvas input is
//! read. The navigation gestures — **wheel to scroll, Ctrl+wheel to zoom about
//! the cursor, middle-drag to pan**, and from Phase 3 the **hand tool with
//! space-to-pan**, **anchored discrete zoom**, **zoom to selection** and
//! **marquee zoom to region** — and, from stage S4, the **selection model**:
//! click, Shift+click, double-click to descend, Escape to ascend, rubber-band
//! marquee, eight grips plus move, **dragging a selection to move it**, and
//! Delete.
//!
//! ## Where the selection model lives
//!
//! | module | subject |
//! |---|---|
//! | [`mapping`] | the ONE screen⟷page conversion, and the hit tolerance |
//! | [`target`] | the provider seam, and the trait re-attached to the salvaged decomposition |
//! | [`selection`] | selection as identity; the level ladder; re-resolution |
//! | [`forms`] | filling a form where it is drawn: why it is not a tool, why its hit test takes no tolerance, and what its editor cannot promise |
//! | [`gesture`] | press / drag / release, the clear that must not happen on a press, Escape's abort, and the one rubber band's two intents |
//! | [`moving`] | which move verb each rung reaches, the canvas→page delta, and the ghost's honesty rule |
//! | [`handles`] | eight grips plus move, and the cursor over each |
//! | [`textsel`] | selecting **text**: why it needs no capability, why it is offered exactly where content selection is not, and the single pass that makes the highlight and the copy one value |
//! | [`menus`] | the right-click: which of the two canvas menus opens, and the select-first rule that makes it about the thing you pointed at |
//! | [`overlay`] | what all of it looks like — and what rule 4 forbids it looking like |
//! | [`geometry`] | the pan and zoom-anchor arithmetic |
//! | [`keys`] | Escape and Delete, and which of Escape's three claimants gets it |
//! | [`tool`] | select or hand, and the space bar that borrows the hand |
//! | [`zoom`] | **the anchor rule**, the two-frame handshake, and the five zoom paths that route through it |
//! | [`interact`](mod@interact) | **what the operator just did, and what happens as a result** — the pointer frame, the gesture application, the right-click, the keys, the re-resolve, the cursor |
//!
//! Everything above is pure except this file, [`interact`](mod@interact),
//! [`overlay`], and [`moving`]'s one wiring function ([`moving::drag`], which
//! is the only thing there that touches the live object model — its rules are
//! pure). That is the point: `PROJECT_PLAN.md`'s split is driven by
//! testability, and the
//! selection invariants are exactly the kind of property a unit test can hold
//! and a running window cannot be trusted to demonstrate.
//!
//! ## What is in this file, and what is next door in [`interact`](mod@interact)
//!
//! This file is **composition**. [`show`] and [`show_in`], the `ScrollArea`,
//! the strip of page rectangles and the raster or the state sentence drawn into
//! each, the fit resolved against this frame's viewport, the placeholder a
//! document with no pages gets, the [`CanvasGeometry`] the rulers are then
//! painted against, and the layout trace. It needs a live `Ui`, and it answers
//! one question: **where does everything go on screen?**
//!
//! [`interact`](mod@interact) is **interaction**. The pointer frame, what a
//! press would land on, the gesture machine's outcome and its application, the
//! right-click, the keys, the re-resolve of the selection, the overlay draw and
//! the cursor. It needs this frame's input, and it answers the other question:
//! **what did the operator just do, and what happens as a result?**
//!
//! The two change for different reasons — a page-display mode is a layout
//! question, a new tool is an interaction one — which is the seam rule R2
//! forced this file along when it reached 1,526 lines. The invariants that
//! belong to the second half travel with it, and are written up in
//! [`interact`](mod@interact)'s header rather than here: **selection survives
//! navigation**, and the two closed seams (where the selection lives, and the
//! one shared decomposition).
//!
//! ## Actions, not mutations — and the two documented exceptions
//!
//! The project's strongest structural invariant is that **no code path runs
//! from a widget to a document**: everything an operator does becomes an
//! [`Action`] that is applied *after* the frame is drawn. It is why the old
//! GUI's undo log is coherent, and it is established here at S0 — with two
//! actions and one widget — because retrofitting it later is expensive.
//!
//! [`show`] therefore takes `&mut OpenDoc` but is permitted to write
//! exactly three fields, none of them document state and none of them
//! expressible as an action. The first two are **frame bookkeeping about the
//! view**, and are impossible to defer for the same reason:
//!
//! 1. **`last_scroll_offset`** — the offset the scroll area settled on this
//!    frame. It is only readable *after* the area is built, and the next
//!    frame's pan needs it *before* the area is built. Storing it is what
//!    lets a pan track the hand instead of lagging it by a frame.
//! 2. **`zoom_anchor`** — which page point a zoom step must hold still, and
//!    where. It has to span two frames because the new zoom is not known when
//!    the step is asked for: the zoom is an [`Action`], applied after the UI
//!    is built, and it *clamps*. Recording the inputs and solving next frame
//!    avoids predicting a clamp we do not control. [`zoom`] owns both ends —
//!    which point ([`zoom::anchor_point`]) and when to spend it
//!    ([`zoom::anchor_step`]).
//!
//! The third is **`selection`**, which arrived here when seam 1 above was
//! closed, and it does not weaken the invariant: a selection *names* parts of
//! a document and changes nothing a save would write. It is settled during the
//! frame, from input that only exists during the frame, so deferring it would
//! make a click land a frame after the operator made it — the same argument as
//! the two above. The line is unmoved and visible in [`canvas_keys`]: Delete
//! removes nothing here, it raises [`VectorAction::DeleteSelection.into()`] carrying the
//! operand list, applied after the frame through the one funnel. Nothing that
//! touches `EditSession` runs from a widget.
//!
//! A fourth value is derived rather than stored: [`crate::viewer::ViewState::apply_fit`]
//! is called inline, because a fit mode is a pure function of this frame's
//! viewport and turning it into an action would apply it one frame late —
//! the page would visibly lag every window resize.
//!
//! ## Input conventions, and why breaking them feels wrong
//!
//! - **Plain wheel scrolls; Ctrl+wheel zooms.** egui routes these apart at
//!   the input-state level: a wheel event carrying the zoom modifier
//!   becomes `zoom_delta` and contributes *nothing* to
//!   `smooth_scroll_delta`, so the scroll area cannot pan and zoom off the
//!   same gesture. Breaking this is the single most common way a
//!   from-scratch viewer feels wrong.
//! - **Middle-drag pans** — the CAD / Inkscape / Illustrator / browser
//!   convention, requested by the operator on 2026-08-04. It is implemented
//!   against the scroll offset directly rather than by enabling
//!   `ScrollSource.drag`, because that knob is button-agnostic: turning it
//!   on would also make a *left*-drag pan, and the left button is reserved
//!   for the selection marquee that arrives at S4.
//! - **Panning triggers no re-raster.** It moves the viewport over an
//!   existing texture.
//!
//! ## The zoom anchor — decided once, in [`zoom`]
//!
//! `DEFECTS.md`'s "Not defects" table records that *"zoom buttons pin the
//! page's top-left, not the centre or the cursor"*. The wheel path was fixed
//! at S0; Phase 3.1 closes the rest, and the rule that governs all five zoom
//! paths (wheel, in, out, actual size, and the two framing commands) lives in
//! [`zoom`]'s header and in [`zoom::anchor_point`] — **the pointer when it is
//! over the canvas, the viewport's centre when it is not**. This file no
//! longer decides an anchor; it arms one ([`zoom::arm_anchor`]) and consumes
//! one ([`zoom::consume_anchor`]).

// Filling an interactive form where it is drawn: the boxes, the hit test that
// deliberately takes no tolerance, and the one editor a focused field gets.
pub mod form_marks;
pub mod formfield;
pub mod forms;
/// ★ Pointing at the page instead of typing coordinates — `OPERATOR_REQUESTS.md`
/// O66. A shared arm, not a feature of one dialog: his sentence was about
/// *"anything we are inserting"*.
pub mod placing;
// pdfcer's OWN crosshair bitmap, supplied to the OS as a real cursor. The
// platform's stock crosshair is monochrome and its colour belongs to the
// operator's pointer scheme, which is how it came to be white on white paper.
pub mod cursor;

/// **How deep the last click reached, and how deep it could have** — the
/// readout that turns "all I get is the page" into a diagnosis. See its header.
pub mod depth;
// The `f64` position tier and, above all, the two hand-overs between it and
// the `egui` scroll offset. Its own file because the seam is where the defects
// are, and O24f and O26e were hundreds of lines apart inside `show`.
mod deep;
// Who decides where the view is this frame, in one ranked list -- R2.
mod offset;
// Spending a fit command's request to place the view -- O28.
mod fit;
pub mod geometry;
pub mod gesture;
// Draggable alignment lines: what a guide belongs to, where it lives on disk,
// and why grabbing one cannot also start a marquee.
pub mod guides;
// The drawing grid, in each page's own space. Split from `rulers` under R2
// along the seam that module's header already drew: a ruler is chrome beside
// the canvas that reserves layout space, a grid is chrome over the page that
// reserves none.
/// ★★★ **The annotation half of the canvas clipboard** — split out of
/// `clipboard` on 2026-09-05 under R2, along the annotation-versus-content
/// seam. Its header carries the finding that made the split worth making:
/// `pdfcer-core`'s lossless annotation route is **lossy for exactly the
/// annotations this shell could already copy**, because a markup it models
/// travels as a `MarkupSpec` and is planted with `add_markup` rather than
/// `add_markup_with`. The fork between the two routes is read off the
/// engine's own carrier choice, never off a subtype list here.
pub mod annotclip;
/// A placement drag on a selected ce dimension - the operator's report of
/// 2026-08-20, *"I need to be able to move the dimension after it has been laid
/// down"*. Reaches `place_dimension`, never `move_dimension`; its header says
/// why that distinction is the whole design.
/// Dragging an ordinary markup annotation — the other half of the annotation
/// fork `dimdrag` opened. Its header carries the reason the shell sends a
/// DELTA and not a rectangle: a move has two halves and a renderer can only
/// see one of them.
pub mod annotdrag;
/// ★ Dragging a **Bézier handle** — the last Phase 1 row, and one `pdfcer`'s
/// own `gui` column ticked `[x]` while nothing here drew a handle at all.
/// `EditSession::move_handle` had existed since Pass 30.1; what was missing was
/// a way to see one and a way to grab one.
/// ★ **Cut, copy and paste on the canvas** — the operator's report of
/// 2026-08-19. Implements the row the engine can express (markup) and records
/// the one it cannot (page content) as a dated citation rather than a promise.
/// ★ **What a click MEANS** — the eight-rung ladder that decides whether a
/// completed click places an anchor, a caret, a vertex, a sticky, a dimension
/// pick, a text sweep, an annotation selection or a content selection. Split
/// out of `interact` under R2 on 2026-08-20; its header carries the order and
/// why each rung sits where it does.
pub mod clicking;
pub mod clipboard;
/// ★ **What Shift does to a drag** - the axis lock and the aspect lock, written
/// down once for the five drags that share them. `ui-conventions/drag-moves.md`
/// D5, found absent from every one of them by the conventions sweep of
/// 2026-08-20. Its header carries why one module rather than five call sites.
pub mod constrain;
/// ★ **Would a cut survive?** — the pre-press gate the engine asked for, asked
/// from the cheap side. Its header carries why it mirrors their rule instead of
/// calling it, and why the mirror is deliberately permissive.
pub mod cutgate;
/// Arriving where a bookmark points, once the viewport is known. Split from
/// `interact` under R2: landing on a destination is its own subject.
pub mod destination;
pub mod dimdrag;
/// Which of the three move verbs one drag reaches. Split out of `interact`
/// under R2; its header carries the argument that a fork whose branches can
/// all answer "not mine" eats the gesture.
pub mod dragroute;
/// ★ **The FORM-FIELD clipboard** (O58) — separate from [`clipboard`] because a
/// `/Widget` is not an annotation selection here, so nothing there can see one.
pub mod fieldclip;
pub mod grid;
pub mod handledrag;
pub mod handles;
/// ★ **What a press would land on, and what it would mean.** Split out of
/// `interact` under R2; its header carries the four-way precedence between a
/// Bézier handle, an anchor, a resize grip and the selection body — the single
/// most bug-prone rule on this canvas, learned three separate times in one day.
pub mod pressing;
pub mod presspick;
/// Dragging a form field's box. The third module on the annotation branch of
/// `dragroute`'s fork, and its header records that it exists because the
/// ADDRESS differs from a markup's, not because the geometry does.
pub mod widgetdrag;
// Reading this frame's pointer — what a click landed on at every rung, which
// of the two panning gestures is in flight, and where the in-flight press is
// kept between frames. Split out under R2 when the rulers landed; see its
// header on why the forced seam is a real one.
pub mod input;
// What the operator just did, and what happens as a result: the seven ordered
// steps of the one gesture function, the `Frame` of settled facts it is handed,
// and the two invariants it is accountable for. Split from this file under R2
// along the seam the two subjects already drew — composition needs a live `Ui`
// and answers *where does everything go?*, interaction needs this frame's input
// and answers *what did the operator just do?* Its items are `pub(super)`: this
// module is the only caller and nothing outside `canvas` can name them.
pub mod interact;
// Escape and Delete, and the precedence between the three things that would
// like Escape. Split from this file along the seam every other split here
// follows: that module is drivable by a headless `egui::Context`, this one
// needs a window.
pub mod keys;
/// ★★ **Following a `/Link`** — the hit test, the pointing hand, and the
/// four sentences for the four destinations this program cannot perform.
/// New on 2026-09-01: until the engine shipped `DestinationReader` a
/// link's destination could not be READ at all, so there was no
/// link-following code path anywhere in the shell. Its header carries why
/// collapsing the five destination variants into two behaviours is the
/// defect, and why the affordance is a cursor and never a mark on the page.
pub mod links;
pub mod mapping;
/// ★★ **What a rubber-band takes, and why the DIRECTION decides it** —
/// `OPERATOR_REQUESTS.md` O88. Left to right encloses, right to left
/// touches; AutoCAD's window / crossing-window rule. Split out of
/// [`interact`] on 2026-09-02 under R2. Its header carries the operator's
/// report and the reason the fix is geometric rather than about hit tests,
/// and `without_page_wrappers` carries the hazard a crossing band
/// introduces that an enclosing one could not.
pub mod marquee;
// Drawing a markup annotation where the operator points: the rubber band, the
// four kinds it can author, and the raw endpoints an arrow's head depends on.
pub mod markup;
pub mod measure;
pub mod menus;
// Dragging a selection: which verb each rung reaches, the canvas→page delta,
// and the ghost's honesty rule. Kept out of `selection` deliberately — that
// module is already 1,352 lines and owns *what is selected*, while this owns
// *what happens when you drag it*.
pub mod moving;
pub mod overlay;
/// ★ The application's own colour ROLES — `preview` and `dimension_selected` —
/// built from the resolved theme's palette and published per frame.
///
/// `egui_shell::theme::Overlays` is a generic role map because **R7** forbids
/// the shell learning what a ce dimension is; the roles are pdfcer's, exactly as
/// the ribbon manifest's command ids are. Its header carries the mapping
/// argument and the distinctness test the shell says the application owes.
pub mod overlays;

/// **Dropping pages onto the page view** — the caret between two sheets, and
/// the release that inserts or reorders there.
///
/// The operator's request of 2026-08-19: *"…or onto the canvas to add pages
/// and insert them in between the pages we've dragged to"*. The drag itself
/// lives in [`crate::pagedrag`], which is what lets a gesture that began in a
/// panel — possibly in another document — end here.
pub mod pagedrop;

/// ★★★ **Reading a comment where the comment is** — the pop-up window a click
/// on a note opens, and the tooltip a hover shows.
///
/// The operator, 2026-09-05: *"I could add a yellow sticky note but even in
/// read mode I don't think I could figure out how to read it."* He was right,
/// and the measurement was worse than the report: the only route to a comment
/// was the Comments panel, on the `markup` tab, which Read is not shown.
///
/// It lives on the **canvas** rather than on the ribbon precisely so that it
/// is mode-independent by construction — no future edit to a tab list can take
/// reading away from Read mode again. Its header carries the whole argument,
/// including why the pop-up is chrome rather than content under rule 4.
pub mod notepopup;
// The wheel as a page turn, under a one-page-at-a-time display mode -- O30.
mod paging;
/// ★ Everything the canvas draws, once everything is decided — lifted out of
/// [`interact`] when that file crossed R2's ceiling. Its header carries the
/// layer order and the argument for each position in it.
mod painting;
/// ★★ **What a click is ALLOWED to land on** — the operator's selection
/// filter, and the eleven classes it switches.
///
/// `OPERATOR_REQUESTS.md` O17. This is the replacement for Edit ▸ Content's
/// declare-your-intention-then-point model, and its header carries the whole
/// argument: why a filter belongs on the status bar rather than the ribbon,
/// why it is **subtractive only** (so `default()` reproduces today's behaviour
/// and R6 holds by construction), and why it composes with
/// [`crate::app::modes::capability::Capabilities`] as an `AND` rather than an
/// override.
///
/// Pure: no egui, no pointer, no document. Which is exactly why the popup that
/// drives it still has to be driven before any of it counts — R1.
pub mod pick;
pub mod resizing;
pub mod rightclick;
/// ★★ **The ninth grip** — the rotate handle above the selection box, and the
/// one gesture the eight could never express. `ui-conventions/handles.md` H2,
/// and the third word of the operator's *"reposition, resize, or rotate"*. Its
/// header carries why a rotation is not a resize with different arithmetic:
/// the pointer's DISTANCE from the centre must mean nothing.
pub mod rotating;
/// ★★ The eight resize grips, finally committing — built out of `move_nodes`
/// because `pdfcer-core` has no scale verb, which was re-derived against its
/// source rather than taken from a note.
/// **Which of the four canvas menus a secondary click opens.** Its header
/// carries the frame-ordering hazard that makes the question subtle: egui opens
/// a popup ON the click, so a menu keyed on state the click is about to change
/// shows the previous answer for ever.
/// **What rides along when a resize scales an annotation** — the three Tool-row
/// switches of `OPERATOR_REQUESTS.md` O51. Its header carries the correction
/// they are: convergence among reference implementations argues for a DEFAULT,
/// not against an OPTION.
pub mod scaling;
// The ruler gutters, the 1-2-5 tick ladder they and the grid share, and what
// unit the whole thing reads in. Its header carries the three decisions this
// feature turns on: the unit, the space the grid lives in, and why the
// reservation it takes out of the viewport is a constant (R128).
/// **The copied selection as a picture**, for programs that are not pdfcer —
/// `OPERATOR_REQUESTS.md` O71. Renders the clip's own one-page PDF rather than
/// cropping the page, and composites onto white because `CF_DIB` has no alpha
/// consumers agree about.
pub mod clipimage;
pub mod rulers;
pub mod selection;
/// **Smart-Selector** — a click selects a container, a double-click goes
/// inside it. `OPERATOR_REQUESTS.md` O70, following Inkscape's group context,
/// which is the convention the operator named.
pub mod smart;
// The GUI half of snapping: the zoom-invariant catch radius, the master/Alt
// gates, the Tab cycle, the two-click confirm, and the indicator glyph.
/// ★★★ **The shape itself, following your hand** — the live geometry preview
/// (`OPERATOR_REQUESTS.md` O63).
///
/// Its header carries the convention it **reverses** by operator ruling —
/// `handledrag.rs`'s *"a preview shows the cursor, the render shows the
/// document"* — and the measurement that makes it possible: a rasterised
/// preview is a second away on a CAD sheet, and this never touches the
/// rasteriser.
/// The fourteen **pre-commit** slots one frame of the canvas might fill — the
/// marquee, the ghosts, the shape preview, the snap marker, the ink trail.
///
/// Extracted from [`interact`] under R2; its header carries the one argument
/// they all share (why each is its own value and not a variant of another) and
/// the Rule 4 reading that makes every one of them permitted.
pub mod previews;
pub mod shapes;
pub mod snap;
// Which page the frame is about, in what order the rest should be drawn, and
// where a navigated-to page lands. The canvas's half of Phase 4's strip.
mod backdrop;
pub mod strip;
pub mod target;
// Selecting TEXT on the page, and copying it: the mode gate that needs no
// capability, the interaction decisions and which of Acrobat / Inkscape /
// SolidWorks each came from, and the one derivation that makes what is
// highlighted and what is copied the same value.
/// The three markup kinds that carry WORDS — text box, sticky note and
/// stamp. A different gesture (place, then type) and a different engine spec
/// from the seven geometric kinds; its header carries the argument.
pub mod textannot;
pub mod textsel;
// EDITING the page's own words, and placing new ones: the caret, the draft, and
// — in its `disposition` submodule — the two cases `DEFECTS.md` D4b records as
// wrong on commit, where the engine had the mechanism and the old GUI never
// selected it.
pub mod textedit;
// The `PDFCER_DIAG` lines the canvas writes, and the shape contract
// `tools/ui-verify` reads them under.
pub mod trace;
// Which pointer tool the canvas is in — select or hand — and the space bar
// that borrows the hand for as long as it is held.
pub mod tool;
// The anchor rule, the two-frame handshake it rides on, and the five zoom
// paths that route through it.
pub mod zoom;

/// ★ **Drawing the canvas** — the scroll area, the pages in it and the
/// geometry a frame hands back. Split out on 2026-08-29 when this file hit
/// R2's ceiling for the second time in one day; its header carries why the
/// seam is here and not in a shorter doc comment.
mod present;

pub use present::{CANVAS_MARGIN, Sampled, show};

// ★ `canvas::viewer` was a path before the split, because `canvas/mod.rs` had
// `use crate::viewer;` at its top and `canvas::measure::resolve` reaches for it
// by that name. Preserved as a re-export rather than fixing the caller: the
// split was meant to move code, not to rename anything anybody says.
pub use crate::viewer;
