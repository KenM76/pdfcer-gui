//! # `app::status::decline::textedit` — the two declines the TEXT CARET raises
//!
//! `OPERATOR_REQUESTS.md` **O127**, defects 2 and 3. Two recording functions,
//! and the argument they share.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline/textedit.md`.

use super::{Declined, LAST};

/// **Record that a paragraph reflow did not happen, and why** —
/// `OPERATOR_REQUESTS.md` **O127**, defect 3.
pub(crate) fn record_reflow(why: crate::text::textedit::ReflowRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::Reflow(why)));
}

/// **Record that Enter could not make a line break here** —
/// `OPERATOR_REQUESTS.md` **O127**, defect 2.
pub(crate) fn record_enter_cannot_split(why: crate::text::textedit::EnterRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::EnterCannotSplit(why)));
}

/// **Record that a key was declined before it reached the draft** —
/// `OPERATOR_REQUESTS.md` **O140/O141**, pre-commit half, 2026-09-09.
///
/// The operator's report was *"if I try to edit the edit is not accepted"*, and
/// [`record_edit_text_refusal`] below answered it at the point the engine
/// refused: at `Ctrl+Enter`, after a whole word had been typed and with that
/// word already gone, because `commit_into` calls `abandon` whether or not the
/// engine accepted. That answer is correct and it arrives too late to be much
/// use.
///
///
/// # Which sentence
///
/// A letter in `RunRepertoire::ambiguous` (`two_ways`) is one the font has
/// and draws with more than one code, so it gets
/// [`EditRefusal::RunDrawsTwoWays`]. Any other absence from
/// `RunRepertoire::accepted` is one of several causes — no glyph for the
/// scalar, the embedded subset's floor, a scalar above the BMP — which the
/// repertoire does not tell apart, so [`EditRefusal::RunCannotTake`] reports
/// the measurement (the letter is not in the set) rather than borrowing
/// `FontLacksTheCharacter`'s cause. All of them share one remedy: another face.
///
/// # `typed: None`, and it is not an omission
///
/// [`crate::panels::properties::refusedchar::record`]'s last argument is *what
/// the operator was trying to write*, and it exists because the commit-time
/// refusal destroys the draft: without it, taking the offer changes the face
/// and leaves the operator to retype the word. **Here the draft is still
/// alive.** He is standing in the text, his caret has not moved, and every
/// character before the refused one is still in the box. Handing the offer a
/// copy of the text to re-apply would make the offer re-write text that has not
/// been lost — the same word twice.
///
/// So this recorder is the *pre*-commit twin of [`record_edit_text_refusal`],
/// and the difference between them is exactly one fact: whether the draft
/// survived. That difference is also the one clause
/// [`EditRefusal::RunCannotTake`]'s sentence carries and its two neighbours
/// cannot — *"nothing you have already typed is lost"* — and it is pinned by a
/// test in `text::editrefusal` so a later edit cannot quietly give it to one of
/// them.
///
/// [`EditRefusal::RunCannotTake`]: crate::text::textedit::EditRefusal::RunCannotTake
/// [`EditRefusal::RunDrawsTwoWays`]: crate::text::textedit::EditRefusal::RunDrawsTwoWays
pub(crate) fn record_key_refused(
    page: usize,
    run: usize,
    character: char,
    two_ways: bool,
    base_font: String,
) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        format!(
            "text-edit-key-declined page={page} run={run} character='{character}' two_ways={} \
             character_font={base_font}",
            u8::from(two_ways)
        )
    });
    record_edit_text(if two_ways {
        crate::text::textedit::EditRefusal::RunDrawsTwoWays(character)
    } else {
        crate::text::textedit::EditRefusal::RunCannotTake(character)
    });
    // O141's offer, raised in the same breath as the sentence — one event, two
    // surfaces, written together here so a build cannot say one without the
    // other. `typed: None`: see the note above.
    crate::panels::properties::refusedchar::record(page, run, character, base_font, two_ways, None);
}

/// **Record that a committed text edit was refused, and which kind of
/// refusal it was** — `OPERATOR_REQUESTS.md` **O140**, 2026-09-05.
fn record_edit_text(why: crate::text::textedit::EditRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::EditText(why)));
}

/// **Classify the engine's refusal of a committed text edit, say it on the
/// trace, and put the sentence in the slot** — the one entry point
/// `app::actions::apply`'s `edit_text` arm calls.
pub(crate) fn record_edit_text_refusal(
    page: usize,
    run: usize,
    one_operator: bool,
    error: &pdfcer_core::text_edit::EditError,
) {
    use pdfcer_core::text_edit::RefusalClass;

    let kind = error.refusal_kind();
    //
    // `NoMatch` and `PinnedSpanNotFound` both arrive as
    // `RefusalKind::NotFound` and mean opposite things — *the text does not
    // begin at your pin* versus *your pin names no operator at all*. This is
    // the only place holding the `EditError`, so it is the only place that can
    // tell them apart, and `EditRefusal::of` takes the answer as a fact.
    //
    // ⚠ Matched on the variant rather than on its `Display`: a message match
    // is prose, and prose is the engine's to reword.
    let stale_pin = matches!(
        error,
        pdfcer_core::text_edit::EditError::PinnedSpanNotFound { .. }
    );
    let missing = missing_character(error);
    let typed = crate::canvas::textedit::last_commit().filter(|c| c.page == page && c.run == run);
    let char_kind = refused_char_kind(error, typed.as_ref());
    let two_ways = matches!(
        char_kind,
        Some(
            crate::text::textedit::RefusedCharacter::TwoGlyphsFor(_)
                | crate::text::textedit::RefusedCharacter::HeldTwoGlyphsFor(_)
        )
    );
    let why = crate::text::textedit::EditRefusal::of(kind, one_operator, char_kind, stale_pin);
    crate::diag::trace(|| {
        //
        //
        //   1. **It contains spaces and a comma**, so a reader splitting on
        //      whitespace gets `character=Some(('q',` and drops the rest.
        //   2. **It is a different shape from the field it must be compared
        //      with.** `panels::properties::refusedchar` emits
        //      `character='q'`. A check asserting *"the offer names the
        //      character the refusal named"* compared a debug-formatted tuple
        //      against `'q'` and reported **THE OFFER DOES NOT NAME THE
        //      CHARACTER** — while quoting, in its own failure message, the
        //      offer naming it. The application was correct throughout.
        //   3. `{:?}` on a domain type makes the trace's vocabulary a
        //      consequence of a Rust derive, so it changes silently when the
        //      type does.
        //
        // ⇒ This is the project's standing finding once more: **a driven
        // failure is a claim about the check too**, and the first run of a
        // check written without the ability to drive it found an emitter
        // defect rather than an application one. The repair is on this side
        // deliberately — loosening the comparison would have left two surfaces
        // describing the same character in two languages.
        //
        // `character` now carries the character alone, in the same `'c'`
        // spelling the offer uses, and the font it was refused for gets its own
        // field. `character=none` when the refusal named none, which is the
        // R-INV-2/3/4 case where the encoding itself is unreadable.
        // ui-text-exempt: diagnostic trace, never displayed.
        // `said` is `EditRefusal::name`, NOT `{why:?}` — see that function.
        // A payload added to a variant changed this field's spelling once, and
        // two driven checks went red against a build that was working.
        let said = why.name();
        let (character, character_font) = match &missing {
            Some((c, font)) => (format!("'{c}'"), font.clone()),
            None => ("none".to_owned(), "none".to_owned()),
        };
        //
        // A bare `0`/`1`, never `{:?}` on the `bool` — the same rule that
        // banned a debug-formatted `Option` from this line in the first place.

        format!(
            "edit-text-classified page={page} run={run} kind={kind:?} \
             one_operator={one_operator} stale_pin={} character={character} \
             character_font={character_font} said={said}",
            u8::from(stale_pin)
        )
    });
    record_edit_text(why);
    offer_workaround(page, run, error);
    // **O141 — the offer, raised in the same breath as the sentence.**
    //
    // One event, two surfaces: the bar says *what stopped* and names where the
    // answer is; `panels::properties::refusedchar` names the character and
    // holds the control. They are written together here so a build cannot say
    // one without the other — which is the state O141 was filed about, where
    // the engine, the refusal, the character and the chooser all existed and
    // nothing joined them.
    if let Some((character, base_font)) = missing {
        //
        // Read from `canvas::textedit::last_commit` rather than passed in,
        // and that module's [`Committing`] doc carries the whole argument: the
        // one call site of this function is inside `vector_edit`'s closure in
        // `app::actions::apply`, where nothing but the session and the error is
        // in scope — and *what this edit is trying to write* is a fact about the
        // edit, owned by the function that planned it, not by the router that
        // dispatched it.
        //
        // [`Committing`]: crate::canvas::textedit::Committing
        crate::panels::properties::refusedchar::record(
            page, run, character, base_font, two_ways, typed,
        );
    }
}

/// **The character the engine could not encode, and the face it was refused
/// against** — or `None` when the refusal was not about one character.
fn missing_character(error: &pdfcer_core::text_edit::EditError) -> Option<(char, String)> {
    match error {
        pdfcer_core::text_edit::EditError::Refused(refusal) => {
            Some((refusal.character?, refusal.base_font.clone()))
        }
        _ => None,
    }
}

/// Which character-level refusal this is. An ambiguous character the commit
/// did not add is one the text already held. `edit_text` keeps a held letter's
/// code, so this arises only where it cannot tell which code belongs to which
/// letter: the matched text occurs more than once in its show operator, or a
/// code shows a ligature.
fn refused_char_kind(
    error: &pdfcer_core::text_edit::EditError,
    typed: Option<&crate::canvas::textedit::Committing>,
) -> Option<crate::text::textedit::RefusedCharacter> {
    use crate::text::textedit::RefusedCharacter;
    use pdfcer_core::text_edit::RInvTrigger;

    let pdfcer_core::text_edit::EditError::Refused(refusal) = error else {
        return None;
    };
    let c = refusal.character?;
    Some(if refusal.trigger == RInvTrigger::Ambiguous {
        if typed.is_none_or(|t| RefusedCharacter::added(c, &t.original, &t.replacement)) {
            RefusedCharacter::TwoGlyphsFor(c)
        } else {
            RefusedCharacter::HeldTwoGlyphsFor(c)
        }
    } else {
        RefusedCharacter::NotInTheFont(c)
    })
}

/// Raise the Properties panel's offer when the engine names a workaround for
/// this refusal, or say the workaround itself was refused.
fn offer_workaround(page: usize, run: usize, error: &pdfcer_core::text_edit::EditError) {
    use crate::panels::properties::workaround;
    if let pdfcer_core::text_edit::EditError::WorkaroundRefused { why, .. } = error {
        workaround::record_failed(why.clone());
        return;
    }
    let Some(offered) = error.workaround() else {
        return;
    };
    let typed = crate::canvas::textedit::last_commit().filter(|c| c.page == page && c.run == run);
    if let Some(typed) = typed {
        workaround::record(typed, offered);
    }
}
