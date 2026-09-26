//! # `text::panels::layersearch` — the four strings the Layers search says
//!
//! **Surface:** the search field at the top of the Layers panel, and the
//! two lines that describe what it did.
//! **Consumer:** [`crate::panels::layers`], and nothing else.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/panels/layersearch.md`.

/// The hint inside the empty search field.
///
/// *"Search layers"* and not *"Filter…"* or *"Type to filter"*. The
/// operator asked for *"a search to implement on the layers"* and that is
/// the word they used; a control whose label is not the word the person
/// asking for it used is a control they have to translate. "Filter" is also
/// the wrong promise in a small way — a filter usually implies a set of
/// fixed criteria you choose from, which is what a state filter would be
/// (see `panels::layers::search`'s Decision 1) and what this is not.
///
/// No ellipsis: it is a hint, not a command that opens something.
#[must_use]
pub fn field_hint() -> &'static str {
    "Search layers"
}

/// The accessible name and hover text for the field.
///
/// It states what is matched, because Decision 1 in
/// `panels::layers::search` is a decision an operator can otherwise only
/// discover by typing `hidden` and being surprised. One clause, at the one
/// moment they are looking at the control.
#[must_use]
pub fn field_tooltip() -> &'static str {
    "Show only the layers whose name contains what you type. Capitals do not matter."
}

/// The label on the control that empties the field.
///
/// It exists at all because the field is drawn **above a list the
/// search may have emptied**, and an empty list is the one state in which
/// the operator most needs to undo the thing that caused it. Clearing a
/// text field by selecting and deleting is three gestures; this is one, and
/// it is beside the state it repairs.
///
/// A word rather than a `×` glyph: the icon set has no clear-field art, and
/// `icons::paint` draws a visible mark for an unknown key rather than
/// nothing — so an invented key would ship a placeholder rectangle, which
/// is precisely what R9 forbids.
#[must_use]
pub fn clear_label() -> &'static str {
    "Clear"
}

/// Hover text for [`clear_label`].
#[must_use]
pub fn clear_tooltip() -> &'static str {
    "Empty the search box and show every layer again."
}

/// **How many layers the search is showing, out of how many there are.**
///
/// `None` when the search removed nothing, which is the whole of rule 3: a
/// panel with no query in it says exactly what it said before this feature
/// existed.
///
/// It reports `shown of total` rather than `hidden`, and the two are not
/// interchangeable. The operator is looking at the list; the useful number
/// is the size of the thing in front of them and how much of the whole it
/// is. *"13 layers hidden"* makes them do the subtraction to find out
/// whether the one they want could still be there.
#[must_use]
pub fn narrowed(shown: usize, total: usize) -> Option<String> {
    if shown >= total {
        return None;
    }
    Some(format!("Showing {shown} of {total} layers."))
}

/// **The sentence an empty result owes the operator.**
///
/// R9 says an unavailable capability renders nothing, and an empty list is
/// not a placeholder — but *"no rows"* and *"no rows because of what you
/// typed"* are different states, and the operator can see the layers are in
/// the document because they were on screen a moment ago.
///
/// The query is quoted back, per rule 2. The count of what was hidden
/// follows it, because the two together are the complete answer: *what you
/// asked for*, and *what is still there behind it*.
///
/// The query is **not** truncated. A pasted paragraph would make a long
/// line, and a long line in a narrow panel wraps — which is ugly and
/// correct. Eliding it would produce a sentence quoting something the
/// operator did not type, which for the one string whose job is to prove
/// the search ran is the one thing it must never do.
#[must_use]
pub fn none_matched(query: &str, total: usize) -> String {
    if total == 1 {
        format!("No layer matches \u{201c}{query}\u{201d}. The document has 1 layer.")
    } else {
        format!("No layer matches \u{201c}{query}\u{201d}. The document has {total} layers.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Nothing is said when nothing was filtered** — rule 3, which is
    /// what keeps an unfiltered panel identical to what it was.
    #[test]
    fn an_unfiltered_list_discloses_nothing() {
        assert_eq!(narrowed(16, 16), None);
        assert_eq!(narrowed(0, 0), None);
        assert_eq!(
            narrowed(17, 16),
            None,
            "a shown count above the total is nonsense, and nonsense must not become a sentence"
        );
    }

    /// **A narrowed list says both numbers** — rule 1.
    #[test]
    fn a_narrowed_list_says_how_many_of_how_many() {
        let line = narrowed(3, 16).expect("a narrowed list owes a sentence");
        assert!(line.contains('3'), "{line}");
        assert!(line.contains("16"), "{line}");
    }

    /// **The empty case quotes the query back** — rule 2, and the only
    /// way an operator can be sure the search ran rather than the panel
    /// failing.
    #[test]
    fn the_empty_case_repeats_what_was_typed() {
        let line = none_matched("A-ANNO", 16);
        assert!(
            line.contains("A-ANNO"),
            "the query must appear verbatim in the sentence: {line}"
        );
        assert!(
            line.contains("16"),
            "and so must what is still there: {line}"
        );
    }

    /// **A query with quotation marks or punctuation survives verbatim.**
    #[test]
    fn a_query_containing_quotes_is_still_shown_as_typed() {
        let line = none_matched("say \"hi\"", 4);
        assert!(line.contains("say \"hi\""), "{line}");
    }

    /// **Singular and plural are both grammatical**, which every count in
    /// this catalog is required to be.
    #[test]
    fn the_layer_count_agrees_with_its_number() {
        assert!(none_matched("x", 1).contains("1 layer."));
        assert!(none_matched("x", 2).contains("2 layers."));
    }

    /// **Every string is distinct**, the property every text module here
    /// asserts: two controls with one label are two controls an operator
    /// cannot tell apart.
    #[test]
    fn no_two_strings_are_the_same() {
        let all = [
            field_hint(),
            field_tooltip(),
            clear_label(),
            clear_tooltip(),
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }

    /// **The field's tooltip states what is matched.**
    #[test]
    fn the_field_tooltip_says_it_matches_the_name() {
        assert!(
            field_tooltip().contains("name"),
            "the tooltip is where Decision 1 is disclosed: {}",
            field_tooltip()
        );
    }
}
