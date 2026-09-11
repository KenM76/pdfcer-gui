//! # `text::panels::properties` — the Properties panel
//!
//! `RIBBON_IA.md` §5.8 commissions two surfaces for a selection's
//! properties, and is explicit about which is built first:
//!
//! > The division of labour: the **tab** carries what a user changes *while
//! > working* — colour, width, style, align, delete. The **panel** carries
//! > everything, including the read-only facts (winding rule, node count,
//! > embedded-font status, exact geometry) that belong beside the Objects
//! > panel's inventory rather than in a ribbon band.
//! >
//! > Build order: **panel first, tab second.** The panel is the harder half
//! > and the tab's contents are a subset of it, so building the tab first
//! > would mean writing the property editors twice.
//!
//! This is that panel's copy — **the read-only half of it**, which is all of
//! it at stage S3.
//!
//! ## What is deliberately absent, and why it is absent rather than greyed
//!
//! §5.8 also says the panel is *"where the **editable geometry** lives — X,
//! Y, W, H as typed values"*, and calls that the surface through which
//! `/Rect` move-and-resize becomes reachable without a drag. **None of that
//! is here.**
//!
//! Not because typed geometry is hard, but because there is nothing to edit:
//! [`crate::app::actions::Action`] carries zoom and page navigation and
//! nothing else, and the panel that would host the editors has no selection
//! to host them for. Four spinners bound to nothing would render, accept
//! typing, and discard it — which is not a placeholder in the harmless sense
//! but a control that silently loses an operator's work.
//!
//! `RIBBON_IA.md` P3 states the rule this follows: *"An unavailable
//! capability renders nothing, not a disabled stub. Greying is reserved for
//! **temporarily** unavailable — no document open, document encrypted, undo
//! stack empty — and is always explained on hover."* "The selection model
//! does not exist" is not temporary unavailability; it is absence.
//!
//! So the geometry is stated as **facts**, in the same field list as
//! everything else, and becomes editable when there is something to edit.
//!
//! ## The panel is the disclosure surface
//!
//! Every `ObjectNote` an object carries is spelled out here in full, at the
//! foot of the field list. That placement is the disclosure rule's, not a
//! layout preference: inference reporting belongs **off-canvas** — *"a
//! status line, a results panel, a report after the command, a properties
//! field"* — and the page view must carry no badge, tint, dashed outline or
//! "provisional" layer at all.
//!
//! The one-line test the rule offers: *would a screenshot of the editing
//! canvas differ from a screenshot of the same document saved and reopened?*
//! Nothing in this panel can make it differ, because nothing in this panel
//! draws on the page.
//!
//! ## Field wording lives next door
//!
//! The *values* — kind names, paint dispositions, winding rules, colours,
//! font labels, note sentences — are all [`super::objects`]'s, and are
//! reached from here rather than re-worded. That is the same
//! single-description discipline
//! [`crate::panels::objects::summary`] exists to enforce, applied one layer
//! up: a path's fill colour must not be described one way in an Objects row
//! and another way in a Properties field.
//!
//! This module owns only the **labels** — the left-hand column — and the
//! panel's own chrome.

/// Heading over the selected object's properties.
///
/// Says **object**, and it matters — for a reason that has now changed twice
/// and is worth carrying rather than re-deriving.
///
/// It was written because `file.properties` had *two* scopes under one command
/// — `RIBBON_IA.md` §5.1 gave it the tooltip *"The document's own title,
/// author, subject and keywords, and the properties of whatever is selected on
/// the page"* — so an unheaded field list invited the reading "these are the
/// document's properties", which is exactly wrong for a fill colour and exactly
/// wrong in the other direction for `/Title`.
///
/// ★ **Since 2026-09-05 the two scopes are two panels** — `file.properties` and
/// `file.document_properties`, the second being [`super::docprops`] — so this
/// panel has one subject again and that argument has expired. The heading stays
/// anyway, for a different and smaller reason: this panel draws **several**
/// sections at once (*This markup*, *This text*, *Position and size*), each
/// headed with what it is about, and one unheaded field list among them would
/// read as the continuation of whichever section happened to be above it.
#[must_use]
pub fn properties_object_heading() -> &'static str {
    "Object properties"
}

/// Shown when no object is being shown properties for.
///
/// The panel is never blanked: a blank region is indistinguishable from a
/// broken one, so the honest answer is a sentence naming the precondition —
/// and naming the surface that satisfies it, because the Objects panel is
/// the only route to this one at S3 and an operator has no way to guess
/// that.
#[must_use]
pub fn properties_nothing_focused() -> &'static str {
    "Pick a row in the Objects panel to see what it is made of."
}

// ★★★ **The document's own copy MOVED to [`super::docprops`] on 2026-09-05**,
// with the section it belongs to — the operator: *"the document properties are
// still always visible in the properties tab. it needs to get out of there and
// be in its own document properties tab."*
//
// Seventeen functions went: the "This document" heading and its note, the four
// `/Info` field labels, the inexact-decode disclosure, and the seven read-only
// facts about the file (name, size, version, page count, sheet size,
// encryption, and its note). The three `recovered_*` functions went with them.
//
// ★ Moved rather than re-exported. A `pub use` here would have kept
// `t::properties_document_heading()` resolving from a module that no longer
// draws it, which is precisely the stale route this project keeps finding — and
// there was exactly one caller of each, so the move cost one import line.
//
// ★★ It also bought R2 headroom that was about to be needed anyway: this file
// stood at **1,469 lines against the 1,500 ceiling** on the day of the move,
// and `super::textobject` records having been split off at 1,446 for the same
// gate. The seam is a subject boundary rather than an arithmetic one — what is
// left here describes **what is selected**; what left describes **the file**.

/// The line stating that this panel reports and does not change.
///
/// **Shown once, at the top, and never repeated per field.** An operator
/// looking at a list of exact numbers with no input boxes will reasonably
/// wonder whether the boxes failed to draw; saying so costs one line and
/// removes the question.
///
/// It states the boundary without naming a future control (P3 again — a
/// promise is a placeholder made of prose).
#[must_use]
pub fn properties_read_only_note() -> &'static str {
    "These are the facts pdfcer read from the file. Nothing here can be changed in this build."
}

/// Sub-heading over the disclosure sentences at the foot of the list.
///
/// A heading rather than an unlabelled run of paragraphs, because the
/// sentences are long and an operator scanning for a number needs to know
/// where the numbers stop. "Worth knowing" rather than "Warnings": every one
/// of these is a fact about the document, and warning styling would make a
/// property of the file read as a pdfcer failure.
#[must_use]
pub fn properties_notes_heading() -> &'static str {
    "Worth knowing about this object"
}

// ---------------------------------------------------------------------------
// Field labels
//
// The left-hand column, and nothing else. Every VALUE in this panel is
// worded by `super::objects`, so a fact cannot be described one way in an
// Objects row and another way in a Properties field.
//
// Each is a noun, sentence case, with no trailing colon: the colon is
// layout, and putting it in the string means a future two-column layout has
// to strip it back out.
// ---------------------------------------------------------------------------

/// The object's kind.
#[must_use]
pub fn field_type() -> &'static str {
    "Type"
}

/// The object's paint-order index — the handle every command-line verb
/// takes.
#[must_use]
pub fn field_index() -> &'static str {
    "Index"
}

/// How the path is painted (§8.5.3, Table 60).
#[must_use]
pub fn field_paint() -> &'static str {
    "Paint"
}

/// The colour a viewer actually sees for this object.
///
/// "Colour", not "Fill" or "Stroke", because which of the two is showing
/// depends on the paint disposition — a stroke-only path never shows its
/// fill colour, so a field labelled "Fill" would name a colour that appears
/// nowhere on the page. The Paint field directly above says which it is.
#[must_use]
pub fn field_colour() -> &'static str {
    "Colour"
}

/// The fill winding rule (§8.5.3.3).
#[must_use]
pub fn field_winding() -> &'static str {
    "Winding rule"
}

/// Stroke width in user-space units at paint time.
#[must_use]
pub fn field_line_width() -> &'static str {
    "Line width"
}

/// Anchor count across every part of the object.
#[must_use]
pub fn field_nodes() -> &'static str {
    "Points"
}

/// How many separate pieces the object is drawn from.
#[must_use]
pub fn field_parts() -> &'static str {
    "Parts"
}

/// The text a text object shows.
#[must_use]
pub fn field_text() -> &'static str {
    "Text"
}

/// The font in effect at the object's first show operator.
#[must_use]
pub fn field_font() -> &'static str {
    "Font"
}

/// Whether the document carries the font's program.
#[must_use]
pub fn field_font_embedded() -> &'static str {
    "Font embedded"
}

/// An image's sample count.
#[must_use]
pub fn field_pixels() -> &'static str {
    "Image samples"
}

/// The object's lower-left corner in PDF user space.
#[must_use]
pub fn field_position() -> &'static str {
    "Position"
}

/// The object's width and height in PDF points.
#[must_use]
pub fn field_size() -> &'static str {
    "Size"
}

// ---------------------------------------------------------------------------
// Field values that are this panel's own
// ---------------------------------------------------------------------------

/// A position, in PDF points.
///
/// **PDF user space, y-UP, origin at the page's lower left** — the same
/// frame `pdfcer` prints and the same frame the object model stores. Not
/// the screen's y-down frame, and not adjusted for `/CropBox` or `/Rotate`.
/// An operator comparing this number against one from the CLI must get the
/// same number, and that is worth more than matching the direction their
/// mouse moves.
///
/// One decimal: enough to tell a 0.0-pt-tall rule from a 0.5-pt one, which
/// is precisely the distinction that makes a hairline look like nothing at
/// all.
#[must_use]
pub fn value_position(x: f64, y: f64) -> String {
    format!("{x:.1}, {y:.1} pt")
}

/// The object's paint-order index, as an operator reads it.
///
/// The `#` is not decoration: it is the form the Objects panel's row label
/// uses and the form `pdfcer object-list` prints, so an operator can
/// match a properties field against a row and against a command line without
/// translating. Formatting a number is a catalog decision for exactly this
/// reason — one place decides, and every surface inherits it.
#[must_use]
pub fn value_index(index: usize) -> String {
    format!("#{index}")
}

/// A stroke width, in PDF points.
///
/// Two decimals, unlike the one [`value_size`] uses, and the difference is
/// deliberate: a line width is routinely 0.25 or 0.75 pt, and rounding to
/// one decimal makes a quarter-point hairline and a half-point one the same
/// number. A bounding box is never that fine.
#[must_use]
pub fn value_line_width(width: f64) -> String {
    format!("{width:.2} pt")
}

/// A width and height, in PDF points.
///
/// `×` rather than `x`, and one decimal for the same reason
/// [`value_position`] uses one. A zero on either axis is a real answer, not
/// a missing measurement — the note list below the fields says which shape
/// it is.
#[must_use]
pub fn value_size(width: f64, height: f64) -> String {
    format!("{width:.1} × {height:.1} pt")
}

/// An image's sample count.
///
/// "px" and never "pt": these are SAMPLES (§8.9.5, Table 89), and the Size
/// field a few rows above is in points. An image occupies the unit square
/// under the CTM, so the two numbers describe genuinely different things —
/// where it is, and what it is made of — and the pair is what lets an
/// operator judge effective resolution. They must not look alike.
#[must_use]
pub fn value_pixels(width: u32, height: u32) -> String {
    format!("{width} × {height} px")
}

/// Shown for a field whose value the file does not state.
///
/// One sentence fragment for every such field rather than a per-field
/// wording, because the answer is the same in every case and the *reason*
/// belongs in the note list rather than duplicated across four rows.
///
/// It is not a blank. A blank field is indistinguishable from a field pdfcer
/// forgot to fill in, and this panel's entire value is that its silences are
/// as legible as its numbers.
#[must_use]
pub fn value_not_stated() -> &'static str {
    "not stated in the file"
}

/// The font's program is in the document.
#[must_use]
pub fn value_font_embedded_yes() -> &'static str {
    "Yes — the document carries this font's program."
}

/// The font's program is not in the document.
///
/// States the consequence, not just the fact: a font the reader has to
/// supply is the difference between a file that prints as designed anywhere
/// and one that does so only on the machine it was made on. That is the
/// question an operator is actually asking when they look at this field.
#[must_use]
pub fn value_font_embedded_no() -> &'static str {
    "No — this document relies on the reader having a copy of it."
}

/// pdfcer could not decide whether the font is embedded.
///
/// ★ **The honest answer to a name-matching problem, and it is disclosed
/// rather than resolved.**
///
/// A text object records the `/BaseFont` in effect; the document's font
/// inventory records a program per font *dictionary*. Joining the two by
/// name is the only join available — the object model does not carry the
/// font dictionary's object id — and a name is not a key: one document can
/// declare two font dictionaries with the same `/BaseFont` (two independent
/// subsets of one face, which the survey behind the Fonts panel found in
/// 87 % of embedding files), and they can differ in whether they embed.
///
/// So when the name matches more than one record, or none, pdfcer says it
/// could not tell rather than picking one. Picking would be an inference
/// presented as a fact, which is precisely what rule 4 exists to stop — and
/// unlike most inferences this one is invisible: a confidently wrong "Yes"
/// looks exactly like a right one.
///
/// The Fonts panel is where the per-dictionary truth lives, so this points
/// at it.
#[must_use]
pub fn value_font_embedded_ambiguous() -> &'static str {
    "pdfcer could not tell — this document declares more than one font under that name, and they need not agree. The Fonts panel lists each one separately."
}

/// The **markup style** section's words — the largest of this module's four
/// subjects, split out under R2 on 2026-09-07. Re-exported so every existing
/// `t::markup_*` call site is unchanged: the seam is in the file system, not in
/// the catalogue's shape.
mod markup;

pub use markup::*;

// ===========================================================================
// ★ The selected object's geometry — X, Y, W, H typed rather than dragged
//
// Every string here names a **PDF user-space point**, and none of them says
// so more than once. The units live in one note under the heading rather than
// as a suffix on four fields, because "40.00 pt" repeated four times is three
// repetitions of a fact the operator learned from the first one, and a
// properties panel is read top to bottom.
// ===========================================================================

/// The heading over the four geometry fields.
///
/// *"Position and size"* rather than *"Geometry"*: the second is the word a
/// draughtsman uses for the shape of the thing, and this section changes where
/// it is and how big it is. The standing rule in `text::commands` is that a
/// label is the operator's vocabulary.
#[must_use]
pub const fn geometry_heading() -> &'static str {
    "Position and size"
}

/// The units line under the heading.
///
/// ★ It names the corner as well as the unit, and that is the load-bearing
/// half. PDF's Y axis points **up**, so a panel showing `Y` without saying
/// which edge it measures is ambiguous in the one direction that matters — an
/// operator who reads it as a top edge and types a smaller number to move the
/// object up will watch it go down.
#[must_use]
pub const fn geometry_units_note() -> &'static str {
    "Points, measured to the bottom-left corner. Y increases upward."
}

/// The X field's label.
#[must_use]
pub const fn geometry_x() -> &'static str {
    "Left"
}

/// The Y field's label.
///
/// *"Bottom"* rather than *"Y"*, for the reason [`geometry_units_note`] gives:
/// naming the edge makes the axis direction unmistakable at the point of use,
/// not just in a note the operator may have scrolled past.
#[must_use]
pub const fn geometry_y() -> &'static str {
    "Bottom"
}

/// The width field's label.
#[must_use]
pub const fn geometry_w() -> &'static str {
    "Width"
}

/// The height field's label.
#[must_use]
pub const fn geometry_h() -> &'static str {
    "Height"
}

/// The **angle** field's label, for a markup annotation.
///
/// *"Angle"* rather than *"Rotation"*, on the standing tie-breaker: the
/// operator's reference applications label the number on a shape's properties
/// panel *Angle*, and the convergence of the product class is the spec. It is
/// also the word he used — *"the angle should be editable from the
/// properties"*.
///
/// ⚠ **Not written as a bare "dimension" anywhere near this**, per Rule 15.
/// This number is a property of a **markup annotation**, not of a ce dimension;
/// a ce dimension's orientation is part of its measurement and is turned by a
/// different verb entirely.
#[must_use]
pub const fn geometry_angle() -> &'static str {
    "Angle"
}

/// The unit suffix on the angle field, and the direction it counts in.
///
/// ★ The direction is stated, and it has to be. PDF user space measures
/// anticlockwise from the positive x axis (§8.3.3), which is the mathematical
/// convention and the **opposite** of what a CAD operator reading a compass
/// bearing expects. A field labelled only *"Angle"* showing `30` is ambiguous
/// between two readings 60° apart, and the operator finds out which by typing a
/// number and watching the mark go the wrong way.
#[must_use]
pub const fn geometry_angle_note() -> &'static str {
    "Degrees anticlockwise. 0 is the orientation the mark was drawn in."
}

/// The commit button.
///
/// One button for up to two commands, and it does not say how many — *"Apply"*
/// is what the operator is doing; *"raise a move and a scale"* is what the
/// program is doing, and `RIBBON_IA.md` §2's rule is that a control is named
/// for the first.
#[must_use]
pub const fn geometry_apply() -> &'static str {
    "Apply"
}

/// Why Apply is greyed when nothing was typed.
///
/// R9 reserves greying for *temporarily* unavailable and requires the reason on
/// hover. This is the ordinary case — the section has just drawn, the fields
/// hold the object's current numbers, and there is nothing to do until one of
/// them changes.
#[must_use]
pub const fn geometry_nothing_typed() -> &'static str {
    "Type a different number in one of the four fields first."
}

/// Why Apply is greyed when a typed extent would collapse the object.
///
/// ★ It says what the floor IS rather than only that one was hit, because
/// *"too small"* leaves the operator guessing at a threshold, and the whole
/// point of a typed field is that they can hit an exact number.
#[must_use]
pub const fn geometry_too_small() -> &'static str {
    "Width and height must each be at least a quarter of a point — a smaller \
     value would collapse the object onto a line."
}

// ★ `recovered_heading`, `recovered_detail` and `recovered_tooltip` were here
// and are now in [`super::docprops`]. A rebuilt cross-reference table is a fact
// about the FILE, so it moved with the rest of the file's own copy; see the
// note above the read-only line for the whole move.

// ===========================================================================
// The selected TEXT's style — `format_text`, O37
// ===========================================================================

/// The heading over the text restyle controls.
///
/// *"This text"* rather than *"Font"*, matching [`markup_heading`]'s *"This
/// markup"*. The panel can show several sections at once and the operator has
/// to be able to tell which selection each is about; a section headed with the
/// name of a *property* would read as a category, not as a subject.
#[must_use]
pub const fn text_heading() -> &'static str {
    "This text"
}

/// How much of the page the restyle will act on.
///
/// ★ It says *"pieces of text"* rather than *"runs"*. A run is a show operator,
/// which is a fact about the file's structure that no operator asked to learn;
/// what they need to know is that their one press will change more than one
/// thing, and how many.
#[must_use]
pub fn text_covers(count: usize) -> String {
    if count == 1 {
        "Changes apply to the text you selected.".to_owned()
    } else {
        format!("Changes apply to all {count} pieces of text you selected.")
    }
}

/// The section's own refusal: the selection is real and cannot be pinned.
///
/// ★ It draws the heading and this sentence rather than drawing nothing,
/// deliberately. An operator with text selected who saw the section vanish
/// would conclude the feature is missing; an operator who sees it say why is
/// told the truth about one selection.
#[must_use]
pub const fn text_unreadable() -> &'static str {
    "pdfcer cannot tell exactly which piece of text this is, so it will not offer to change it — a change might land on different text that reads the same."
}

/// Shown under [`text_heading`] when a piece of TEXT is selected as an object
/// and nothing has been swept.
///
/// ★★★ **The sentence this module's own header claimed existed and did not.**
///
/// `panels::properties::text`'s header has said since it shipped:
///
/// > That is a real gap and it is named rather than hidden: clicking a text
/// > object with the Select tool does not raise this section; sweeping across
/// > the text does. **The empty state says so in those words**, because an
/// > operator who cannot find a control assumes it is missing.
///
/// There was no empty state. `section` returned `false` before drawing
/// anything whenever `doc.text_selection` was `None`, which is exactly the
/// state that paragraph describes — so the panel said nothing at all, and the
/// operator it was written for concluded the feature was missing. That is O37's
/// *"nothing on screen tells you to press T"*, and it was a documented
/// intention that no code carried.
///
/// # ★★ Why it names the tool and the KEY, when nothing else in this file does
///
/// `crate::text::tool`'s rule 2 forbids a tip and requires a statement of fact,
/// and this is one: the Text tool is what selects a range of words, and `T`
/// arms it. It is also the one place in the application where the operator is
/// **demonstrably** looking for this control — they have just clicked the text
/// they want to change — so the route belongs here rather than in a tooltip on
/// a control they have not found.
///
/// The chord is written into the sentence rather than fetched from the keymap
/// because this file has no `MenuHost` to ask. That is a real duplication and
/// it is bounded: `shell::manifest`'s keymap binds `T` to `view.tool_text`, and
/// `the_text_route_sentence_names_the_bound_chord` fails if the two ever part.
///
/// # ★★★ RE-AIMED 2026-09-05 — it used to say *"how these words look"*, and
/// # that became false the day it was written
///
/// `OPERATOR_REQUESTS.md` **O89**: *"I don't see where I am able to edit the
/// color of text, vectors, etc."* The answer built for it,
/// `crate::panels::properties::textobject`, puts a **working colour control on
/// the clicked text object** — so *"to change how these words look, press T"*
/// now stands directly above a control that changes how these words look
/// without pressing anything.
///
/// ⇒ Corrected in place rather than left beside the new control, because a
/// sentence that contradicts the widget under it is worse than no sentence:
/// this project's own rule is that when prose an earlier session wrote becomes
/// wrong, it is corrected where it stands and dated, so there are not two
/// answers on screen.
///
/// ★ It now names the four properties that genuinely still need the sweep —
/// font, size, bold, italic — and `crate::panels::properties::textobject`'s
/// header carries why those four cannot have a whole-object control and colour
/// can: each of them needs a reading of **one run** to be honest, and *"they
/// disagree"* is a displayable answer for a colour and for nothing else.
#[must_use]
pub const fn text_object_route() -> &'static str {
    "To change the font, size, bold or italic of these words, press T for the Text tool and sweep across them. Clicking picks the shape they are drawn in, which is not the same thing."
}

// ★★★ `text_face_label`, `text_face_none` and `text_face_ambiguous` were HERE
// until 2026-08-29 and now live in [`super::face`], with the two group headings
// and the standard-14 disclosure that joined them.
//
// They moved because the chooser did. `Pass 162.0` made the face list carry
// faces the document does NOT contain, which turned one combo box into a
// two-group control with a disclosure of its own — and the strings for it were
// then the largest single subject in this file, on a surface that is drawn by
// `crate::panels::properties::face` and consumed by two separate callers.
//
// ★ Moved rather than duplicated, and the doc comments moved with them. This
// project's salvage rule is that a doc comment is usually the record of a
// defect the wording was changed to fix — `text_face_ambiguous`'s 87 % survey
// is exactly that — so a re-typed copy would be a second wording with none of
// the reasons attached.

/// Label for the size field.
#[must_use]
pub const fn text_size_label() -> &'static str {
    "Size"
}

/// The unit shown inside the size field.
#[must_use]
pub const fn text_size_suffix() -> &'static str {
    " pt"
}

/// What a Format ▸ Font control shows when it is greyed and has no operand.
///
/// ★★★ **A screenshot found this and no trace could have**, 2026-08-27, which
/// is `D:/dev/rag/egui/`'s standing rule arriving in person: *layout and
/// clipping defects have exactly one oracle, a rendered screenshot.*
///
/// The Font group's size field is an `egui::DragValue` over the shared read-back
/// draft. With nothing swept the draft holds its `Default` — zero — and the
/// widget's own `range(1.0..=1440.0)` clamps that up, so the greyed control
/// rendered **`1.0 pt`**. The driven check saw a region at the right place and
/// passed, correctly: it was asserting that the control is drawn, and it was.
///
/// ★★ A greyed control showing a **false value** is worse than one showing
/// none. Greyed says *"not right now"*; `1.0 pt` says *"this text is one point
/// tall"*, which is a claim about the operator's document and it is wrong. The
/// same argument the Properties panel's `text_colour_not_plain` makes about a
/// converted swatch: a control that shows an approximation invites a press that
/// writes it back.
///
/// ★ An em dash, and the convention is the reason. Word leaves its font-size
/// box **blank** with nothing selected; every property grid in this class —
/// Acrobat, SolidWorks, Figma — shows a blank or a dash for *no value* and for
/// *mixed values*, which are the same state as far as a single field is
/// concerned. A dash is chosen over a blank because an empty framed control on
/// a ribbon reads as a rendering fault, and because it is what the operator's
/// own tools do.
#[must_use]
pub const fn text_value_absent() -> &'static str {
    "—"
}

/// Label for the bold / italic buttons.
#[must_use]
pub const fn text_weight_label() -> &'static str {
    "Style"
}

/// The bold button.
#[must_use]
pub const fn text_bold() -> &'static str {
    "Bold"
}

/// The bold button's hover text.
///
/// ★★ It promises the *outcome* and names the fallback, because the fallback is
/// the thing the operator would otherwise discover as a surprise. Both routes
/// are honest: a page carrying a real Bold gets the real face, and one that does
/// not gets a thickened version of what is there. Neither is greyed, because
/// between pdfcer's two verbs every page is covered.
#[must_use]
pub const fn text_bold_hint() -> &'static str {
    "Set this text in bold. If this page already carries a real bold face, pdfcer uses it; if it does not, pdfcer thickens the letters and tells you it did."
}

/// The italic button.
#[must_use]
pub const fn text_italic() -> &'static str {
    "Italic"
}

/// The italic button's hover text.
#[must_use]
pub const fn text_italic_hint() -> &'static str {
    "Slant this text. If this page already carries a real italic face, pdfcer uses it; if it does not, pdfcer slants the letters and tells you it did."
}

/// Label for the colour swatch.
#[must_use]
pub const fn text_colour_label() -> &'static str {
    "Colour"
}

/// Shown where the swatch would be, for a run painted in a space this control
/// cannot round-trip.
///
/// ★★ The sentence protects the operator's ink. A swatch showing DeviceCMYK as
/// its nearest RGB would write that RGB back on the next press, moving the run
/// out of its original space for ever on a document heading for a printer that
/// cares. pdfcer deliberately stores the space it was given rather than
/// force-converting the way Acrobat does, and this control must not undo that.
#[must_use]
pub const fn text_colour_not_plain() -> &'static str {
    "Set in CMYK or a spot colour — pdfcer will not offer to change it here, because doing so would convert the ink to screen colour permanently."
}

// ---------------------------------------------------------------------------
// ★★★ What Bold and Italic would ACTUALLY do to this run —
// `EditSession::preview_style_ladder`, consumed 2026-09-11
// ---------------------------------------------------------------------------
//
// # What these replace, and why the sentence they replace was not wrong
//
// `text_bold_hint` and `text_italic_hint` are still here and still used, for
// the run whose ladder cannot be planned. They say:
//
// > If this page already carries a real bold face, pdfcer uses it; if it does
// > not, pdfcer thickens the letters and tells you it did.
//
// That is an accurate statement of the **mechanism** and a poor answer to the
// operator's actual question, which is *what is going to happen to my drawing
// when I press this?* It hands them a conditional and leaves them to evaluate
// it against facts they cannot see — which font resources this page carries,
// whether any of them covers the characters they swept, and whether the one
// that does belongs to the same typeface as the text they are looking at.
//
// # ★★★ The instrument changed on 2026-09-11, and so did the number of answers
//
// These sentences were first written on 2026-08-29 against
// `preview_style_resolution`, which previews the **R90 synthesis gate**: one
// bit, *"is there a real face on this page that claims this style"*. The gate
// is one input to the decision, not the decision. `Pass 179.0` had already
// turned the commit path into a four-rung ladder, and the gate cannot see rung
// 2 by construction — the standard-14 sibling of the run's own family is
// **not on the page**, which is the whole point of it.
//
// So the old hover, on the commonest CAD page there is — a title block set in
// `Helvetica` with no bold resource anywhere — answered *"no real bold face,
// pdfcer will thicken the letters"* about a press that binds `Helvetica-Bold`
// and produces genuinely bold type. It was not a hedge that could be tightened;
// it was the wrong question, answered accurately.
//
// `EditSession::preview_style_ladder` is the right question. It runs
// `plan_style_ladder` — **the function `format_text` runs** — read-only
// against the staged content, walks the page once, and stages nothing. The
// preview and the commit are two readings of one answer rather than two answers
// kept in step by hand. Requested as
// `request_the_style_ladder_has_no_read_only_preview.md`; delivered in
// `Pass 295.0`.
//
// # ★★ The passed-over clause, and why it lives HERE and not on the status line
//
// A ladder that lands on rung 2 or rung 4 usually got there by stepping over a
// face that claimed the style and could not show the text. The engine
// discloses that **after** the commit, in `FormatReport::disclosures`, and this
// shell already surfaces those verbatim — so repeating it in the status line
// would be the same fact twice, which this project treats as a disclosure
// skipped rather than a disclosure doubled.
//
// Before the press there was nothing, and that is the gap
// [`text_hint_faces_tried`] fills. It is the same information one gesture
// earlier, where it can still change what the operator does.
//
// # ★★★ NONE of these greys a button, and the engine's ruling is why
//
// `pdfcer-core`, verbatim and unchanged: *"Do not grey out a bold button. Offer
// it, and surface the disclosure when synthesis fires."*
//
// `crate::panels::properties::text`'s header carries the fuller argument and it
// survives this work intact. What changes is only **which sentence the hover
// carries**, and that is exactly the right size of change: R83 asks that the
// operator be able to know before the gesture, not that every foreseeable
// refusal become an absent control.
//
// ⇒ The one case where greying could now be argued is
// [`text_bold_hint_declined`], which is a **measured** prediction of a refusal
// rather than a guess. It is still a sentence, because what produces it is a
// setting the operator owns: R9 reserves greying for the *temporarily*
// unavailable and demands the reason on hover, and the reason here is *"you
// told pdfcer never to fake it"*, which is a sentence by nature.
//
// ★ The pair this block replaces, `text_bold_hint_face_cannot_cover` and its
// italic twin, is **deleted rather than retargeted**. Its subject is gone: a
// face that claims the style and cannot show the run is no longer an outcome
// at all, it is an entry in `passed_over` on the way to a rung that works. The
// engine defect it previewed — `gate_synthesis` naming `Times-Bold` and
// `set_font` then refusing it, so neither verb reached bold — was fixed by the
// ladder itself. A sentence kept alive past its subject is how a shell ends up
// warning about a limit that no longer exists.

/// The bold button's hover text when **this text is already bold**.
///
/// `StyleRung::AlreadyStyled` — the run's own face already claims the weight,
/// so the press is a no-op rather than a change. Said plainly and without a
/// warning tone: pressing it costs nothing and does nothing, which is what a
/// toggle showing a state it is already in should say.
#[must_use]
pub const fn text_bold_hint_already() -> &'static str {
    "This text is already bold, so pressing this will not change it."
}

/// The italic button's twin of [`text_bold_hint_already`].
#[must_use]
pub const fn text_italic_hint_already() -> &'static str {
    "This text is already italic, so pressing this will not change it."
}

/// The bold button's hover text when **the bold form of this text's own
/// typeface is already on the page**.
///
/// Rung 1 with `StyleLadder::same_family == Some(true)`. The best outcome the
/// ladder has: nothing is added to the file, nothing is embedded, and the
/// letterforms are the ones the document already uses.
///
/// ★ It names the face *and* the relationship. *"pdfcer will use Arial-Bold"*
/// is checkable; *"the bold form of this text's own typeface"* is the part that
/// tells the operator the result will look like the rest of their drawing. The
/// shell does not work that relationship out — `same_family` is the engine's
/// verdict, and engine invariant R74 forbids re-deriving it here.
#[must_use]
pub fn text_bold_hint_sibling_face(face: &str) -> String {
    format!(
        "Set this text in bold. This page already carries {face}, the bold form of this text's own typeface, so pdfcer will use it — nothing is added to the file and the letterforms stay in the family."
    )
}

/// The italic button's twin of [`text_bold_hint_sibling_face`].
///
/// ★ *"Slant"*, not *"thicken"* — the two synthetic operations are different
/// and an operator who has read one sentence should not have to guess that the
/// other means something else.
#[must_use]
pub fn text_italic_hint_sibling_face(face: &str) -> String {
    format!(
        "Set this text in italic. This page already carries {face}, the italic form of this text's own typeface, so pdfcer will use it — nothing is added to the file and the letterforms stay in the family."
    )
}

/// The bold button's hover text when **the only real bold face on the page
/// belongs to a different typeface**.
///
/// Rung 1 with `StyleLadder::same_family == Some(false)`. Still a real face,
/// still nothing embedded — but the letterforms will not match the rest of the
/// run, and that is a visible change the operator should be able to expect
/// rather than discover.
///
/// ★★ It says *"the letters will be shaped differently"*, which is the thing
/// that distinguishes this from the sibling case. Both sentences would
/// otherwise read *"pdfcer will use a real bold face"* and the operator would
/// have no way to tell from the hover which of two quite different results is
/// coming.
#[must_use]
pub fn text_bold_hint_other_family(face: &str) -> String {
    format!(
        "Set this text in bold. No bold form of this text's own typeface is here, so pdfcer will use {face} — a real bold face from a different typeface. The letters will be shaped differently, not just heavier."
    )
}

/// The italic button's twin of [`text_bold_hint_other_family`].
#[must_use]
pub fn text_italic_hint_other_family(face: &str) -> String {
    format!(
        "Set this text in italic. No italic form of this text's own typeface is here, so pdfcer will use {face} — a real italic face from a different typeface. The letters will be shaped differently, not just slanted."
    )
}

/// The bold button's hover text when **pdfcer will add a standard PDF face**.
///
/// Rung 2: the standard-14 sibling of the run's own family, bound as a new
/// `/Font` resource with **no font file embedded** (ISO 32000-1 §9.6.2.2 —
/// every conforming reader is required to have these fourteen). This is the
/// rung the old `preview_style_resolution` hover could not see at all, and it
/// is the one that fires on the commonest CAD page there is: a title block set
/// in `Helvetica` carrying no bold resource.
///
/// ★★ *"the file does not grow"* is in the sentence deliberately. The
/// operator's standing worry about font work is what it does to a drawing they
/// have to email, and a rung that adds a resource but not a font program is
/// exactly the reassurance that worry wants — and it is true, which is the
/// only reason it is here.
#[must_use]
pub fn text_bold_hint_standard_sibling(face: &str) -> String {
    format!(
        "Set this text in bold. This page carries no bold face, so pdfcer will add {face} — one of the fourteen typefaces every PDF reader already has. Real bold letters, and no font file is embedded, so the file does not grow."
    )
}

/// The italic button's twin of [`text_bold_hint_standard_sibling`].
#[must_use]
pub fn text_italic_hint_standard_sibling(face: &str) -> String {
    format!(
        "Set this text in italic. This page carries no italic face, so pdfcer will add {face} — one of the fourteen typefaces every PDF reader already has. Real italic letters, and no font file is embedded, so the file does not grow."
    )
}

/// The bold button's hover text when **the letters will be thickened**.
///
/// Rung 4, the last rung: no real face anywhere on the ladder could show this
/// run, so pdfcer strokes the regular face. R90 makes that declinable rather
/// than a preference, which is why the sentence says what *will* happen rather
/// than merely offering to do it.
///
/// # ★★★ It became a measurement on 2026-09-11, and the hedge came out
///
/// From 2026-08-29 to 2026-09-11 this read *"… pdfcer will use a real bold
/// typeface if it can find one and thicken the letters if it cannot — and it
/// will tell you which it did."* That hedge was correct and unavoidable: the
/// shell was previewing the **R90 gate**, which cannot see rung 2, so it knew
/// the gate had found nothing and did not know what the ladder would do next.
/// The doc comment of the day argued at length that predicting the rung needed
/// an instrument that did not exist.
///
/// ⇒ It exists now. `preview_style_ladder` returns the rung the commit will
/// land on, so this sentence is only ever shown when the answer is **rung 4**,
/// and a hedge that offers a possibility the preview has already ruled out is
/// worse than the conditional it replaced. Both halves of that argument are
/// recorded because the hedge was right when it was written; the fix was a new
/// measurement, not better wording.
#[must_use]
pub const fn text_bold_hint_synthetic() -> &'static str {
    "Set this text in bold. No real bold face can show this text, so pdfcer will thicken the letters instead — and it will tell you it did."
}

/// The italic button's twin of [`text_bold_hint_synthetic`], measured on the
/// same day and for the same reason — read that one for the argument.
#[must_use]
pub const fn text_italic_hint_synthetic() -> &'static str {
    "Set this text in italic. No real italic face can show this text, so pdfcer will slant the letters instead — and it will tell you it did."
}

/// ★★★ The bold button's hover text when **the press will be refused, because
/// the operator said so**.
///
/// `FormatError::SynthesisRefusedByPosture` — the ladder reached rung 4, and
/// `StylePolicy::Refuse` is set. The refusal is not a defect and not a limit:
/// it is the setting working, and the sentence says which setting so the
/// operator can change it in one move if this is the run they want it for.
///
/// ★ It names the setting rather than describing it, because a hover that says
/// *"your settings prevent this"* sends the operator hunting through a
/// preferences dialog for a phrase that may not be there. Matching the words on
/// the control is the difference between a disclosure and a riddle.
#[must_use]
pub const fn text_bold_hint_declined() -> &'static str {
    "Bold will be refused for this text. No real bold face can show it, and you have set pdfcer never to fake a style, so it will not thicken the letters. Change that setting to allow it."
}

/// The italic button's twin of [`text_bold_hint_declined`].
#[must_use]
pub const fn text_italic_hint_declined() -> &'static str {
    "Italic will be refused for this text. No real italic face can show it, and you have set pdfcer never to fake a style, so it will not slant the letters. Change that setting to allow it."
}

/// ★★ The clause appended to any style hint when **the ladder will step over
/// faces on the way**.
///
/// `StyleLadder::passed_over` — each entry a face that claimed the style and
/// could not show this run's characters. Written as an addendum rather than
/// folded into the seven sentences because it is orthogonal to all of them: a
/// ladder can pass over faces on its way to any rung, including the one that
/// ends in a refusal.
///
/// # ★★★ The character, and why it earns its own parenthesis
///
/// The engine's own `reason` string is accurate and technical —
/// *"R-INV-1: character U+006F 'o' has no code in font 'Times-Bold'"*. On a
/// hover the operator wants **no 'o'**, which is the same fact in the form that
/// answers *why not*. `Refusal::character` is the engine handing that over as a
/// field, so this is a reformatting of an engine answer rather than a parse of
/// its prose — the distinction decision 058 turns on, and the reason
/// `PassedOver` was asked for as a struct instead of a `Vec<String>`.
///
/// ★ A face with no character named gets no parenthesis rather than an empty
/// one. `Refusal::character` is `Option`, and a refusal about the whole run
/// rather than one glyph is a real case; *"Times-Bold ()"* would be this shell
/// rendering an absence as a presence.
///
/// ★ Leading space, and it is not an oversight. The clause is pushed onto a
/// sentence that already ends in a full stop, and owning the separator here is
/// what keeps the call sites from each getting it right independently.
#[must_use]
pub fn text_hint_faces_tried(tried: &[(&str, Option<char>)]) -> String {
    let list: Vec<String> = tried
        .iter()
        .map(|(face, ch)| match ch {
            Some(ch) => format!("{face} (no '{ch}')"),
            None => (*face).to_owned(),
        })
        .collect();
    format!(
        " It will pass over {}, which cannot show this text.",
        list.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The route sentence names the chord the keymap actually binds.**
    ///
    /// [`text_object_route`] writes `T` into its prose because this module has
    /// no `MenuHost` to ask, which is a real duplication of the keymap and the
    /// kind that rots silently: rebinding the text tool would leave one
    /// sentence in the application telling the operator to press a key that
    /// does something else, and nothing would fail.
    ///
    /// So the duplication is **bounded** rather than merely admitted. This
    /// reads the shipped manifest's keymap, finds whatever chord is bound to
    /// `view.tool_text`, and asserts the sentence contains it. Rebinding to
    /// `Y` fails here and the failure names the sentence.
    ///
    /// ★ It asserts the **chord in the sentence**, not the sentence in full,
    /// deliberately: the copy is a design surface and must stay free to be
    /// reworded, while the one fact it borrows from somewhere else must not
    /// drift. Pinning the whole string would turn every rewording into a test
    /// edit and teach the next person to update the literal without reading it.
    #[test]
    fn the_text_route_sentence_names_the_bound_chord() {
        let shell = crate::shell::manifest::built_in();
        let keymap = shell.keymap.as_ref().expect("the manifest binds keys");
        let chord = keymap
            .iter()
            .find(|(_, id)| *id == "view.tool_text")
            .map(|(chord, _)| chord)
            .expect("the text tool is bound to something");
        // ★★ The needle is `press <chord> `, not the bare chord, and the
        // difference is the whole worth of this test.
        //
        // Every pointer-tool chord in this manifest is a **single letter**, and
        // this sentence is forty words of English. `contains("A")` would be
        // satisfied by the `A` in "change"; `contains("T")` is satisfied by the
        // "To" the sentence opens with, so the first draft of this test passed
        // for a reason that had nothing to do with the keymap. It was found by
        // rebinding the tool to `Y` and watching it fail — which proved only
        // that `Y` is a rare letter.
        //
        // Anchoring on the phrase the sentence actually uses makes the check
        // ask what it means to ask: *does the instruction name the key?* A
        // rewording that drops the word "press" fails here, which is correct —
        // the sentence would no longer be an instruction naming a key, and
        // this test would no longer be able to tell whether it named the right
        // one.
        let sentence = text_object_route();
        let needle = format!("press {chord} ");
        assert!(
            sentence.contains(&needle),
            "the text tool is bound to `{chord}`, so the route sentence should say              {needle:?}, and it says: {sentence}"
        );
    }

    /// **Every field label is a bare noun phrase with no trailing colon.**
    ///
    /// The colon is layout. Baking it into the string means a future
    /// two-column or grid layout has to strip it back out of every entry,
    /// and the one that gets missed renders as `Type::`.
    #[test]
    fn no_field_label_carries_its_own_punctuation() {
        for label in ALL_FIELD_LABELS {
            assert!(!label.ends_with(':'), "`{label}` carries a colon");
            assert!(
                !label.ends_with('.'),
                "`{label}` is a label, not a sentence"
            );
            assert!(!label.is_empty());
        }
    }

    /// **No two fields share a label.**
    ///
    /// Two rows reading "Size" — one for the bounding box and one for the
    /// image's samples — is exactly the confusion [`value_pixels`]'s "px vs
    /// pt" comment is about, arriving through the label column instead of
    /// the value column.
    #[test]
    fn every_field_label_is_distinct() {
        let mut seen: Vec<&str> = Vec::new();
        for label in ALL_FIELD_LABELS {
            assert!(!seen.contains(&label), "two fields share the label {label}");
            seen.push(label);
        }
    }

    /// The catalog of field labels, for the sweeps above.
    ///
    /// Hand-written, like every enumeration of things Rust cannot enumerate
    /// for us. It is only used by tests, so an entry missed here weakens a
    /// check rather than shipping a defect — but it is listed in the same
    /// order as the panel draws them so a reader can diff the two.
    const ALL_FIELD_LABELS: [&str; 15] = [
        "Type",
        "Index",
        "Paint",
        "Colour",
        "Winding rule",
        "Line width",
        "Points",
        "Parts",
        "Text",
        "Font",
        "Font embedded",
        "Image samples",
        "Position",
        "Size",
        // Not a field: the note heading. Included so a rename of it is
        // caught by the distinctness sweep alongside the fields, since it
        // shares the same column.
        "Worth knowing about this object",
    ];

    /// The label list and the functions agree.
    ///
    /// Without this the sweeps above would silently test a stale copy of the
    /// catalog — the classic failure of a hand-written enumeration.
    #[test]
    fn the_label_catalog_matches_the_functions() {
        let from_fns = [
            field_type(),
            field_index(),
            field_paint(),
            field_colour(),
            field_winding(),
            field_line_width(),
            field_nodes(),
            field_parts(),
            field_text(),
            field_font(),
            field_font_embedded(),
            field_pixels(),
            field_position(),
            field_size(),
            properties_notes_heading(),
        ];
        assert_eq!(from_fns, ALL_FIELD_LABELS);
    }

    /// Position and size are in points, to one decimal, and a zero extent is
    /// a real answer.
    ///
    /// The decimal is not decoration: a horizontal rule is 0.0 pt tall and a
    /// hairline is 0.5 pt tall, and rounding to whole points makes those the
    /// same object.
    #[test]
    fn geometry_values_keep_one_decimal_and_state_their_unit() {
        assert_eq!(value_position(72.0, 144.26), "72.0, 144.3 pt");
        assert_eq!(value_size(200.0, 0.0), "200.0 × 0.0 pt");
        assert!(value_size(1.0, 1.0).ends_with(" pt"));
    }

    /// An image's samples are labelled px, never pt.
    ///
    /// The Size field a few rows above is in points and describes a
    /// different thing. Two numbers of the same shape with the same unit
    /// would read as one measurement stated twice.
    #[test]
    fn image_samples_are_never_labelled_in_points() {
        let px = value_pixels(640, 480);
        assert_eq!(px, "640 × 480 px");
        assert!(!px.contains("pt"));
    }

    /// **The three embedded-font answers are three different answers.**
    ///
    /// The ambiguous one is the load-bearing case: a confidently wrong "Yes"
    /// is indistinguishable from a right one, so the panel has to be able to
    /// decline. It must not read like either of the definite answers, and it
    /// must point at the surface that can be definite.
    #[test]
    fn the_embedded_font_answers_include_an_honest_dont_know() {
        let yes = value_font_embedded_yes();
        let no = value_font_embedded_no();
        let dunno = value_font_embedded_ambiguous();
        assert_ne!(yes, no);
        assert_ne!(no, dunno);
        assert_ne!(yes, dunno);
        assert!(
            dunno.contains("could not tell"),
            "the ambiguous answer must decline in words: {dunno}"
        );
        assert!(
            dunno.contains("Fonts panel"),
            "an honest don't-know has to say where the answer is: {dunno}"
        );
    }

    /// An unstated value is a sentence, never a blank.
    ///
    /// A blank field is indistinguishable from one pdfcer forgot to fill in,
    /// and this panel's whole value is that its silences are as legible as
    /// its numbers.
    #[test]
    fn an_absent_value_says_so() {
        assert!(!value_not_stated().trim().is_empty());
    }

    /// **The panel must not promise typed geometry it cannot accept.**
    ///
    /// `RIBBON_IA.md` §5.8 specifies editable X/Y/W/H here, and it is not
    /// built: there is no selection model and no mutating action to carry
    /// the edit. The read-only note is the one string that says so, and a
    /// well-meaning copy edit that turns it into "editing coming soon" would
    /// make it a promise — which P3 forbids in prose exactly as it forbids
    /// in a widget.
    #[test]
    fn the_read_only_note_states_the_boundary_without_promising_a_control() {
        let note = properties_read_only_note();
        assert!(note.contains("can be changed"), "{note}");
        assert!(note.contains("Nothing here"), "{note}");
        for promise in ["coming soon", "not yet available", "will be", "future"] {
            assert!(
                !note.to_lowercase().contains(promise),
                "the note promises a control instead of stating a boundary: {note}"
            );
        }
    }
}
