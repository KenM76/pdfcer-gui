//! # `text::fieldclip` - the sentences the FORM-FIELD clipboard can say
//!
//! ## ★★★ This file lost half its job on 2026-08-29, and that was the win
//!
//! It used to carry two families: the refusals, and a **loss note** listing
//! which properties a paste could not carry - the font, the alignment, the
//! default value, the calculation, the border colours. That list existed
//! because the paste **re-authored** through `New*Field`, a spec that can only
//! express geometry and a dozen booleans.
//!
//! `Pass 167.0` shipped `pdfcer_core::formclip` and every one of those
//! properties now travels. The loss note is **deleted**, not softened, on the
//! engine's own instruction: *"you should not be maintaining a hand-written map
//! of which properties survive, because it rots silently every time we add an
//! authoring key."*
//!
//! What replaced it is `FieldPasteOutcome::disclosures` - a `Vec<String>` the
//! **engine** writes, covering a dropped value, a carried calculation and its
//! `/CO` registration, a renamed font resource, an ignored rectangle size, the
//! tab-order position, a dropped structure-tree link and a reused accessibility
//! name. It reaches the status row through `vector_edit` like every other verb's
//! disclosures, and **not one word of it is written here**.
//!
//! ⇒ The rule that decided it is this shell's own and it has now removed two
//! sentences from this file in one day: **one fact, one wording.** The engine's
//! version is authoritative - it reports what the operation *did*, not what the
//! shell *intended* - and a second phrasing is a divergence waiting to happen.
//!
//! ## What remains
//!
//! [`refusal`] - why nothing happened. Same posture as `text::clipboard`: a
//! keystroke that does nothing and says nothing is indistinguishable from a
//! broken keyboard.
//!
//! [`os_marker`] - the sentence a copy leaves on the *operating system's*
//! clipboard, which is not a courtesy but a **requirement**; see its own header.
//!
//! [`candidate_name`] - the spelling of a pasted field's name.
//!
//! ## ★★ Rule 4, in one line, because it still governs
//!
//! A pasted field renders exactly as a saved-and-reopened one would - no badge,
//! no tint, nothing drawn on the page. The disclosure lives off-canvas, on the
//! status row. *Render normally; report separately.* **Both.**

use crate::canvas::fieldclip::Refusal;

/// The sentence for a refusal.
///
/// Returns an owned `String` rather than a `&'static str` because
/// [`Refusal::EngineRefused`] carries the engine's own wording, which is not
/// static and must not be paraphrased. The four shell-owned variants are still
/// literals here, so `check-ui-strings` still sees them.
#[must_use]
pub fn refusal(reason: &Refusal) -> String {
    match reason {
        Refusal::NothingSelected => {
            "No form field is selected. Click a field on the page first.".to_owned()
        }
        // ★ Not "an error occurred". The document changed underneath the
        // selection — an undo, a deletion from the Forms panel — and the
        // operator's next act is to click the field again, so the sentence
        // says that rather than describing the internal state.
        Refusal::Vanished => {
            "That field is no longer in the document. Click a field on the page again.".to_owned()
        }
        Refusal::NoGeometry => "That field has no box on the page, so there is nothing to copy. Fields like this are reached from the Forms panel.".to_owned(),
        // ★★★ THE ENGINE'S OWN WORDS, and this variant replaced two of this
        // shell's.
        //
        // It used to say *"signature fields cannot be copied"* and *"a radio
        // button needs its own export value"*. Both were true when written and
        // one stopped being true within the hour: `formclip` copies an UNSIGNED
        // signature field normally - which hands this shell signature-field
        // authoring it never had, since there is still no `add_signature_field`
        // - and refuses a SIGNED one at the copy, because what would travel is
        // the baked "signed by" artwork into a file nobody signed. The engine
        // declines to make that object rather than making it and warning about
        // it, which is the posture redaction takes.
        //
        // ⇒ Passing the engine's sentence through is not laziness. Its refusals
        // are written by the party that knows why, kept current by the party
        // that changes the rule, and a shell that paraphrased them would be
        // maintaining a second copy of a taxonomy that moves.
        Refusal::EngineRefused(why) => why.clone(),
        Refusal::NothingCopied => {
            "Nothing has been copied. Select a field and press Ctrl+C.".to_owned()
        }
    }
}

/// **The paste is bringing a script with it** — said BEFORE the press.
///
/// ★★★ The one pre-press disclosure this shell owes, and it exists because the
/// fact is **invisible**: a form field carrying a calculation, a format script
/// or a validation looks exactly like one that does not, on the page and in
/// every screenshot of it. Everything else about a paste is reported afterwards
/// by the engine, which knows what actually happened; this has to come first,
/// because after the press the operator has already committed the gesture.
///
/// # Why it does not say WHICH script, or what it references
///
/// Because the engine deliberately does not resolve the field names inside it,
/// and that restraint is right. Acrobat is documented silently dropping a copied
/// JavaScript reference to a field the target document lacks — discovered only
/// on reopen, with nothing said at the time. Naming the uncertainty beats
/// half-analysing it and reporting a confident half-answer.
///
/// # Why it is not a warning, and does not block
///
/// It is usually what the operator wants. A title-block field that computes a
/// sheet count *should* bring its calculation to the next drawing — that is the
/// reason for copying it. The sentence exists so the outcome is not a surprise,
/// not to discourage the act.
#[must_use]
pub const fn brings_a_script() -> &'static str {
    "This field carries a calculation or format script, and the paste brings it along. \
     If it refers to other fields by name, those fields need to exist here too."
}

/// **A name whose group is already an ordinary field** — refused, with the
/// reason and the remedy.
///
/// ★★★ This guards **silent data loss**, measured on 2026-08-29 against
/// `pdfcer` at engine `3ac9dd7`:
///
/// ```text
/// field "Text" = "K. Mantle"        then   add a field named "Text.2"
/// -> field "Text" is GONE, its value is GONE, its box is still on the page
/// ```
///
/// A period separates levels (§12.7.3.2), so `Text.2` asks for a field `2`
/// inside a group `Text` — and §12.7.3.1 does not let one dictionary be both a
/// terminal field and a group. Adding the child **converts** the parent, which
/// discards its `/FT`, its `/V` and its widget's field-ness. The engine reports
/// success and says nothing.
///
/// # Why this lives here rather than being left to the engine
///
/// It should be the engine's, it is filed as
/// `request_a_dotted_name_silently_swallows_an_existing_terminal_field.md`, and
/// the guard there is already **half present** — `add_text_field` refuses the
/// mirror case (*"the name belongs to a group that contains other fields"*) and
/// not this one, which is the destructive direction.
///
/// Until that lands, the path is reachable from this shell in two gestures: the
/// placement dialog's name box, and the Properties panel's rename. Both take a
/// name the operator typed. A shim that prevents an unrecoverable loss is worth
/// its own deletion later; `actions::forms::group_is_a_field` carries the
/// tripwire that will name the day it can go.
///
/// # The wording
///
/// It names the field that would be destroyed, because *"invalid name"* would
/// leave the operator guessing which part offended. And it offers the remedy
/// that is almost always what they meant — a plain name — rather than
/// explaining PDF field hierarchies to a draughtsman.
#[must_use]
pub fn name_would_swallow(existing: &str) -> String {
    format!(
        "A name with a dot puts the field inside a group, and \u{201c}{existing}\u{201d} is \
         already an ordinary field rather than a group. Using this name would turn it into a \
         group and lose what is in it. Pick a name without a dot, or rename \u{201c}{existing}\u{201d} first."
    )
}

/// **What a field copy leaves on the OPERATING SYSTEM's clipboard.**
///
/// ★★★ This exists because of a toolkit constraint, not a design wish, and
/// without it `Ctrl+V` does not work at all. `egui-winit-0.35.0` synthesises
/// `Event::Paste` **only when the OS clipboard holds non-empty text**, and
/// swallows the keystroke entirely otherwise — no key event, no paste event,
/// nothing. So a paste of something pdfcer holds in its own memory would depend
/// on whether the operator had recently copied text in some other application.
///
/// ⇒ **It was found by driving, not by reading**, on 2026-08-29: the whole
/// field clipboard was written, unit-tested, gate-clean and shipped-looking, and
/// the first driven run reported `fieldclip-copy` present and `fieldclip-paste`
/// absent with nothing between them. The RAG entry
/// `egui_winit_swallows_ctrl_c_x_v_so_a_keymap_binding_for_them_is_dead_on_arrival.md`
/// predicted exactly this and it still happened, because the field path is a
/// *new* copy path and the marker lives at each copy site rather than in one
/// place. That is the finding worth keeping: **a documented platform trap does
/// not protect a code path that did not exist when it was documented.**
///
/// # The wording
///
/// For a human who pastes into a text editor and wonders what they got. It names
/// the field, because a form has many and *"a form field"* would not say which.
/// It names both chords, because the second one is the whole feature and an
/// operator who reads this sentence in an email has just been taught it.
#[must_use]
pub fn os_marker(field: &str) -> String {
    format!(
        "The form field “{field}” was copied from pdfcer. Paste it back into pdfcer \
         with Ctrl+V for a new field, or Ctrl+Shift+V for another box that fills with \
         the same value."
    )
}

/// **A candidate name for a pasted field** — `Text` + `2` -> `Text2`.
///
/// # ★★★ NO SEPARATOR, and above all NO DOT
///
/// Corrected 2026-08-29 from `Drawn By 2` (a space) after reading the Acrobat
/// reference `forms__field_copy_paste_and_duplication.md`. Two things it
/// settles, and the second is a correctness matter rather than a taste one.
///
/// **1. The convention is a plain numeric suffix.** Acrobat's bulk duplication
/// ("Create Multiple Copies") auto-names its copies `Date1`, `Date2`, `Date3`,
/// and the sourced rationale is explicitly about scripting: the suffix exists so
/// a script can loop over every field sharing *"the non-number part of the field
/// name"*. **A space breaks exactly that property** — the non-number part of
/// `Drawn By 2` is `Drawn By ` with a trailing space, which no author would
/// write and every string comparison would trip over. So the separator is not a
/// house style to pick; it is load-bearing, and the convention has a reason.
///
/// **2. A DOT DESTROYS THE FIELD YOU COPIED. Measured, not reasoned.**
///
/// The other sourced account has Acrobat numbering copies `Text.0`, `Text.1`
/// with a **dot**, and the two accounts are flagged as contested. The operator
/// asked, reasonably, why pdfcer does not simply follow it. The answer turned
/// out to be a four-command experiment rather than a spec argument:
///
/// ```text
/// add-text-field --name "Text"     ...   -> field "Text",  value -
/// fill-field     --set "Text=K. Mantle"  -> field "Text",  value "K. Mantle"
/// add-text-field --name "Text.2"   ...   -> field "Text.2", value -
///                                           and "Text" IS GONE
/// ```
///
/// A period separates levels (§12.7.3.2), so `Text.2` asks for a field `2`
/// inside a group `Text` — and §12.7.3.1 does not let one dictionary be both a
/// terminal field and a group. Adding the child therefore **converts** `Text`
/// into a group: its `/FT`, its `/V` and its widget stop being a field's. The
/// filled-in value is discarded and the box stays on the page belonging to
/// nothing, still drawn, no longer fillable.
///
/// ⇒ Acrobat's version of the scheme must rename the *original* to `Text.0` in
/// the same operation, which is coherent — and is a **rename of a field that
/// already exists**. A field's name is its identity to every calculation, FDF
/// import and external mapping that references it, and a *copy* has no business
/// changing it. The plain suffix has neither problem: both fields exist, both
/// are independent, and neither is renamed.
///
/// ★ The engine's silent conversion is a defect in its own right and is filed
/// as `request_a_dotted_name_silently_swallows_an_existing_terminal_field.md`.
/// This rule does not depend on that being fixed: even with a clean refusal,
/// the dotted convention would make a paste fail rather than work.
///
/// # Why the catalog and not the call site
///
/// A field name is shown to the operator in three places — the Forms panel, the
/// tab-order list and the Properties header — so it is operator-facing text and
/// `check-ui-strings` was right to insist. The numbering itself is *logic* and
/// lives with the caller; only the spelling is here.
///
/// The name is a placeholder the operator is expected to change, which is why a
/// paste generates one rather than opening a dialog. Four boxes down a column is
/// four keystrokes, not four interruptions.
#[must_use]
pub fn candidate_name(stem: &str, n: u32) -> String {
    format!("{stem}{n}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Every shell-owned refusal says what to do next.
    #[test]
    fn every_shell_refusal_names_the_operators_next_move() {
        for r in [
            Refusal::NothingSelected,
            Refusal::Vanished,
            Refusal::NoGeometry,
            Refusal::NothingCopied,
        ] {
            let s = refusal(&r);
            assert!(
                s.contains("Click") || s.contains("Select") || s.contains("Forms panel"),
                "a refusal that does not say what to do next is a dead end. {r:?} -> {s}"
            );
        }
    }

    /// ★★ The engine's wording passes through UNCHANGED.
    ///
    /// No prefix, no suffix, no rewording. A shell that decorated the engine's
    /// refusal would be maintaining a second copy of a taxonomy that moves - and
    /// this variant exists because two of this file's own hand-written refusals
    /// went stale within an hour of being written.
    #[test]
    fn an_engine_refusal_is_passed_through_verbatim() {
        let engine = "a signed signature field cannot be copied";
        assert_eq!(
            refusal(&Refusal::EngineRefused(engine.to_owned())),
            engine,
            "verbatim, not decorated"
        );
    }

    /// The OS marker names the field and both chords.
    #[test]
    fn the_os_marker_teaches_both_chords() {
        let m = os_marker("Revision");
        assert!(m.contains("Revision"), "a form has many fields; say which");
        assert!(
            m.contains("Ctrl+V") && m.contains("Ctrl+Shift+V"),
            "the second chord is the whole feature, and somebody reading this in an email has just been taught it. Got: {m}"
        );
    }
}
