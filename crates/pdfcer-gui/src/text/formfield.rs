//! # `text::formfield` — every string the form-field placement dialog shows
//!
//! One area of the catalog described in [`crate::text`]'s header, covering
//! [`crate::dialogs::formfield`] — the pop-up that collects a control's details
//! after it has been placed on the page.
//!
//! It sits beside [`crate::text::forms`] rather than inside it because the two
//! answer opposite questions. That file is about **filling** a form somebody
//! else authored, and its copy is dominated by disclosures about what a document
//! does or does not support. This one is about **authoring**, and its copy is
//! dominated by labels for choices the operator is making right now.
//!
//! ## ★★ The vocabulary rule this file follows
//!
//! Every label here is the word the operator's other programs use, not the word
//! the PDF specification uses. The standing tie-breaker — *make it work the way
//! other programs do* — applies to vocabulary as much as to behaviour, and the
//! spec's names for these things are unusually bad for a UI:
//!
//! | spec | here | why |
//! |---|---|---|
//! | choice field (`/Ch`) | drop-down list | nobody outside the spec says "choice field" |
//! | `/TU` | tooltip | "alternate field name" describes the mechanism |
//! | `/AS` on state | value when ticked | "on state" is a name in a dictionary |
//! | comb | equal cells | the word means nothing; the picture is obvious |
//!
//! ## ★ What is deliberately NOT here
//!
//! The field-name stems (`Text`, `Check Box`, `Group`, …) that auto-generated
//! names are built from. Those are `/T` strings written into the file and keyed
//! on by form-filling scripts and FDF imports; translating them would rename
//! every field for an operator running a different language, invisibly, until an
//! import failed. They are literals on `FormFieldKind::name_prefix`.

use crate::canvas::formfield::FormFieldKind;

/// The window title, which names the kind being placed.
#[must_use]
pub fn title(kind: FormFieldKind) -> String {
    match kind {
        FormFieldKind::Text => "New text field".to_owned(),
        FormFieldKind::CheckBox => "New check box".to_owned(),
        FormFieldKind::Radio => "New radio button".to_owned(),
        FormFieldKind::Choice => "New drop-down list".to_owned(),
        FormFieldKind::PushButton => "New button".to_owned(),
    }
}

/// One line above the fields, saying what the operator is about to make.
#[must_use]
pub fn intro(kind: FormFieldKind) -> String {
    match kind {
        FormFieldKind::Text => "A box people can type into.".to_owned(),
        FormFieldKind::CheckBox => "A box people can tick.".to_owned(),
        FormFieldKind::Radio => {
            "One of a set of alternatives — picking one clears the others.".to_owned()
        }
        FormFieldKind::Choice => "A list people can pick from.".to_owned(),
        FormFieldKind::PushButton => "A button on the page.".to_owned(),
    }
}

/// The label above the name box.
///
/// ★★ It reads differently for a radio button, and that is the most important
/// wording decision in this file. For every other kind the name identifies
/// **this control**; for a radio it identifies **the group**, and two radios
/// sharing it is what makes them exclusive. An operator who reads the same
/// label on both will place three radios that are all separately tickable and
/// wonder why.
#[must_use]
pub fn name_label(kind: FormFieldKind) -> String {
    match kind {
        FormFieldKind::Radio => "Group name — all the buttons in one set share this".to_owned(),
        _ => "Name".to_owned(),
    }
}

/// Why Accept is greyed.
#[must_use]
pub fn accept_disabled() -> String {
    "Give the field a name first — it is how the form refers to it.".to_owned()
}

/// The Accept control.
#[must_use]
pub fn accept() -> String {
    "Add".to_owned()
}

/// The Cancel control.
#[must_use]
pub fn cancel() -> String {
    "Cancel".to_owned()
}

/// The label above the tooltip box.
#[must_use]
pub fn tooltip_label() -> String {
    "Tooltip".to_owned()
}

/// What goes in the tooltip box.
#[must_use]
pub fn tooltip_hint() -> String {
    "What this field is for".to_owned()
}

/// ★ The consequence of leaving the tooltip empty, stated always.
///
/// Not a warning and not conditional on the box being empty: it is a fact about
/// what a tooltip *does*, which is entirely invisible on screen. Rule 4's
/// surviving half asks for exactly this — report what cannot be seen, and do
/// not nag about it.
#[must_use]
pub fn tooltip_note() -> String {
    "Shown on hover, and read aloud by a screen reader. Leave it blank if the \
     field needs no explanation."
        .to_owned()
}

/// The label above a text field's starting value.
#[must_use]
pub fn value_label() -> String {
    "Starting value".to_owned()
}

/// What goes in the starting-value box.
#[must_use]
pub fn value_hint() -> String {
    "Usually left blank".to_owned()
}

/// The multi-line checkbox.
#[must_use]
pub fn multiline() -> String {
    "Allow more than one line".to_owned()
}

/// The password checkbox.
#[must_use]
pub fn password() -> String {
    "Hide what is typed".to_owned()
}

/// ★★ What "hide what is typed" does **not** mean.
///
/// Salvaged in substance from the old shell's `form_field_password_tooltip`,
/// which exists because a masked box reads as "secure" to anyone not told
/// otherwise. It is not: the value is stored as plain text in the file, and
/// anybody with the file can read it. Getting this wrong is the difference
/// between a UI convention and a false security claim.
#[must_use]
pub fn password_hover() -> String {
    "Shows dots instead of characters on screen. The value is still stored as \
     plain text in the file — this is not encryption."
        .to_owned()
}

/// The maximum-length checkbox.
#[must_use]
pub fn max_len() -> String {
    "Limit to".to_owned()
}

/// The comb checkbox.
#[must_use]
pub fn comb() -> String {
    "Space characters into equal cells".to_owned()
}

/// What comb looks like, for anyone who has not seen one.
#[must_use]
pub fn comb_hover() -> String {
    "Divides the box into one cell per character, the way a form asks for a \
     postcode or a serial number."
        .to_owned()
}

/// The check box's starts-ticked control.
#[must_use]
pub fn checked() -> String {
    "Starts ticked".to_owned()
}

/// The label above a check box's export value.
#[must_use]
pub fn export_label() -> String {
    "Value when ticked".to_owned()
}

/// What the export value is for.
#[must_use]
pub fn export_note() -> String {
    "What the form submits when the box is ticked. \u{201c}Yes\u{201d} is what \
     most software expects."
        .to_owned()
}

/// ★★ How a radio group works, said before the operator names one.
///
/// The single sentence that stops the most common form-authoring mistake:
/// placing three radio buttons with three different names and getting three
/// independent tick boxes that happen to be round.
#[must_use]
pub fn radio_group_note() -> String {
    "Place each button in the set with the same group name above, and a \
     different value below. Picking one then clears the others."
        .to_owned()
}

/// The label above a radio's export value.
#[must_use]
pub fn radio_export_label() -> String {
    "Value when this one is picked".to_owned()
}

/// The radio's starts-selected control.
#[must_use]
pub fn radio_selected() -> String {
    "Start with this one picked".to_owned()
}

/// The label above a drop-down's options.
#[must_use]
pub fn options_label() -> String {
    "Options".to_owned()
}

/// How to type the options.
#[must_use]
pub fn options_hint() -> String {
    "One per line".to_owned()
}

/// The drop-down alternative.
#[must_use]
pub fn combo() -> String {
    "Drop-down".to_owned()
}

/// The list-box alternative.
#[must_use]
pub fn list_box() -> String {
    "List".to_owned()
}

/// The editable-combo control.
#[must_use]
pub fn editable() -> String {
    "Allow typing an answer that is not listed".to_owned()
}

/// The multi-select control, offered for a list only.
#[must_use]
pub fn multi_select() -> String {
    "Allow picking more than one".to_owned()
}

/// The sort control.
#[must_use]
pub fn sort() -> String {
    "Sort the options".to_owned()
}

/// Who does the sorting, and when.
///
/// ★ Worth a hover because the answer is surprising: the flag asks the *viewer*
/// to sort, so what the operator typed and what a reader sees can differ, and
/// pdfcer is not the one doing it.
#[must_use]
pub fn sort_hover() -> String {
    "Asks the viewer to show them in alphabetical order rather than the order \
     typed above."
        .to_owned()
}

/// The label above a push button's caption.
#[must_use]
pub fn caption_label() -> String {
    "Words on the button".to_owned()
}

/// The required control.
#[must_use]
pub fn required() -> String {
    "Required".to_owned()
}

/// What "required" actually enforces, and where.
///
/// ★ Not in pdfcer. The flag is a request to whatever software submits the form,
/// and nothing stops a document being saved with the field empty — which is
/// worth saying, because "required" reads as a guarantee.
#[must_use]
pub fn required_hover() -> String {
    "Marks the field as one the form should not be submitted without. Nothing \
     stops the document being saved with it empty."
        .to_owned()
}

/// The read-only control.
#[must_use]
pub fn read_only() -> String {
    "Read-only".to_owned()
}

/// What read-only means here.
#[must_use]
pub fn read_only_hover() -> String {
    "The field is shown but cannot be changed — useful for a value filled in \
     from somewhere else."
        .to_owned()
}

/// The label beside the border width.
#[must_use]
pub fn border_label() -> String {
    "Border width".to_owned()
}

/// What zero means.
#[must_use]
pub fn border_hover() -> String {
    "In points. Zero draws no border at all.".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every kind gets its own title and its own opening line**, because the
    /// window is the only confirmation the operator gets that the button they
    /// pressed was the one they meant.
    #[test]
    fn every_kind_is_named_distinctly() {
        let mut titles: Vec<String> = FormFieldKind::ALL.into_iter().map(title).collect();
        let before = titles.len();
        titles.sort();
        titles.dedup();
        assert_eq!(titles.len(), before, "two kinds share a title");

        let mut intros: Vec<String> = FormFieldKind::ALL.into_iter().map(intro).collect();
        let before = intros.len();
        intros.sort();
        intros.dedup();
        assert_eq!(intros.len(), before, "two kinds share an opening line");
    }

    /// ★★★ **The radio button's name label says "group", and no other does.**
    ///
    /// The wording that prevents the most common form-authoring mistake. Tested
    /// rather than left to review because "Name" is the obvious label, it is
    /// correct for four of the five kinds, and unifying them would look like a
    /// tidy-up rather than a regression.
    #[test]
    fn only_the_radio_asks_for_a_group_name() {
        for kind in FormFieldKind::ALL {
            let label = name_label(kind);
            let mentions_group = label.to_lowercase().contains("group");
            assert_eq!(
                mentions_group,
                matches!(kind, FormFieldKind::Radio),
                "{kind:?} label: {label}"
            );
        }
    }

    /// ★★ **The password hover refuses the word "secure" and says "not
    /// encryption".**
    ///
    /// A masked box reads as secure to anyone not told otherwise, and the value
    /// really is plain text in the file. This is a false-claim guard, not a
    /// style test.
    #[test]
    fn the_password_hover_does_not_imply_security() {
        let hover = password_hover();
        assert!(
            hover.contains("not encryption"),
            "the disclaimer must be explicit: {hover}"
        );
        assert!(
            !hover.to_lowercase().contains("secure"),
            "\u{201c}secure\u{201d} is the claim this sentence exists to refuse: {hover}"
        );
    }

    /// ★★★ **The inert-button note is gone, and this test is its headstone.**
    ///
    /// It said pdfcer *"cannot yet give it something to do"*.
    /// `set_button_action` shipped on 2026-08-30 and that sentence stayed on
    /// screen for two days, because **nothing in this repository fails when a
    /// capability lands**. The engine's own reply had warned in as many words:
    /// *"if your surface tells the operator that pdfcer never authors an action,
    /// it is now saying something untrue in the direction that matters."*
    ///
    /// The replacement is `text::buttonaction`, which says what the button WILL
    /// do. What is asserted here is the guard that would have caught the
    /// staleness: **no string a push button's rows draw may claim a button
    /// cannot be given an action.** A sentence that reintroduces the claim
    /// fails here rather than shipping.
    #[test]
    fn no_string_here_claims_a_button_cannot_be_given_an_action() {
        for s in [
            caption_label(),
            title(crate::canvas::formfield::FormFieldKind::PushButton),
            intro(crate::canvas::formfield::FormFieldKind::PushButton),
        ] {
            let lower = s.to_lowercase();
            assert!(
                !(lower.contains("cannot") || lower.contains("not yet")),
                "a button CAN be given an action since 2026-08-30: {s}"
            );
        }
    }
}
