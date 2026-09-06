//! # `text::commands::annotate` — the labels and tooltips of the **Markup** and
//! **Measure** tabs
//!
//! ## ★ Why this is a file of its own, and what the seam actually is
//!
//! **R2** (no `.rs` file over 1,500 lines) forced a split when the three
//! unblocked Phase 6 markup kinds arrived: [`super`] reached 1,520 lines. But a
//! line count only says *that* something had to move; it does not say what, and
//! `tools/gates/check-file-size.sh` says in its own header that shaving prose to
//! fit a threshold is the behaviour it exists to refuse. So the question was
//! which subject was separable, and this one is:
//!
//! > **Markup and Measure are the tabs about what you ADD ON TOP of the page.
//! > Everything left in [`super`] is about the file, the view, the pages, or the
//! > content that is already there.**
//!
//! That is not a line drawn here for convenience. It is the line
//! [`crate::app::modes::Capabilities`] already draws — `edit_content` on one
//! side, `author_markup` and `author_measure` on the other — and it is why
//! Review mode exists at all: a reviewer may add a comment and a dimension to a
//! drawing they may not otherwise touch. It is also the line
//! [`crate::shell::manifest`] already draws, which keeps `markup.rs` and
//! `measure.rs` as files of their own beside `edit.rs` and `pages.rs`. A reader
//! adding a Markup command now edits `text/commands/annotate.rs` and
//! `manifest/markup.rs`, which sit one directory apart and describe the same
//! band.
//!
//! ## Nothing else changed
//!
//! Every function came across verbatim, and [`super`] re-exports all twenty-one
//! by name — so every call site still writes `t::markup_rectangle()` and nothing
//! outside `text/` learns that the catalog was split. The re-export is explicit
//! rather than a glob for the reason `shell::commands`' own `pub use` is: a glob
//! would let a function added here arrive in the crate's namespace without
//! anybody naming it, and the catalog's whole discipline is that every
//! operator-visible string is named somewhere a reviewer looks.
//!
//! The tests stay in [`super`], with the list they walk. They are about the
//! catalog as a whole — no two labels alike, every tooltip a sentence, every
//! command reachable — and splitting them across the two files would let a
//! duplicate label pass by being in the other half.

use super::CommandText;

// ===========================================================================
// MARKUP TAB
//
// The four shapes shared one tooltip in the salvage source — "Draw this
// shape on the page. Click the button, then drag on the page where you
// want it." Each gets its own here, because the gesture is not the same
// for all four: a highlight is dragged across words, an arrow from tail to
// head, a rectangle corner to corner.
// ===========================================================================

/// `markup.rectangle`
#[must_use]
pub const fn markup_rectangle() -> CommandText {
    CommandText::new(
        "Rectangle",
        "Draw a rectangle on the page. Click the button, then drag from one corner to the \
         other.",
    )
}

/// `markup.ellipse`
#[must_use]
pub const fn markup_ellipse() -> CommandText {
    CommandText::new(
        "Ellipse",
        "Draw an ellipse on the page. Click the button, then drag out the box it fits inside.",
    )
}

/// `markup.arrow`
#[must_use]
pub const fn markup_arrow() -> CommandText {
    CommandText::new(
        "Arrow",
        "Draw an arrow on the page. Click the button, then drag from the tail to the head.",
    )
}

/// `markup.highlight`
#[must_use]
pub const fn markup_highlight() -> CommandText {
    CommandText::new(
        "Highlight",
        "Draw a highlight band over the page. Click the button, then drag across what you want \
         marked.",
    )
}

// ---------------------------------------------------------------------------
// The two kinds drawn by a RUN OF CLICKS, and the command that ends the run.
//
// Their tooltips have to carry one thing the four drag-shaped tooltips above do
// not: **how the gesture stops.** "Click the button, then drag" is a complete
// instruction because a drag ends when the button comes up; "click each corner"
// is not, because nothing in it says when to stop, and an operator left clicking
// forever is the exact failure the two endings exist to prevent. So each tooltip
// names the double-click, and `markup.finish`'s names the double-click back —
// the two are one instruction written from both ends, which is how a discoverable
// ending and a fast one stay the same feature rather than becoming two.
// ---------------------------------------------------------------------------

/// `markup.polyline`
#[must_use]
pub const fn markup_polyline() -> CommandText {
    CommandText::new(
        "Polyline",
        "Draw a line with corners in it. Click the button, then click each corner in turn and \
         double-click the last one.",
    )
}

/// `markup.polygon`
#[must_use]
pub const fn markup_polygon() -> CommandText {
    CommandText::new(
        "Polygon",
        "Draw a closed shape with corners of your choosing. Click the button, then click each \
         corner in turn and double-click the last one; the shape closes itself.",
    )
}

/// `markup.cloud`
///
/// ★ **"Revision cloud", not "Cloud".** The operator's own words, three times,
/// were *"still no revision cloud tool"* — never "cloud" alone — and in AEC the
/// two-word phrase is the term of art: it means *this area changed on this
/// revision*, which a one-word "Cloud" beside "Polygon" and "Freehand" does not
/// say. It is also the longest label in the Shapes band and that is accepted,
/// because a band of one-word labels with a two-word outlier reads as the
/// outlier being the specific one, which it is.
///
/// The tooltip repeats Polygon's gesture sentence almost verbatim, deliberately.
/// The two tools take the identical run of clicks and the identical ending, and
/// a reader who learns one has learned the other; wording it differently would
/// imply a difference that does not exist. What it adds is the last clause —
/// what makes it a cloud rather than a polygon is the border, which is the only
/// thing that differs in the file too.
#[must_use]
pub const fn markup_cloud() -> CommandText {
    CommandText::new(
        "Revision cloud",
        "Draw a closed shape with a cloudy border, to mark what changed. Click the button, then \
         click each corner in turn and double-click the last one; the shape closes itself.",
    )
}

/// `markup.ink`
///
/// **Freehand, not Ink.** The type and the specification say `/Ink`
/// (§12.5.6.12) and the operator says freehand, which is the same split
/// `Rectangle`/`/Square` makes in the other direction — see
/// `canvas::markup`'s header on whose vocabulary the names follow.
#[must_use]
pub const fn markup_ink() -> CommandText {
    CommandText::new(
        "Freehand",
        "Draw a line that follows the pointer. Click the button, then press and draw; let go \
         when you are done.",
    )
}

/// `markup.finish`
#[must_use]
pub const fn markup_finish() -> CommandText {
    CommandText::new(
        "Finish shape",
        "Place the polyline or polygon you have been clicking out. Double-clicking the last \
         corner does the same thing. Available once there are enough corners to draw.",
    )
}

// ---------------------------------------------------------------------------
// THE TWO NODE COMMANDS — the right-click route to a drawn shape's corners.
//
// ★★★ Their words are the ENGINE'S words, and that is deliberate rather than
// lazy. `pdfcer-core`'s note on the vertex verbs describes them as *"add a
// point here"* and *"remove this point"*, and the shell's own filed note asked
// for exactly those two phrases on the right-click menu. Using them unchanged
// means the operator, the shell and the engine's own documentation all call one
// operation one thing.
//
// ★★ **"Point", not "vertex" and not "node".** `/Vertices` is the PDF key,
// `node` is what this crate's modules are named after, and *point* is the word
// on the tool that arms them — `view.tool_node` is labelled **Points**. The same
// split as Rectangle/`/Square` and Freehand/`/Ink`, resolved the same way: the
// operator's vocabulary wins on a label, the specification's wins in the code.
//
// ★ Both labels are DEICTIC — "here", "this" — where every other label in this
// file names a thing in the abstract. That is correct for these two and only
// these two: they are the only commands in the catalog whose operand is *the
// place the operator was pointing at when they opened the menu*, and a label
// that said "Add a point" would be describing a different, general command that
// this build does not have. `manifest::TAB_SCOPED` carries the same fact from
// the other side — it is why neither has a ribbon home.
// ---------------------------------------------------------------------------

/// `markup.add_node`
///
/// ★ The tooltip names the three shapes it works on rather than the one it
/// does not, because a `/Line`'s row is **absent** and not greyed — nobody
/// reads a tooltip for a row they cannot see. What it does have to explain is
/// where the new corner lands, since the answer is *on the outline*, not under
/// the pointer: the click is allowed to be several points off the line.
#[must_use]
pub const fn markup_add_node() -> CommandText {
    CommandText::new(
        "Add a point here",
        "Split the edge you right-clicked and put a new corner on it, at the place you \
         pointed. Works on a polyline, a polygon and a revision cloud.",
    )
}

/// `markup.remove_node`
///
/// ★★ The tooltip carries **the floor**, and it is the reason this command is
/// greyed rather than absent when the shape is down to its last corners. R9
/// asks that a greyed control always explain itself on hover, and the
/// explanation has to say what would make it live again — *draw another
/// corner* — or greying is just a locked door.
#[must_use]
pub const fn markup_remove_node() -> CommandText {
    CommandText::new(
        "Remove this point",
        "Take away the corner you right-clicked. Greyed once the shape is down to its last \
         corners: a closed shape keeps three and an open one keeps two.",
    )
}

// ---------------------------------------------------------------------------
// The three kinds that mark a SELECTION rather than a drag.
//
// Their tooltips are written the other way round from the four above, and
// deliberately: a shape's tooltip says *"click the button, then drag"* because
// the button arms a tool, and these say *"select the text first"* because the
// button acts at once on what is already selected. Getting that backwards would
// describe Acrobat's other model — the arm-then-sweep comment tools — which is
// not what these do (`canvas::markup::text` §1).
//
// Each also names its own mark rather than sharing one sentence, because the
// three differ in exactly that one respect and a shared tooltip would make the
// band read as three ways to do the same thing.
// ---------------------------------------------------------------------------

/// `markup.underline`
#[must_use]
pub const fn markup_underline() -> CommandText {
    CommandText::new(
        "Underline",
        "Draw a line under the text you have selected. Select the words on the page first, then \
         press this.",
    )
}

/// `markup.strikeout`
#[must_use]
pub const fn markup_strikeout() -> CommandText {
    CommandText::new(
        "Strikeout",
        "Draw a line through the text you have selected. Select the words on the page first, \
         then press this.",
    )
}

/// `markup.squiggly`
#[must_use]
pub const fn markup_squiggly() -> CommandText {
    CommandText::new(
        "Squiggly",
        "Draw a wavy line under the text you have selected, for wording that needs a second \
         look. Select the words on the page first, then press this.",
    )
}

/// `markup.text_box`
#[must_use]
pub const fn markup_text_box() -> CommandText {
    CommandText::new(
        "Text box",
        "Place a box of text on the page as an annotation. It sits on top of the document \
         rather than becoming part of it, and takes the markup colour.",
    )
}

/// `markup.sticky_note`
#[must_use]
pub const fn markup_sticky_note() -> CommandText {
    CommandText::new(
        "Sticky note",
        "Place a collapsed note on the page, which opens when a reader clicks it. Sticky notes \
         use their own standard colours.",
    )
}

/// `markup.stamp`
#[must_use]
pub const fn markup_stamp() -> CommandText {
    CommandText::new(
        "Stamp",
        "Place a stamp on the page. Stamps use their own standard colours.",
    )
}

/// `markup.comments`
#[must_use]
pub const fn markup_comments() -> CommandText {
    CommandText::new(
        "Comments",
        "List the notes and markup on this document and jump to any of them.",
    )
}

// ===========================================================================
// MEASURE TAB
//
// The four controls that had no tooltip at all in the salvage source. Each
// one now says what it measures and what the measurement is read against,
// because the group model — named groups carrying a shared scale, number
// format and drafting standard — is the part of pdfcer's measuring that a
// user of any other product will not expect.
// ===========================================================================

/// `measure.linear`
#[must_use]
pub const fn measure_linear() -> CommandText {
    CommandText::new(
        "Linear",
        "Measure a straight distance and place a dimension on the page. The result is read \
         against the current dimension group's scale.",
    )
}

/// `measure.radius_diameter`
#[must_use]
pub const fn measure_radius_diameter() -> CommandText {
    CommandText::new(
        "Radius / diameter",
        "Measure a circle or an arc and place a radius or diameter dimension on the page.",
    )
}

/// `measure.perimeter`
///
/// ★ The description names all three endings, because a tool with three ways
/// to stop has to say so before the first click. Discovering the closing
/// convention by accident works; discovering it after tracing thirty vertices
/// the wrong way does not.
///
/// It also names what the number IS - the whole way round, added up - because
/// the operator asked for exactly that ("it adds the distance of all the
/// segments together for the dimension display") and a label reading only
/// "Perimeter" leaves an open path looking like the wrong tool for a pipe run.
#[must_use]
pub const fn measure_perimeter() -> CommandText {
    CommandText::new(
        "Perimeter",
        "Click around a shape to measure the whole way round, added up as one number. Click the first point again to close it, or double-click to finish an open path. The result is read against the current dimension group's scale, like every other dimension.",
    )
}

/// `measure.length`
///
/// ★ The operator's ask of 2026-08-20: *"add a length tool that works like the
/// perimeter tool without needing to close the profile."*
///
/// The label is `Length`, not `Path length` or `Open perimeter`: the operator
/// asked for a *length tool*, and the word they used is the word to put on it.
/// The description names what it is FOR - a run of something - because "click
/// along and add it up" describes the gesture and not the reason.
#[must_use]
pub const fn measure_length() -> CommandText {
    CommandText::new(
        "Length",
        "Click along a run - a pipe, a cable, a kerb line - to measure how far it goes, added \
         up as one number. Double-click the last point to finish. Use Perimeter instead when \
         the shape closes.",
    )
}

/// `measure.two_line`
#[must_use]
pub const fn measure_two_line() -> CommandText {
    CommandText::new(
        "Two-line",
        "Pick two lines already on the drawing and dimension the distance between them. Use \
         this rather than Linear when the geometry is there to be measured — the dimension \
         follows the lines rather than the two points you happened to click.",
    )
}

/// `measure.finish`
///
/// # Why the tooltip names the double-click
///
/// Because the double-click is the ending most operators will actually use,
/// and a control that exists *because* a gesture has no natural end is the one
/// place the other ending has to be taught. A tooltip that said only "finish
/// the current dimension" would leave an operator reaching for the ribbon on
/// every circle they place — which works, and is slower than the tool is meant
/// to be.
///
/// It says *radius or diameter* rather than "the current measurement" because
/// this command is not general: Linear and Two-line finish themselves at a
/// known click count, so Finish is greyed while either is armed and an
/// operator who read a general promise here would be right to call that a bug.
#[must_use]
pub const fn measure_finish() -> CommandText {
    CommandText::new(
        "Finish",
        "Place the radius or diameter dimension for the objects picked so far. Double-clicking \
         on the page does the same thing. Available once the picked objects define a circle.",
    )
}

/// `measure.set_scale`
#[must_use]
pub const fn measure_set_scale() -> CommandText {
    CommandText::new(
        "Set scale",
        "Set the scale the current dimension group's measurements are read against — how much \
         real-world length one unit on the drawing stands for.",
    )
}

/// `measure.manage_groups`
///
/// ★ **The trailing ellipsis was removed on 2026-08-19 and its removal is the
/// label doing its job.** A `…` is a promise that the control opens something
/// with a start and an end — a dialog you finish and dismiss. This one now
/// toggles a dock panel (`crate::panels::Panel::DimensionGroups`), and a
/// toggle whose label promises a dialog is a small lie told sixty times a day.
///
/// The label is also the **dock tab caption**, because
/// `crate::app::PdfcerApp::new` builds the panel registry from the command
/// catalog — one string, so the tab and the ribbon control can never disagree
/// about what the surface is called. "Manage dimension groups…" was a
/// reasonable ribbon label and an unreadable tab; "Dimension groups" is both.
#[must_use]
pub const fn measure_manage_groups() -> CommandText {
    CommandText::new(
        "Dimension groups",
        "Add, rename and remove dimension groups, and see the scale, number format and \
         drafting standard each one carries.",
    )
}
