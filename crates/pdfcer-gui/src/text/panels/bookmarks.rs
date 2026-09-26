//! # `text::panels::bookmarks` — the words for **moving** a bookmark and for
//! **expanding or collapsing** one
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/panels/bookmarks.md`.

// ---------------------------------------------------------------------------
// Expanding and collapsing — `EditSession::set_outline_open`
// ---------------------------------------------------------------------------

/// The glyph on a row whose children are **hidden** — press to reveal them.
#[must_use]
pub const fn bookmark_collapsed_glyph() -> &'static str {
    "\u{23f5}"
}

/// The glyph on a row whose children are **showing** — press to hide them.
#[must_use]
pub const fn bookmark_expanded_glyph() -> &'static str {
    "\u{23f7}"
}

/// Hover text on the triangle of a **collapsed** row.
#[must_use]
pub const fn bookmark_expand_tooltip() -> &'static str {
    "Show the bookmarks filed under this one. Whether a bookmark is open or \
     closed is stored in the document, so this is a change you can undo."
}

/// Hover text on the triangle of an **expanded** row.
#[must_use]
pub const fn bookmark_collapse_tooltip() -> &'static str {
    "Hide the bookmarks filed under this one. They stay in the document; the \
     list stops showing them. Whether a bookmark is open or closed is stored \
     in the document, so this is a change you can undo."
}

// ---------------------------------------------------------------------------
// Moving one — `EditSession::move_outline_item`
// ---------------------------------------------------------------------------

/// The standing hint that says the rows can be dragged.
#[must_use]
pub const fn bookmark_drag_hint() -> &'static str {
    "Drag a bookmark to move it. Dropping on the top or bottom edge of another \
     bookmark puts it beside that one; dropping in the middle files it inside. \
     Whatever is filed under it comes with it."
}

/// **What the move did**, said after the press from the engine's own report.
#[must_use]
pub fn bookmark_moved(visible_items: usize, reparented: bool) -> String {
    match (visible_items, reparented) {
        (0 | 1, false) => "Bookmark moved.".to_owned(),
        (0 | 1, true) => "Bookmark moved, and it now sits under a different one.".to_owned(),
        (n, false) => format!(
            "Bookmark moved, and the {} shown under it moved with it.",
            n - 1
        ),
        (n, true) => format!(
            "Bookmark moved under a different one, and the {} shown under it went too.",
            n - 1
        ),
    }
}

/// **The subtree that travelled and was never counted**, because it was
/// collapsed.
#[must_use]
pub fn bookmark_move_took_hidden(descendants: usize) -> String {
    if descendants == 1 {
        "It was collapsed, so the 1 bookmark hidden inside it moved as well.".to_owned()
    } else {
        format!("It was collapsed, so the {descendants} bookmarks hidden inside it moved as well.")
    }
}

/// **The bookmark landed somewhere it cannot be seen**, and the panel is
/// right to show it that way.
#[must_use]
pub const fn bookmark_move_into_collapsed() -> &'static str {
    "Its new parent is collapsed, so the bookmark will not show in the list \
     until you open that parent with its triangle. It is in the document."
}

/// **The move was asked for and changed nothing**, because the bookmark was
/// already there.
#[must_use]
pub const fn bookmark_move_no_change() -> &'static str {
    "That bookmark was already in that place, so nothing changed and there is \
     nothing to undo."
}

/// **A bookmark cannot be filed inside itself** — the decline for a drop that
/// landed on the dragged row or somewhere in its own subtree.
#[must_use]
pub const fn bookmark_move_declined_own_subtree() -> &'static str {
    "A bookmark cannot be filed inside itself, or inside anything filed under \
     it, so nothing moved. Drop it on a bookmark outside that branch."
}

/// **The engine refused the move** — the residue the shell's own forecast
/// cannot cover.
#[must_use]
pub const fn bookmark_move_declined_engine() -> &'static str {
    "That bookmark was not moved \u{2014} pdfcer declined the change, and the \
     outline is exactly as it was. The document may be encrypted or signed in \
     a way that forbids changing it."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The move disclosure never says "0 bookmarks".**
    #[test]
    fn the_move_disclosure_never_reads_as_a_template() {
        for reparented in [false, true] {
            for visible in [0usize, 1] {
                let said = bookmark_moved(visible, reparented);
                assert!(!said.contains('0'), "{said}");
                assert!(!said.contains(" 1 "), "{said}");
            }
        }
        // And with a real subtree it names the number, one lower than the
        // engine's count because that count includes the bookmark itself.
        assert!(bookmark_moved(8, false).contains('7'));
        assert!(bookmark_moved(8, true).contains('7'));
    }

    /// **Re-parenting and reordering do not read the same.**
    #[test]
    fn a_reparent_and_a_reorder_are_worded_differently() {
        for n in [1usize, 2, 9] {
            assert_ne!(bookmark_moved(n, false), bookmark_moved(n, true));
        }
    }

    /// **The hidden-subtree sentence is about the branch, not about the
    /// screen**, and it is the one place the two `/Count` quantities are
    /// visibly different.
    #[test]
    fn the_hidden_subtree_sentence_names_the_branch_size() {
        let moved = bookmark_moved(1, true);
        let hidden = bookmark_move_took_hidden(40);
        assert!(hidden.contains("40"), "{hidden}");
        assert!(hidden.contains("collapsed"), "{hidden}");
        assert!(
            !moved.contains("40"),
            "the engine's count knows nothing about the hidden branch: {moved}"
        );
        // One is not spelled as a plural.
        assert!(bookmark_move_took_hidden(1).contains("1 bookmark hidden"));
    }

    /// **The collapsed-destination sentence names the remedy and does not
    /// read as a failure.**
    #[test]
    fn the_collapsed_destination_sentence_matches_the_add_rows_posture() {
        let said = bookmark_move_into_collapsed();
        assert!(said.contains("collapsed"), "{said}");
        assert!(said.contains("triangle"), "the remedy is named: {said}");
        assert!(
            said.contains("in the document"),
            "the move WORKED, and the sentence must not read as a failure: {said}"
        );
        // The add row's sentence, on the same two counts.
        let add = super::super::bookmark_add_under_collapsed();
        assert!(add.contains("collapsed"), "{add}");
        assert!(add.contains("still be in the file"), "{add}");
    }

    /// **Neither decline names a bookmark**, which is what keeps
    /// `Declined` `Copy`.
    #[test]
    fn the_declines_carry_no_title() {
        let own: &'static str = bookmark_move_declined_own_subtree();
        let engine: &'static str = bookmark_move_declined_engine();
        assert_ne!(own, engine, "two moments, two remedies, two sentences");
        // Each says that nothing happened, which is the whole speech act.
        assert!(own.contains("nothing moved"), "{own}");
        assert!(own.contains("Drop it"), "the remedy is named: {own}");
        assert!(engine.contains("exactly as it was"), "{engine}");
    }

    /// **The two triangles differ**, and both are one character.
    #[test]
    fn the_disclosure_triangles_are_two_different_glyphs() {
        assert_ne!(bookmark_collapsed_glyph(), bookmark_expanded_glyph());
        assert_eq!(bookmark_collapsed_glyph().chars().count(), 1);
        assert_eq!(bookmark_expanded_glyph().chars().count(), 1);
    }

    /// **Both triangle tooltips say the state is stored in the document.**
    #[test]
    fn both_triangle_tooltips_disclose_that_the_state_is_saved() {
        for tip in [bookmark_expand_tooltip(), bookmark_collapse_tooltip()] {
            assert!(tip.contains("stored in the document"), "{tip}");
            assert!(tip.contains("undo"), "{tip}");
        }
        assert_ne!(bookmark_expand_tooltip(), bookmark_collapse_tooltip());
    }

    /// **The drag hint names all three landings.**
    #[test]
    fn the_drag_hint_teaches_the_three_landings() {
        let hint = bookmark_drag_hint();
        assert!(hint.contains("Drag"), "{hint}");
        assert!(hint.contains("edge"), "{hint}");
        assert!(hint.contains("middle"), "{hint}");
        assert!(
            hint.contains("comes with it"),
            "the subtree travels, and the hint is where that is said before any \
             press: {hint}"
        );
    }
}
