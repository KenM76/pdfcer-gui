//! # `text::panels::formfield` — the words in the form-field properties section
//!
//! Covers [`crate::panels::properties::formfield`], the section that appears
//! when an operator clicks a form field on the page in Edit mode.
//!
//! ## ★★★ The sentence this file exists for
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
//! ★★ It is worded as a **statement about pdfcer**, not about PDF. The format
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
///
/// ★ 1-based, because that is what the page strip shows and what the operator
/// would say out loud. Every 0-based index in this shell stops at the boundary
/// with the person using it.
#[must_use]
pub fn page_number(one_based: usize) -> String {
    format!("{one_based}")
}

/// How many boxes the field is drawn as, and which one was clicked.
///
/// ★★ Shown only when there is more than one, and it is a **disclosure** rather
/// than a statistic: a field drawn in three places can be changed from three
/// pages, and the two the operator is not looking at change with it. Nothing on
/// the page says so.
#[must_use]
pub fn box_count(total: usize, clicked: usize) -> String {
    format!("{total} — you clicked #{}", clicked.saturating_add(1))
}

/// What kind of control this is, in the operator's vocabulary.
///
/// ★ A `/Btn` is three different controls and the spec tells them apart by
/// flag bits, not by type. Reporting all three as "button" would be accurate
/// about the format and useless to a person: a check box and a push button have
/// nothing in common from where they sit.
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
        // ★★ Not "unknown" and not blank. A field with no `/FT` is a real and
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
    // ★★ `display_text` and not a decode of our own. A `/V` is raw bytes, and
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
///
/// ★ Only the ones that are **set**. A list of every flag with yes/no beside it
/// would be six rows of "No" on a typical field, and the reader has to search
/// it to learn anything. Naming the exceptions is what a person would do.
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
    // ★ The type-specific bits are read only for the type they belong to,
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
///
/// ★ It asks for the **short** name and says so, because `rename_field` takes a
/// partial name and rebuilds the qualified one from the parent chain. An
/// operator who copied `Address.Line1` out of the row above and pasted it here
/// would author a `/T` containing a dot, which nothing can address again.
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
///
/// # ★★★ Drawn INSTEAD of the control, not under it
///
/// This is the sentence R9 asks for when a capability is refused *permanently
/// for this document*: the box and the button are not drawn, and this line
/// takes their place. A greyed box would say the state is temporary — a
/// certification signature is not — and would hide its own explanation behind a
/// hover.
///
/// ★★ It names **renaming**, not "editing", and that is deliberate. The pane
/// around it still offers seven editable properties, because a certified form
/// at `/P 2` permits filling and forbids restructuring (§12.8.2.2 Table 257). A
/// sentence saying "this document cannot be changed" would be false about the
/// controls directly below it, and an operator who believed it would stop
/// trying things that work.
///
/// ★ It does not name the *cause* by name. `rename_refusal` answers for
/// encryption as well as certification, and this shell has already shipped one
/// structural-refusal string that names certification unconditionally — which
/// is silently wrong on an encrypted file and sends the operator looking for a
/// signature that is not there. Here the sentence says what is true of both:
/// the document forbids it.
#[must_use]
pub fn rename_refused() -> String {
    "This document does not allow its form fields to be renamed. The values below can still \
     be filled in and changed."
        .to_owned()
}

/// **Why there are no delete buttons** — `EditSession::deletion_refusal`
/// answered `Some` (R83).
///
/// ★★ One sentence for both buttons, because both are refused by one gate:
/// deleting a field and deleting one of its boxes are both *structural* changes
/// to the form, which is precisely what a certification signature exists to
/// freeze. Two sentences saying the same thing twice, one above the other,
/// would read as two separate problems.
///
/// ★★★ It says what the operator can do **instead**, and the answer is not
/// "delete it anyway" — it is that the form's structure is fixed and its
/// contents are not. Without that clause the sentence is a dead end, and a dead
/// end in a properties pane reads as a broken program rather than as a
/// protected document.
#[must_use]
pub fn delete_refused() -> String {
    "This document does not allow form fields to be removed. Its structure is fixed; the \
     values in it can still be filled in and changed."
        .to_owned()
}

/// Why Rename is greyed.
#[must_use]
pub fn rename_disabled() -> String {
    "Type a name with no dots in it. A dot separates a field from its parent, \
     so a name containing one cannot be addressed."
        .to_owned()
}

/// The delete-the-field button.
#[must_use]
pub fn delete_field() -> String {
    "Delete field".to_owned()
}

/// What deleting the field will take with it.
///
/// ★★ Names the count in the hover rather than after the fact, because that is
/// where it can still change the operator's mind. A confirmation that said
/// "deleted from 3 pages" afterwards is a report; this is a warning.
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

/// ★★★ **RETIRED 2026-08-27 — this sentence was false, and it recommended a
/// destructive workaround.**
///
/// It read:
///
/// > ~~Required, read-only, the tooltip and the border can only be set when a
/// > field is placed. To change one, delete this field and place a new one.~~
///
/// `EditSession::edit_field` and `edit_widget` landed on **2026-08-26**, the
/// same day this sentence was written, three commits before the revision the
/// shell compiles against — and the engine wrote a full pane design brief into
/// the request channel saying so. Nothing consumed it, so for a day the program
/// told an operator to **delete their field and start again** for a capability
/// it already had. Delete-and-replace loses the field's name, its filled value
/// and its place in the tab order, every one of which an FDF import or a
/// filling script keys on.
///
/// The function is **kept and rewritten** rather than deleted, because there is
/// still something true to say in the same place: the properties that remain
/// out of reach are the *widget*-scoped ones — the box, the border, where it is
/// visible — and an operator who has just found six editable flags will
/// reasonably wonder where the seventh is. An absence with no explanation is
/// indistinguishable from an oversight; that was the right instinct in the old
/// sentence and it is the only part of it that survives.
///
/// ★ It names no remedy now, because there is no honest one. Delete-and-replace
/// still "works" for a border and it is not advice this program should give.
#[must_use]
pub fn not_editable_note() -> String {
    "A box's size, border and visibility belong to that one placement rather than to the field, \
     and are not editable here yet."
        .to_owned()
}

// ===========================================================================
// The editable properties — `EditSession::edit_field`, consumed 2026-08-27
//
// ★★ Every hover answers "what does this DO to the document", never "what is
// this called". `crate::text::tool`'s rule 2 — a sentence states a fact about
// the program, never a tip — and the practical test each one below passes: an
// operator who does not know what `/Ff` bit 2 is should be able to decide
// whether they want it from the hover alone.
// ===========================================================================

/// The heading over the editable properties.
///
/// ★ *"Properties"*, not *"Editable properties"*. The section directly above it
/// is headed with the facts that are genuinely read-only, and a heading that
/// advertised editability would invite the question of why the other section is
/// not editable — which is a fact about the file (a name, a type, a page) and
/// not a limitation.
#[must_use]
pub const fn editable_heading() -> &'static str {
    "Properties"
}

/// `/Ff` bit 2.
#[must_use]
pub const fn flag_required() -> &'static str {
    "Required"
}

/// ★ It says what happens **at submit**, because that is the only moment the
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

/// ★★ It names what read-only does **not** do, which is the half that gets
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

/// ★★★ It states the security fact, because the control's name invites exactly
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

/// ★★ It names the maximum-length requirement, because the standard makes them
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

/// ★ It says what **zero** means, because that is the one thing the control's
/// appearance cannot say. A spinner reading 0 looks like a limit of nothing;
/// the pane spells zero as *no limit* because `/MaxLen` of zero is not
/// meaningful in a file and the value is free to carry the absence.
#[must_use]
pub const fn label_max_len_hover() -> &'static str {
    "How many characters this field accepts. Zero means no limit."
}

/// `/TU`.
///
/// ★ *"Tooltip"* is the word the standard's own name (`/TU`, "alternate field
/// name") does not use and every application does. It is also what a screen
/// reader announces, which is the fact the hover carries.
#[must_use]
pub const fn label_tooltip() -> &'static str {
    "Tooltip"
}

/// See [`label_tooltip`].
#[must_use]
pub const fn label_tooltip_hint() -> &'static str {
    "What this field is for"
}

// ===========================================================================
// The BOX — `EditSession::edit_widget`, consumed 2026-08-27
// ===========================================================================

/// The heading over the widget-scoped properties.
///
/// ★★ *"This box"*, not *"Widget"*. A widget annotation is what the file calls
/// it and is a word no operator has any use for; what they are looking at is a
/// rectangle on a page. The distinction the heading has to carry is not the
/// spec's vocabulary but the **scope** — that these properties belong to this
/// one rectangle and the ones above belong to the field — and
/// [`widget_scope_note`] says that in the one state where it is visible.
#[must_use]
pub const fn widget_heading() -> &'static str {
    "This box"
}

/// Shown only when the field is drawn in more than one place.
///
/// ★★★ **The one sentence that makes the field/widget split legible**, and it
/// is deliberately conditional. On a one-widget field — the overwhelming
/// majority — there is no distinction to explain and the sentence would be
/// noise. On a radio group it is the difference between changing one button and
/// changing the answer, which is exactly the state where an operator would
/// otherwise expect this section to behave like the one above it.
#[must_use]
pub fn widget_scope_note(boxes: usize) -> String {
    format!(
        "This field is drawn in {boxes} places. What follows changes only the one you clicked; \
         the properties above change all {boxes}."
    )
}

/// Lower-left x of the box.
///
/// ★ The four are labelled X / Y / Width / Height rather than with the
/// standard's `/Rect` corners, because a corner pair is a spelling and a
/// position-and-size is what an operator is thinking about. `super::geometry`
/// made the same call for page objects and this matches it, so the two
/// surfaces read the same way.
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

/// ★★★ It says **which of two acts** is about to happen, before the press.
///
/// Moving and resizing are the same gesture on this pane and different acts on
/// the file: a pure translation moves the baked artwork exactly and for
/// nothing, while a changed extent makes §12.5.5's algorithm *scale* it, so a
/// text field made twice as wide is redrawn rather than given room for more
/// text. An operator who expected the second and got the first — or the other
/// way round — has been surprised by something the program knew in advance.
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
///
/// R9: greying is for a **temporarily** unavailable capability and must be
/// explained on hover. The capability is present and the operand — a number
/// the operator has changed — is not.
#[must_use]
pub const fn widget_apply_disabled() -> &'static str {
    "Change one of the four numbers above first."
}

/// `/MK` `/CA`.
#[must_use]
pub const fn label_caption() -> &'static str {
    "Caption"
}

/// ★ The hint names the push-button case, because that is the one where a
/// caption is not decoration: a push button has no value at all (§12.7.4.2.2),
/// so the caption is the only thing telling anyone reading the field list what
/// the button does.
#[must_use]
pub const fn label_caption_hint() -> &'static str {
    "The words on a button"
}

// ===========================================================================
// The BORDER and the VISIBILITY — `Pass 146.0`, consumed 2026-08-27
//
// ★★★ Filed at 22:40 as *"a widget's border can be written and not read, so a
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

/// ★★ Shown when `Widget::border` is `None` — *the file says nothing about a
/// border.*
///
/// **Not "Solid, 1 pt".** `BorderSpec::default()` is solid/1 pt because that
/// reproduces the bytes pdfcer authors, which is correct for a writer and a lie
/// from a reader. The engine made the field an `Option` specifically so this
/// distinction survives, and their load-bearing test —
/// `a_widget_whose_file_states_no_border_reads_a_dash_not_a_default` — goes red
/// if the reader substitutes.
///
/// ★ Distinct from a border of **width 0**, which Table 166 states as a value
/// meaning *no border*. That is the file saying something definite and it reads
/// as `0 pt`, not as this. Collapsing the two would tell an operator the file
/// is silent when it has spoken.
#[must_use]
pub const fn border_unstated() -> &'static str {
    "—  (this file says nothing about a border)"
}

/// One border style, in the operator's words.
///
/// ★ Beveled and Inset are named by their **appearance** rather than by the
/// standard's word, because *"beveled"* describes a 3-D raised edge that a
/// person recognises on sight and cannot name, while *"inset"* is the same edge
/// the other way up. The other three need no help.
#[must_use]
pub fn border_style_label(style: pdfcer_core::edit::BorderStyle) -> &'static str {
    use pdfcer_core::edit::BorderStyle;
    match style {
        BorderStyle::Solid => "Solid",
        BorderStyle::Dashed => "Dashed",
        BorderStyle::Beveled => "Raised edge",
        BorderStyle::Inset => "Sunken edge",
        BorderStyle::Underline => "Underline only",
        // ★ NO catch-all arm, and that is deliberate rather than an oversight
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

/// ★ It states what **zero** means, because Table 166 makes zero a value —
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
///
/// ★ Named by **where you see it** rather than by the flag combination, which
/// is the only framing that answers the question an operator is asking. `/F`'s
/// four settable combinations are Hidden, Print, NoView and their pairings;
/// *"on screen and on paper"* is what those mean.
#[must_use]
pub fn visibility_label(visibility: pdfcer_core::edit::Visibility) -> &'static str {
    use pdfcer_core::edit::Visibility;
    match visibility {
        Visibility::VisibleAndPrints => "On screen and on paper",
        Visibility::ScreenOnly => "On screen only",
        Visibility::PrintOnly => "On paper only",
        Visibility::Hidden => "Nowhere — hidden",
        // ★ Exhaustive, for [`border_style_label`]'s reason: `Visibility` is
        // the four combinations pdfcer can SET, and a fifth would be a decision
        // this file must be forced to make rather than allowed to paper over.
    }
}

/// ★★★ Shown when `Widget::visibility` is `None` — *the file's flags are ones
/// pdfcer cannot set.*
///
/// The engine's mapping is **exact-or-`None`**, never nearest, and their note
/// on why is the same argument as the border's: `/F` admits dozens of
/// combinations and `Visibility` is the four pdfcer can write, so a file
/// carrying `Print | NoZoom` has no nearest of the four that is not a lie.
///
/// ★ `None` here can never mean *absent*: Table 164 makes an absent `/F` equal
/// to `0`, which **is** one of the four. So this sentence is always about a
/// file that has said something pdfcer cannot express, and it says exactly that
/// rather than showing nothing.
#[must_use]
pub fn visibility_unmappable(flags: u32) -> String {
    format!(
        "This box uses display flags pdfcer cannot set (0x{flags:04X}), so they are left alone."
    )
}

/// **The box's current rotation**, and the fact that `None` is not zero.
///
/// ★★ `Widget::rotation` is `Option<i64>`, and the distinction is the same one
/// `Widget::border`'s docs call *"a fact to display, not a value to
/// substitute"*: `None` means **the file states none**, `Some(0)` means the
/// file says zero. They render identically and they are different facts, and an
/// operator debugging why a box looks wrong in another viewer wants to know
/// which their file carries.
#[must_use]
pub fn widget_rotation_label(rotation: Option<i64>) -> String {
    match rotation {
        None => "Turned: not set in this file (which draws the same as 0\u{00b0}).".to_owned(),
        Some(0) => "Turned: 0\u{00b0}.".to_owned(),
        Some(d) => format!("Turned: {d}\u{00b0} anticlockwise."),
    }
}

/// Turn the box a quarter turn to the LEFT.
///
/// ★★★ *Left* and *right*, never *clockwise* and *anticlockwise*, and never a
/// signed number. `/MK /R` is counterclockwise while the page's `/Rotate` is
/// clockwise, and the standard's two sentences differ by exactly one word — so
/// a label that named a direction convention would be asking the operator to
/// hold the trap that caught the engine's own reviewers. *Left* is what they
/// watch the box do.
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
///
/// ★ It says the box stays put, because that is the surprise: `/MK /R` turns
/// what is drawn INSIDE the rectangle and leaves the rectangle itself alone
/// (§12.5.5 maps the appearance's `/BBox` into `/Rect`). An operator expecting
/// a tall box to become a wide one needs telling once.
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
///
/// ★★ Not a failure: the file is correct and carries the new angle. The baked
/// appearance could not be regenerated, so the box keeps drawing at its old
/// orientation until something regenerates it — and an operator watching a box
/// refuse to turn is owed the reason rather than a mystery.
#[must_use]
pub fn widget_rotation_stale(why: &str) -> String {
    format!("The box will keep drawing the old way round until its appearance is rebuilt: {why}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The limitation note does not tell the operator to delete their
    /// field**, which is what it did until 2026-08-27.
    ///
    /// The test it replaces asserted the opposite — it required the string
    /// `"delete this field"` to be present, on the reasoning that *"a note that
    /// only said 'cannot be changed' would leave the operator stuck"*. That
    /// reasoning was sound and its premise was false: the capability existed,
    /// so the operator was not stuck, and the test was pinning a sentence that
    /// recommended destroying a field's name, value and tab position for
    /// nothing.
    ///
    /// ★ A test can pin a sentence and cannot know whether the sentence is
    /// true. This one is written in the negative for that reason: it does not
    /// try to say what the note should claim, only that it must not send an
    /// operator down the destructive route again.
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

    /// ★★ **A field with no type is described as a defect, not as "unknown".**
    ///
    /// A `/FT`-less field is what a bare kid that lost its `/Parent` becomes,
    /// and no viewer can fill it. "Unknown" would read as pdfcer failing to
    /// look; this says what is true of the document.
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
