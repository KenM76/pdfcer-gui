//! # `text::panels::layersearch` — the four strings the Layers search says
//!
//! **Surface:** the search field at the top of the Layers panel, and the
//! two lines that describe what it did.
//! **Consumer:** `pdfcer_gui::panels::layers`, and nothing else.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/layersearch.md`.

/// The hint inside the empty search field.
#[must_use]
pub fn field_hint() -> &'static str {
    "Search layers"
}

/// The accessible name and hover text for the field.
#[must_use]
pub fn field_tooltip() -> &'static str {
    "Show only the layers whose name contains what you type. Capitals do not matter."
}

/// The label on the control that empties the field.
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
#[must_use]
pub fn narrowed(shown: usize, total: usize) -> Option<String> {
    if shown >= total {
        return None;
    }
    Some(format!("Showing {shown} of {total} layers."))
}

/// **The sentence an empty result owes the operator.**
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
