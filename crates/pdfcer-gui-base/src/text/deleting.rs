//! # `text::deleting` — the sentences a Delete that removed nothing shows
//!
//! Four of them, for `pdfcer_gui::canvas::deleting`, plus the rule that decides
//! which refusals get one at all.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/deleting.md`.

use crate::refusals::delete::Refusal;

/// The sentence for a refusal to delete, or `None` when the state is one the
/// operator can already see.
///
///
/// `Option` rather than an empty string, because *"there is deliberately
/// nothing to say"* and *"the sentence is missing"* must not be the same value.
#[must_use]
pub const fn refusal(reason: Refusal) -> Option<&'static str> {
    match reason {
        // §9.4.2, and the one refusal in this file that names a remedy —
        // which is the entire reason it is asked before the press rather than
        // left to the engine. See `canvas::deleting`'s header on R83.
        //
        // The mechanism, in the operator's terms: a label that carries no
        // position of its own starts wherever the label before it ended. Remove
        // the earlier one and the later one slides somewhere nobody put it. The
        // remedy always works and cannot fail — a label that follows a label
        // that is already gone is not a case this can produce.
        Refusal::RunWouldMoveNext(_) => Some(
            "The label after this one has no position of its own — it starts where this one \
             ends — so removing this one would move it somewhere you did not put it. Delete the \
             later label first, then this one.",
        ),
        // Four points highlighted, one press, and pdfcer would remove one of
        // them. Refusing and saying how many is the honest answer; acting on
        // the first is the defect that let a four-anchor drag move one anchor
        // for months.
        Refusal::ManyNodes(_) => {
            Some("pdfcer removes one corner point at a time. Click a single point, then Delete.")
        }
        // The twin one rung up, and the asymmetry it has to survive: the
        // same set of lines CAN be dragged together, so an operator who has just
        // moved four of them at once has every reason to expect Delete to
        // remove four. The sentence says the limit is Delete's rather than the
        // selection's, so the set they built does not look wrong to them.
        Refusal::ManyLines(_) => Some(
            "pdfcer removes one line at a time, although it can move several together. Click a \
             single line, then Delete.",
        ),
        // The page will not decompose, so nothing INSIDE an object can be
        // named — see this module's header for why this one is new. The
        // sentence names what the operator can still do, because they can: the
        // Object rung never needed the decomposition, so Escape and Delete
        // removes the whole shape on a page whose interior pdfcer cannot read.
        Refusal::NoObjectModel => Some(
            "pdfcer could not read the inside of this page, so it cannot remove one piece of a \
             shape here. Press Escape to step back out to the whole shape, then Delete.",
        ),
        // The seven that say nothing, listed rather than caught by a wildcard:
        // a new variant must be classified by whoever adds it, and `_ => None`
        // would classify it as "obvious" by default — which is the direction
        // that ships a silent Delete.
        Refusal::NothingSelected
        | Refusal::NoPartEntered
        | Refusal::NoNodeEntered
        | Refusal::UnaddressableObject
        | Refusal::NoPartsInObject
        | Refusal::NoNodeVerbForText => None,
    }
}

#[cfg(test)]
mod tests {
    use super::refusal;
    use crate::refusals::delete::Refusal;

    /// Every sentence this catalogue offers is finished English prose.
    #[test]
    fn every_sentence_is_finished_prose_with_no_baked_gap() {
        for reason in [
            Refusal::RunWouldMoveNext(3),
            Refusal::ManyNodes(4),
            Refusal::ManyLines(3),
            Refusal::NoObjectModel,
        ] {
            let sentence = refusal(reason).expect("this refusal is meant to speak");
            assert!(
                !sentence.contains("  "),
                "{reason:?} has a run of spaces in it: {sentence}"
            );
            assert!(
                sentence.ends_with('.'),
                "{reason:?} does not end in a full stop: {sentence}"
            );
        }
    }

    /// **No sentence may say "dimension"** — R8b Rule 15, mechanically.
    #[test]
    fn no_sentence_writes_a_bare_dimension() {
        for reason in [
            Refusal::RunWouldMoveNext(0),
            Refusal::ManyNodes(2),
            Refusal::ManyLines(2),
            Refusal::NoObjectModel,
        ] {
            if let Some(sentence) = refusal(reason) {
                assert!(
                    !sentence.to_lowercase().contains("dimension"),
                    "{reason:?} writes a bare \"dimension\": {sentence}"
                );
            }
        }
    }

    /// The seven that are deliberately silent stay silent — so that a future
    /// edit which starts narrating "nothing selected" has to change a test that
    /// says why it should not.
    #[test]
    fn the_states_the_operator_can_see_say_nothing() {
        for reason in [
            Refusal::NothingSelected,
            Refusal::NoPartEntered,
            Refusal::NoNodeEntered,
            Refusal::UnaddressableObject,
            Refusal::NoPartsInObject,
            Refusal::NoNodeVerbForText,
        ] {
            assert!(
                refusal(reason).is_none(),
                "{reason:?} narrates a state the operator can already see"
            );
        }
    }
}
