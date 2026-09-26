//! # `text::panels::properties` — the Properties panel
//!
//! `RIBBON_IA.md` §5.8 commissions two surfaces for a selection's
//! properties, and is explicit about which is built first:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/properties.md`.

/// Heading over the selected object's properties.
#[must_use]
pub fn properties_object_heading() -> &'static str {
    "Object properties"
}

/// Shown when no object is being shown properties for.
#[must_use]
pub fn properties_nothing_focused() -> &'static str {
    "Pick a row in the Objects panel to see what it is made of."
}

// **The document's own copy MOVED to [`super::docprops`] on 2026-09-05**,
// with the section it belongs to — the operator: *"the document properties are
// still always visible in the properties tab. it needs to get out of there and
// be in its own document properties tab."*
//
// Seventeen functions went: the "This document" heading and its note, the four
// `/Info` field labels, the inexact-decode disclosure, and the seven read-only
// facts about the file (name, size, version, page count, sheet size,
// encryption, and its note). The three `recovered_*` functions went with them.
//
// Moved rather than re-exported. A `pub use` here would have kept
// `t::properties_document_heading()` resolving from a module that no longer
// draws it, which is precisely the stale route this project keeps finding — and
// there was exactly one caller of each, so the move cost one import line.
//
// It also bought R2 headroom that was about to be needed anyway: this file
// stood at **1,469 lines against the 1,500 ceiling** on the day of the move,
// and `super::textobject` records having been split off at 1,446 for the same
// gate. The seam is a subject boundary rather than an arithmetic one — what is
// left here describes **what is selected**; what left describes **the file**.

/// The line stating that this panel reports and does not change.
#[must_use]
pub fn properties_read_only_note() -> &'static str {
    "These are the facts pdfcer read from the file. Nothing here can be changed in this build."
}

/// Sub-heading over the disclosure sentences at the foot of the list.
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
#[must_use]
pub fn value_position(x: f64, y: f64) -> String {
    format!("{x:.1}, {y:.1} pt")
}

/// The object's paint-order index, as an operator reads it.
#[must_use]
pub fn value_index(index: usize) -> String {
    format!("#{index}")
}

/// A stroke width, in PDF points.
#[must_use]
pub fn value_line_width(width: f64) -> String {
    format!("{width:.2} pt")
}

/// A width and height, in PDF points.
#[must_use]
pub fn value_size(width: f64, height: f64) -> String {
    format!("{width:.1} × {height:.1} pt")
}

/// An image's sample count.
#[must_use]
pub fn value_pixels(width: u32, height: u32) -> String {
    format!("{width} × {height} px")
}

/// Shown for a field whose value the file does not state.
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
#[must_use]
pub fn value_font_embedded_no() -> &'static str {
    "No — this document relies on the reader having a copy of it."
}

/// pdfcer could not decide whether the font is embedded.
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

/// Render mode and single-run width words.
mod runtext;

pub use runtext::*;

// ===========================================================================
// The selected object's geometry — X, Y, W, H typed rather than dragged
//
// Every string here names a **PDF user-space point**, and none of them says
// so more than once. The units live in one note under the heading rather than
// as a suffix on four fields, because "40.00 pt" repeated four times is three
// repetitions of a fact the operator learned from the first one, and a
// properties panel is read top to bottom.
// ===========================================================================

/// The heading over the four geometry fields.
#[must_use]
pub const fn geometry_heading() -> &'static str {
    "Position and size"
}

/// The units line under the heading.
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
#[must_use]
pub const fn geometry_angle() -> &'static str {
    "Angle"
}

/// The unit suffix on the angle field, and the direction it counts in.
#[must_use]
pub const fn geometry_angle_note() -> &'static str {
    "Degrees anticlockwise. 0 is the orientation the mark was drawn in."
}

/// The commit button.
#[must_use]
pub const fn geometry_apply() -> &'static str {
    "Apply"
}

/// Why Apply is greyed when nothing was typed.
#[must_use]
pub const fn geometry_nothing_typed() -> &'static str {
    "Type a different number in one of the four fields first."
}

/// Why Apply is greyed when a typed extent would collapse the object.
#[must_use]
pub const fn geometry_too_small() -> &'static str {
    "Width and height must each be at least a quarter of a point — a smaller \
     value would collapse the object onto a line."
}

// `recovered_heading`, `recovered_detail` and `recovered_tooltip` were here
// and are now in [`super::docprops`]. A rebuilt cross-reference table is a fact
// about the FILE, so it moved with the rest of the file's own copy; see the
// note above the read-only line for the whole move.

// ===========================================================================
// The selected TEXT's style — `format_text`, O37
// ===========================================================================

/// The heading over the text restyle controls.
#[must_use]
pub const fn text_heading() -> &'static str {
    "This text"
}

/// How much of the page the restyle will act on.
#[must_use]
pub fn text_covers(count: usize) -> String {
    if count == 1 {
        "Changes apply to the text you selected.".to_owned()
    } else {
        format!("Changes apply to all {count} pieces of text you selected.")
    }
}

/// The section's own refusal: the selection is real and cannot be pinned.
#[must_use]
pub const fn text_unreadable() -> &'static str {
    "pdfcer cannot tell exactly which piece of text this is, so it will not offer to change it — a change might land on different text that reads the same."
}

// `text_object_route` WAS HERE, AND IT IS DELETED RATHER THAN MOVED
//
// It read: *"To change the font, size, bold or italic of these words, press T
// for the Text tool and sweep across them. Clicking picks the shape they are
// drawn in, which is not the same thing."*
//
// Written 2026-08-29 for O37's *"nothing on screen tells you to press T"*,
// re-aimed 2026-09-05 when O89 gave the clicked object a working colour
// control, and **deleted 2026-09-14** when `OPERATOR_REQUESTS.md` O198 gave
// it a working font, size, bold and italic control as well. Every clause of
// the sentence had become false: clicking now picks the words as well as the
// shape, and the four properties it named are editable without arming
// anything.
//
//
// ⚠ Its test, `the_text_route_sentence_names_the_bound_chord`, went with it,
// and so did the only reader of `view.tool_text`'s chord outside the keymap.
// If a future sentence writes a chord into its prose, that test is worth
// restoring from git — including its comment about why `contains("T")` passed
// for the wrong reason.

//
// They moved because the chooser did. `Pass 162.0` made the face list carry
// faces the document does NOT contain, which turned one combo box into a
// two-group control with a disclosure of its own — and the strings for it were
// then the largest single subject in this file, on a surface that is drawn by
// `pdfcer_gui::panels::properties::face` and consumed by two separate callers.
//
// Moved rather than duplicated, and the doc comments moved with them. This
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
#[must_use]
pub const fn text_colour_not_plain() -> &'static str {
    "Set in CMYK or a spot colour — pdfcer will not offer to change it here, because doing so would convert the ink to screen colour permanently."
}

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
// # The passed-over clause, and why it lives HERE and not on the status line
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
// # NONE of these greys a button, and the engine's ruling is why
//
// `pdfcer-core`, verbatim and unchanged: *"Do not grey out a bold button. Offer
// it, and surface the disclosure when synthesis fires."*
//
// `pdfcer_gui::panels::properties::text`'s header carries the fuller argument and it
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
// The pair this block replaces, `text_bold_hint_face_cannot_cover` and its
// italic twin, is **deleted rather than retargeted**. Its subject is gone: a
// face that claims the style and cannot show the run is no longer an outcome
// at all, it is an entry in `passed_over` on the way to a rung that works. The
// engine defect it previewed — `gate_synthesis` naming `Times-Bold` and
// `set_font` then refusing it, so neither verb reached bold — was fixed by the
// ladder itself. A sentence kept alive past its subject is how a shell ends up
// warning about a limit that no longer exists.

/// The bold button's hover text when **this text is already bold**.
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
#[must_use]
pub fn text_bold_hint_sibling_face(face: &str) -> String {
    format!(
        "Set this text in bold. This page already carries {face}, the bold form of this text's own typeface, so pdfcer will use it — nothing is added to the file and the letterforms stay in the family."
    )
}

/// The italic button's twin of [`text_bold_hint_sibling_face`].
#[must_use]
pub fn text_italic_hint_sibling_face(face: &str) -> String {
    format!(
        "Set this text in italic. This page already carries {face}, the italic form of this text's own typeface, so pdfcer will use it — nothing is added to the file and the letterforms stay in the family."
    )
}

/// The bold button's hover text when **the only real bold face on the page
/// belongs to a different typeface**.
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

/// The bold button's hover text when **the press will be refused, because
/// the operator said so**.
#[must_use]
pub const fn text_bold_hint_declined() -> &'static str {
    "Bold will be refused for this text. No real bold face can show it, and you have set pdfcer never to fake a style, so it will not thicken the letters. Change that setting to allow it."
}

/// The italic button's twin of [`text_bold_hint_declined`].
#[must_use]
pub const fn text_italic_hint_declined() -> &'static str {
    "Italic will be refused for this text. No real italic face can show it, and you have set pdfcer never to fake a style, so it will not slant the letters. Change that setting to allow it."
}

/// The clause appended to any style hint when **the ladder will step over
/// faces on the way**.
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

    /// **Every field label is a bare noun phrase with no trailing colon.**
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
    #[test]
    fn every_field_label_is_distinct() {
        let mut seen: Vec<&str> = Vec::new();
        for label in ALL_FIELD_LABELS {
            assert!(!seen.contains(&label), "two fields share the label {label}");
            seen.push(label);
        }
    }

    /// The catalog of field labels, for the sweeps above.
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
    #[test]
    fn geometry_values_keep_one_decimal_and_state_their_unit() {
        assert_eq!(value_position(72.0, 144.26), "72.0, 144.3 pt");
        assert_eq!(value_size(200.0, 0.0), "200.0 × 0.0 pt");
        assert!(value_size(1.0, 1.0).ends_with(" pt"));
    }

    /// An image's samples are labelled px, never pt.
    #[test]
    fn image_samples_are_never_labelled_in_points() {
        let px = value_pixels(640, 480);
        assert_eq!(px, "640 × 480 px");
        assert!(!px.contains("pt"));
    }

    /// **The three embedded-font answers are three different answers.**
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
    #[test]
    fn an_absent_value_says_so() {
        assert!(!value_not_stated().trim().is_empty());
    }

    /// **The panel must not promise typed geometry it cannot accept.**
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
