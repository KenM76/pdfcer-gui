//! # `text::panels::formfield` — the words in the form-field properties section
//!
//! Covers `pdfcer_gui::panels::properties::formfield`, the section that appears
//! when an operator clicks a form field on the page in Edit mode.
//!
//! ## The sentence this file exists for
//!
//! [`not_editable_note`]. `pdfcer-core` has four verbs for an existing field —
//! rename, delete, delete-a-widget, fill — and **none** for its flags. Required,
//! read-only, multiline, comb, the border and the tooltip are settable only when
//! the field is created.
//!
//! A panel that showed those as facts and offered no way to change them would
//! read as unfinished, and an operator would spend real time looking for the
//! control. So the limit is **stated**: what cannot be changed, and what to do
//! instead. Saying it costs one line; not saying it costs a search that ends in
//! the operator concluding the program is broken.
//!
//! It is worded as a **statement about pdfcer**, not about PDF. The format
//! permits changing every one of them; it is this engine that has no verb yet.
//! Blaming the format would be a false claim, and the kind that is never
//! corrected because nobody can check it.
//!
//! ## Vocabulary
//!
//! The same rule [`crate::text::formfield`]'s header sets: the word the
//! operator's other programs use, not the spec's. A `/Ch` is a drop-down, a
//! `/Btn` is a check box or a button, `/TU` is a tooltip. The one place the
//! spec's vocabulary survives is the word **box** for a widget, because there
//! is no better one — "widget" is jargon and "annotation" is wrong.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/formfield.md`.

use pdfcer_core::forms::{Field, FieldType};

/// The section heading.
#[must_use]
pub fn heading() -> String {
    "Form field".to_owned()
}

/// The label for the field's name.
#[must_use]
pub fn label_name() -> String {
    "Name".to_owned()
}

/// The label for the field's type.
#[must_use]
pub fn label_type() -> String {
    "Type".to_owned()
}

/// The label for the page the clicked box is on.
#[must_use]
pub fn label_page() -> String {
    "Page".to_owned()
}

/// The label for the box count.
#[must_use]
pub fn label_boxes() -> String {
    "Boxes".to_owned()
}

/// The label for the field's current value.
#[must_use]
pub fn label_value() -> String {
    "Value".to_owned()
}

/// The label for the set flags.
#[must_use]
pub fn label_flags() -> String {
    "Set".to_owned()
}

/// A 1-based page number.
#[must_use]
pub fn page_number(one_based: usize) -> String {
    format!("{one_based}")
}

/// How many boxes the field is drawn as, and which one was clicked.
#[must_use]
pub fn box_count(total: usize, clicked: usize) -> String {
    format!("{total} — you clicked #{}", clicked.saturating_add(1))
}

/// What kind of control this is, in the operator's vocabulary.
#[must_use]
pub fn field_type(field: &Field) -> String {
    use pdfcer_core::forms::ButtonKind;
    match field.field_type {
        Some(FieldType::Text) => "Text field".to_owned(),
        Some(FieldType::Button) => match field.button_kind {
            Some(ButtonKind::Check) => "Check box".to_owned(),
            Some(ButtonKind::Radio) => "Radio button".to_owned(),
            Some(ButtonKind::Push) => "Button".to_owned(),
            None => "Button".to_owned(),
        },
        Some(FieldType::Choice) => "Drop-down list".to_owned(),
        Some(FieldType::Signature) => "Signature".to_owned(),
        // Not "unknown" and not blank. A field with no `/FT` is a real and
        // specific defect — no viewer knows how to fill it — and it is exactly
        // the shape `EditSession::adopt_widget` produces from a bare kid that
        // lost its `/Parent`. Saying so here is the same disclosure
        // `text::status::adopted` makes at the moment one is created.
        None => "No type — no viewer knows how to fill it".to_owned(),
    }
}

/// The field's current value, if it has one worth showing.
///
/// `None` for an empty field, which draws no row at all rather than an empty
/// one — R9 applied to a fact instead of to a control.
#[must_use]
pub fn field_value(field: &Field) -> Option<String> {
    use pdfcer_core::forms::FieldValue;
    if matches!(field.value, FieldValue::Absent) {
        return None;
    }
    // `display_text` and not a decode of our own. A `/V` is raw bytes, and
    // turning them into characters is §7.9.2 / Annex D.3 text-string decoding —
    // PDFDocEncoding or UTF-16BE with a BOM, not UTF-8. `String::from_utf8_lossy`
    // over a UTF-16 value produces interleaved NULs and replacement characters,
    // which is a wrong answer that looks like a corrupt document rather than
    // like a bug in this panel.
    //
    // The engine's own helper is explicit that it is for display and that
    // export uses the raw bytes, which is exactly this use.
    let text = field.value.display_text();
    (!text.trim().is_empty()).then_some(text)
}

/// The flags that are set, named, or `None` when none are.
#[must_use]
pub fn field_flags(field: &Field) -> Option<String> {
    use pdfcer_core::forms::FieldFlags;
    let f = field.flags;
    let mut set: Vec<&str> = Vec::new();
    if f.read_only() {
        set.push("read-only");
    }
    if f.required() {
        set.push("required");
    }
    if f.no_export() {
        set.push("not exported");
    }
    // The type-specific bits are read only for the type they belong to,
    // because the SAME BIT means different things on different field types —
    // bit 18 is `Edit` on a choice and `DoNotSpellCheck`-adjacent territory
    // elsewhere. Reporting a text flag on a check box would be inventing a
    // property from a bit that is not about it.
    if matches!(field.field_type, Some(FieldType::Text)) {
        if f.has(FieldFlags::MULTILINE) {
            set.push("multi-line");
        }
        if f.has(FieldFlags::PASSWORD) {
            set.push("hidden as typed");
        }
        if f.has(FieldFlags::COMB) {
            set.push("equal cells");
        }
    }
    if matches!(field.field_type, Some(FieldType::Choice)) {
        if f.has(FieldFlags::COMBO) {
            set.push("drop-down");
        }
        if f.has(FieldFlags::EDIT) {
            set.push("free text allowed");
        }
        if f.has(FieldFlags::MULTI_SELECT) {
            set.push("multi-select");
        }
    }
    (!set.is_empty()).then(|| set.join(", "))
}

/// The label above the rename box.
#[must_use]
pub fn rename_label() -> String {
    "Rename — its short name, with no dots".to_owned()
}

/// The rename button.
#[must_use]
pub fn rename_button() -> String {
    "Rename".to_owned()
}

/// **Why there is no rename box at all** — `EditSession::rename_refusal`
/// answered `Some` (R83).
#[must_use]
pub fn rename_refused() -> String {
    "This document does not allow its form fields to be renamed. The values below can still \
     be filled in and changed."
        .to_owned()
}

/// **Why there are no delete buttons** — `EditSession::deletion_refusal`
/// answered `Some` (R83).
#[must_use]
pub fn delete_refused() -> String {
    "This document does not allow form fields to be removed. Its structure is fixed; the \
     values in it can still be filled in and changed."
        .to_owned()
}

/// Why Rename is greyed, in the terms of the rule that actually refused it.
#[must_use]
pub fn rename_disabled(refusal: &pdfcer_core::forms_author::FormAuthorError) -> String {
    use pdfcer_core::forms_author::FormAuthorError as F;
    match refusal {
        // The empty box is the state this panel OPENS in, so this is the
        // sentence most often read. It asks for the thing that is missing and
        // says nothing about periods.
        F::EmptyName => "Type the new name for this field. It cannot be left blank.".to_owned(),
        // `Text.2` — a well-formed path supplied where one segment was required.
        // The consequence, not the clause: a name with a dot in it is read as a
        // path to somewhere else, so the field it would author is one nobody can
        // address.
        F::DottedPartialName { .. } => "Type a name with no dots in it. A dot separates a field from its parent, so a name containing one cannot be addressed.".to_owned(),
        // `a..b`, `.x`, `x.` — a period that starts, ends or doubles up. A
        // different rule from the one above with a different remedy, so it gets
        // its own sentence rather than being folded into the dot one.
        //
        F::EmptyNameSegment { .. } => "That name has a dot with nothing beside it. Remove the dot, or put a name on both sides of it.".to_owned(),
        // The catch-all does not guess. Reaching here means the engine refused
        // for a reason this shell has not met, and a wrong reason in a hover is
        // worse than none — it sends the operator to fix something that is not
        // broken.
        _ => "pdfcer cannot use that name. Try a different one.".to_owned(),
    }
}

/// The delete-the-field button.
#[must_use]
pub fn delete_field() -> String {
    "Delete field".to_owned()
}

/// What deleting the field will take with it.
#[must_use]
pub fn delete_field_hover(widgets: usize) -> String {
    if widgets <= 1 {
        "Removes the field from the form and the box from the page.".to_owned()
    } else {
        format!(
            "Removes the field from the form and all {widgets} of its boxes, on \
             every page they are drawn on."
        )
    }
}

/// The delete-this-box button.
#[must_use]
pub fn delete_box() -> String {
    "Delete this box".to_owned()
}

/// What deleting one box does, and the case where it does more.
#[must_use]
pub fn delete_box_hover() -> String {
    "Removes only the box you clicked. The field stays in the form, drawn \
     wherever else it appears."
        .to_owned()
}

/// **RETIRED 2026-08-27 — this sentence was false, and it recommended a
/// destructive workaround.**
#[must_use]
pub fn not_editable_note() -> String {
    "A box's size, border and visibility belong to that one placement rather than to the field, \
     and are not editable here yet."
        .to_owned()
}

//
// Every hover answers "what does this DO to the document", never "what is
// this called". `crate::text::tool`'s rule 2 — a sentence states a fact about
// the program, never a tip — and the practical test each one below passes: an
// operator who does not know what `/Ff` bit 2 is should be able to decide
// whether they want it from the hover alone.
// ===========================================================================

/// The heading over the editable properties.
#[must_use]
pub const fn editable_heading() -> &'static str {
    "Properties"
}

/// `/Ff` bit 2.
#[must_use]
pub const fn flag_required() -> &'static str {
    "Required"
}

/// It says what happens **at submit**, because that is the only moment the
/// flag does anything. A required field is not enforced while typing and is not
/// enforced on save; a reader checks it when the form is sent.
#[must_use]
pub const fn flag_required_hover() -> &'static str {
    "The form cannot be submitted while this field is empty. It does not stop anyone leaving it \
     empty while filling in the rest."
}

/// `/Ff` bit 1.
#[must_use]
pub const fn flag_read_only() -> &'static str {
    "Read only"
}

/// It names what read-only does **not** do, which is the half that gets
/// people: the value is still there, still exported and still printed. An
/// operator who sets this expecting the field to disappear has misread it.
#[must_use]
pub const fn flag_read_only_hover() -> &'static str {
    "Nobody filling in the form can change this field. Its value is still stored, still exported \
     and still printed."
}

/// `/Ff` bit 13, text fields only.
#[must_use]
pub const fn flag_multiline() -> &'static str {
    "Multiple lines"
}

/// See [`flag_multiline`].
#[must_use]
pub const fn flag_multiline_hover() -> &'static str {
    "Text wraps and Enter starts a new line. A single-line field ignores Enter and scrolls \
     sideways instead."
}

/// `/Ff` bit 14, text fields only.
#[must_use]
pub const fn flag_password() -> &'static str {
    "Hide as typed"
}

/// It states the security fact, because the control's name invites exactly
/// the wrong conclusion. `/Ff` bit 14 changes how a *reader draws* the value;
/// the characters are stored in the file in plain text and anyone with the file
/// can read them. An operator who used this for a password because it is called
/// one has been misled by the standard's own name for it, and this program is
/// not going to repeat the mistake silently.
#[must_use]
pub const fn flag_password_hover() -> &'static str {
    "Shows bullets instead of the characters while someone types. It does NOT protect anything \
     — the text is stored in the file in the clear and can be read out of it."
}

/// `/Ff` bit 25, text fields only.
#[must_use]
pub const fn flag_comb() -> &'static str {
    "Equal cells"
}

/// It names the maximum-length requirement, because the standard makes them
/// inseparable (Table 228) and the pane sends both — so an operator who ticks
/// this on a field with no limit will see a number appear above and should know
/// why rather than think the program changed something they did not ask for.
#[must_use]
pub const fn flag_comb_hover() -> &'static str {
    "Spreads the characters into equally-spaced boxes, the way a form asks for a postcode one \
     letter per square. It needs a maximum length, so turning it on sets one if there is none."
}

/// `/Ff` bit 15, radio groups only.
#[must_use]
pub const fn flag_no_toggle_off() -> &'static str {
    "Cannot be cleared"
}

/// See [`flag_no_toggle_off`].
#[must_use]
pub const fn flag_no_toggle_off_hover() -> &'static str {
    "Once one of these buttons is chosen, clicking it again does not clear it — the only way \
     to change the answer is to choose a different button."
}

/// `/Ff` bit 18, choice fields only.
#[must_use]
pub const fn flag_combo() -> &'static str {
    "Drop-down"
}

/// See [`flag_combo`].
#[must_use]
pub const fn flag_combo_hover() -> &'static str {
    "One line that opens a list when clicked. Turn it off for a box that shows several options \
     at once."
}

/// `/Ff` bit 22, choice fields only.
#[must_use]
pub const fn flag_multi_select() -> &'static str {
    "Allow several"
}

/// See [`flag_multi_select`].
#[must_use]
pub const fn flag_multi_select_hover() -> &'static str {
    "More than one option in the list can be chosen at the same time."
}

/// `/MaxLen`.
#[must_use]
pub const fn label_max_len() -> &'static str {
    "Maximum length"
}

/// It says what **zero** means, because that is the one thing the control's
/// appearance cannot say. A spinner reading 0 looks like a limit of nothing;
/// the pane spells zero as *no limit* because `/MaxLen` of zero is not
/// meaningful in a file and the value is free to carry the absence.
#[must_use]
pub const fn label_max_len_hover() -> &'static str {
    "How many characters this field accepts. Zero means no limit."
}

/// `/TU`.
#[must_use]
pub const fn label_tooltip() -> &'static str {
    "Tooltip"
}

/// See [`label_tooltip`].
#[must_use]
pub const fn label_tooltip_hint() -> &'static str {
    "What this field is for"
}

/// `/DV` — the value a Reset button puts back.
#[must_use]
pub const fn label_default_value() -> &'static str {
    "Default value"
}

/// See [`label_default_value`].
///
/// It says **empty** rather than *blank* or *none*, because the box being
/// empty is exactly the state it describes and the operator is looking at it.
#[must_use]
pub const fn label_default_value_hint() -> &'static str {
    "Empty — Reset will clear this field"
}

/// See [`label_default_value`].
#[must_use]
pub const fn label_default_value_hover() -> &'static str {
    "What a Reset button puts back in this field. Leave it empty and Reset clears the field \
     instead. This is separate from what the field says now."
}

/// `/Q` — which end of the box the field's text sits against.
#[must_use]
pub const fn label_alignment() -> &'static str {
    "Alignment"
}

/// The three justifications, named for the operator.
#[must_use]
pub const fn quadding_name(q: pdfcer_core::vartext::Quadding) -> &'static str {
    use pdfcer_core::vartext::Quadding as Q;
    match q {
        Q::Left => "Left",
        Q::Center => "Centred",
        Q::Right => "Right",
    }
}

/// The heading over the widget-scoped properties.
#[must_use]
pub const fn widget_heading() -> &'static str {
    "This box"
}

/// Shown only when the field is drawn in more than one place.
#[must_use]
pub fn widget_scope_note(boxes: usize) -> String {
    format!(
        "This field is drawn in {boxes} places. What follows changes only the one you clicked; \
         the properties above change all {boxes}."
    )
}

/// Lower-left x of the box.
#[must_use]
pub const fn label_widget_x() -> &'static str {
    "X"
}

/// Lower-left y of the box.
#[must_use]
pub const fn label_widget_y() -> &'static str {
    "Y"
}

/// Width of the box.
#[must_use]
pub const fn label_widget_w() -> &'static str {
    "Width"
}

/// Height of the box.
#[must_use]
pub const fn label_widget_h() -> &'static str {
    "Height"
}

/// The button that commits the four numbers.
#[must_use]
pub const fn widget_apply() -> &'static str {
    "Apply"
}

/// It says **which of two acts** is about to happen, before the press.
#[must_use]
pub fn widget_apply_hover(resizes: bool) -> &'static str {
    if resizes {
        "Resizes the box. What is drawn inside it is redrawn to fit, which for a stamp or a \
         signature may not be possible."
    } else {
        "Moves the box. What is drawn inside it moves with it, unchanged."
    }
}

/// Why Apply is greyed.
#[must_use]
pub const fn widget_apply_disabled() -> &'static str {
    "Change one of the four numbers above first."
}

/// `/MK` `/CA`.
#[must_use]
pub const fn label_caption() -> &'static str {
    "Caption"
}

/// The hint names the push-button case, because that is the one where a
/// caption is not decoration: a push button has no value at all (§12.7.4.2.2),
/// so the caption is the only thing telling anyone reading the field list what
/// the button does.
#[must_use]
pub const fn label_caption_hint() -> &'static str {
    "The words on a button"
}

// ===========================================================================
// `/MK` `/BG` and `/BC` — a widget's two colours. `OPERATOR_REQUESTS.md` O202.
//
// Table 189 gives each key four states and only two of them are a colour a
// swatch can draw, so each key needs a MARK for the button face and a NOTE for
// the popup in the states it cannot draw. The two keys get their own strings
// rather than one pair parameterised by a noun, because the sentences differ
// by more than the noun:
//
//   * a background stating *no colour* paints nothing at all, including on a
//     push button, which otherwise keeps its grey plate;
//   * a border stating *no colour* is still drawn BLACK, because a border's
//     thickness lives in `/BS` `/W` and a widget wanting no border says so
//     there. `WidgetChrome::stroke` resolves the empty array and the absent
//     key to the same black, and says why at length.
//
// ⚠ That second bullet contradicts O202's own decision 1, which was written
// against the engine as it stood before `Pass 308.0` and assumed an empty
// `/BC` would mean *draw no border*. It does not. The text below states what
// the engine measurably does.
// ===========================================================================

/// `/MK` `/BG` — the widget's background colour.
#[must_use]
pub const fn label_background() -> &'static str {
    "Background"
}

/// Names the two kinds whose background is not a plain rectangle, because
/// those are the two where an operator picking a colour would otherwise be
/// surprised by the shape of what appears.
#[must_use]
pub const fn label_background_hover() -> &'static str {
    "What is painted behind this box. A radio button's background is a disc rather than a \
     rectangle, and a push button's is the grey plate it sits on. Changing it redraws the box."
}

/// `/MK` `/BC`.
#[must_use]
pub const fn label_border_colour() -> &'static str {
    "Border and mark"
}

/// Says where the *thickness* comes from, because this row and the Border
/// width row above are two keys in two different dictionaries and an operator
/// who set a colour and saw no outline would otherwise have no way to learn
/// that the width is zero.
#[must_use]
pub const fn label_border_colour_hover() -> &'static str {
    "The ink the outline is drawn in — and the ink a check box's tick and a radio button's dot \
     are drawn in, because a box carries no separate colour for its mark. How thick the outline \
     is comes from Border width above, not from here."
}

/// The button face when the file states nothing about this colour.
#[must_use]
pub const fn colour_mark_unstated() -> &'static str {
    super::properties::text_value_absent()
}

/// The button face when the file states Table 189's empty array.
#[must_use]
pub const fn colour_mark_no_colour() -> &'static str {
    "None"
}

/// The popup entry that writes Table 189's empty array into `/BG`.
#[must_use]
pub const fn background_no_colour_entry() -> &'static str {
    "No background"
}

/// Why that entry is greyed. R9: greying is for a **temporarily** unavailable
/// capability and is always explained on hover.
#[must_use]
pub const fn background_no_colour_unavailable() -> &'static str {
    "This box already states that it has no background."
}

/// The popup entry that takes `/BG` out of the file altogether.
#[must_use]
pub const fn background_remove_entry() -> &'static str {
    "Back to the usual background"
}

/// Why that entry is greyed. R9: greying is for a **temporarily** unavailable
/// capability and is always explained on hover.
#[must_use]
pub const fn background_remove_unavailable() -> &'static str {
    "This box already says nothing about its background."
}

/// The popup entry that takes `/BC` out of the file altogether.
#[must_use]
pub const fn border_colour_remove_entry() -> &'static str {
    "Back to a black outline"
}

/// Why that entry is greyed. R9: greying is for a **temporarily** unavailable
/// capability and is always explained on hover.
#[must_use]
pub const fn border_colour_remove_unavailable() -> &'static str {
    "This box already says nothing about its outline colour, so it is drawn black."
}

/// The popup note over a `/BG` the file is silent about.
#[must_use]
pub const fn background_unstated_note() -> &'static str {
    "This file says nothing about a background colour, so the box is painted the way its kind is \
     normally painted — nothing behind a text field, the grey plate on a push button."
}

/// The popup note over a `/BG` that states the empty array.
#[must_use]
pub const fn background_no_colour_note() -> &'static str {
    "This box states that it has no background, so nothing is painted behind it — not even a push \
     button's plate. Picking a colour below replaces that."
}

/// The popup note over a `/BC` the file is silent about.
#[must_use]
pub const fn border_colour_unstated_note() -> &'static str {
    "This file says nothing about a border colour, so the outline and any mark are drawn in black."
}

/// The popup note over a `/BC` that states the empty array.
#[must_use]
pub const fn border_colour_no_colour_note() -> &'static str {
    "This box states no border colour, and pdfcer still draws the outline and any mark in black: \
     a box with no border says so through a border width of 0, not through this. Picking a colour \
     below replaces it."
}

/// The button face for a four-ink separation — the file's own four numbers.
#[must_use]
pub fn colour_cmyk_mark(c: f32, m: f32, y: f32, k: f32) -> String {
    format!("{c:.2} {m:.2} {y:.2} {k:.2}")
}

/// The popup note over a four-ink separation.
#[must_use]
pub const fn colour_cmyk_note() -> &'static str {
    "This colour is a four-ink CMYK separation, kept exactly as the file states it and never \
     converted, so there is no one screen colour to show for it. Picking a colour below replaces \
     the separation."
}

// ===========================================================================
// What the operator touched, for a refusal and for the receipt.
//
// These are OPERATOR-VISIBLE and live here for that reason. They reach the
// status line through `text::forms::field_widget_property_changed`, and they
// reach a refusal through the engine's §6 rule — *"the gates are checked
// against the RESULT, not against your request"* — which can name a property
// the request never mentioned, so the shell has to carry which control was
// actually pressed.
//
// They read as sentence fragments because they are one: *"Changed the border
// width."* is the whole line.
// ===========================================================================

/// The four geometry numbers, committed together by Apply.
#[must_use]
pub const fn touched_box() -> &'static str {
    "the box"
}

/// The border style combo.
#[must_use]
pub const fn touched_border() -> &'static str {
    "the border"
}

/// The dash picker.
#[must_use]
pub const fn touched_dash() -> &'static str {
    "the dash"
}

/// The dash row's label.
#[must_use]
pub const fn label_dash() -> &'static str {
    "Dash"
}

/// What the dash picker changes.
#[must_use]
pub const fn dash_hover() -> &'static str {
    "The pattern a dashed border is drawn with."
}

/// The border width spinner.
#[must_use]
pub const fn touched_border_width() -> &'static str {
    "the border width"
}

/// What an edit of a check box's mark touched, for the status line.
#[must_use]
pub const fn touched_mark() -> &'static str {
    "the mark"
}

/// A check box's mark when another program chose a symbol pdfcer has no
/// name for.
#[must_use]
pub fn mark_unnamed() -> String {
    "Another program's symbol".to_owned()
}

/// The caption field.
#[must_use]
pub const fn touched_caption() -> &'static str {
    "the caption"
}

/// The visibility combo.
#[must_use]
pub const fn touched_visibility() -> &'static str {
    "where the box is shown"
}

/// The `/BG` swatch.
#[must_use]
pub const fn touched_background() -> &'static str {
    "the background colour"
}

/// The `/BC` swatch. Named the way its label is, for the reason
/// [`label_border_colour`] gives.
#[must_use]
pub const fn touched_border_colour() -> &'static str {
    "the border and mark colour"
}

//
// Filed at 22:40 as *"a widget's border can be written and not read, so a
// properties control would lie"*, shipped by the engine within the hour, and
// consumed here. The whole exchange turned on one sentence of the request, and
// the engine quoted it back in three places of their own:
//
//   A properties control has to show the current value. One seeded from a
//   default would display *Solid 1 pt* over a widget whose file says
//   *Dashed 3 pt* and write the invention back on the first press.
//
// So `border: None` means **the file states no border**, and this pane renders
// that as a dash. It is a fact to display, never a value to substitute.
// ===========================================================================

/// `/BS` — the border style.
#[must_use]
pub const fn label_border() -> &'static str {
    "Border"
}

/// Shown when `Widget::border` is `None` — *the file says nothing about a
/// border.*
#[must_use]
pub const fn border_unstated() -> &'static str {
    "—  (this file says nothing about a border)"
}

/// The border combo's entry that takes the frame away: `/BS /W 0`.
#[must_use]
pub const fn border_none() -> &'static str {
    "No border"
}

/// Its tip. `keeps_mark` for a check box or radio button, whose mark is drawn
/// in the border colour.
#[must_use]
pub const fn border_none_hover(keeps_mark: bool) -> &'static str {
    if keeps_mark {
        "Draw no frame around this box. The border colour stays, because it is the colour of \
         the check mark or dot. A box another program drew is redrawn in pdfcer's style."
    } else {
        "Draw no frame around this box: the border width becomes 0 and the border colour is \
         taken out."
    }
}

/// One border style, in the operator's words.
#[must_use]
pub fn border_style_label(style: pdfcer_core::edit::BorderStyle) -> &'static str {
    use pdfcer_core::edit::BorderStyle;
    match style {
        BorderStyle::Solid => "Solid",
        BorderStyle::Dashed => "Dashed",
        BorderStyle::Beveled => "Raised edge",
        BorderStyle::Inset => "Sunken edge",
        BorderStyle::Underline => "Underline only",
        // NO catch-all arm, and that is deliberate rather than an oversight
        // the compiler let through. `BorderStyle` is **not**
        // `#[non_exhaustive]`, so exhaustiveness here means a sixth style added
        // to `pdfcer-core` fails to build in this file — which is exactly where
        // the decision belongs. A `_ => "Another style"` would compile for ever
        // and put a word in the operator's mouth about a style nobody had
        // looked at.
        //
        // A malformed `/S` cannot reach here: the engine degrades an
        // unrecognised name to Solid on the way in, per Table 166's own
        // default.
    }
}

/// The border width field.
#[must_use]
pub const fn label_border_width() -> &'static str {
    "Border width"
}

/// It states what **zero** means, because Table 166 makes zero a value —
/// *no border* — rather than an absence, and a spinner showing 0 cannot say
/// which it is on its own.
#[must_use]
pub const fn label_border_width_hover() -> &'static str {
    "How thick the border is drawn, in points. Zero means the file asks for no border at all, \
     which is different from the file saying nothing about one."
}

/// `/F` — where the widget is visible.
#[must_use]
pub const fn label_visibility() -> &'static str {
    "Shown"
}

/// One visibility, in the operator's words.
#[must_use]
pub fn visibility_label(visibility: pdfcer_core::edit::Visibility) -> &'static str {
    use pdfcer_core::edit::Visibility;
    match visibility {
        Visibility::VisibleAndPrints => "On screen and on paper",
        Visibility::ScreenOnly => "On screen only",
        Visibility::PrintOnly => "On paper only",
        Visibility::Hidden => "Nowhere — hidden",
        // Exhaustive, for [`border_style_label`]'s reason: `Visibility` is
        // the four combinations pdfcer can SET, and a fifth would be a decision
        // this file must be forced to make rather than allowed to paper over.
    }
}

/// Shown when `Widget::visibility` is `None` — *the file's flags are ones
/// pdfcer cannot set.*
#[must_use]
pub fn visibility_unmappable(flags: u32) -> String {
    format!(
        "This box uses display flags pdfcer cannot set (0x{flags:04X}), so they are left alone."
    )
}

/// **The box's current rotation**, and the fact that `None` is not zero.
#[must_use]
pub fn widget_rotation_label(rotation: Option<i64>) -> String {
    match rotation {
        None => "Turned: not set in this file (which draws the same as 0\u{00b0}).".to_owned(),
        Some(0) => "Turned: 0\u{00b0}.".to_owned(),
        Some(d) => format!("Turned: {d}\u{00b0} anticlockwise."),
    }
}

/// Turn the box a quarter turn to the LEFT.
#[must_use]
pub const fn widget_rotate_left() -> &'static str {
    "Turn left"
}

/// Turn the box a quarter turn to the right.
#[must_use]
pub const fn widget_rotate_right() -> &'static str {
    "Turn right"
}

/// What turning does and does not affect.
#[must_use]
pub const fn widget_rotation_hint() -> &'static str {
    "Turns what is drawn inside the box. The box itself stays where it is."
}

/// **The rotation landed** — the receipt.
#[must_use]
pub fn widget_rotated(now: i64, siblings: usize) -> String {
    if siblings == 0 {
        format!("Turned to {now}\u{00b0} anticlockwise.")
    } else if siblings == 1 {
        format!(
            "Turned to {now}\u{00b0} anticlockwise. This field has one other box and it was left \
             alone."
        )
    } else {
        format!(
            "Turned to {now}\u{00b0} anticlockwise. This field has {siblings} other boxes and they \
             were left alone."
        )
    }
}

/// **The rotation was written and the drawing did not follow.**
#[must_use]
pub fn widget_rotation_stale(why: &str) -> String {
    format!("The box will keep drawing the old way round until its appearance is rebuilt: {why}")
}

// ===========================================================================
// `/DA` — the ink, face and size of a field's own text. `OPERATOR_REQUESTS.md`
// O202's other half.
//
// The box's fill and outline are `/MK`, above; this is what is written INSIDE
// it. They are different dictionaries set by different verbs, and the engine's
// own note says conflating them is the likeliest way to end up with a field
// that looks nothing like what was intended — so the two groups are labelled
// so that no row here could be read as a third property of the box.
// ===========================================================================

/// The heading over the `/DA` rows.
#[must_use]
pub const fn text_heading() -> &'static str {
    "Text"
}

/// The face chooser's label.
#[must_use]
pub const fn label_text_font() -> &'static str {
    "Font"
}

/// Says the list is the fourteen faces every reader has built in, because an
/// operator who has just come from their word processor's font menu would
/// otherwise read a fourteen-entry list as pdfcer failing to find the rest.
#[must_use]
pub const fn label_text_font_hover() -> &'static str {
    "The face this field's own text is drawn in. These fourteen are built into every PDF reader, \
     so a form using them looks the same everywhere and carries no embedded font."
}

/// The size spinner's label.
#[must_use]
pub const fn label_text_size() -> &'static str {
    "Size"
}

/// Names what zero means, which is the one thing about this control that is
/// not guessable from looking at it.
#[must_use]
pub const fn label_text_size_hover() -> &'static str {
    "Points. Zero means the reader picks a size that fits the box, and re-picks it as the value \
     changes — which is what a new field normally wants."
}

/// How a size of zero is written on the face of the spinner.
#[must_use]
pub const fn text_size_auto() -> &'static str {
    "Auto"
}

/// The text-colour swatch's label.
#[must_use]
pub const fn label_text_colour() -> &'static str {
    "Text colour"
}

/// Says it is the value and not the box, for the reason the label above gives.
#[must_use]
pub const fn label_text_colour_hover() -> &'static str {
    "The ink the field's own value is drawn in — not the box behind it, which is Background."
}

/// A face the document embedded, named by the key its own `/DA` uses.
#[must_use]
pub fn text_font_embedded(key: &str) -> String {
    format!("{key} (this document's own)")
}

/// The operator-facing name of one of the fourteen built-in faces.
#[must_use]
pub const fn text_font_name(font: pdfcer_core::fontdata::Std14) -> &'static str {
    use pdfcer_core::fontdata::Std14 as F;
    match font {
        F::Helvetica => "Helvetica",
        F::HelveticaBold => "Helvetica Bold",
        F::HelveticaOblique => "Helvetica Italic",
        F::HelveticaBoldOblique => "Helvetica Bold Italic",
        F::TimesRoman => "Times",
        F::TimesBold => "Times Bold",
        F::TimesItalic => "Times Italic",
        F::TimesBoldItalic => "Times Bold Italic",
        F::Courier => "Courier",
        F::CourierBold => "Courier Bold",
        F::CourierOblique => "Courier Italic",
        F::CourierBoldOblique => "Courier Bold Italic",
        F::Symbol => "Symbol",
        F::ZapfDingbats => "Zapf Dingbats",
    }
}

/// Stands in for the swatch when the file states an ink no swatch can draw.
#[must_use]
pub const fn text_colour_unshowable() -> &'static str {
    "This field's text is set in an ink with no single screen colour — a four-ink separation, or \
     a colour space pdfcer keeps exactly as the file states it. It is left as it is."
}

/// What the operator touched, for a refusal and for the receipt. Reads as a
/// sentence fragment because it is one: *"Changed the text colour."*
#[must_use]
pub const fn touched_text_colour() -> &'static str {
    "text colour"
}

/// What the operator touched, for a refusal and for the receipt.
#[must_use]
pub const fn touched_text_font() -> &'static str {
    "font"
}

/// What the operator touched, for a refusal and for the receipt.
#[must_use]
pub const fn touched_text_size() -> &'static str {
    "text size"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The limitation note does not tell the operator to delete their
    /// field**, which is what it did until 2026-08-27.
    #[test]
    fn the_limitation_note_never_advises_deleting_the_field() {
        let note = not_editable_note();
        assert!(
            !note.contains("delete this field"),
            "the note recommends a destructive workaround: {note}"
        );
        assert!(
            !note.contains("place a new one"),
            "the note recommends a destructive workaround: {note}"
        );
        assert!(
            note.contains("placement"),
            "it still says what IS out of reach: {note}"
        );
    }

    /// **A field with no type is described as a defect, not as "unknown".**
    #[test]
    fn a_typeless_field_is_described_as_unfillable() {
        let mut field = sample();
        field.field_type = None;
        let described = field_type(&field);
        assert!(
            described.contains("No type"),
            "must name the defect: {described}"
        );
        assert!(
            !described.to_lowercase().contains("unknown"),
            "\u{201c}unknown\u{201d} blames pdfcer for a property of the file: {described}"
        );
    }

    /// **A `/Btn` is told apart into its three real controls**, because the
    /// spec's one type is three different things to a person.
    #[test]
    fn the_three_button_kinds_are_named_separately() {
        use pdfcer_core::forms::ButtonKind;
        let mut described = Vec::new();
        for kind in [ButtonKind::Check, ButtonKind::Radio, ButtonKind::Push] {
            let mut field = sample();
            field.field_type = Some(FieldType::Button);
            field.button_kind = Some(kind);
            described.push(field_type(&field));
        }
        let before = described.len();
        described.sort();
        described.dedup();
        assert_eq!(before, described.len(), "two button kinds share a name");
    }

    /// **A field with nothing set shows no flags line at all** — R9 applied to
    /// a fact: an empty row is worse than no row.
    #[test]
    fn no_flags_means_no_line() {
        assert_eq!(field_flags(&sample()), None);
    }

    /// **The box count discloses the click as well as the total**, because a
    /// field drawn three times can be changed from any of them.
    #[test]
    fn the_box_count_says_which_one_was_clicked() {
        let line = box_count(3, 1);
        assert!(line.contains('3'), "the total: {line}");
        assert!(
            line.contains("#2"),
            "1-based, as the operator counts: {line}"
        );
    }

    /// A plain text field with nothing set.
    fn sample() -> Field {
        use pdfcer_core::forms::{FieldFlags, FieldValue};
        use pdfcer_core::object::ObjId;
        use pdfcer_core::vartext::Quadding;
        Field {
            id: ObjId::new(1, 0),
            fully_qualified_name: "Name".to_owned(),
            partial_name: None,
            alternate_name: None,
            mapping_name: None,
            rich_value: None,
            default_style: None,
            field_type: Some(FieldType::Text),
            button_kind: None,
            flags: FieldFlags(0),
            value: FieldValue::Absent,
            default_value: FieldValue::Absent,
            default_appearance: None,
            quadding: Quadding::Left,
            max_len: None,
            options: Vec::new(),
            top_index: 0,
            selected_indices: Vec::new(),
            widgets: Vec::new(),
            merged: false,
            has_additional_actions: false,
            shares_parent_name: false,
            parent: None,
        }
    }
}
