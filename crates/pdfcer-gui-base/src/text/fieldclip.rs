//! # `text::fieldclip` - the sentences the FORM-FIELD clipboard can say
//!
//!
//!
//! `Pass 167.0` shipped `pdfcer_core::formclip` and every one of those
//! properties now travels. The loss note is **deleted**, not softened, on the
//! engine's own instruction: *"you should not be maintaining a hand-written map
//! of which properties survive, because it rots silently every time we add an
//! authoring key."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/fieldclip.md`.

use crate::refusals::fieldclip::Refusal;

/// The sentence for a refusal.
#[must_use]
pub fn refusal(reason: &Refusal) -> String {
    match reason {
        Refusal::NothingSelected => {
            "No form field is selected. Click a field on the page first.".to_owned()
        }
        // Not "an error occurred". The document changed underneath the
        // selection — an undo, a deletion from the Forms panel — and the
        // operator's next act is to click the field again, so the sentence
        // says that rather than describing the internal state.
        Refusal::Vanished => {
            "That field is no longer in the document. Click a field on the page again.".to_owned()
        }
        Refusal::NoGeometry => "That field has no box on the page, so there is nothing to copy. Fields like this are reached from the Forms panel.".to_owned(),
        // THE ENGINE'S OWN WORDS, and this variant replaced two of this
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
#[must_use]
pub const fn brings_a_script() -> &'static str {
    "This field carries a calculation or format script, and the paste brings it along. \
     If it refers to other fields by name, those fields need to exist here too."
}

/// **A name whose group is already an ordinary field** — refused, with the
/// reason and the remedy.
///
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
///
///
/// So the old sentence had become **false in the operator's direction**, which
/// is the worst way for a refusal to be wrong. It said *"Using this name would
/// turn it into a group and lose what is in it"*, warning about a loss that can
/// no longer occur, and it said nothing about what actually happened, which is
/// that the edit was refused and his document is untouched. An operator reading
/// it would reasonably believe he had just been stopped at the edge of
/// something dangerous rather than told about a name he cannot have.
///
/// # Why the sentence survives the fix at all
///
/// Because the engine's own — which names the victim and cites §12.7.3.1 —
/// cannot reach him. `check-ui-strings.sh`'s exclusion 3 says in as many words
/// that an error type's `Display` is not permission to route operator text
/// through it, so the funnel sends the engine's prose to `PDFCER_DIAG` and puts
/// its own generic *"That change was refused"* on the bar. This function is
/// what turns that shrug back into an answer, fed by
/// `Declined::FieldPathCrossesTerminal`, which carries the name **the engine
/// resolved** rather than one this shell re-derived.
///
/// Also corrected: the old text said the hole was reachable "in two gestures:
/// the placement dialog's name box, and the Properties panel's rename". **The
/// rename was never one of them.** `rename_field` takes a *partial* name and
/// refuses a dotted one outright (`DottedPartialName`), for a different reason
/// — a `/T` containing a dot is unaddressable by every fill verb, pdfcer's
/// included — and it has done so for longer than this comment existed.
///
/// # The wording
///
/// It names the field that would be destroyed, because *"invalid name"* would
/// leave the operator guessing which part offended. And it offers the remedy
/// that is almost always what they meant — a plain name — rather than
/// explaining PDF field hierarchies to a draughtsman.
#[must_use]
pub fn name_crosses_a_field(terminal: &str) -> String {
    format!(
        "That name was refused and the document is unchanged: a dot puts the field inside a \
         group, and \u{201c}{terminal}\u{201d} is already an ordinary field rather than a \
         group. Pick a name without a dot, or rename \u{201c}{terminal}\u{201d} first."
    )
}

/// **A dotted name where a single name was required** — refused, with the
/// reason and the remedy.
#[must_use]
pub fn name_is_a_path(supplied: &str) -> String {
    format!(
        "That name was refused and the document is unchanged: \u{201c}{supplied}\u{201d} is a \
         path, not a name. A dot separates the levels of a form's field names, so a field whose \
         own name contains one can be clicked but never filled in \u{2014} not by pdfcer, and \
         not by any other reader. Pick a name without a dot."
    )
}

/// **What a field copy leaves on the OPERATING SYSTEM's clipboard.**
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
/// # NO SEPARATOR, and above all NO DOT
///
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
/// The engine's silent conversion is a defect in its own right and is filed
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

    /// Every shell-owned refusal says what to do next.
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

    /// The engine's wording passes through UNCHANGED.
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
