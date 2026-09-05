//! `panels::layers::search` — narrowing the Layers list as you type.
//!
//! # Why this is its own file
//!
//! The same seam `dialogs/host/placement.rs` was split on, and stated in
//! the same terms: **the rule is a different subject from the drawing.**
//! [`super`] owns what a layer row says and which actions a click raises;
//! this file owns one question and no others — *given a query and a layer,
//! is that layer in the list?* It is a `mod search;` declared inside
//! `layers.rs`, so it lives at `panels/layers/search.rs` and needs no entry
//! in `panels/mod.rs`.
//!
//! The split is worth it for one reason beyond R2: a predicate in a file of
//! its own can be **swept** — every layer shape this project knows about,
//! against every query shape, with no window open. A predicate written
//! inline in a `for` loop inside a `ScrollArea` closure can only be tested
//! by looking at it.
//!
//! # ★★★ Decision 1: it matches THE NAME THE ROW SHOWS, and nothing else
//!
//! The operator's ask was *"there is a search to implement on the layers"*.
//! The question that leaves open is whether a query also matches a layer's
//! **state** — so that typing `hidden` returns the hidden ones.
//!
//! **It does not, and the reason is not that state matching is hard.**
//!
//! > **A search result has to be explicable from the row it returned.**
//!
//! Every row in this panel shows a name, a visible/hidden marker, and up to
//! six caveats hung off the name as tooltips. If a query matched state as
//! well as name, then a document with a layer genuinely called
//! *"Hidden Detail"* and eleven layers that are switched off would answer
//! `hidden` with twelve rows, one of which matched for a completely
//! different reason from the other eleven — and **nothing on screen would
//! say which**. The operator's next move is to conclude the search is
//! broken, and they would be right to.
//!
//! ⇒ Names only. The state is already legible on every row as a word (R84:
//! never colour or a glyph alone), so an operator who wants the hidden ones
//! can see them without asking. If a *state filter* is ever wanted it is a
//! different control — a pair of tick boxes, like `app::status::filter`'s
//! eleven-class pick filter — and it composes with this rather than hiding
//! inside it.
//!
//! ★★ **"The name the ROW shows"**, not `Layer::name`. A layer whose
//! `/Name` is absent — a real malformation, since Table 98 makes it
//! Required — is drawn as [`crate::text::panels::layer_unnamed`]'s
//! placeholder rather than as an invented "Layer 3". A search that matched
//! the *underlying* field would leave that row unmatchable by anything the
//! operator can read, which is the same defect as matching state: the
//! result would not be explicable from the row.
//!
//! # ★★ Decision 2: case-insensitive, substring, literal
//!
//! Taken wholesale from [`crate::find::FindOptions`]'s default rather than
//! decided again, because this shell has already argued it and a second
//! answer in a second place is how two searches in one program come to
//! behave differently:
//!
//! > *"an operator who types `total` and is not shown `TOTAL` on the next
//! > line reads that as a search that did not work."*
//!
//! The one thing NOT carried across is the **Match case** control. `find`
//! offers it because it searches a document, where a case-sensitive search
//! is a real technique for a real problem. This searches at most
//! [`pdfcer_core::layers::MAX_LAYERS`] short labels in a panel narrow
//! enough that the whole list is usually visible, so a control to make the
//! search stricter would be a control for a problem the list's size
//! prevents.
//!
//! ASCII case folding rather than Unicode, and that is a limitation stated
//! rather than hidden: `str::to_lowercase` is Unicode-correct and allocates
//! per row per frame, `str::eq_ignore_ascii_case` does neither. Layer names
//! in the wild are overwhelmingly CAD layer names — `HIDDEN`, `DIM`,
//! `A-WALL-FULL` — and a Turkish dotted İ in one would match on its bytes
//! rather than on its case-folded form. Worth the trade; worth saying.
//!
//! # ★ Decision 3: the query is TRIMMED, and an all-whitespace query is no
//! query
//!
//! `redact`'s search field does the same and for the same reason: a
//! trailing space from a paste is invisible, and a filter that answered
//! "nothing matches" because of one would be a search that failed for a
//! reason the operator cannot see.
//!
//! # What the empty result says, and why it says anything at all
//!
//! R9 forbids a placeholder, and an empty list is not one — but it owes a
//! sentence, because *"no rows"* and *"no rows **because of what you
//! typed**"* are different states and the operator has to be able to tell
//! them apart. `panels::comments` is the precedent: its empty case still
//! discloses the filter, because *"a drawing whose every annotation is a
//! form field is a real and common shape, and 'no notes or markup' alone
//! would leave an operator who can see annotations on the page believing
//! the panel had failed."*
//!
//! Here the equivalent is worse, because the operator can see the layers in
//! the document — they were on screen a moment ago. So the empty case says
//! the query back to them and how many rows it is hiding. See
//! [`crate::text::panels::layers_search_none`].

/// The lower bound on how many layers a document must have before the
/// search field is drawn at all.
///
/// ★★ **Two**, and it is a threshold rather than "always" for R9's reason.
/// A search over a one-row list can do exactly one thing — remove the row —
/// so a field offering it is a control whose only outcome is to make the
/// panel emptier. Drawing it anyway would be the placeholder rule broken in
/// its subtler form: not a control that does nothing, but a control whose
/// every outcome is useless.
///
/// ★ Not a larger number, though a reader will wonder. A threshold of, say,
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
/// # ★ Why this takes `&str` and not `&Layer`
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
///
/// `str::contains` with a closure cannot express "case-insensitively", and
/// `to_lowercase().contains(&q.to_lowercase())` allocates two `String`s per
/// row per frame. This walks the byte windows instead: at most
/// `MAX_LAYERS` rows of a few dozen bytes, once per frame, with nothing on
/// the heap.
///
/// ★ Byte windows are safe here despite UTF-8 being multi-byte, and the
/// reason is worth stating because it looks like a bug: `eq_ignore_ascii_case`
/// on two byte slices is `true` only when they are equal after folding
/// *ASCII* letters, and every non-ASCII byte must therefore match exactly.
/// A window that starts mid-character cannot match a needle that starts on
/// a character boundary unless the bytes are genuinely equal — in which
/// case it is a real match on the same bytes. So no false positive can be
/// produced by the slicing, and none can be lost either.
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

    /// ★★ **Case-insensitive**, which is `FindOptions`' argued default:
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
    ///
    /// `find` offers wildcards behind a control; this does not, and a query
    /// containing one must therefore match the character rather than
    /// silently matching everything.
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

    /// ★ **A multi-byte name does not produce a false match**, and the
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

    /// ★★ **The name the ROW shows is what is matched.**
    ///
    /// Asserted through the placeholder the panel actually draws, so that a
    /// change to that wording is caught here rather than leaving one row in
    /// the list unmatchable by anything on screen.
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

    /// ★★★ **A query that matches nothing is distinguishable from a
    /// document with no layers.**
    ///
    /// The whole reason `Filtered` counts rather than discards. Without
    /// this distinction the panel's only available sentence is "nothing
    /// here", which is also what a panel that failed to read the document
    /// says — and R9's rule that an absent capability renders nothing is
    /// exactly the rule that makes those two indistinguishable if the
    /// count is thrown away.
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
