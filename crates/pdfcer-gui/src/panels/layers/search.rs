//! `panels::layers::search` — narrowing the Layers list as you type.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/layers/search.md`.

/// The lower bound on how many layers a document must have before the
/// search field is drawn at all.
///
/// **Two**, and it is a threshold rather than "always" for R9's reason.
/// A search over a one-row list can do exactly one thing — remove the row —
/// so a field offering it is a control whose only outcome is to make the
/// panel emptier. Drawing it anyway would be the placeholder rule broken in
/// its subtler form: not a control that does nothing, but a control whose
/// every outcome is useless.
///
/// Not a larger number, though a reader will wonder. A threshold of, say,
/// eight would be a judgement about when a list becomes hard to scan, and
/// this panel is the wrong place to make it: a CAD sheet with three layers
/// called `A-ANNO-TEXT`, `A-ANNO-DIMS` and `A-ANNO-NOTE` is genuinely
/// easier to work with a filter than without one, and a threshold that hid
/// the field would be deciding for the operator on the basis of a count
/// that does not describe their problem. Two is the only value that follows
/// from an argument rather than from taste.
pub const MIN_LAYERS_FOR_SEARCH: usize = 2;

/// **Does `name` match `query`?**
///
/// `name` is the text the row displays — see the module header on why that
/// is not the same as `Layer::name`.
///
/// An empty or all-whitespace query matches **everything**, which is what
/// makes "clearing the box restores the list" true by construction rather
/// than by a branch somewhere else remembering to skip the filter.
///
/// # Why this takes `&str` and not `&Layer`
///
/// So that the rule cannot quietly grow a second input. A predicate handed
/// the whole layer could be extended to consult `visible_by_default` or
/// `locked` in one line, by someone who had not read the module header, and
/// nothing would fail — the search would simply start returning rows for
/// reasons the operator cannot see. Decision 1 is enforced by the
/// signature, which is stronger than enforcing it by a comment.
#[must_use]
pub fn matches(name: &str, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return true;
    }
    contains_ignore_ascii_case(name, query)
}

/// Case-insensitive substring test, ASCII folding, no allocation.
fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    let (h, n) = (haystack.as_bytes(), needle.as_bytes());
    if n.is_empty() {
        return true;
    }
    if n.len() > h.len() {
        return false;
    }
    h.windows(n.len()).any(|w| w.eq_ignore_ascii_case(n))
}

/// What one frame's filtering removed.
///
/// The [`crate::panels::comments::model::Excluded`] shape: a count rather
/// than a discard, because the panel **discloses** it. A filter that threw
/// away the number of rows it hid could only say "nothing here", and
/// "nothing here" is what a broken panel says too.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Filtered {
    /// How many layers matched the query and are drawn.
    pub shown: usize,
    /// How many layers the query removed.
    ///
    /// Zero when the query is empty, always — which is what makes "an
    /// unfiltered panel says nothing extra" true rather than conditional.
    pub hidden: usize,
}

impl Filtered {
    /// Every layer, unfiltered.
    ///
    /// The value the panel builds when no query is in force, so that the
    /// "nothing was filtered" case is a named construction rather than a
    /// `hidden: 0` a reader has to interpret.
    #[must_use]
    pub const fn all(total: usize) -> Self {
        Self {
            shown: total,
            hidden: 0,
        }
    }

    /// Whether the query removed anything, i.e. whether the panel owes the
    /// operator a sentence about it.
    #[must_use]
    pub const fn is_narrowed(self) -> bool {
        self.hidden > 0
    }

    /// Whether the query removed **everything**.
    ///
    /// Distinguished from `shown == 0` on an empty document, which cannot
    /// happen here — the panel returns before it filters when the document
    /// has no optional content — but the distinction is named anyway so a
    /// caller cannot accidentally use one for the other.
    #[must_use]
    pub const fn is_empty_because_of_the_query(self) -> bool {
        self.shown == 0 && self.hidden > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **An empty query matches everything**, which is what makes clearing
    /// the box restore the list.
    #[test]
    fn an_empty_query_matches_every_layer() {
        for name in ["Dimensions", "", "A-WALL", "図面"] {
            assert!(matches(name, ""));
            assert!(matches(name, "   "), "and whitespace is not a query");
            assert!(matches(name, "\t\n"));
        }
    }

    /// **Case-insensitive**, which is `FindOptions`' argued default:
    /// *"an operator who types `total` and is not shown `TOTAL` on the next
    /// line reads that as a search that did not work."*
    #[test]
    fn matching_ignores_case_in_both_directions() {
        assert!(matches("HIDDEN", "hidden"));
        assert!(matches("hidden", "HIDDEN"));
        assert!(matches("Hidden Detail", "dEtAiL"));
    }

    /// **Substring, not prefix and not whole-word.**
    ///
    /// A CAD layer set is `A-ANNO-TEXT`, `A-ANNO-DIMS`, `S-GRID-IDEN`; the
    /// useful query is `anno`, which is a prefix of nothing.
    #[test]
    fn matching_is_a_substring_test_anywhere_in_the_name() {
        assert!(matches("A-ANNO-TEXT", "anno"));
        assert!(matches("A-ANNO-TEXT", "text"));
        assert!(matches("A-ANNO-TEXT", "A-ANNO-TEXT"));
        assert!(!matches("A-ANNO-TEXT", "annotext"));
    }

    /// **Literal, not a pattern.** A layer called `A*` is searched for by
    /// typing `A*`, and an asterisk is not a wildcard.
    #[test]
    fn a_query_is_literal_and_a_star_is_a_character() {
        assert!(matches("A*B", "*"));
        assert!(!matches("AB", "*"));
        assert!(!matches("Dimensions", "Dim*"));
    }

    /// **The query is trimmed**, so a pasted trailing space is not a
    /// search that mysteriously fails.
    #[test]
    fn surrounding_whitespace_in_the_query_is_ignored() {
        assert!(matches("Dimensions", "  dim  "));
        assert!(
            !matches("Dimensions", "  d im  "),
            "but INNER whitespace is part of the query, or a two-word layer name is unsearchable"
        );
    }

    /// **A multi-byte name does not produce a false match**, and the
    /// window walk does not panic on one.
    ///
    /// The slicing argument in [`contains_ignore_ascii_case`]'s docs, made
    /// falsifiable.
    #[test]
    fn a_non_ascii_name_matches_only_on_its_real_bytes() {
        assert!(matches("図面レイヤ", "レイ"));
        assert!(!matches("図面レイヤ", "面レイヤー"));
        // A window starting mid-character cannot match a needle that does
        // not have those exact bytes.
        assert!(!matches("é", "e"));
        assert!(matches("é", "é"));
    }

    /// **A query longer than the name matches nothing**, without panicking
    /// on the window walk.
    #[test]
    fn a_query_longer_than_the_name_does_not_match_or_panic() {
        assert!(!matches("Dim", "Dimensions"));
        assert!(!matches("", "x"));
    }

    /// **The name the ROW shows is what is matched.**
    #[test]
    fn the_unnamed_placeholder_is_searchable_by_what_it_says() {
        let shown = crate::text::panels::layer_unnamed();
        assert!(
            matches(shown, shown),
            "a layer with no /Name is drawn as {shown:?}; typing that must find it"
        );
        // And a fragment of it, since the operator types what they see.
        let fragment: String = shown.chars().take(4).collect();
        assert!(matches(shown, &fragment));
    }

    /// **An unfiltered panel reports no hiding**, so it says nothing extra.
    #[test]
    fn an_unfiltered_listing_is_not_narrowed() {
        let f = Filtered::all(16);
        assert_eq!(f.shown, 16);
        assert!(!f.is_narrowed());
        assert!(!f.is_empty_because_of_the_query());
    }

    /// **A query that matches nothing is distinguishable from a
    /// document with no layers.**
    #[test]
    fn an_empty_result_knows_it_was_the_query_that_emptied_it() {
        let f = Filtered {
            shown: 0,
            hidden: 16,
        };
        assert!(f.is_empty_because_of_the_query());
        assert!(f.is_narrowed());
        let no_layers = Filtered::all(0);
        assert!(
            !no_layers.is_empty_because_of_the_query(),
            "a document with no layers must not claim the search emptied it"
        );
    }

    /// **The threshold is a real one**: one layer gets no field, two do.
    #[test]
    fn the_search_field_needs_something_to_search() {
        // Written as a sweep over counts rather than as two comparisons
        // against the constant, because clippy is right that
        // `assert!(1 < CONST)` is a constant expression and proves nothing
        // at run time. This asserts the SHAPE of the rule instead: no field
        // below the threshold, a field at and above it.
        for total in 0..8usize {
            assert_eq!(
                total >= MIN_LAYERS_FOR_SEARCH,
                total > 1,
                "a search over {total} layer(s) must be offered iff there is more than one"
            );
        }
    }
}
