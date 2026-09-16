//! # `text::panels::choiceopts` — a drop-down's option list, in the operator's
//! words
//!
//! The copy for `panels::properties::choiceopts`: the `/Opt` editor's column
//! headers and row buttons, the three `/Ff` flags Acrobat groups with them, the
//! default-choice chooser, and the two sentences the shell owes when it refuses
//! an option or empties the list.
//!
//! Its own module rather than more of [`super::formfield`] under R2 — that file
//! stood at 1,434 of its 1,500-line ceiling when this was written — and the
//! seam is the one the code takes: `properties::fieldedit` draws a field's
//! flags, `properties::choiceopts` draws the list those flags describe.
//!
//! ## The vocabulary decision that governs every string here
//!
//! An `/Opt` entry is a **pair**: what the reader shows and what a submit
//! sends. §12.7.4.4 lets the two coincide, and in most files they do, which is
//! why an editor offering one box teaches an operator the pair does not exist.
//! Collapsing them is named in the engine's own note as something that *"would
//! silently break forms"* — the document opens, the list reads correctly, and
//! the submitted data is wrong.
//!
//! So both columns are always drawn, and they are named for what they *do*
//! rather than for the standard's keys: **Shown** and **Sent**. Acrobat calls
//! them *Item* and *Export Value*; "export value" is a word for a thing the
//! operator has never exported, and "item" does not say it is the visible half
//! of a pair.

/// The section heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Options"
}

/// The left column: what a reader displays for this entry.
#[must_use]
pub const fn column_shown() -> &'static str {
    "Shown"
}

/// See [`column_shown`].
#[must_use]
pub const fn column_shown_hover() -> &'static str {
    "What the person filling the form sees in the list."
}

/// The right column: what a submit sends for this entry.
#[must_use]
pub const fn column_sent() -> &'static str {
    "Sent"
}

/// See [`column_sent`].
///
/// It says the two are **usually the same** because that is the state the
/// operator is looking at, and an editor showing two identical columns with no
/// explanation reads as a mistake rather than as a capability.
#[must_use]
pub const fn column_sent_hover() -> &'static str {
    "What this answer is worth when the form is submitted or exported. Usually the same as \
     what is shown — set it differently when the form has to send a code."
}

/// The move-up button.
///
/// `\u{23f6}`, not `\u{25b2}` BLACK UP-POINTING
/// TRIANGLE, and the reason is
/// [`super::bookmarks::bookmark_collapsed_glyph`]'s: U+25B2 is in
/// `icons::glyphs`'s genuinely-absent row, so it would draw as a
/// substitution box in front of the operator. The U+23F4-U+23F7 block is
/// supplied by `emoji-icon-font`, and taking both halves of the pair from
/// one face is what stops a missing glyph reading as a direction.
#[must_use]
pub const fn row_up() -> &'static str {
    "\u{23f6}"
}

/// See [`row_up`].
#[must_use]
pub const fn row_up_hover() -> &'static str {
    "Move this option one place up the list."
}

/// The move-down button. The same triangle turned down, from the same
/// face; see [`row_up`].
#[must_use]
pub const fn row_down() -> &'static str {
    "\u{23f7}"
}

/// See [`row_down`].
#[must_use]
pub const fn row_down_hover() -> &'static str {
    "Move this option one place down the list."
}

/// The remove button.
///
/// `\u{00d7}` MULTIPLICATION SIGN, for
/// [`crate::text::find::close`]'s reason rather than a new one:
/// `\u{2715}` MULTIPLICATION X is in `icons::glyphs`'s absent
/// row, and U+00D7 is supplied by `Ubuntu-Light`.
#[must_use]
pub const fn row_remove() -> &'static str {
    "\u{00d7}"
}

/// See [`row_remove`].
///
/// It says what removing an option does to an answer already given, because
/// that is the consequence the button's appearance cannot carry and pdfcer
/// deliberately does not repair: re-pointing a selection would be inventing an
/// answer the operator never gave.
#[must_use]
pub const fn row_remove_hover() -> &'static str {
    "Take this option off the list. If the field is already set to it, the field keeps that \
     answer and the panel says so."
}

/// The new-option box's placeholder.
#[must_use]
pub const fn add_hint() -> &'static str {
    "New option"
}

/// The add button.
#[must_use]
pub const fn add_button() -> &'static str {
    "Add"
}

/// See [`add_button`].
#[must_use]
pub const fn add_button_hover() -> &'static str {
    "Add this to the end of the list. Pressing Enter in the box does the same."
}

/// What the section says in place of an empty table.
///
/// Not *"No options."* — the fact worth stating is the **consequence**, which
/// is that the field cannot be filled in at all. `EditSession` allows a
/// zero-option choice field and discloses it, for the same reason.
#[must_use]
pub const fn no_options_yet() -> &'static str {
    "No options yet — a drop-down with an empty list cannot be filled in."
}

/// `/Ff` bit 20.
///
/// **"Keep sorted", not "Sorted"**, because the control does both halves: it
/// puts the list in order now and keeps a new option in order as it arrives.
/// The flag on its own sorts nothing — Table 230 makes it *"intended for use by
/// writers, not by readers"* — so a label naming only the flag would describe a
/// checkbox that appears to do nothing.
#[must_use]
pub const fn flag_sort() -> &'static str {
    "Keep sorted"
}

/// See [`flag_sort`].
///
/// It states what turning it **off** does, because that is the half an operator
/// gets wrong: there is no stored "original order" to come back to.
#[must_use]
pub const fn flag_sort_hover() -> &'static str {
    "Puts the list in A-to-Z order by what is shown, and keeps new options in order. Turning \
     it off leaves the list in whatever order it is in now."
}

/// Why the move-up button is greyed on the first option.
///
/// R9 wants every greyed control explained, and this one is
/// self-explanatory only to someone who has already noticed which row
/// they are on. It names the state rather than the rule, because the
/// state is the whole reason.
#[must_use]
pub const fn already_first_hover() -> &'static str {
    "This is already the first option."
}

/// Why the move-down button is greyed on the last option. See
/// [`already_first_hover`].
#[must_use]
pub const fn already_last_hover() -> &'static str {
    "This is already the last option."
}

/// Why the reorder buttons are greyed while the list is kept sorted.
#[must_use]
pub const fn reorder_locked_hover() -> &'static str {
    "The list is kept sorted, so its order comes from what is shown. Turn off Keep sorted to \
     move options by hand."
}

/// `/DV` for a choice field.
#[must_use]
pub const fn label_default() -> &'static str {
    "Default choice"
}

/// See [`label_default`].
#[must_use]
pub const fn label_default_hover() -> &'static str {
    "Which option a Reset button puts back. This is separate from what the field says now."
}

/// The default-choice chooser's "no default" entry.
///
/// *"Nothing"* rather than *"None"*: the state it names is the field being
/// **emptied** by a reset, not the chooser having no entry selected.
#[must_use]
pub const fn default_none() -> &'static str {
    "Nothing — Reset clears the field"
}

/// `/Ff` bit 19.
#[must_use]
pub const fn flag_editable() -> &'static str {
    "Allow typing"
}

/// See [`flag_editable`].
#[must_use]
pub const fn flag_editable_hover() -> &'static str {
    "The person filling the form can type an answer that is not in the list."
}

/// Why [`flag_editable`] is greyed on a list box.
///
/// Table 230 makes bit 19 *"shall be used only if"* bit 18 is set, so the
/// engine refuses the press by name. Greyed rather than absent because this is
/// R9's **temporary** unavailability — one checkbox above makes it available —
/// and the hover names that checkbox.
#[must_use]
pub const fn flag_editable_needs_combo_hover() -> &'static str {
    "Turn on Drop-down first. The PDF standard only allows typing in a drop-down, not in a \
     list that shows several options at once."
}

/// The hover when the FILE already breaks Table 230 — typing allowed on
/// something that is not a drop-down.
///
/// The control stays live in this state, which is the whole point: clearing the
/// flag is the only way to make the field conform, and greying it would leave
/// the operator looking at a defect they cannot fix.
#[must_use]
pub const fn flag_editable_without_combo_hover() -> &'static str {
    "This file allows typing on a field that is not a drop-down, which the PDF standard does \
     not permit. Clearing this, or turning on Drop-down, fixes it."
}

/// `/Ff` bit 23, written as the positive the operator expects.
///
/// The flag is `DoNotSpellCheck`, so the checkbox shows its **inverse**. Worth
/// the inversion: *"Check spelling"* is what Acrobat's field properties says and
/// what every other application says, and a checkbox labelled "Do not check
/// spelling" is read wrongly by roughly half of everybody.
#[must_use]
pub const fn flag_spell_check() -> &'static str {
    "Check spelling"
}

/// See [`flag_spell_check`].
///
/// It says **pdfcer does not spell-check**, because the setting otherwise reads
/// as a promise this program makes. It is a note in the file for whichever
/// reader fills the form in.
#[must_use]
pub const fn flag_spell_check_hover() -> &'static str {
    "Records in the file that what is typed here should be spell-checked. pdfcer does not \
     spell-check; the reader used to fill the form in does."
}

/// `/Ff` bit 27.
#[must_use]
pub const fn flag_commit_now() -> &'static str {
    "Apply choice immediately"
}

/// See [`flag_commit_now`].
#[must_use]
pub const fn flag_commit_now_hover() -> &'static str {
    "The answer takes effect the moment an option is picked, rather than when the field is \
     left. Turn it on when picking an option is meant to change something else on the form."
}

/// The shell's worded refusal: two options cannot send the same value.
///
/// # This sentence exists because the engine's refusal names nothing
///
/// Both `add_choice_field` and `edit_field` refuse a repeated export, so the
/// document is safe either way. What the engine's refusal cannot do is reach
/// the operator as anything but the funnel's un-categorised line — *"That
/// change was refused"* — and a person looking at a thirty-row drop-down needs
/// the value, not the verdict. The panel asks first and says which one.
#[must_use]
pub fn note_duplicate_sent(value: &str) -> String {
    format!(
        "Two options cannot send the same value. \"{value}\" is already in the list, and a \
         form filling itself in would only ever find the first one."
    )
}

/// The disclosure owed when the last option is removed.
#[must_use]
pub const fn note_list_now_empty() -> &'static str {
    "That was the last option. This field cannot be filled in until the list has something \
     in it."
}
