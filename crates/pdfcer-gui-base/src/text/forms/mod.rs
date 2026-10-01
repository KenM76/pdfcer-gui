//! # `text::forms` — every string the Forms panel shows
//!
//! One area of the catalog described in [`crate::text`]'s header, covering
//! `pdfcer_gui::panels::forms` — the panel that lists an `/AcroForm`'s fields
//! and lets an operator **fill** them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/forms/mod.md`.

/// Every word the Tab-order section shows.
mod authoring;
/// Every word the Field-groups section shows.
pub mod groups;
mod tab_order;

pub use authoring::{
    field_appearance_not_repainted, field_options_reordered, field_siblings_untouched,
    field_sort_claim_unmet, field_widget_moved, field_widget_property_changed,
    field_widgets_affected, form_field_actions_orphaned, form_field_actions_retargeted,
    form_field_added, form_field_deleted, form_field_merged, form_field_no_options,
    form_field_no_tooltip, form_field_renamed, form_field_tagged_document, form_noun_check_box,
    form_noun_choice, form_noun_push_button, form_noun_radio, form_noun_text, form_widget_deleted,
    form_widget_deleted_last,
};

pub use groups::*;
pub use tab_order::*;

// ---------------------------------------------------------------------------
// The panel's three empty states
// ---------------------------------------------------------------------------

/// Shown when the open document carries no `/AcroForm` at all.
#[must_use]
pub fn forms_no_acroform() -> &'static str {
    "This document has no interactive form fields. A page can look like a form — boxes and \
     labels printed on it — without carrying any fields you can type into; that is a picture \
     of a form, and filling it is not what this panel does."
}

/// Shown when there is an `/AcroForm` but it declares no fields.
#[must_use]
pub fn forms_empty_acroform() -> &'static str {
    "This document declares an interactive form but lists no fields in it."
}

/// The count line at the top of a populated list.
#[must_use]
pub fn forms_field_count(total: usize, fillable: usize) -> String {
    format!("{total} field(s), {fillable} you can fill here.")
}

/// Disclosure that the form declares fields this panel cannot list.
#[must_use]
pub fn forms_inline_field_roots_note(count: usize) -> String {
    format!(
        "⚠ {count} more entry(ies) in this form are written in a way the PDF standard does not \
         allow — as values rather than as references — so they have no identity pdfcer can \
         write to. They are not listed here, and another reader may count them."
    )
}

/// Shown under [`forms_field_count`] when the fillable count is zero.
#[must_use]
pub fn forms_no_fillable_fields() -> &'static str {
    "None of them can be filled here. Each row below says why."
}

// ---------------------------------------------------------------------------
// Document-wide disclosures — stated once, above every control
// ---------------------------------------------------------------------------

/// The `/NeedAppearances` disclosure.
#[must_use]
pub fn forms_need_appearances_note() -> &'static str {
    "⚠ This form asks viewers to draw field values themselves. Some viewers do and some don't, \
     so a filled value may look different — or not appear — depending on what opens the file."
}

/// The JavaScript-computed-value disclosure.
#[must_use]
pub fn forms_javascript_note(count: usize) -> String {
    format!(
        "⚠ {count} field(s) carry scripts that would normally calculate, format or validate \
         their value. pdfcer does not run them, so a value you type here stays exactly as typed \
         and any field that would have been computed from it is left alone."
    )
}

/// The XFA disclosure — **new in this build**, and moved earlier on purpose.
#[must_use]
pub fn forms_xfa_note() -> &'static str {
    "⚠ This form also carries an XFA packet, which describes the same fields a second time. \
     pdfcer fills the part most viewers read and cannot write the XFA part, so an XFA-aware \
     viewer may still show the old value."
}

/// Shown above the list when a certification signature forbids filling.
#[must_use]
pub fn forms_certification_note() -> &'static str {
    "⚠ A certification signature on this document forbids changing form values. The fields are \
     listed so you can read them; none of them can be filled here."
}

// ---------------------------------------------------------------------------
// One field row
// ---------------------------------------------------------------------------

/// One field row's tooltip — always the RAW fully-qualified name.
#[must_use]
pub fn form_field_row_tooltip(fqn: &str) -> String {
    format!("Field name in the file: {fqn}")
}

/// Page suffix on a field label.
#[must_use]
pub fn form_field_page_suffix(page_number: usize) -> String {
    format!(" (p. {page_number})")
}

/// The `(required)` marker, as TEXT — never a colour-only cue (R84).
#[must_use]
pub fn form_field_required_marker() -> &'static str {
    " (required)"
}

/// Tooltip on a row disabled because the field is read-only.
#[must_use]
pub fn form_field_readonly_tooltip() -> &'static str {
    "This field is marked read-only in the document, so its value is not meant to be changed."
}

/// Tooltip on a row disabled by a certification signature.
#[must_use]
pub fn form_field_certification_disabled_tooltip() -> &'static str {
    "A certification signature on this document forbids changing form values. Filling this \
     field would invalidate that signature, so pdfcer will not do it."
}

/// Note on a signature-field row.
#[must_use]
pub fn form_field_signature_note() -> &'static str {
    "Signature field — click its box on the page to sign it. The Signatures panel reports on the \
     signatures a document already carries."
}

/// Hover tip on an unsigned signature box on the page.
#[must_use]
pub fn sign_box_tooltip() -> &'static str {
    "Click to sign — draw your signature"
}

/// Note on a pushbutton row.
///
/// A pushbutton holds no value, so there is nothing to fill; saying so is the
/// difference between "recognised and has no value" and "pdfcer missed it".
#[must_use]
pub fn form_field_pushbutton_note() -> &'static str {
    "Button — it runs an action rather than holding a value, so there is nothing to fill in."
}

/// Caption under a `/MaxLen` text editor.
#[must_use]
pub fn form_field_length_caption(len: usize, max: i64) -> String {
    format!("{len}/{max}")
}

/// Tooltip on a password-masked field.
#[must_use]
pub fn form_field_password_tooltip() -> &'static str {
    "Typing here is masked on screen. That does NOT encrypt it — the value is stored as plain \
     text inside the PDF."
}

/// Tooltip on a fillable text row, saying when the typing is written.
#[must_use]
pub fn form_field_commit_tooltip() -> &'static str {
    "Type here, then click or tab away to write the value into the document. Nothing reaches \
     the file until you save."
}

// ---------------------------------------------------------------------------
// Check boxes and radio groups
// ---------------------------------------------------------------------------

/// Caveat on a check box whose widgets declare no `/AP` on-state.
#[must_use]
pub fn form_field_no_on_state_note() -> &'static str {
    "This document records no ticked state for this box — it defines the drawn appearance for \
     only one of the two. pdfcer will not invent one, so the box can be read here but not \
     changed."
}

/// Shown when a radio group declares no on-state anywhere in its widgets.
///
/// R83: an empty exclusive cluster looks broken. This says the field is
/// recognised and that the DOCUMENT is what offers nothing to pick.
#[must_use]
pub fn form_field_radio_no_states() -> &'static str {
    "This radio group has no selectable options recorded in the document."
}

/// The clear-selection button on a radio group.
#[must_use]
pub fn form_field_radio_clear() -> &'static str {
    "Clear"
}

/// Its tooltip.
#[must_use]
pub fn form_field_radio_clear_tooltip() -> &'static str {
    "Deselect every option in this group, leaving it unanswered."
}

// ---------------------------------------------------------------------------
// Choice fields
// ---------------------------------------------------------------------------

/// Shown when a choice field lists no options.
#[must_use]
pub fn form_field_choice_no_options() -> &'static str {
    "This drop-down lists no options in the document, so there is nothing to choose."
}

/// The combo's placeholder when nothing is selected.
#[must_use]
pub fn form_field_choice_unset() -> &'static str {
    "— not set —"
}

/// Caveat under a choice field whose stored value is not one of its options.
#[must_use]
pub fn form_field_choice_value_not_listed() -> &'static str {
    "The value stored in this field is not one of the options the document lists. It is shown \
     as it is stored; picking an option below replaces it."
}

/// The same caveat on a **multi-select** list, where the wording above would
/// be false.
#[must_use]
pub fn form_field_choice_multi_value_not_listed() -> &'static str {
    "One of the values stored in this field is not an option the document lists, so it has no \
     box below. Ticking any box drops it."
}

// ---------------------------------------------------------------------------
// Rich text (`/RV`) — the disclosed downgrade
// ---------------------------------------------------------------------------

/// Note on a rich-text field row.
#[must_use]
pub fn form_field_rich_text_note() -> &'static str {
    "This field holds formatted text. pdfcer cannot edit that formatting yet, and typing plain \
     text into it would leave the stored formatting deciding what other viewers show — so the \
     box above is read-only."
}

/// The convert-to-plain-text button on a rich-text row.
#[must_use]
pub fn form_field_rich_text_convert() -> &'static str {
    "Convert to plain text…"
}

/// Its tooltip. Delete-shaped weight: says what is lost, plainly, before the
/// press — this discards formatting the operator may not be able to recreate.
#[must_use]
pub fn form_field_rich_text_convert_tooltip() -> &'static str {
    "Turn this into an ordinary text field so you can type in it. The stored bold, colours and \
     fonts are DISCARDED — only the plain words are kept. One undo reverses it."
}

/// Names the formatting THIS field actually holds, above the Convert button
/// that would discard it.
#[must_use]
pub fn form_field_rich_text_summary(runs: &[pdfcer_core::richtext::Run]) -> String {
    use pdfcer_core::richtext::Align;

    let mut emphasis: Vec<String> = Vec::new();
    let mut typography: Vec<String> = Vec::new();
    let mut layout: Vec<String> = Vec::new();
    let push = |bucket: &mut Vec<String>, s: String| {
        if !bucket.contains(&s) {
            bucket.push(s);
        }
    };

    for r in runs {
        let st = &r.style;
        if st.weight.is_some_and(|w| w >= 700) {
            push(&mut emphasis, "bold".to_owned());
        }
        if st.italic == Some(true) {
            push(&mut emphasis, "italic".to_owned());
        }
        if st.underline == Some(true) {
            push(&mut emphasis, "underlined".to_owned());
        }
        if st.strikethrough == Some(true) {
            push(&mut emphasis, "struck through".to_owned());
        }
        if let Some(v) = st.baseline_shift_pt {
            // Named by MEANING. Table 225's positive-is-superscript is the
            // opposite of the intuition CSS gives, so the sign alone would
            // mislead anyone who checked.
            let s = if v > 0.0 { "superscript" } else { "subscript" };
            push(&mut emphasis, s.to_owned());
        }
        if let Some(sz) = st.size_pt {
            push(&mut typography, format!("{sz} pt"));
        }
        if let Some(f) = st.family.first() {
            push(&mut typography, f.clone());
        }
        if let Some([r, g, b]) = st.color {
            let byte = |v: f64| (v * 255.0).round().clamp(0.0, 255.0) as u8;
            push(
                &mut typography,
                format!("#{:02X}{:02X}{:02X}", byte(r), byte(g), byte(b)),
            );
        }
        if let Some(a) = st.align {
            // Left is this interface's own reading direction, so naming it
            // adds a word without distinguishing anything; the other two are
            // choices someone made.
            match a {
                Align::Center => push(&mut layout, "centred".to_owned()),
                Align::Right => push(&mut layout, "right-aligned".to_owned()),
                Align::Left => {}
            }
        }
    }

    emphasis.extend(typography);
    emphasis.extend(layout);
    if emphasis.is_empty() {
        return "This field is marked as formatted text, but no formatting is actually set on \
                it. Converting it to a plain field loses nothing."
            .to_owned();
    }
    format!(
        "Formatting in this field: {}. Converting to plain text discards all of it.",
        emphasis.join(", ")
    )
}

/// The per-run breakdown, on hover over the summary.
#[must_use]
pub fn form_field_rich_text_runs_tooltip(runs: &[pdfcer_core::richtext::Run]) -> String {
    let mut s = String::from("Each formatted part of this field:");
    for r in runs {
        let text: String = if r.text.chars().count() > 32 {
            let head: String = r.text.chars().take(32).collect();
            format!("{head}…")
        } else {
            r.text.clone()
        };
        let mut bits: Vec<&str> = Vec::new();
        if r.style.weight.is_some_and(|w| w >= 700) {
            bits.push("bold");
        }
        if r.style.italic == Some(true) {
            bits.push("italic");
        }
        if r.style.underline == Some(true) {
            bits.push("underlined");
        }
        if r.style.strikethrough == Some(true) {
            bits.push("struck through");
        }
        // "as the rest" rather than "plain": a run with no emphasis of its own
        // still carries the field's default size, family and colour, and
        // calling it plain would say it has none.
        let how = if bits.is_empty() {
            "as the rest".to_owned()
        } else {
            bits.join(" + ")
        };
        s.push_str(&format!("\n  “{text}” — {how}"));
    }
    s
}

/// The `/RV` bytes are not valid UTF-8, so they cannot even be parsed.
///
/// A separate, COMPLETE entry rather than a reason fragment fed to
/// [`form_field_rich_text_unreadable`]: that function's `reason` comes from
/// core's own `RichTextError` `Display`, which core owns and writes as a whole
/// clause. A fragment hand-written in the shell to look like one is a message
/// assembled from pieces nobody reviews as a sentence.
///
/// Says the same load-bearing thing as its sibling: this is NOT an unformatted
/// field.
#[must_use]
pub fn form_field_rich_text_not_utf8() -> String {
    "This field holds formatted text that pdfcer could not read — the stored formatting is not \
     valid text. It is NOT unformatted: converting it would discard formatting nobody has \
     seen. Consider leaving it alone."
        .to_owned()
}

/// The `/RV` document is valid UTF-8 and would not parse.
#[must_use]
pub fn form_field_rich_text_unreadable(reason: &str) -> String {
    format!(
        "This field holds formatted text that pdfcer could not read ({reason}). It is NOT \
         unformatted — converting it would discard formatting nobody has seen. Consider \
         leaving it alone."
    )
}

// ---------------------------------------------------------------------------
// Calculated fields — decision 009 posture B
// ---------------------------------------------------------------------------

/// Heading for the recompute section of the Forms panel.
#[must_use]
pub const fn recompute_heading() -> &'static str {
    "Calculated fields"
}

/// The standing explanation, shown whenever the section is open.
///
/// Says the two things an operator cannot infer from the numbers on screen:
/// pdfcer did not run the scripts, and the values shown are as last saved.
#[must_use]
pub const fn recompute_explainer() -> &'static str {
    "pdfcer never runs a document's JavaScript. Where a field is computed by a recognised \
     Acrobat built-in, pdfcer can reproduce the arithmetic natively instead. The source script \
     stays in the file either way."
}

/// Summary line when a plan has pending changes.
#[must_use]
pub fn recompute_pending(changes: usize, coerced: usize) -> String {
    let blanks = if coerced == 0 {
        String::new()
    } else {
        format!(
            " {coerced} operand(s) are blank or non-numeric and count as zero, matching Acrobat."
        )
    };
    format!("{changes} field(s) would change.{blanks}")
}

/// Summary line when everything already holds its computed value.
#[must_use]
pub const fn recompute_up_to_date() -> &'static str {
    "Every recognised calculation already holds its computed value."
}

/// Shown when the document has no calculation pdfcer recognises.
#[must_use]
pub const fn recompute_nothing_recognised() -> &'static str {
    "No recognised Acrobat calculation in this form."
}

/// One proposed change, as a single reviewable line.
#[must_use]
pub fn recompute_change_row(field: &str, from: &str, to: &str) -> String {
    format!("{field}: {from} -> {to}")
}

/// One skipped calculation and its reason.
#[must_use]
pub fn recompute_skip_row(field: &str, reason: &str) -> String {
    format!("{field}: {reason}")
}

/// The button that commits the plan.
#[must_use]
pub const fn recompute_apply_button() -> &'static str {
    "Recompute these fields"
}

/// Tooltip for that button.
#[must_use]
pub const fn recompute_apply_tooltip() -> &'static str {
    "Writes the values listed above — one undo step per field, because pdfcer writes them one \
     at a time. The source scripts are left in place, so a JavaScript-running reader \
     recomputes independently."
}

/// Warning when pdfcer had to invent part of the evaluation order.
#[must_use]
pub fn recompute_order_is_a_guess(unlisted: usize) -> String {
    format!(
        "This form does not list {unlisted} of its calculated field(s) in its calculation \
         order, which the PDF standard requires. pdfcer ordered them by their own dependencies; \
         another reader may compute different values."
    )
}

/// Note counting the scripts pdfcer did not consider.
#[must_use]
pub fn recompute_not_considered(count: usize) -> String {
    format!(
        "{count} other script(s) were not considered — pdfcer recognises no built-in in them, so \
         those fields keep the values last saved."
    )
}

// ---------------------------------------------------------------------------
// Reset to defaults (§12.7.5.3)
// ---------------------------------------------------------------------------

/// Heading for the reset section of the Forms panel.
#[must_use]
pub const fn reset_heading() -> &'static str {
    "Reset to defaults"
}

/// The standing explanation, shown whenever the section is open.
#[must_use]
pub const fn reset_explainer() -> &'static str {
    "This DISCARDS what has been typed. Each field goes back to the default stored in the \
     document, and a field with no stored default is emptied completely. Signature, read-only \
     and button fields are left alone."
}

/// One field the reset would clear.
#[must_use]
pub fn reset_row(field: &str, from: &str, to: &str) -> String {
    format!("{field}: {from} -> {to}")
}

/// What a field with no stored default becomes.
#[must_use]
pub const fn reset_to_empty() -> &'static str {
    "(emptied)"
}

/// Summary of what a reset would do.
#[must_use]
pub fn reset_pending(clearing: usize, skipped: usize) -> String {
    if skipped == 0 {
        format!("{clearing} field(s) would be cleared.")
    } else {
        format!("{clearing} field(s) would be cleared; {skipped} left alone.")
    }
}

/// How many fields already hold their reset value.
#[must_use]
pub fn reset_already_default(count: usize) -> String {
    format!("{count} field(s) already hold their default.")
}

/// Shown when nothing is eligible.
#[must_use]
pub const fn reset_nothing_to_do() -> &'static str {
    "No field in this form can be reset."
}

/// The button that performs the reset.
#[must_use]
pub const fn reset_button() -> &'static str {
    "Reset these fields"
}

/// Tooltip for that button.
#[must_use]
pub const fn reset_tooltip() -> &'static str {
    "Clears the values listed above. One undo step."
}

// ---------------------------------------------------------------------------
// Form-wide operations
// ---------------------------------------------------------------------------

/// The regenerate-appearances button.
#[must_use]
pub fn forms_regenerate_button() -> &'static str {
    "Redraw values"
}

/// Its tooltip — leads with the problem it solves, not the mechanism.
#[must_use]
pub fn forms_regenerate_tooltip() -> &'static str {
    "Draw every field's current value into the document, so it looks the same in every viewer \
     instead of depending on each one to render it. Use this if a filled value looks wrong or \
     missing somewhere else. One undo reverses it."
}

/// The flatten button.
#[must_use]
pub fn forms_flatten_button() -> &'static str {
    "Flatten form"
}

/// Its tooltip — delete-shaped weight, so it has to be honest and complete.
#[must_use]
pub fn forms_flatten_tooltip() -> &'static str {
    "Turn every field's current value into ordinary page content and remove the form. The \
     values stay visible but stop being editable, and anything typed into them can no longer \
     be changed. One undo reverses it. Note: with the normal save, the old field values are \
     still recoverable from the file's earlier revision — flatten is not a way to remove \
     sensitive answers."
}

/// Warning beside Flatten when some field has no drawn appearance to burn.
#[must_use]
pub fn forms_flatten_needs_redraw_note(count: usize) -> String {
    format!(
        "⚠ {count} field(s) have no drawn appearance in this document. Flattening turns each \
         field's DRAWN appearance into page content, so those values would vanish rather than \
         be kept. Use “{}” first.",
        forms_regenerate_button()
    )
}

// ---------------------------------------------------------------------------
// What the page can and cannot be filled from — see `pdfcer_gui::canvas::forms`
// ---------------------------------------------------------------------------

/// The panel's note for fields that **cannot be clicked on the page** because
/// nothing is drawn for them.
#[must_use]
pub fn forms_canvas_undrawn_note(count: usize) -> String {
    format!(
        "{count} field(s) are not drawn on the page, so they cannot be clicked there. Fill one \
         here and it becomes drawn — and clickable on the page from then on. “{}” does the same \
         for fields that already hold a value.",
        forms_regenerate_button()
    )
}

/// The panel's note for fields that are drawn but still cannot be **typed
/// into** on the page.
#[must_use]
pub fn forms_canvas_unreachable_note(count: usize) -> String {
    format!(
        "{count} field(s) sit on a rotated page, or on no page this file names, so they are \
         filled here rather than by clicking them."
    )
}

// ---------------------------------------------------------------------------
// What the last fill decided on the operator's behalf
// ---------------------------------------------------------------------------

/// Rule-4 disclosure: **pdfcer chose the point size**, because the field asked
/// it to.
#[must_use]
pub fn forms_fill_autosize_note(field: &str, size: f64) -> String {
    format!(
        "⚠ “{field}” asks for an automatic text size and pdfcer chose {size:.1} pt. Another \
         program filling this field may choose differently."
    )
}

/// Rule-4 disclosure: **the chosen size does not fit, and the text will spill
/// out of the box.**
#[must_use]
pub fn forms_fill_autosize_overflow_note(field: &str, size: f64) -> String {
    format!(
        "⚠ “{field}” is too small for this text. pdfcer held the size at {size:.1} pt so it stays \
         readable, which means the text will overflow the box — make the field taller, or shorten \
         what is in it."
    )
}

/// Rule-4 disclosure: pdfcer chose the size, and **the field's width is what
/// decided it** rather than its height.
#[must_use]
pub fn forms_fill_autosize_width_note(field: &str, size: f64) -> String {
    format!(
        "⚠ “{field}” asks for an automatic text size. pdfcer chose {size:.1} pt to fit the \
         field's WIDTH — making the field taller will not change it. Another program filling \
         this field may choose differently."
    )
}

/// Rule-4 disclosure: **characters were replaced**, and the operator's own
/// text is not what the page now says.
#[must_use]
pub fn forms_fill_unencodable_note(field: &str, count: usize) -> String {
    format!(
        "⚠ {count} character(s) of “{field}” could not be written in this field's font and were \
         stored as “?”. The page now shows those question marks."
    )
}

/// Rule-4 disclosure: a password field was filled and **nothing was stored**.
#[must_use]
pub fn forms_fill_password_withheld_note(field: &str) -> String {
    format!(
        "“{field}” is a password field, so what you typed was not saved in the file. The page \
         shows one * per character, and the box here reads empty."
    )
}

/// Disclosure: turning Password on removed the value the field held.
#[must_use]
pub fn field_password_value_removed(field: &str) -> String {
    format!(
        "“{field}” is now a password field, so the value it held has been removed from the file."
    )
}

/// The button that stores a withheld password after all.
#[must_use]
pub fn forms_store_password_button() -> &'static str {
    "Save it in the file anyway"
}

/// Hover text on [`forms_store_password_button`].
#[must_use]
pub fn forms_store_password_tooltip() -> &'static str {
    "Stores what you typed as plain text in the file. Anyone who opens the file can read it."
}

/// Tooltip on a form-wide control disabled by a certification signature.
#[must_use]
pub fn forms_structural_certification_disabled_tooltip() -> &'static str {
    "A certification signature on this document forbids changing the form's structure. Values \
     can still be filled in; the form itself cannot be removed."
}

/// Disclosure: the field's other boxes stayed where they were.
#[must_use]
pub fn widget_siblings_unmoved(count: usize) -> String {
    format!(
        "This field is drawn in {} other place(s) as well, and those boxes have not moved. That \
         is deliberate — they are separate positions for the same value.",
        count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **No two "you cannot do this" sentences read alike.**
    #[test]
    fn every_refusal_explains_a_different_refusal() {
        let all = [
            form_field_readonly_tooltip(),
            form_field_certification_disabled_tooltip(),
            form_field_signature_note(),
            form_field_pushbutton_note(),
            form_field_rich_text_note(),
            forms_certification_note(),
            forms_structural_certification_disabled_tooltip(),
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "two refusals share a sentence");
            }
        }
    }

    /// **The fill gate and the structural gate say different things.**
    #[test]
    fn the_structural_refusal_does_not_claim_values_are_locked() {
        let structural = forms_structural_certification_disabled_tooltip();
        assert!(
            structural.contains("structure"),
            "the structural refusal must name what is frozen: {structural}"
        );
        assert!(
            structural.contains("still"),
            "it must say that filling remains available, or it reads as the \
             fill refusal: {structural}"
        );
    }

    /// **Every warning survives its glyph being stripped.**
    #[test]
    fn a_warning_glyph_is_never_load_bearing() {
        for s in [
            forms_need_appearances_note(),
            forms_xfa_note(),
            forms_certification_note(),
            &forms_javascript_note(3),
        ] {
            let stripped = s.trim_start_matches(['⚠', ' ']);
            assert!(
                stripped.len() > 40,
                "the sentence is carried by its glyph: {s}"
            );
            // Alphanumeric rather than uppercase: several of these open with
            // a count ("3 field(s) carry scripts…"), which is a sentence and
            // not a fragment. What is being ruled out is a warning that opens
            // with a dash, a colon or nothing — the shape a sentence takes
            // when the glyph was doing the work.
            assert!(
                stripped.starts_with(|c: char| c.is_alphanumeric()),
                "stripping the glyph must leave a sentence, not a fragment: {s}"
            );
            assert!(
                stripped.trim_end().ends_with('.'),
                "a disclosure must be a complete sentence: {s}"
            );
        }
    }

    /// **The rich-text summary distinguishes "no formatting" from
    /// "unreadable formatting".**
    #[test]
    fn an_unreadable_rich_value_is_not_reported_as_an_unformatted_one() {
        let none = form_field_rich_text_summary(&[]);
        let unreadable = form_field_rich_text_unreadable("unexpected end of document");
        let not_text = form_field_rich_text_not_utf8();

        assert!(
            none.contains("loses nothing"),
            "an unformatted field must say the conversion is free: {none}"
        );
        for bad in [&unreadable, &not_text] {
            assert!(
                bad.contains("NOT unformatted") || bad.contains("It is NOT"),
                "an unreadable rich value must deny being unformatted: {bad}"
            );
            assert!(
                !bad.contains("loses nothing"),
                "an unreadable rich value must never say the conversion is \
                 free: {bad}"
            );
        }
    }

    /// **The rich-text summary lists emphasis before typography.**
    #[test]
    fn emphasis_is_grouped_before_typography() {
        use pdfcer_core::richtext::{Run, Style};

        // `paragraph` is 0 on every run: the summary is about STYLE, and
        // grouping runs by paragraph would not change which bucket a feature
        // lands in. Stated rather than left as an unexplained zero, because
        // `Run` is `#[non_exhaustive]`-adjacent — the field list has grown
        // once already, and a future one may matter here.
        let bold_with_ds = Run {
            text: "Total".to_owned(),
            style: Style {
                weight: Some(700),
                size_pt: Some(12.0),
                family: vec!["Helvetica".to_owned()],
                ..Style::default()
            },
            paragraph: 0,
        };
        let plain = Run {
            text: " is ".to_owned(),
            style: Style::default(),
            paragraph: 0,
        };
        let italic = Run {
            text: "urgent".to_owned(),
            style: Style {
                italic: Some(true),
                ..Style::default()
            },
            paragraph: 0,
        };

        let summary = form_field_rich_text_summary(&[bold_with_ds, plain, italic]);
        let bold = summary.find("bold").expect("bold must be listed");
        let italic_at = summary.find("italic").expect("italic must be listed");
        let size = summary.find("12 pt").expect("the size must be listed");
        assert!(
            bold < italic_at && italic_at < size,
            "emphasis must be grouped ahead of the typographic settings, or \
             the two facts an operator compares end up furthest apart: \
             {summary}"
        );
    }

    /// **The count line's own wording admits it is about this panel.**
    #[test]
    fn the_count_line_is_scoped_to_this_panel() {
        let line = forms_field_count(12, 0);
        assert!(line.contains("here"), "{line}");
        assert!(line.starts_with("12 field"), "{line}");
    }

    /// The reset explainer leads with the loss, not the mechanism.
    #[test]
    fn the_reset_explainer_says_the_destructive_part_first() {
        let s = reset_explainer();
        let discards = s.find("DISCARDS").expect("it must name the loss");
        assert!(
            discards < 40,
            "the loss must be in the first clause, not after an explanation \
             of how reset works: {s}"
        );
    }

    /// **The recompute explainer states the standing rule, not a limitation.**
    #[test]
    fn the_recompute_explainer_states_a_rule_rather_than_a_gap() {
        let s = recompute_explainer();
        assert!(s.contains("never runs"), "{s}");
        assert!(
            !s.contains("yet") && !s.contains("not able"),
            "the rule must not be worded as a shortfall: {s}"
        );
    }
}
