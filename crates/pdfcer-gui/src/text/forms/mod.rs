//! # `text::forms` — every string the Forms panel shows
//!
//! One area of the catalog described in [`crate::text`]'s header, covering
//! [`crate::panels::forms`] — the panel that lists an `/AcroForm`'s fields
//! and lets an operator **fill** them.
//!
//! It sits beside [`crate::text::panels`] rather than inside it, which is a
//! deliberate placement rather than an oversight: the Forms panel is not one
//! of the six report panels that module covers, it is the first panel in this
//! build that **changes the document**, and its copy is dominated by
//! *disclosures* and *refusals* rather than by field labels. Keeping it in its
//! own file means the reviewer of a disclosure sentence is reading a file that
//! contains nothing but disclosure sentences.
//!
//! ## Almost every sentence here is salvaged verbatim
//!
//! Carried across from the old shell's `ui_text.rs` **with its doc comments**,
//! because the doc comment is usually the record of a defect the wording was
//! changed to fix. `SALVAGE.md`'s procedure forbids re-deriving a decision
//! already paid for, and this area has more of them per line than any other:
//!
//! - [`forms_no_acroform`] does not say "no fields found". It says a page can
//!   *look* like a form without carrying one, because otherwise "no fields"
//!   reads as pdfcer failing to find something that is plainly on the page.
//! - [`form_field_password_tooltip`] exists because a masked box reads as
//!   "secure" to anyone not told otherwise, and the value really is stored as
//!   plain text in the file.
//! - [`forms_data_export_carries_rich_text`]'s counterpart in the old catalog
//!   carries a `★` recording that the sentence outlived the behaviour it
//!   described by one commit. That string is **not** salvaged here (this build
//!   has no export), and the lesson is: a disclosure that has gone stale is a
//!   false statement the operator has no way to check.
//!
//! ## The three sentences that are NEW, and why each had to be
//!
//! | Function | Replaces | Why |
//! |---|---|---|
//! | [`forms_xfa_note`] | the old shell's post-fill `fill_xfa_may_disagree` | The old note appeared *after* a value was typed. Whether the document carries an XFA packet is a property of the FILE, knowable before the operator touches anything, so it is said up front. |
//! | [`form_field_no_on_state_note`] | the old shell's post-toggle `form_field_no_appearance_for_state` | The old one was computed from a predicate that could never be true (see [`crate::panels::forms::rows`]' header, "★ The salvaged appearance check could not fire"). This one asks a question the model can actually answer. |
//! | [`forms_no_fillable_fields`] | — | This build derives the "N you can fill here" count from what the panel actually offers, and the count can legitimately be zero on a form full of fields. A silent zero looks like a bug. |
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **Name the thing and what the operator can do about it.**
//! - **Never state a capability the build does not have.** This build fills;
//!   it does not create, delete or rename a field, and no string here implies
//!   otherwise.
//! - **A warning glyph is never the only cue.** Every `⚠` sentence reads
//!   correctly with the glyph stripped, because a glyph is a colour-class cue
//!   and `RIBBON_IA.md` R84 forbids carrying state in one.

/// Every word the Tab-order section shows.
///
/// Re-exported below, so this split is invisible at every call site — see that
/// module's header for why this is the seam R2 forced and why it was the right
/// one anyway.
mod authoring;
/// Every word the Field-groups section shows.
///
/// Re-exported below, so the split is invisible at every call site — the same
/// R2 seam [`authoring`] and [`tab_order`] were cut along, and for the same
/// reason: a reviewer of one surface's wording should be reading a file that
/// contains only that surface's wording.
pub mod groups;
mod tab_order;

pub use authoring::{
    field_appearance_stale, field_siblings_untouched, field_sort_claim_unmet, field_widget_moved,
    field_widgets_affected, form_field_added, form_field_deleted, form_field_merged,
    form_field_no_options, form_field_no_tooltip, form_field_renamed, form_field_tagged_document,
    form_noun_check_box, form_noun_choice, form_noun_push_button, form_noun_radio, form_noun_text,
    form_widget_deleted, form_widget_deleted_last,
};

pub use groups::*;
pub use tab_order::*;

// ---------------------------------------------------------------------------
// The panel's three empty states
// ---------------------------------------------------------------------------

/// Shown when the open document carries no `/AcroForm` at all.
///
/// States the distinction that actually matters to the operator: a page can
/// LOOK like a form — ruled boxes, printed labels — without carrying a single
/// interactive field. Without this, "no fields" reads as pdfcer failing to find
/// something that is plainly there on the page.
///
/// Salvaged verbatim except for the final clause, which named the old shell's
/// text tools. This build has none, so the sentence stops at the honest half
/// rather than pointing at a control the operator cannot find — the
/// "no placeholders" invariant (`PROJECT_PLAN.md` §3) applied to prose.
#[must_use]
pub fn forms_no_acroform() -> &'static str {
    "This document has no interactive form fields. A page can look like a form — boxes and \
     labels printed on it — without carrying any fields you can type into; that is a picture \
     of a form, and filling it is not what this panel does."
}

/// Shown when there is an `/AcroForm` but it declares no fields.
///
/// A distinct sentence from [`forms_no_acroform`], because the two are
/// different facts about the file: one says the document never had a form, the
/// other says it declares one and lists nothing in it — which is a
/// malformation worth being able to see.
#[must_use]
pub fn forms_empty_acroform() -> &'static str {
    "This document declares an interactive form but lists no fields in it."
}

/// The count line at the top of a populated list.
///
/// # ★ `fillable` is what THIS PANEL offers, not what the model calls fillable
///
/// The obvious implementation counts `pdfcer_core::forms::Field::is_fillable`,
/// and it is wrong in a way the operator can see. `is_fillable` answers *"could
/// a fill edit change this field's value"* — it excludes read-only, signature
/// and pushbutton fields and nothing else. The panel additionally declines to
/// offer an editable control when a certification signature forbids filling the
/// whole document, and when a field holds rich text pdfcer cannot author.
///
/// So a certified fillable form would read "12 fields, 12 you can fill here"
/// above twelve disabled boxes.
///
/// This is `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s third
/// bite — *"a returned count is not always the count to display"* — one verb
/// over. Its worked example is `set_group_style` returning members
/// **regenerated** rather than members that will visibly **move**; the shape
/// recurs wherever a count comes from a model's predicate and the sentence
/// describes the interface's behaviour. The count displayed must be derived
/// from what the panel will actually draw.
#[must_use]
pub fn forms_field_count(total: usize, fillable: usize) -> String {
    format!("{total} field(s), {fillable} you can fill here.")
}

/// Disclosure that the form declares fields this panel cannot list.
///
/// # ★ The count line understates the file, by exactly this many
///
/// `pdfcer_core::forms::AcroForm::inline_field_roots` counts `/Fields` entries
/// that are **direct dictionaries rather than indirect references**. Table 218
/// admits only references, so such an entry is malformed — and
/// `parse_acroform` skips it, because a field with no object identity has
/// nothing a fill could write to.
///
/// The consequence is that `fields.len()` is not the number of fields in the
/// file. This is the third bite in
/// `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` in its most
/// literal form: the count the model returns is not the count to present as
/// the whole truth. An operator comparing pdfcer's "9 fields" against another
/// reader's "10" must be able to find out why here rather than concluding
/// pdfcer lost one.
#[must_use]
pub fn forms_inline_field_roots_note(count: usize) -> String {
    format!(
        "⚠ {count} more entry(ies) in this form are written in a way the PDF standard does not \
         allow — as values rather than as references — so they have no identity pdfcer can \
         write to. They are not listed here, and another reader may count them."
    )
}

/// Shown under [`forms_field_count`] when the fillable count is zero.
///
/// New in this build. A zero in a count line is indistinguishable from a
/// panel that failed to look, and on a real form — a signed contract, a
/// certified return, a read-only archive copy — zero is the correct and
/// unsurprising answer. Saying so converts a suspicious number into a
/// statement about the document.
///
/// Deliberately does NOT enumerate the reasons: each row already carries its
/// own, and a summary that tried to aggregate four different causes would
/// either be vague or would be a second place to keep them in step.
#[must_use]
pub fn forms_no_fillable_fields() -> &'static str {
    "None of them can be filled here. Each row below says why."
}

// ---------------------------------------------------------------------------
// Document-wide disclosures — stated once, above every control
// ---------------------------------------------------------------------------

/// The `/NeedAppearances` disclosure.
///
/// The real trap this closes: a value pdfcer writes is correct in the file, but
/// a viewer that honours `/NeedAppearances` draws it from the value while one
/// that does not draws the stale baked appearance — so the same document shows
/// two different things depending on who opens it.
#[must_use]
pub fn forms_need_appearances_note() -> &'static str {
    "⚠ This form asks viewers to draw field values themselves. Some viewers do and some don't, \
     so a filled value may look different — or not appear — depending on what opens the file."
}

/// The JavaScript-computed-value disclosure.
///
/// **pdfcer never runs a document's JavaScript**, and that is a standing
/// project rule rather than an unfinished feature. Fields whose value a script
/// would have computed are therefore left exactly as last saved, and this
/// sentence is what stops an operator concluding the form is broken when a
/// total does not move.
///
/// The remedy is [`recompute_heading`]'s section, which reproduces a
/// whitelisted subset of Acrobat's built-in calculations natively.
#[must_use]
pub fn forms_javascript_note(count: usize) -> String {
    format!(
        "⚠ {count} field(s) carry scripts that would normally calculate, format or validate \
         their value. pdfcer does not run them, so a value you type here stays exactly as typed \
         and any field that would have been computed from it is left alone."
    )
}

/// The XFA disclosure — **new in this build**, and moved earlier on purpose.
///
/// # Why this is form-wide and up front rather than per-fill and after
///
/// The old shell surfaced this as `fill_xfa_may_disagree`, a status note
/// produced from `FillOutcome::xfa_may_disagree` **after** a value was
/// committed. That is a faithful reading of the engine's outcome and the wrong
/// moment for the operator: whether the document carries an XFA packet is a
/// property of the FILE (`pdfcer_core::forms::AcroForm::xfa`), knowable before
/// anything is typed, and identical for every field.
///
/// Saying it once, before the list, means the operator learns that their
/// typing may not stick *before* they do it — and it removes the need for this
/// panel to carry a note channel back from the action funnel at all. See
/// [`crate::panels::forms`]' header, "★ Nothing has to travel back from
/// `apply`".
///
/// Leads with the consequence rather than the mechanism, which is the old
/// string's decision kept: an operator cares that a value might not stick, not
/// that the document has two field descriptions.
#[must_use]
pub fn forms_xfa_note() -> &'static str {
    "⚠ This form also carries an XFA packet, which describes the same fields a second time. \
     pdfcer fills the part most viewers read and cannot write the XFA part, so an XFA-aware \
     viewer may still show the old value."
}

/// Shown above the list when a certification signature forbids filling.
///
/// Salvaged from the old shell's `form_field_certification_disabled_tooltip`,
/// and **promoted from a per-row tooltip to a panel-wide line**. The reason is
/// the same one the old panel gave for asking the gate once: a certification
/// signature forbids filling the whole DOCUMENT, not one field, so repeating
/// it on forty rows is forty copies of one fact.
///
/// It is still attached to each disabled row as well (see
/// [`form_field_certification_disabled_tooltip`]), because a row an operator
/// clicks on and cannot type into must explain itself where they are looking.
#[must_use]
pub fn forms_certification_note() -> &'static str {
    "⚠ A certification signature on this document forbids changing form values. The fields are \
     listed so you can read them; none of them can be filled here."
}

// ---------------------------------------------------------------------------
// One field row
// ---------------------------------------------------------------------------

/// One field row's tooltip — always the RAW fully-qualified name.
///
/// The row's visible label prefers `/TU`, which is what a screen reader
/// announces; but an operator diagnosing why a value did not match needs the
/// technical name, and it may differ from the label or be absent from it.
#[must_use]
pub fn form_field_row_tooltip(fqn: &str) -> String {
    format!("Field name in the file: {fqn}")
}

/// Page suffix on a field label.
///
/// A leading space is part of the string because it is appended to a label
/// this catalog does not own — the field's `/TU` or its name, which come from
/// the document. Putting the separator here keeps the whole of the assembled
/// sentence's punctuation in the catalog.
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
///
/// The per-row half of [`forms_certification_note`]. Both exist: the panel-wide
/// line is what an operator reads when scanning, and this is what they get when
/// they click the box that will not accept typing.
#[must_use]
pub fn form_field_certification_disabled_tooltip() -> &'static str {
    "A certification signature on this document forbids changing form values. Filling this \
     field would invalidate that signature, so pdfcer will not do it."
}

/// Note on a signature-field row.
///
/// Listed rather than hidden (R83): an operator scrolling past a signature
/// field should see that pdfcer knows it is there.
///
/// ★★ **CORRECTED 2026-09-05, and it is a worked example of why a conjoined
/// refusal is dangerous.** It read *"pdfcer does not create or verify
/// signatures yet."* Half of that stayed true — pdfcer still cannot **sign**
/// — and half became false the day `signature::verify_all_with_trust` was
/// wired (`crate::panels::signatures`, engine v0.38.0 at `b01964f`). Because
/// the two claims shared one sentence, the true half kept the false half
/// looking true, and this row went on denying a capability the Signatures
/// panel demonstrates two clicks away. The clauses are separated now, and
/// only the one that is still true is a refusal.
#[must_use]
pub fn form_field_signature_note() -> &'static str {
    "Signature field — pdfcer cannot sign a document. The Signatures panel reports on the \
     signatures a document already carries."
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
///
/// Two numbers and a slash, deliberately wordless: it sits under every capped
/// field on the form and a sentence there would be read once and then be
/// noise. The limit itself is enforced live rather than at commit — see
/// [`crate::panels::forms::rows`] — so this caption describes a rule the
/// operator has already felt.
#[must_use]
pub fn form_field_length_caption(len: usize, max: i64) -> String {
    format!("{len}/{max}")
}

/// Tooltip on a password-masked field.
///
/// Says the masking is display-only. A masked box reads as "secure" to anyone
/// not told otherwise, and the value really is stored as plain text in the
/// file — the sneaky half of rule 4 if left unsaid.
#[must_use]
pub fn form_field_password_tooltip() -> &'static str {
    "Typing here is masked on screen. That does NOT encrypt it — the value is stored as plain \
     text inside the PDF."
}

/// Tooltip on a fillable text row, saying when the typing is written.
///
/// New in this build, and it earns its place because the commit rule is
/// invisible: a text field writes its value when focus LEAVES it, not on every
/// keystroke, so an operator who types and then looks at the page sees
/// nothing happen. (The rule itself is
/// [`crate::panels::forms::rows::commit`], and it exists so one typed word is
/// one undo step rather than a dozen.)
#[must_use]
pub fn form_field_commit_tooltip() -> &'static str {
    "Type here, then click or tab away to write the value into the document. Nothing reaches \
     the file until you save."
}

// ---------------------------------------------------------------------------
// Check boxes and radio groups
// ---------------------------------------------------------------------------

/// Caveat on a check box whose widgets declare no `/AP` on-state.
///
/// **New in this build**, replacing a salvaged string that could not fire —
/// see [`crate::panels::forms::rows`]' header for the full account.
///
/// # ★ Why the wording is "cannot", not "will look the same"
///
/// The obvious sentence — *"the value changes but the page will not"* — is
/// what the old shell's equivalent tried to say, and it is **false against
/// this engine**. `EditSession::set_button_state` refuses any state other
/// than `Off` that no widget defines, by name:
/// `EditError::FieldStateUnknown { name, state, available }`
/// (`edit.rs:12607-12629`). `pdfcer_core::forms::Widget::on_states` lists the
/// states the widget's `/AP` `/N` sub-dictionary defines, **excluding `Off`**
/// — so an empty list means there is no state pdfcer may select, and a tick
/// here would be an affordance for a call that always errors (R83).
///
/// The control is therefore drawn **disabled and explained**, exactly like a
/// `/Locked` layer row, and this is the explanation. It says the document is
/// what offers nothing, not pdfcer.
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
///
/// New in this build. `/V` may legitimately hold a value that `/Opt` does not
/// list — set by another program, or left behind when the option list was
/// edited — and the row shows it verbatim rather than blank, because showing
/// blank would claim the field is unanswered when it is not.
///
/// Said out loud because a value that appears in the box and in none of the
/// choices below it looks like a rendering fault.
#[must_use]
pub fn form_field_choice_value_not_listed() -> &'static str {
    "The value stored in this field is not one of the options the document lists. It is shown \
     as it is stored; picking an option below replaces it."
}

// ---------------------------------------------------------------------------
// Rich text (`/RV`) — the disclosed downgrade
// ---------------------------------------------------------------------------

/// Note on a rich-text field row.
///
/// The row is read-only, and the reason is **correctness** rather than
/// fidelity: pdfcer cannot author `/RV`, and §12.7.3.4 / §12.7.3.3 bind
/// appearance generation for these fields to `/RV` rather than `/V`, both with
/// `shall`. Writing plain text and leaving `/RV` behind would make conforming
/// readers rebuild the appearance from the OLD text — the document would
/// display words nobody typed.
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
///
/// # Why a generic warning was not enough
///
/// [`form_field_rich_text_convert_tooltip`] already says "bold, colours and
/// fonts are DISCARDED". That is a category, not this document: it reads the
/// same on a field whose only formatting is 12 pt Helvetica as on one carrying
/// three colours and a superscript. The always-visible summary names what is
/// there; the per-run breakdown ([`form_field_rich_text_runs_tooltip`]) is a
/// hover away. Progressive disclosure, with the frequent question visible.
///
/// # ★ Collected by category, emitted in a fixed order
///
/// Not in the order the runs happen to mention things. A single
/// accumulate-as-you-go list produced, on the old shell's shipped fixture,
/// `"bold, 12 pt, Helvetica, #FF0000, italic"`: run 0 is the bold one and
/// contributes the `/DS` size, family and colour with it, so `italic` from run
/// 2 landed at the far end — the two facts an operator most needs to compare
/// were the two furthest apart. Found by reading the rendered panel, not the
/// code.
///
/// So emphasis first (what the words LOOK like), then the typographic
/// settings, then layout. Within each bucket, first-seen order, which is
/// stable because it comes from document order.
///
/// Only features actually SET appear. `richtext::Style` uses `None` for
/// "neither the run nor `/DS` specified this", which is not the same as a
/// default; listing unset properties would both bury the real ones and assert
/// something the file does not say.
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
///
/// The summary answers "what formatting is in here"; this answers "which words
/// have which". Both are wanted and only one fits on a form row.
///
/// A tooltip is the right home precisely because this is the OCCASIONAL
/// question. It is **not** a disclosure obligation: the destructive act's
/// consequence is already stated in the always-visible summary, so nothing
/// here is a fact the operator must see before clicking. If it were, a hover
/// would be the wrong place for it.
///
/// Text is shown quoted and elided so one long run cannot push the rest off
/// the screen — the point is which run, not the whole value, and the value is
/// already in the read-only box above.
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
///
/// The blank-operand clause is appended rather than being its own line so the
/// two facts an operator weighs together — how many fields move, and how many
/// of the inputs pdfcer had to read as zero — are read together.
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
///
/// `reason` is `pdfcer_core::form_script::recompute::Skip`'s own `Display`,
/// passed through rather than rewritten: core writes each as a complete clause
/// and replacing one with a shell paraphrase throws away the only part of the
/// sentence that helps.
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
///
/// # ★ It says "one undo step per field", and the old catalog said "one undo
/// step"
///
/// The old wording was inherited from a control that writes one command, and
/// it is wrong here. `pdfcer-core` has no batch-recompute verb — part 3 of the
/// core API is explicit that *"applying a plan is a loop the shell writes
/// itself"* — so the shell calls `EditSession::fill_text_field` once per
/// planned change and each of those is its own undo entry. An operator told
/// "one undo step" would press Ctrl+Z once, see one field revert, and
/// reasonably conclude undo is broken.
///
/// Stated rather than fixed, because the fix is core's: a single
/// `apply_recompute` verb would make one command out of the loop. Until then
/// the honest sentence is the one that matches what happens.
#[must_use]
pub const fn recompute_apply_tooltip() -> &'static str {
    "Writes the values listed above — one undo step per field, because pdfcer writes them one \
     at a time. The source scripts are left in place, so a JavaScript-running reader \
     recomputes independently."
}

/// Warning when pdfcer had to invent part of the evaluation order.
///
/// A rule-4 disclosure of the purest kind: pdfcer inferred something (which
/// order to evaluate in), the inference changes the numbers, and no other
/// reader is obliged to agree with it.
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
///
/// Says the destructive part first. A section titled "reset" that opens with
/// how it works has buried the only sentence that changes the operator's
/// decision.
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
///
/// Parenthesised because it is not a value — it is the *absence* of one, and
/// `/V` removed and `/V` set to the empty string are different bytes. A shell
/// that showed both as `""` would be describing the wrong edit.
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
///
/// Stated rather than left as a gap in the list. A field the operator expected
/// to see and does not is a question; "3 already hold their default" is the
/// answer, given before it is asked.
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
///
/// This is the operator-facing answer to [`forms_need_appearances_note`]: a
/// document carrying that flag asks viewers to draw field values themselves,
/// and viewers disagree about whether to. Regenerating bakes an appearance for
/// every field and clears the flag, so every viewer shows the same thing.
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
///
/// Says what is lost, what survives, and — the part an operator cannot guess —
/// that under the default incremental save the old values are still present in
/// the file's previous revision. That last clause is why this is a tooltip and
/// not a blocking confirmation: flatten is not structurally irreversible the
/// way applying a redaction is. See [`crate::panels::forms::edit`]'s header for
/// the full argument, which was made against what each operation actually does
/// rather than by analogy.
#[must_use]
pub fn forms_flatten_tooltip() -> &'static str {
    "Turn every field's current value into ordinary page content and remove the form. The \
     values stay visible but stop being editable, and anything typed into them can no longer \
     be changed. One undo reverses it. Note: with the normal save, the old field values are \
     still recoverable from the file's earlier revision — flatten is not a way to remove \
     sensitive answers."
}

/// Warning beside Flatten when some field has no drawn appearance to burn.
///
/// **New in this build**, and it closes a real data-loss path rather than a
/// cosmetic one.
///
/// Flatten works by invoking each widget's **existing** `/AP` as a page
/// XObject. A field with no normal appearance — which is exactly what
/// `/NeedAppearances` announces, and what
/// `pdfcer_core::forms::Field::has_appearance` reports per field — has nothing
/// to invoke, so flattening burns **nothing** for it and then removes the
/// field. The typed value disappears from the visible page.
///
/// The remedy is [`forms_regenerate_button`], which is why the two controls
/// sit side by side and why this sentence names it. Core's own guidance is
/// the same: regenerate first, then flatten.
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
// What the page can and cannot be filled from — see `crate::canvas::forms`
// ---------------------------------------------------------------------------

/// The panel's note for fields that **cannot be clicked on the page** because
/// nothing is drawn for them.
///
/// # Why this sentence exists at all
///
/// `crate::canvas::forms` lets an operator click a field where it is drawn.
/// The word *drawn* is load-bearing: a widget with no `/AP` `/N` paints
/// nothing, so a click target over it would be an invisible affordance — the
/// operator can only find it by accident and cannot find it again. The canvas
/// therefore declines it, and this is the panel telling them **where the field
/// went**, which is the whole difference between a routing decision and a
/// capability that quietly disappeared.
///
/// # ★ The remedy sentence was wrong on its first draft, and driving the
/// binary is what caught it
///
/// It read: *"Use “Redraw values” to draw them, and they can then be clicked
/// where they sit."* That is **false for the case the sentence is about.**
///
/// Measured on `demo-form.pdf`, which carries exactly one undrawn field
/// (`Full name`, with no value): pressing Redraw values traced
/// `form-regenerate-appearances commands=0 (nothing to do)` and the field
/// stayed undrawn. `EditSession::regenerate_appearances` walks the fields and
/// `continue`s on any text field whose `/V` is not `FieldValue::Text` — an
/// **absent** value has nothing to draw, so an empty field is skipped by
/// design. Redraw only helps a field that is undrawn *and already holds a
/// value*, which is a real and common case (a form filled by another program
/// that never generated appearances, which is what `/NeedAppearances`
/// announces) but is not this one.
///
/// The remedy that always works is the one this panel is: `fill_text_field`
/// writes `/V` **and** regenerates the `/AP` of every widget of the field, so
/// filling an undrawn field here once makes it drawn — and therefore clickable
/// on the page from then on. So the sentence names both, in the order they
/// apply.
///
/// Recorded rather than quietly reworded because the first draft is the shape
/// of mistake `HANDOFF.md` §2 exists for: it was plausible, it read well, no
/// test could contradict it, and it was a promise to the operator that the
/// engine would not keep.
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
///
/// Two causes, said as one sentence because they have one remedy — fill it
/// here — and because an operator does not need to know which of them applies
/// to which field in order to act:
///
/// 1. **A rotated page.** `egui` cannot rotate a text box, so on a `/Rotate 90`
///    page an in-place editor would run horizontally across text the
///    appearance draws vertically. The click and the *placement* are both
///    correct at every rotation; it is only the editor that cannot be.
/// 2. **The file does not say which page the field is on.** `/P` is optional
///    on a widget annotation, and without it there is no page to put a box on.
///
/// Deliberately **not** phrased as a limitation to be fixed ("pdfcer cannot
/// yet…"), because one half of it is a property of the file rather than of
/// pdfcer, and a sentence that promised a future version would be a promise
/// only half of which could ever be kept.
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
///
/// A `/DA` of `0 Tf` means auto-size (§12.7.3.3): the field declines to state a
/// size and leaves the writer to pick one that fits. pdfcer picks one, and the
/// number it picked is what lands in the file — so **nothing in the saved
/// document says the number was pdfcer's rather than the author's**, and no
/// amount of re-reading the field afterwards can recover the distinction. That
/// is precisely the shape of thing rule 4 exists for: an inference, made on the
/// operator's behalf, invisible in the result.
///
/// It matters because another writer, filling the same field, will choose its
/// own number — so a form filled here and a form filled elsewhere can legibly
/// differ, and the operator is entitled to know why before they compare the
/// two.
///
/// The field is named because the disclosure is read somewhere other than
/// where the value was typed — in a panel, possibly beside forty other rows,
/// possibly after a fill made by clicking the page.
#[must_use]
pub fn forms_fill_autosize_note(field: &str, size: f64) -> String {
    format!(
        "⚠ “{field}” asks for an automatic text size and pdfcer chose {size:.1} pt. Another \
         program filling this field may choose differently."
    )
}

/// Rule-4 disclosure: **characters were replaced**, and the operator's own
/// text is not what the page now says.
///
/// The field's font is a Base-14 Latin face and `pdfcer-core` encodes into
/// `WinAnsi`; a character with no code there is written as `?`. The saved value
/// **is** the substituted one, so re-reading the field tells the operator what
/// pdfcer wrote and never that it wrote something other than what they typed.
///
/// This is the more serious of the two fill disclosures and is worded as such:
/// an auto-size is a difference of appearance, this is a difference of
/// *content*. The count is given rather than the characters, because listing
/// them would mean echoing the operator's own typing back into a surface that
/// may be screenshotted, and because the count is what tells them whether it
/// was a stray character or the whole name.
#[must_use]
pub fn forms_fill_unencodable_note(field: &str, count: usize) -> String {
    format!(
        "⚠ {count} character(s) of “{field}” could not be written in this field's font and were \
         stored as “?”. The page now shows those question marks."
    )
}

/// Tooltip on a form-wide control disabled by a certification signature.
///
/// Distinct from [`form_field_certification_disabled_tooltip`], and the
/// distinction is the one the old shell drew and argued for at length: filling
/// takes core's `/P`-aware gate, while flattening is a **structural** change to
/// the form and takes the strict one. On the ordinary real-world shape — a
/// certified fillable form at `/P 2` — filling is offered and flattening is
/// refused, so one sentence could not cover both without being wrong about one
/// of them.
#[must_use]
pub fn forms_structural_certification_disabled_tooltip() -> &'static str {
    "A certification signature on this document forbids changing the form's structure. Values \
     can still be filled in; the form itself cannot be removed."
}

/// Disclosure: the field's other boxes stayed where they were.
///
/// ★★★ The engine's own words for why this is owed: *"A field with widgets on
/// pages 1, 2 and 3 looks like one thing to an operator who asked to move 'the
/// signature box'. Moving one and silently leaving two behind is the kind of
/// partial result that reads as a bug later."*
///
/// ★★ It says the move was **correct**, not that something went wrong. The
/// boxes are separate placements of one value and moving one is exactly what
/// the operator dragged — so the sentence's job is to stop them hunting for a
/// fault, not to apologise for one.
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
    ///
    /// Each exists because, without it, the operator's only available reading
    /// of an inert control is *"pdfcer got it wrong"*. Two that read alike
    /// would send them looking for the wrong cause — and two of these
    /// describe gates that genuinely disagree with each other on the most
    /// common certified document there is.
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

    /// **★ The fill gate and the structural gate say different things.**
    ///
    /// Not a tautology of the test above. These two are the pair most likely
    /// to be collapsed into one string by someone tidying up, because on most
    /// documents they are both absent and on a fully-locked one they are both
    /// present. The case that matters is the ordinary certified fillable form
    /// (`/P 2`), where filling is permitted and flattening is not — and an
    /// operator told "form values cannot be changed" while happily typing
    /// into the fields has been misinformed by their own tool.
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
    ///
    /// R84 — never a colour-class cue alone. A `⚠` is exactly that, and a
    /// sentence whose meaning depended on it would be unreadable to anyone
    /// whose font lacks the glyph or who is listening rather than looking.
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
    ///
    /// The single most consequential distinction in this file. Both cases
    /// render as a row with no formatting listed, and only one of them is a
    /// reason to stop before pressing Convert: an unreadable `/RV` means the
    /// operator is about to discard formatting **nobody has seen**.
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
    ///
    /// Pins the ordering decision recorded in
    /// [`form_field_rich_text_summary`]'s header — the one found by reading a
    /// rendered panel rather than the code. `bold` and `italic` must end up
    /// adjacent even when they arrive on runs separated by a run carrying the
    /// `/DS` size and family.
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
    ///
    /// The sentence says "you can fill **here**", not "fillable fields". That
    /// word is what makes the count honest when a certification signature
    /// disables every row: the panel is describing itself, not the model's
    /// `is_fillable` predicate. See the function's own header for the bite
    /// this closes.
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
    ///
    /// "pdfcer never runs a document's JavaScript" is a project rule, and the
    /// wording has to read as a decision rather than as an unfinished feature
    /// — otherwise an operator waits for a version that will never come.
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
