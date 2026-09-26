//! # `text::arrange` — every sentence the three ways of *arranging* something
//! on the page can owe
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/arrange.md`.

/// **The Markup tab's Arrange group caption.**
#[must_use]
pub const fn group_arrange() -> &'static str {
    "Arrange"
}

/// **Why an arrow key moved nothing**, in the shell's reading of the cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudgeRefusal {
    /// §12.5.3 Table 165 bit 8. The document says the interface may not change
    /// this annotation, and pdfcer honours it rather than letting the engine
    /// refuse — the same ruling `pdfcer_gui::canvas::annotdrag` makes for the
    /// pointer, so a key and a drag cannot disagree about what locked means.
    Locked,
    /// Something is selected on the page and it is **not a markup annotation** —
    /// a line of the drawing, a text run, an image.
    ///
    /// See [`not_a_markup`] for why this is worded as a limit of today's build
    /// rather than as a rule of the program.
    NotAMarkup,
    /// A **ce dimension** is selected (rule 15). It is markup, it is selected,
    /// and it is refused — because moving one has to re-measure it, which is a
    /// different verb.
    Dimension,
    /// The page's device transform will not invert, so there is no honest way
    /// to turn a keystroke into a displacement in the page's own units.
    ///
    /// The same condition under which both halves of the `viewer` bridge
    /// decline and under which a pointer drag refuses
    /// (`pdfcer_gui::canvas::moving::page_delta` answers `None`). A page like this
    /// cannot be drawn correctly either, so the operator is already looking at
    /// something wrong; the sentence exists so the arrow key is not the second
    /// mystery.
    DegeneratePage,
}

/// The sentence for a refused nudge, or `None` when the case is one this
/// catalog deliberately leaves silent.
#[must_use]
pub const fn nudge_refusal(why: NudgeRefusal) -> Option<&'static str> {
    Some(match why {
        NudgeRefusal::Locked => locked_cannot_move(),
        NudgeRefusal::NotAMarkup => not_a_markup(),
        NudgeRefusal::Dimension => dimension_use_the_pointer(),
        NudgeRefusal::DegeneratePage => degenerate_page(),
    })
}

/// **The mark is locked, so it will not move.**
#[must_use]
pub const fn locked_cannot_move() -> &'static str {
    "This mark is locked by the document, so it cannot be moved — by the arrow keys or by \
     dragging it."
}

/// **The arrow keys move a mark, and what is selected is not one.**
///
/// # Worded as *today*, and the reason is that it is true and will change
///
/// `pdfcer_gui::canvas::moving` moves page content on a drag through five verbs, and
/// none of them is out of reach of a keystroke in principle. What is missing is
/// `pdfcer_gui::canvas::modelneed`: the deeper rungs need the page's
/// decomposition, a frame only builds one when something has said it needs one,
/// and *"an arrow key was pressed"* is not yet one of the terms. A nudge that
/// reached those rungs without that term would be the `NoObjectModel` defect
/// this project has now shipped four times — a working verb, silently
/// unreachable, reported as a limit of the document.
///
/// ⇒ So this says *"drag it"*, which works today on everything the arrow keys
/// do not, rather than *"that cannot be moved"*, which would be false.
#[must_use]
pub const fn not_a_markup() -> &'static str {
    "The arrow keys nudge a selected markup. Drag this with the pointer instead."
}

/// **A ce dimension is selected**, and the arrow keys will not move it.
#[must_use]
pub const fn dimension_use_the_pointer() -> &'static str {
    "This is a measurement, and moving one re-measures it. Drag it with the pointer so the \
     value keeps up."
}

/// **The page's geometry will not invert.**
#[must_use]
pub const fn degenerate_page() -> &'static str {
    "pdfcer cannot work out this page's geometry, so nothing on it can be nudged. Other pages \
     are unaffected."
}

// ===========================================================================
// The pointer drag — the one refusal out of eleven that had nothing to say
// ===========================================================================

/// **A drag on one line inside a block of text, refused** — `OPERATOR_REQUESTS.md`
/// O188, 2026-09-15.
#[must_use]
pub const fn run_has_no_position_of_its_own() -> &'static str {
    "That line takes its position from the line before it, so this document has no position \
     for it that pdfcer could change — press Escape to select the whole block of text and \
     drag that."
}

/// **A drag on a line that the NEXT line's position is measured from** — the
/// second of O188's two refusals.
#[must_use]
pub const fn run_would_drag_the_next_line() -> &'static str {
    "The line after this one takes its position from this one, so moving it would drag that \
     line along too — press Escape to select the whole block of text and drag that."
}

/// Said when a line's pieces could not be folded into one undo entry.
#[must_use]
pub fn line_takes_several_undo_presses(pieces: usize) -> String {
    format!(
        "That line is written in {pieces} pieces, so taking this back needs \
         {pieces} presses of Undo rather than one."
    )
}

// ===========================================================================
// Z-order — what an Arrange command has to disclose
// ===========================================================================

/// **The mark was already where the command would have put it.**
#[must_use]
pub const fn already_there(front: bool) -> &'static str {
    if front {
        "This mark was already in front of everything else on the page."
    } else {
        "This mark was already behind everything else on the page."
    }
}

/// **The mark is locked, so pdfcer leaves its depth alone.**
#[must_use]
pub const fn locked_cannot_arrange() -> &'static str {
    "This mark is locked by the document, so pdfcer leaves its place in the drawing order \
     alone."
}

/// **Some of this page's annotations cannot be reordered, and stayed put.**
#[must_use]
pub fn pinned(count: usize) -> String {
    if count == 1 {
        "One annotation on this page is written into the page itself rather than as a \
         separate object, so it has no name to be reordered by and has stayed where it was — \
         this mark may still be behind it."
            .to_owned()
    } else {
        format!(
            "{count} annotations on this page are written into the page itself rather than as \
             separate objects, so they have no name to be reordered by and have stayed where \
             they were — this mark may still be behind them."
        )
    }
}

/// **Form fields on this page changed their tab order.**
#[must_use]
pub fn tab_order_changed(count: usize) -> String {
    if count == 1 {
        "One form field on this page changed its place in the tab order, because the drawing \
         order and the tab order are the same list."
            .to_owned()
    } else {
        format!(
            "{count} form fields on this page changed their place in the tab order, because \
             the drawing order and the tab order are the same list."
        )
    }
}

/// **The page shared its annotation list, and pdfcer copied it first.**
#[must_use]
pub const fn copied_shared_list() -> &'static str {
    "This page shared its list of annotations with another page. pdfcer copied the list first, \
     so the other page's drawing order is unchanged."
}

/// **A trap network is on this page and has to stay last.**
#[must_use]
pub const fn trap_net_stays_last() -> &'static str {
    "A prepress trap network on this page must stay last of all, so this mark is now in front \
     of everything except that."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every refusal has a sentence, and no two of them are the same one.**
    #[test]
    fn every_nudge_refusal_has_its_own_sentence() {
        let all = [
            NudgeRefusal::Locked,
            NudgeRefusal::NotAMarkup,
            NudgeRefusal::Dimension,
            NudgeRefusal::DegeneratePage,
        ];
        let mut sentences: Vec<&str> = all
            .iter()
            .map(|why| nudge_refusal(*why).expect("every variant speaks today"))
            .collect();
        let total = sentences.len();
        sentences.sort_unstable();
        sentences.dedup();
        assert_eq!(
            sentences.len(),
            total,
            "two refusals share a sentence — one of them is telling the operator about a \
             state they are not in"
        );
    }

    /// **No sentence here names a thing the operator cannot see.**
    #[test]
    fn no_sentence_speaks_in_the_file_formats_vocabulary() {
        let sentences = [
            locked_cannot_move(),
            not_a_markup(),
            dimension_use_the_pointer(),
            degenerate_page(),
            already_there(true),
            already_there(false),
            locked_cannot_arrange(),
            copied_shared_list(),
            trap_net_stays_last(),
            pinned(1).leak(),
            pinned(4).leak(),
            tab_order_changed(1).leak(),
            tab_order_changed(4).leak(),
        ];
        for text in sentences {
            for probe in [
                "/Annots",
                "ObjId",
                "indirect",
                "dictionary",
                "subtype",
                "/Rect",
                "matrix",
                "permutation",
                "/F ",
                "flag",
            ] {
                assert!(
                    !text.contains(probe),
                    "`{probe}` is the file format's vocabulary, not the operator's: {text:?}"
                );
            }
        }
    }

    /// **The two ends of the pair read differently.**
    #[test]
    fn front_and_back_are_not_the_same_sentence() {
        assert_ne!(already_there(true), already_there(false));
        assert!(already_there(true).contains("in front"));
        assert!(already_there(false).contains("behind"));
    }

    /// **The counted sentences agree in number.**
    #[test]
    fn the_counted_sentences_agree_in_number() {
        assert!(pinned(1).starts_with("One annotation "));
        assert!(pinned(3).starts_with("3 annotations "));
        assert!(tab_order_changed(1).starts_with("One form field "));
        assert!(tab_order_changed(2).starts_with("2 form fields "));
    }

    /// **A refusal blames the document, never the operator and never pdfcer.**
    ///
    /// The lock sentence is the one that could most easily read as a program
    /// failure, and it is the one an operator meets on somebody else's drawing.
    #[test]
    fn the_lock_sentences_name_the_document() {
        for text in [locked_cannot_move(), locked_cannot_arrange()] {
            assert!(text.contains("locked by the document"));
            for alarm in ["error", "failed", "cannot be done", "unsupported"] {
                assert!(!text.to_lowercase().contains(alarm), "{text}");
            }
        }
        assert_ne!(
            locked_cannot_move(),
            locked_cannot_arrange(),
            "the two refuse different things — an operator who pressed Send to back and read \
             'it cannot be moved' would think the program had misheard them"
        );
    }
}
