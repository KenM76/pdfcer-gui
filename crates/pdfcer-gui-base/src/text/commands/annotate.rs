//! # `text::commands::annotate` — the labels and tooltips of the **Markup** and
//! **Measure** tabs
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/commands/annotate.md`.

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
#[must_use]
pub const fn markup_cloud() -> CommandText {
    CommandText::new(
        "Revision cloud",
        "Draw a closed shape with a cloudy border, to mark what changed. Click the button, then \
         click each corner in turn and double-click the last one; the shape closes itself.",
    )
}

/// `markup.ink`
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
// Their words are the ENGINE'S words, and that is deliberate rather than
// lazy. `pdfcer-core`'s note on the vertex verbs describes them as *"add a
// point here"* and *"remove this point"*, and the shell's own filed note asked
// for exactly those two phrases on the right-click menu. Using them unchanged
// means the operator, the shell and the engine's own documentation all call one
// operation one thing.
//
// **"Point", not "vertex" and not "node".** `/Vertices` is the PDF key,
// `node` is what this crate's modules are named after, and *point* is the word
// on the tool that arms them — `view.tool_node` is labelled **Points**. The same
// split as Rectangle/`/Square` and Freehand/`/Ink`, resolved the same way: the
// operator's vocabulary wins on a label, the specification's wins in the code.
//
// Both labels are DEICTIC — "here", "this" — where every other label in this
// file names a thing in the abstract. That is correct for these two and only
// these two: they are the only commands in the catalog whose operand is *the
// place the operator was pointing at when they opened the menu*, and a label
// that said "Add a point" would be describing a different, general command that
// this build does not have. `manifest::TAB_SCOPED` carries the same fact from
// the other side — it is why neither has a ribbon home.
// ---------------------------------------------------------------------------

/// `markup.add_node`
#[must_use]
pub const fn markup_add_node() -> CommandText {
    CommandText::new(
        "Add a point here",
        "Split the edge you right-clicked and put a new corner on it, at the place you \
         pointed. Works on a polyline, a polygon, a revision cloud and a freehand mark.",
    )
}

/// `markup.remove_node`
#[must_use]
pub const fn markup_remove_node() -> CommandText {
    CommandText::new(
        "Remove this point",
        "Take away the corner you right-clicked. Greyed once the shape is down to its last \
         corners: a closed shape keeps three, an open one keeps two, and each stroke of a \
         freehand mark keeps two.",
    )
}

/// `markup.flatten`
#[must_use]
pub const fn markup_flatten() -> CommandText {
    CommandText::new(
        "Make part of the page",
        "Burn this markup into the page's own drawing. It looks the same, but it is no longer a markup: it cannot be moved, edited or deleted as one. Ctrl+Z undoes it.",
    )
}

/// `markup.flatten_page`
#[must_use]
pub const fn markup_flatten_page() -> CommandText {
    CommandText::new(
        "Make all part of the page",
        "Burn every markup on this page into the page's own drawing. They look the same, but they are no longer markups. Links, form fields and anything pdfcer cannot burn stay as they are, and the status bar says which. Ctrl+Z undoes it.",
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

/// `markup.attach_file`
#[must_use]
pub const fn markup_attach_file() -> CommandText {
    CommandText::new(
        "Attach file",
        "Store a file inside the PDF, with a marker on the page that opens it.",
    )
}

/// `markup.sound`
#[must_use]
pub const fn markup_sound() -> CommandText {
    CommandText::new(
        "Attach sound",
        "Store a WAV recording inside the PDF, with an icon on the page that plays it.",
    )
}

/// `markup.screen`
#[must_use]
pub const fn markup_screen() -> CommandText {
    CommandText::new(
        "Media clip",
        "Store a video or audio clip inside the PDF, played in a region you drag on the page.",
    )
}

/// `markup.insert_text`
#[must_use]
pub const fn markup_insert_text() -> CommandText {
    CommandText::new(
        "Insert text",
        "Mark where words should be added, with a caret and the words in a comment.",
    )
}

/// `markup.replace_text`
#[must_use]
pub const fn markup_replace_text() -> CommandText {
    CommandText::new(
        "Replace text",
        "Strike through the selected text and propose new words in a comment.",
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

/// `markup.paste_image_stamp`
#[must_use]
pub const fn markup_paste_image_stamp() -> CommandText {
    CommandText::new(
        "Paste picture as stamp",
        "Put the picture copied in another program on the page as a stamp, at its own size. In Review, Ctrl+V does the same at the pointer.",
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
#[must_use]
pub const fn measure_perimeter() -> CommandText {
    CommandText::new(
        "Perimeter",
        "Click around a shape to measure the whole way round, added up as one number. Click the first point again to close it, or double-click to finish an open path. The result is read against the current dimension group's scale, like every other dimension.",
    )
}

/// `measure.area`
#[must_use]
pub const fn measure_area() -> CommandText {
    CommandText::new(
        "Area",
        "Click the corners of a region to measure the area it encloses, in the current dimension group's units squared. Click the first point again, or double-click the last one, to close it. The page gets the outline labelled with its area.",
    )
}

/// `measure.length`
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
#[must_use]
pub const fn measure_manage_groups() -> CommandText {
    CommandText::new(
        "Dimension groups",
        "Add, rename and remove dimension groups, and see the scale, number format and \
         drafting standard each one carries.",
    )
}
