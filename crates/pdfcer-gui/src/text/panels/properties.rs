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

// ===========================================================================
// The selected markup's style — `set_markup_style`
// ===========================================================================

/// The heading over the markup restyle controls.
#[must_use]
pub const fn markup_heading() -> &'static str {
    "This markup"
}

/// The line under it: what kind of mark is selected.
///
/// ★ The file's own `/Subtype`, translated. An operator placed a *rectangle*
/// and the file calls it `Square`; they placed an *arrow* and the file calls it
/// `Line`. Showing the file's word would be correct and useless — the standing
/// rule in `text::commands` is that a label is the operator's vocabulary and an
/// id is the format's.
#[must_use]
pub fn markup_subtype(subtype: &str) -> String {
    let name = match subtype {
        "Square" => "Rectangle",
        "Circle" => "Ellipse",
        "Line" => "Arrow or line",
        "Polygon" => "Polygon or revision cloud",
        "PolyLine" => "Polyline",
        "Ink" => "Freehand",
        "Highlight" => "Highlight",
        "Underline" => "Underline",
        "StrikeOut" => "Strikeout",
        "Squiggly" => "Squiggly",
        "FreeText" => "Text box",
        "Text" => "Sticky note",
        "Stamp" => "Stamp",
        // ★ Not "Unknown". A subtype this catalogue has no word for is still a
        // real mark the operator can see and is about to restyle, and the
        // file's own spelling is the most honest thing left to show them.
        other => other,
    };
    format!("{name} on this page")
}

/// The colour control's label.
#[must_use]
pub const fn markup_colour_label() -> &'static str {
    "Colour"
}

/// The width control's label.
#[must_use]
pub const fn markup_width_label() -> &'static str {
    "Line width"
}

/// The suffix on the width control.
#[must_use]
pub const fn markup_width_suffix() -> &'static str {
    " pt"
}

/// The Line style row's label.
///
/// ★ *"Line style"* is `RIBBON_IA.md` §5.8's own name for the row and is what
/// the Format tab's command is called, so the two surfaces agree. It sits
/// directly under *Line width*, and the shared first word is doing work: the two
/// rows are one subject and read as a pair.
///
/// ★★ Not *"Dash pattern"*. The chooser's first entry is **Solid**, and under a
/// label reading *Dash pattern* that entry would read as *no dash pattern* — the
/// absence of the thing the label names rather than one of its values. The entry
/// names themselves are `crate::text::markup`'s, because three surfaces show
/// them and only one of the three is this panel.
#[must_use]
pub const fn markup_line_style_label() -> &'static str {
    "Line style"
}

/// The opacity control's label.
#[must_use]
pub const fn markup_opacity_label() -> &'static str {
    "Opacity"
}

/// The suffix on the opacity control.
///
/// A percentage, because that is the unit every application an operator has
/// used states opacity in. `/CA`'s own `0.0..=1.0` is a file-format detail they
/// should never meet.
#[must_use]
pub const fn markup_opacity_suffix() -> &'static str {
    " %"
}

/// The button that removes a property, restoring the file's own default.
///
/// ★ *"Clear"*, not *"Reset"* or *"Default"*. It removes the key from the
/// annotation dictionary, and what happens then is that the **standard's**
/// default applies — which is not necessarily what the mark looked like when
/// the operator placed it. "Reset" would promise a return to a previous state
/// that pdfcer does not remember.
#[must_use]
pub const fn markup_clear() -> &'static str {
    "Clear"
}

/// ★★ What restyling costs, said once under the whole section.
///
/// Two facts an operator cannot see and would otherwise discover from a
/// changed file:
///
/// 1. **The appearance is regenerated.** `set_markup_style` redraws the mark
///    from the geometry pdfcer models, so anything the original expressed
///    *outside* that model — a border effect pdfcer does not author, a producer's
///    own decoration — is gone from the new appearance even though its
///    dictionary key survives. The engine reports each one, and those arrive
///    verbatim on the status row; this sentence is the standing warning that
///    such a report is possible at all.
/// 2. **A wider line moves the box.** For every subtype except a rectangle and
///    an ellipse, `/Rect` is derived from the geometry plus a margin that
///    contains the stroke and any arrowheads — so widening the pen makes the
///    annotation's rectangle bigger. That is the engine's own ⚠, and it is the
///    difference between a mark that looks the same and a mark that occupies
///    the same space.
#[must_use]
pub const fn markup_note() -> &'static str {
    "Changing any of these redraws the mark from the shape pdfcer has recorded for it. A wider \
     line also makes the mark's own box bigger, except on rectangles and ellipses."
}

/// Why the controls are greyed on a locked annotation.
///
/// Names the standard, because an operator who meets this wants to know whether
/// pdfcer is refusing or the document is — and it is the document. It also names
/// the one thing that is still possible, which is the rule a refusal follows
/// everywhere in this shell.
#[must_use]
pub const fn markup_locked() -> &'static str {
    "This mark is locked by the document, so its appearance cannot be changed here. You can \
     still delete it."
}

/// ★★★ **What is possible on a mark this shell cannot restyle** — the sentence
/// that replaced three live controls that could not commit.
///
/// The defect, the reachability test that fixes it, and why R9 wants a sentence
/// here rather than an empty space, are all in
/// `crate::panels::properties::markup`'s header. This doc records the part that
/// belongs to the **words**: that each of the four claims was checked against
/// the engine on 2026-09-06 before it was written, because a limitation
/// sentence has an hours-long shelf life on this project and a false one is
/// worse than none.
///
/// - **move** — `EditSession::move_annotation` refuses a ce dimension and a
///   form widget by name, then works from `/Rect` and whatever geometry keys
///   are present. A `/Text`, `/FreeText` or `/Stamp` has a `/Rect`.
/// - **resize** — `EditSession::resize_annotation` refuses the same two, and
///   otherwise **carries** a foreign appearance rather than rebuilding it; a
///   uniform scale is exact. (A *non-uniform* scale of a foreign appearance is
///   refused unless distortion is allowed. That refusal arrives from the engine
///   with its own message and is not this sentence's subject.)
/// - **delete** — [`markup_locked`] already promises it, and
///   `crate::panels::properties::annotdelete` speaks for an annotation of any
///   kind: the verb is document-wide rather than per-subtype.
/// - **the note** — `EditSession::set_markup_note` refuses a ce dimension and a
///   widget, and nothing else. `/Contents` on a sticky note is the whole point
///   of a sticky note.
#[must_use]
pub const fn markup_not_restylable() -> &'static str {
    "pdfcer does not redraw this kind of mark, so its colour, line width and opacity cannot be \
     changed here. You can still move it, resize it, delete it, and edit the note it carries."
}

/// The fill control's label — `/IC`, the interior colour.
///
/// *"Fill"* rather than *"Interior"*: `/IC` is the format's word and every
/// drawing application an operator has used calls it fill. The standing rule in
/// `text::commands` is that a label is the operator's vocabulary and an id is
/// the format's.
#[must_use]
pub const fn markup_fill_label() -> &'static str {
    "Fill"
}

/// What sits beside the fill swatch when the mark has no `/IC` at all.
///
/// ★ It exists because a swatch cannot show *absence*. With no `/IC` the swatch
/// falls back to black, and a black square beside the word "Fill" says "this
/// shape is filled black" — which is the opposite of the truth. Acrobat draws a
/// red diagonal through its no-colour swatch for exactly this reason; this
/// shell says the word instead, which survives a theme change and a screen
/// reader where a drawn diagonal does not.
#[must_use]
pub const fn markup_fill_none() -> &'static str {
    "None"
}

/// The label over the chooser for the ending drawn at a line's **start** —
/// `/LE`'s first element (§12.5.6.7, Table 176).
#[must_use]
pub const fn markup_line_start_label() -> &'static str {
    "Line start"
}

/// The label over the chooser for the ending drawn at a line's **end** —
/// `/LE`'s second element.
#[must_use]
pub const fn markup_line_end_label() -> &'static str {
    "Line end"
}

/// One line-ending style, in the operator's words.
///
/// ★ Table 176 names ten endings and pdfcer authors three of them —
/// `annot_author::LineEnding` has exactly `None`, `OpenArrow` and `ClosedArrow`,
/// and its own doc comment calls the rest "a documented not-yet-authored
/// remainder". The chooser offers what the engine can draw, because a
/// fourth entry that produced a `/Butt` the appearance did not show would be
/// the inert control this project forbids, one level down.
///
/// The words are the operator's rather than the file's: *"Open arrow"*, not
/// `/OpenArrow`.
#[must_use]
pub const fn markup_line_ending_name(
    ending: pdfcer_core::annot_author::LineEnding,
) -> &'static str {
    use pdfcer_core::annot_author::LineEnding as L;
    // Exhaustive on purpose, with no wildcard: `LineEnding` is NOT
    // `#[non_exhaustive]`, so an ending the engine learns to draw breaks this
    // match at compile time rather than silently reaching a fallback word. That
    // is the whole reason the list is not written out in the panel module.
    match ending {
        L::None => "No end",
        L::OpenArrow => "Open arrow",
        L::ClosedArrow => "Closed arrow",
    }
}

/// The disclosure under the two line-ending choosers.
///
/// ★ It is owed because the readback is **lossy in one direction and silent
/// about it**: `annot_author::read_line_endings` degrades any Table 176 ending
/// pdfcer does not author down to `None`, so a `/Line` a foreign producer gave
/// a `/Butt` or a `/Diamond` end reads here as *No end* — and the mark on the
/// page plainly has one. Without this sentence the operator's conclusion is
/// that the chooser is broken.
#[must_use]
pub const fn markup_line_ending_note() -> &'static str {
    "pdfcer draws three of the standard's line ends. A mark that carries any other one shows as \
     No end here, and redrawing it does not put that end back."
}

/// **The fifth state of the arrowhead controls — take the setting OUT of the
/// file rather than write "none" into it.**
///
/// # ★★★ Two states, one picture, and why the operator is offered both
///
/// `MarkupStyle::endings` became a `StyleEdit` on 2026-09-06, in answer to this
/// shell's own request (`pdfcer-core` `edit.rs:4390`, and the field's own doc
/// comment at `edit.rs:4373`): `Set` writes `/LE`, and `Clear` **removes the
/// key**. Until that day *"draw no arrowheads"* was expressible and *"have no
/// `/LE` at all"* was not, so an operator who turned an arrow's heads off got a
/// document that no longer matched the one they opened, differing in a key
/// neither this panel nor the Format tab shows.
///
/// Table 176 makes `/None` the default for both ends, so `/LE [/None /None]`
/// and an absent `/LE` draw the same line. The difference is **bytes**, and
/// this project does not treat different bytes as the same document. The
/// engine's reply names the argument that decided it over a cheaper
/// doc-comment fix: a **signed** drawing, and the question *"is this
/// byte-identical to what my client sent me"* — a question undo does not
/// answer, because undo covers the session and not the round trip.
///
/// # ★★ Why these words
///
/// *"Clear"* is the verb this panel already uses for `/C`, `/IC` and `/CA`, and
/// [`markup_clear`] argues it: the word is honest about the **act** — the key
/// goes, and what applies afterwards is the standard's default rather than
/// anything pdfcer remembers — where *"Reset"* or *"Default"* would promise a
/// return to a previous state pdfcer never recorded.
///
/// ★ It carries a noun where the other three do not. A bare *"Clear"* in a list
/// whose first entry is already *"No arrowheads"* reads as a second name for
/// that entry, which is the one misreading this control cannot afford: those
/// two produce identical pictures and different files, so an operator who
/// confuses them cannot discover the mistake by looking.
///
/// ⚠ It deliberately does **not** say `/LE`, *key*, or *dictionary*. The
/// operator is a drafter; the fact they need is *the file goes back out the way
/// it came in*, and [`markup_endings_clear_hint`] says exactly that.
#[must_use]
pub const fn markup_endings_clear() -> &'static str {
    "Clear the setting"
}

/// Why an operator would press [`markup_endings_clear`] when the line looks
/// identical either way.
///
/// ★ The hover carries the whole distinction, because the control cannot: the
/// two states are indistinguishable on the page and the difference only shows
/// up in a byte comparison of the saved file. `REVIEW_TRIAGE.md`'s rule — a
/// caveat below the thing it qualifies arrives after the operator has drawn
/// their conclusion — is why it is a hover on the control rather than a note
/// underneath it.
///
/// ★ It names the **consequence** an operator has met (a drawing that comes
/// back different from the one that went out) rather than the mechanism (a
/// dictionary key). Both surfaces read this one string, so the Format tab and
/// this panel cannot come to explain the same act two different ways.
#[must_use]
pub const fn markup_endings_clear_hint() -> &'static str {
    "Takes the arrowhead setting out of the file instead of writing \"no arrowheads\" into it. \
     The line looks the same either way. Use it when a mark arrived without arrowheads and you \
     want it to go back out the way it came in — on a signed or issued drawing, a setting that \
     was not there before is a difference someone will find."
}

/// ★★★ The narrowing the colour swatches perform, said where the operator can
/// see it — **before** the click rather than after.
///
/// §12.5.2 lets `/C` and `/IC` be a 0-, 1-, 3- or **4**-component array, and the
/// four-component case is CMYK, which is not rare on a CAD sheet where the
/// producer is plotter-bound. The swatches convert one for display and a change
/// made through them writes RGB in its place, which is a real narrowing of the
/// colour space and is disclosed rather than performed quietly — the engine's
/// own posture on every conversion it makes.
///
/// The full argument, including the refuse-to-show behaviour this replaced and
/// why that was worse than an approximation, is on
/// `crate::panels::properties::markup`'s `swatch_of`.
#[must_use]
pub const fn markup_colour_narrowed() -> &'static str {
    "This mark's colour is recorded in CMYK and the swatches above are an approximation of it. \
     Picking a new colour here records an RGB one in its place."
}

/// ★★ What regenerating an appearance LOST, in the operator's terms.
///
/// `set_markup_style` redraws a mark from the geometry pdfcer models, so
/// anything the original expressed *outside* that model is gone from the new
/// appearance even though its dictionary key survives. The engine names each
/// one; this is the sentence that reaches the operator, and it is owed under
/// rule 4's surviving half — **an inference the operator cannot see still owes
/// an off-canvas report.**
///
/// ★ Every sentence says **what they will see**, not what a key is called. An
/// operator who is told *"the `/BE` border effect was dropped"* has been told
/// nothing; one who is told *"its cloudy edge is now a plain outline"* can look
/// at the page and decide whether they mind.
///
/// # ★★★ TWO OF THESE NARROWED ON 2026-09-06, AND ONE OF THEM WAS LEFT SAYING
/// # SOMETHING FALSE
///
/// `DroppedProperty::BorderStyle` and `::DashPattern` *"now fire much less
/// often"* — the engine's words — because a dashed border is read back and
/// re-authored rather than solidified
/// (`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:4220-4241`, and the variants'
/// own docs at `edit.rs:4884-4904`). That is the same narrowing `BorderEffect`
/// took after `Pass 98.0`, and it is disclosed here because **a narrowed
/// disclosure whose wording was written for the wide case is a false
/// disclosure**, which rule 4 forbids in exactly the direction it forbids
/// silence.
///
/// What each one now means, read out of the emission site rather than assumed:
///
/// | variant | fires when |
/// |---|---|
/// | `BorderStyle` | `/BS` `/S` names a style pdfcer does not redraw — `/B`, `/I`, `/U` — **or `/S /D` whose dash was not carried** |
/// | `DashPattern` | `/BS` `/D` is present and the dash was not carried: an array §8.4.3.6 does not admit, or a caller that **cleared** it |
///
/// ⚠ **The `BorderStyle` string used to name only a bevel, an inset and an
/// underline, and after the narrowing that became wrong.** The `/S /D` row is
/// new to it: an operator who presses **Solid** on a dashed mark clears the
/// dash, the original dictionary still says `/S /D`, and both variants fire — so
/// the old wording would have told them their mark had a *bevel*. It now names
/// the dash case too, and both sentences are phrased as facts about the new
/// outline rather than about a cause, so each is true whether the change was
/// asked for or merely disclosed.
///
/// ★ The redundancy on a requested clear is accepted rather than engineered
/// away. Suppressing a disclosure when the shell believes the operator asked for
/// it would put the decision *"was this loss requested?"* into
/// `app::actions::apply`'s routing arm, which that arm's own note forbids — it
/// routes and does not compute — and a suppression rule that got it wrong would
/// hide a real loss. A true sentence twice beats a missing one once.
#[must_use]
pub const fn markup_dropped(dropped: pdfcer_core::edit::DroppedProperty) -> &'static str {
    use pdfcer_core::edit::DroppedProperty as D;
    match dropped {
        D::BorderEffect => {
            "This mark had a cloudy or hand-drawn edge that pdfcer does not redraw. It is now a plain outline."
        }
        D::BorderStyle => {
            "This mark's border was declared in a style that is not in the new outline — a bevel, an inset, an underline, or a dash. It is drawn as a plain line now."
        }
        D::DashPattern => {
            "This mark carried a dash pattern that is not in the new outline. Its border is drawn solid now."
        }
        D::RectDifferences => {
            "This mark's own box was inset from the area it covered, and that inset is gone. The mark is drawn to the box now."
        }
        D::LineEnding => {
            "This mark had arrowheads or line ends pdfcer does not redraw, and they are gone."
        }
        // `DroppedProperty` is `#[non_exhaustive]`, so a wildcard is required
        // rather than optional. It answers with the general form of the same
        // fact, which is true of every member: something the file expressed is
        // not in the picture pdfcer just drew, and saying so imprecisely is far
        // better than saying nothing.
        _ => {
            "This mark carried something pdfcer does not redraw, and it is not in the new appearance."
        }
    }
}

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
// `EditSession::preview_style_resolution`, consumed 2026-08-29
// ---------------------------------------------------------------------------
//
// # What these six replace, and why the sentence they replace was not wrong
//
// `text_bold_hint` and `text_italic_hint` are still here and still used. They
// say:
//
// > If this page already carries a real bold face, pdfcer uses it; if it does
// > not, pdfcer thickens the letters and tells you it did.
//
// That is an accurate statement of the **mechanism** and a poor answer to the
// operator's actual question, which is *what is going to happen to my drawing
// when I press this?* It hands them a conditional and leaves them to evaluate
// it against a fact they cannot see — which font resources this page carries,
// and whether any of them covers the characters they swept.
//
// `preview_style_resolution` evaluates that conditional. It is `&self`,
// side-effect-free, and derives every field by calling `gate_synthesis` itself
// — the same function the commit path calls — so the answer here and the
// outcome there cannot disagree. The engine's own account of why it exists:
//
// > A caller could only learn the answer *after* acting — the wrong side of
// > rule 4 for a change that alters how the operator's document renders. R90's
// > own word for synthesis is "declinable", and declining sensibly means
// > knowing what is on offer before the click, not after it.
//
// # ★★★ NONE of these greys the button, and the engine's ruling is why
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
// ⇒ The one case where greying would now be defensible is
// [`text_bold_hint_face_cannot_cover`] — the shell can, for the first time,
// predict that refusal, because `preview_font_resources` runs the per-run
// glyph-coverage test the old argument said it could not. It is still a
// sentence, deliberately: the engine has a **queued fix** that will turn that
// case into ordinary synthesis, and a control withheld on the strength of a
// defect that is about to be fixed is a control that stays withheld for
// months. A sentence degrades to a stale sentence; a greyed button degrades to
// a missing feature.

/// The bold button's hover text when **a real bold face resolves and will be
/// used**.
///
/// `StyleOutcome::RealFaceResolves`, whose `selector` the run's own font
/// pre-flight also accepts. One press takes two verbs: `set_synthetic` is
/// refused *because* a real face is available, and
/// `crate::app::actions::textstyle` retries with the face the refusal names —
/// so the operator gets a genuine typeface rather than thickened letters.
///
/// ★ It names the face. That is the whole value over the conditional it
/// replaces: *"pdfcer will use Arial-Bold"* is checkable by the operator against
/// what they see afterwards, where *"if this page carries a bold face"* is not.
#[must_use]
pub fn text_bold_hint_real_face(face: &str) -> String {
    format!(
        "Set this text in bold. This page carries {face}, so pdfcer will use that real \
         typeface rather than thickening the letters."
    )
}

/// The italic button's twin of [`text_bold_hint_real_face`].
///
/// ★ *"Slant"*, not *"thicken"* — the two synthetic operations are different
/// and an operator who has read one sentence should not have to guess that the
/// other means something else. `crate::app::actions::textstyle`'s own table
/// keeps them distinct for the same reason.
#[must_use]
pub fn text_italic_hint_real_face(face: &str) -> String {
    format!(
        "Set this text in italic. This page carries {face}, so pdfcer will use that real \
         typeface rather than slanting the letters."
    )
}

/// The bold button's hover text when **no real bold face covers this text**, so
/// the letters will be thickened.
///
/// `StyleOutcome::WouldSynthesize`. A synthetic weight is the regular face
/// stroked, and R90 makes it declinable rather than a preference — which is why
/// the sentence says what will happen rather than merely offering to do it.
///
/// ★ It says *"for this text"*, not *"on this page"*, and the distinction is
/// the engine's: acceptance is **per run**, because a face that covers `Hello`
/// may not cover `Hellö`. A sentence claiming the page has no bold face at all
/// would be a stronger claim than was tested.
#[must_use]
pub const fn text_bold_hint_synthetic() -> &'static str {
    "Set this text in bold. No real bold face on this page covers this text, so pdfcer will \
     thicken the letters and tell you it did."
}

/// The italic button's twin of [`text_bold_hint_synthetic`].
#[must_use]
pub const fn text_italic_hint_synthetic() -> &'static str {
    "Set this text in italic. No real italic face on this page covers this text, so pdfcer will \
     slant the letters and tell you it did."
}

/// ★★★ The bold button's hover text for the case in which **the press will be
/// refused**, said before the press.
///
/// # This is a shipped engine defect, previewed rather than hidden
///
/// `crate::app::actions::textstyle`'s header carries the retraction in full.
/// The short form: `gate_synthesis` prefers a real face by **family**, so for a
/// run set in `Times` it names `Times-Bold` and gates synthesis off — and if
/// `Times-Bold` does not map every character in that run, `set_font` then
/// refuses it too. Neither verb reaches bold. It is reproduced on pdfcer's own
/// `textedit/format_family.pdf`, confirmed by the engine, and a fix is queued.
///
/// Until then the operator's experience was: press Bold, nothing happens to the
/// text, and a refusal naming a font appears in the status bar. This says it
/// first.
///
/// # ★★ How the shell knows, without re-deriving a single engine rule
///
/// Two engine answers, joined by a string the engine itself issues:
///
/// * `preview_style_resolution` returns `RealFaceResolves { selector, .. }` —
///   *"the string to hand to `set_font` to reach that face"*;
/// * `preview_font_resources` returns, for **this run's characters**, every
///   resource `set_font` would accept, each with the same kind of `selector`.
///
/// If the first selector is not among the second's, the retry cannot succeed.
/// That is a comparison of two engine-issued selectors, not a second
/// implementation of the family heuristic or of the coverage test — which is
/// the line `StyleResolution`'s own invariant draws: *"No matching rule is
/// re-derived here or — critically — in `pdfcer-gui`."*
///
/// ★ It does **not** tell the operator to pick a different font, though the
/// face chooser is two rows up and would work. Naming a remedy that depends on
/// which faces this particular page carries would be this shell guessing at
/// pdfcer's font selection — decision 058's exact case. Saying what will happen
/// is the honest half; choosing the way round is the operator's.
#[must_use]
pub fn text_bold_hint_face_cannot_cover(face: &str) -> String {
    format!(
        "Bold is not available for this text. pdfcer would use {face}, but that face has no \
         shape for every character here, so the change would be refused. This is a known \
         limit and a fix is on the way."
    )
}

/// The italic button's twin of [`text_bold_hint_face_cannot_cover`].
#[must_use]
pub fn text_italic_hint_face_cannot_cover(face: &str) -> String {
    format!(
        "Italic is not available for this text. pdfcer would use {face}, but that face has no \
         shape for every character here, so the change would be refused. This is a known \
         limit and a fix is on the way."
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
