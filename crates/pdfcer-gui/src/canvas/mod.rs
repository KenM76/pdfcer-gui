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
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/mod.md`.

// Filling an interactive form where it is drawn: the boxes, the hit test that
// deliberately takes no tolerance, and the one editor a focused field gets.
pub mod form_marks;
pub mod formfield;
pub mod forms;
pub mod inkpick;
/// Pointing at the page instead of typing coordinates — `OPERATOR_REQUESTS.md`
/// O66. A shared arm, not a feature of one dialog: his sentence was about
/// *"anything we are inserting"*.
pub mod placing;
pub mod snapshot;
// pdfcer's OWN crosshair bitmap, supplied to the OS as a real cursor. The
// platform's stock crosshair is monochrome and its colour belongs to the
// operator's pointer scheme, which is how it came to be white on white paper.
pub use pdfcer_gui_base::cursor;

/// **How deep the last click reached, and how deep it could have** — the
/// readout that turns "all I get is the page" into a diagnosis. See its header.
pub use pdfcer_gui_base::pickdepth as depth;
// The `f64` position tier and, above all, the two hand-overs between it and
// the `egui` scroll offset. Its own file because the seam is where the defects
// are, and O24f and O26e were hundreds of lines apart inside `show`.
mod deep;
// Who decides where the view is this frame, in one ranked list -- R2.
mod offset;
// Spending a fit command's request to place the view -- O28.
mod fit;
pub use pdfcer_gui_base::viewgeometry as geometry;
#[cfg(test)]
mod geometry_tests;
pub mod gesture;
mod viewpos;
// Draggable alignment lines: what a guide belongs to, where it lives on disk,
// and why grabbing one cannot also start a marquee.
/// **The annotation half of the canvas clipboard** — split out of
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
pub mod guides;
// The nodes of a markup shape a `/Polygon`, `/PolyLine` or `/Line` and the
// drag that moves, adds or removes one. The operator's *"I also can't edit or
// delete nodes of a markup shape once it is drawn."*
pub mod annotnodes;
/// **Where an annotation's artwork ACTUALLY sits** — its four page-space
/// corners, at whatever angle its appearance `/Matrix` puts them, plus the
/// angle itself when the matrix is one. The operator's *"the box outlined when
/// an object is selected should be in the same angled orientation as the
/// object"* (O147). Runs §12.5.5's placement algorithm forwards, because
/// `/Rect` is required upright and therefore cannot answer the question.
/// **A declared workaround** — its header says what is filed at the engine and
/// carries the tripwire that fires when the answer arrives.
pub use pdfcer_gui_base::annotquad;
#[cfg(test)]
mod annotquad_tests;
/// **The boxes that show what a text block is made of** — one outline per
/// chunk of the selected text, so the unit a click is aiming at is visible
/// before the click. `OPERATOR_REQUESTS.md` O215 ask 3; its header carries why
/// that is a prerequisite for ask 1 rather than decoration.
pub mod chunks;
/// Dragging a **Bézier handle** — the last Phase 1 row, and one `pdfcer`'s
/// own `gui` column ticked `[x]` while nothing here drew a handle at all.
/// `EditSession::move_handle` had existed since Pass 30.1; what was missing was
/// a way to see one and a way to grab one.
/// **Cut, copy and paste on the canvas** — the operator's report of
/// 2026-08-19. Implements the row the engine can express (markup) and records
/// the one it cannot (page content) as a dated citation rather than a promise.
/// **What a click MEANS** — the eight-rung ladder that decides whether a
/// completed click places an anchor, a caret, a vertex, a sticky, a dimension
/// pick, a text sweep, an annotation selection or a content selection. Split
/// out of `interact` under R2 on 2026-08-20; its header carries the order and
/// why each rung sits where it does.
pub mod clicking;
pub mod clipboard;
/// Whether another program wrote the clipboard since pdfcer last copied.
pub mod clipseq;
/// A selection another pdfcer-gui window copied, adopted for a paste here.
pub mod clipshared;
/// **What Shift does to a drag** - the axis lock and the aspect lock, written
/// down once for the five drags that share them. `ui-conventions/drag-moves.md`
/// D5, found absent from every one of them by the conventions sweep of
/// 2026-08-20. Its header carries why one module rather than five call sites.
pub use pdfcer_gui_base::constrain;
#[cfg(test)]
mod constrain_tests;
/// **Would a cut survive?** — the pre-press gate the engine asked for, asked
/// from the cheap side. Its header carries why it mirrors their rule instead of
/// calling it, and why the mirror is deliberately permissive.
pub mod cutgate;
/// **Which delete verb the rung the operator is on reaches** — the twin of
/// [`moving`]'s `eligible`, and the answer to a Delete that traced
/// `no-verb-for-rung` and did nothing at all for the whole life of this shell.
pub mod deleting;
/// Arriving where a bookmark points, once the viewport is known. Split from
/// `interact` under R2: landing on a destination is its own subject.
pub mod destination;
/// Landing on a destination that named a POINT rather than a rectangle: the
/// per-axis scroll that must not acquire a magnification. O200, D47.
pub mod destscroll;
pub mod dimdrag;
pub mod dimext;
pub mod dimlabel;
pub mod dimpreview;
/// Which of the three move verbs one drag reaches. Split out of `interact`
/// under R2; its header carries the argument that a fork whose branches can
/// all answer "not mine" eats the gesture.
pub mod dragroute;
/// **The FORM-FIELD clipboard** (O58) — separate from [`clipboard`] because a
/// `/Widget` is not an annotation selection here, so nothing there can see one.
pub mod fieldclip;
pub mod grid;
pub mod handledrag;
pub use pdfcer_gui_base::handles;
#[cfg(test)]
mod handles_body_tests;
/// **What a press would land on, and what it would mean.** Split out of
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
pub mod interact;
pub mod keys;
/// **Following a `/Link`** — the hit test, the pointing hand, and the
/// four sentences for the four destinations this program cannot perform.
/// New on 2026-09-01: until the engine shipped `DestinationReader` a
/// link's destination could not be READ at all, so there was no
/// link-following code path anywhere in the shell. Its header carries why
/// collapsing the five destination variants into two behaviours is the
/// defect, and why the affordance is a cursor and never a mark on the page.
pub mod links;
/// A click on a 3D model in Read or Review opens the viewer.
#[cfg(feature = "3d")]
pub mod models3d;
pub use pdfcer_gui_base::canvasmapping as mapping;
/// **What a rubber-band takes, and why the DIRECTION decides it** —
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
/// Bringing a rectangle into view with the LEAST movement — `OPERATOR_REQUESTS.md`
/// O204's reveal. A one-shot parked on the document and spent by [`offset`]'s
/// ranked chain; its header carries why it cannot be `destscroll` with a flag.
pub mod minreveal;
// The frame's ONE question about the page's object model: does anything
// this frame does need a decomposition? It was four lines inside
// `canvas::interact` — a hand-maintained `matches!` over `GestureOutcome` that
// had been the defect four separate times, most recently for a subject a list
// of gestures structurally cannot hold, because Delete is a keystroke. Its own
// header carries the table of all four and the 531 ms measurement that decided
// the keyboard term's shape.
pub mod modelneed;
// Dragging a selection: which verb each rung reaches, the canvas→page delta,
// and the ghost's honesty rule. Kept out of `selection` deliberately — that
// module is already 1,352 lines and owns *what is selected*, while this owns
// *what happens when you drag it*.
pub mod moving;
/// **Tab walks the objects on the page** — `OPERATOR_REQUESTS.md`
/// O204, the canvas half. Pure over a provider and a pick filter; its
/// header carries why the ring is scoped to whatever the selection is
/// standing in rather than to the page’s own paint order.
pub mod objring;
/// **The invisible text a scan carries, drawn** — `OPERATOR_REQUESTS.md`
/// O226 and O229. Two layers at two places in `painting`'s order: a paper veil
/// that fades the raster without re-rendering it, and the recognised runs laid
/// out as vector text in a colour the operator chooses. Its header carries why
/// nothing here is capped and why a run too small to read becomes a bar rather
/// than a smudge.
pub mod ocrlayer;
pub mod overlay;
/// The application's own colour ROLES — `preview` and `dimension_selected` —
/// built from the resolved theme's palette and published per frame.
pub use pdfcer_gui_base::overlayroles as overlays;

/// **Dropping pages onto the page view** — the caret between two sheets, and
/// the release that inserts or reorders there.
pub mod pagedrop;

/// **Reading a comment where the comment is** — the pop-up window a click
/// on a note opens, and the tooltip a hover shows.
pub mod notepopup;
// The wheel as a page turn, under a one-page-at-a-time display mode -- O30.
mod paging;
// The two gestures that must still work on a frame that drew no page at all,
// run above `present`'s early return -- O186. Its header carries why the
// return is not simply moved, and which handlers are deliberately absent.
mod escape;
/// **Reaching an object that is off the page** — which of the canvas's
/// two interactive rectangles owns this frame's gesture.
pub use pdfcer_gui_base::pasteboard;

/// The page is a keyboard focus owner — `OPERATOR_REQUESTS.md` O204
/// decision 2. One keyboard-only widget per drawn page, so `Memory::focused`
/// names a page instead of nothing and Tab stops falling to the ribbon.
mod pagefocus;
/// Everything the canvas draws, once everything is decided — lifted out of
/// [`interact`] when that file crossed R2's ceiling. Its header carries the
/// layer order and the argument for each position in it.
mod painting;
/// **What a click is ALLOWED to land on** — the operator's selection
/// filter, and the eleven classes it switches.
pub use pdfcer_gui_base::pick;
pub mod resizing;
pub mod rightclick;
/// **The ninth grip** — the rotate handle above the selection box, and the
/// one gesture the eight could never express. `ui-conventions/handles.md` H2,
/// and the third word of the operator's *"reposition, resize, or rotate"*. Its
/// header carries why a rotation is not a resize with different arithmetic:
/// the pointer's DISTANCE from the centre must mean nothing.
pub mod rotating;
/// **The discoverable route to ONE LINE of a text block** — the
/// right-click row that descends to the Part rung on a multi-line text
/// object. O188(A): that rung was reachable by exactly one chord-armed
/// gesture and was announced on no surface. Its header carries the operand
/// problem (a menu row carries a command id and nothing else) and where the
/// pick is parked to solve it, plus why this does not breach
/// `DESIGNS.md`'s *do not add a context-menu item*: that rules out a row
/// that EDITS, not one that re-aims the selection.
pub use pdfcer_gui_base::runmenu;
/// The operand and preflight of `format.merge_text_runs`.
pub mod runmerge;
/// The operand and engine preflight of `format.split_text_lines`.
pub mod runsplit;
/// The eight resize grips, finally committing — built out of `move_nodes`
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
pub use pdfcer_gui_base::scaling;
// The ruler gutters, the 1-2-5 tick ladder they and the grid share, and what
// unit the whole thing reads in. Its header carries the three decisions this
// feature turns on: the unit, the space the grid lives in, and why the
// reservation it takes out of the viewport is a constant (R128).
/// **The copied selection as a picture**, for programs that are not pdfcer —
/// `OPERATOR_REQUESTS.md` O71. Renders the clip's own one-page PDF rather than
/// cropping the page, and composites onto white because `CF_DIB` has no alpha
/// consumers agree about.
pub use pdfcer_gui_base::clipimage;
pub mod rulers;
pub mod selection;
/// **Smart-Selector** — a click selects a container, a double-click goes
/// inside it. `OPERATOR_REQUESTS.md` O70, following Inkscape's group context,
/// which is the convention the operator named.
pub use pdfcer_gui_base::smartselect as smart;
#[cfg(test)]
mod smart_tests;
// The GUI half of snapping: the zoom-invariant catch radius, the master/Alt
// gates, the Tab cycle, the two-click confirm, and the indicator glyph.
/// **The shape itself, following your hand** — the live geometry preview
/// (`OPERATOR_REQUESTS.md` O63).
pub mod previews;
pub mod shapes;
pub use pdfcer_gui_base::snapmark as snap;
// Which page the frame is about, in what order the rest should be drawn, and
// where a navigated-to page lands. The canvas's half of Phase 4's strip.
mod backdrop;
/// The operator's standing answer to *"what happens when a stamp's words do
/// not fit its box?"* — one preference, shared by the placing dialog and the
/// properties panel, because nothing in the file records an author's fit
/// intent and a per-stamp control would be showing a value it invented.
pub use pdfcer_gui_base::stampfit;
pub mod strip;
pub use pdfcer_gui_base::canvastarget as target;
/// **Tab moves through what the operator clicked on, not through the
/// ribbon** — `OPERATOR_REQUESTS.md` O204. The seam that takes the press
/// off `egui` before its focus walk latches, and the pure ring step both
/// canvas rings share. Its header carries why no other seam can work.
pub use pdfcer_gui_base::tabnav;
/// A target provider assembled from plain rectangles, for the tests.
#[cfg(test)]
pub mod targetstub;
pub mod tier;
// Selecting TEXT on the page, and copying it: the mode gate that needs no
// capability, the interaction decisions and which of Acrobat / Inkscape /
// SolidWorks each came from, and the one derivation that makes what is
// highlighted and what is copied the same value.
/// The three markup kinds that carry WORDS — text box, sticky note and
/// stamp. A different gesture (place, then type) and a different engine spec
/// from the seven geometric kinds; its header carries the argument.
pub use pdfcer_gui_base::wordmarkup as textannot;
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
// Which of the two node-edit verb families one drag reaches — a ce dimension's
// corner or a markup shape's node. The R2 seam that kept `interact` under its
// ceiling when markup node editing landed.
pub mod vertexroute;
// The anchor rule, the two-frame handshake it rides on, and the five zoom
// paths that route through it.
pub mod zoom;

/// **Drawing the canvas** — the scroll area, the pages in it and the
/// geometry a frame hands back. Split out on 2026-08-29 when this file hit
/// R2's ceiling for the second time in one day; its header carries why the
/// seam is here and not in a shorter doc comment.
mod present;

pub use present::{CANVAS_MARGIN, Sampled, show};

// `canvas::viewer` was a path before the split, because `canvas/mod.rs` had
// `use crate::viewer;` at its top and `canvas::measure::resolve` reaches for it
// by that name. Preserved as a re-export rather than fixing the caller: the
// split was meant to move code, not to rename anything anybody says.
pub use crate::viewer;
