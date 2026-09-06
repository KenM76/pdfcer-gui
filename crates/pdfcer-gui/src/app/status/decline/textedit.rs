//! # `app::status::decline::textedit` — the two declines the TEXT CARET raises
//!
//! `OPERATOR_REQUESTS.md` **O127**, defects 2 and 3. Two recording functions,
//! and the argument they share.
//!
//! ## Why it is its own file
//!
//! **R2.** [`super`] crossed 1,500 lines the day these two arrived — the second
//! split from it, after `decline/floor.rs` — and the seam is a real one rather
//! than a size-driven cut. Everything else in that file answers *"what is a
//! decline, and how long does it owe its sentence?"*; this answers *"what does
//! the text caret decline, and who says so?"*, which is a subject with two
//! call-site families and an argument about **channels** that nothing else on
//! that surface shares.
//!
//! ## ★★★ The argument, once, for both: a sentence in the wrong slot is silence
//!
//! Every cause below was **already being reported** before O127. Four of the
//! reflow refusals went through `crate::app::actions::record_note`, which the
//! bar draws under **`⚑ About your last edit:`**; four more collapsed into
//! [`super::Declined::EditRefused`]'s nine cause-free words; and Enter in an
//! existing run said nothing at all, because it quietly committed instead.
//!
//! The operator's verdict on the first family was *"I haven't seen the reflow
//! option actually work with anything when I press it."* **He was answered
//! every single time.** In a slot whose whole contract is *an edit happened;
//! here is the part you cannot see* — for a press where nothing happened — in
//! the past tense, about an earlier gesture, truncated to 45 % of the bar
//! (`NOTES_WIDTH_FRACTION`).
//!
//! [`super`]'s own header had already ruled on this exact swap, for two other
//! sentences, in these words:
//!
//! > *"an operator who reads 'About your last edit' after a gesture that did
//! > nothing has been told a small lie confidently."*
//!
//! ⇒ So the fix for two of O127's three defects is **not new copy**. It is
//! these two functions, which put existing sentences in the slot that wears
//! `⊗` and means *nothing happened*. That is worth a file of prose because it
//! is the second time this project has proved the same thing: **a control that
//! answers in the wrong place is indistinguishable, from the operator's chair,
//! from a control that does not answer at all.**
//!
//! ## ★★ Written unconditionally, overwriting whatever was live
//!
//! [`super::record_text_style`]'s rule, and it matters more here: reflow is a
//! control an operator presses **twice** when nothing appears to happen. The
//! second press must produce the second press's sentence, and `LAST` is a slot
//! rather than a queue precisely so the most recent answer is the visible one.

use super::{Declined, LAST};

/// **Record that a paragraph reflow did not happen, and why** —
/// `OPERATOR_REQUESTS.md` **O127**, defect 3.
///
/// ★★ Called from **two positions**, which is deliberate rather than untidy:
/// `app::dispatch::text` resolves the caret and can decline before any verb
/// runs, and `app::actions::textstyle` maps the engine's own refusal after one
/// has. Both name a cause the operator can act on, and neither can see the
/// other's — so merging them would mean one of the two speaking for a condition
/// it cannot observe.
///
/// ★ The `textstyle` call site is what makes the engine's half legible at all.
/// It writes through `Result::inspect_err` **inside** the funnel's closure, and
/// the ordering is what makes that work: `vector_edit` takes the decline floor
/// *before* running the closure, and `BeforeTheVerb::refused` fills the slot
/// only `if slot.is_none()` — so this specific sentence survives and the
/// generic *"that change was refused"* stands aside. That is the floor's
/// documented purpose; reflow is the first verb to use it.
pub(crate) fn record_reflow(why: crate::text::textedit::ReflowRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::Reflow(why)));
}

/// **Record that Enter could not make a line break here** —
/// `OPERATOR_REQUESTS.md` **O127**, defect 2.
///
/// ★★ The one decline in this module raised by a **keystroke** rather than by a
/// command or a verb, and it belongs here for exactly the reason the others do:
/// the operator did something, nothing happened, and the slot that says so
/// wears `⊗`. A key that quietly does something else is the same defect class
/// as a button that quietly does nothing — this project's founding one.
///
/// ★ It arrives by `Action`, not by a direct call. `canvas::textedit::keys` is
/// outside `crate::app`, and [`super`] is `pub(super)` there on purpose — *"a
/// decline is written by the one dispatcher and read by the one bar"*. So the
/// keystroke raises `TextAction::EnterCannotSplit` and the apply arm calls
/// this. Widening the module's visibility so a keystroke handler could reach
/// the store would have traded a real invariant for two saved lines.
///
/// ★ The draft is left **alive** afterwards, which matters to the wording as
/// well as to the gesture: the sentence is about the key just pressed, the
/// operator is still in the text it is about, and *"press Ctrl+Enter to finish
/// this edit"* therefore names something they can do right now.
pub(crate) fn record_enter_cannot_split() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::EnterCannotSplit));
}

/// ★★★ **Record that a committed text edit was refused, and which kind of
/// refusal it was** — `OPERATOR_REQUESTS.md` **O140**, 2026-09-05.
///
/// The operator: *"if I try to edit the edit is not accepted."*
///
/// ## ★★ It belongs in this file, and the seam holds
///
/// This module answers *"what does the text caret decline, and who says so?"*
/// and this is the third such decline — the one raised when the caret's own
/// **commit** comes back refused. It shares [`record_reflow`]'s call-site shape
/// exactly: written from **inside** `vector_edit`'s closure, through
/// `Result::inspect_err`, so the funnel's decline floor lets it stand and the
/// generic *"That change was refused"* stands aside.
///
/// ★ [`record_reflow`]'s note on the ordering is the whole mechanism and is not
/// repeated here: `vector_edit` takes the floor **before** running the closure,
/// and `BeforeTheVerb::refused` fills the slot only `if slot.is_none()`. Reflow
/// was the first verb to use that; this is the second, and the second is what
/// turns a one-off into the documented route for any verb that learns to
/// classify its own refusal.
///
/// ## ★★★ Why this is a *narrower* decline than the one it sits beside
///
/// [`Declined::EditRefused`] — the funnel's own floor — is still there and is
/// still what every other verb in the shell falls back to. What is different
/// about `edit_text` is that its error type can now be **classified**:
/// `pdfcer_core::text_edit::RefusalKind` is a coarse, stable, exhaustively
/// matchable discriminant, and `crate::text::textedit::EditRefusal::of` joins
/// it with the one fact the engine cannot see — whether the run the shell
/// pinned was a single show operator.
///
/// ⇒ So this recorder is not "a better `EditRefused`". It is what a verb writes
/// when it has actually *understood* the refusal, and the difference is visible
/// in the sentence: one says the document is unchanged, the other says why it
/// is and whether anything can be done.
///
/// ## ★ Written unconditionally, overwriting whatever was live
///
/// [`super::record_text_style`]'s rule, for the reason this file's header
/// gives: an operator who commits twice must see the second commit's answer,
/// and `LAST` is a slot rather than a queue precisely so the most recent one is
/// the visible one.
fn record_edit_text(why: crate::text::textedit::EditRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::EditText(why)));
}

/// **Classify the engine's refusal of a committed text edit, say it on the
/// trace, and put the sentence in the slot** — the one entry point
/// `app::actions::apply`'s `edit_text` arm calls.
///
/// # Why the CLASSIFICATION is here and not at the call site
///
/// Two reasons, and the second is the one that decided it.
///
/// **R2.** `app/actions/apply.rs` is a router sitting a handful of lines under
/// the 1,500 ceiling. The arm routes; it does not decide — the rule every other
/// arm in that file follows — and the judgement below is a decision.
///
/// **The decline module is `pub(super)` of `app::status` on purpose**, and that
/// invariant is what ruled out the tidier-looking home. This was first written
/// into `canvas::textedit::report`, beside `trace_target`, on the argument that
/// a refusal is the failure half of *"what an edit report is worth telling
/// anyone"*. That argument is good and it does not survive contact with the
/// visibility: `canvas::` is outside `crate::app`, so it cannot reach `LAST`.
/// This module's own header records the same collision for the Enter keystroke
/// and the same resolution — *"widening the module's visibility so a keystroke
/// handler could reach the store would have traded a real invariant for two
/// saved lines."* A decline is written by the one dispatcher and read by the
/// one bar, and that is worth more than where these lines sit.
///
/// # What it refuses to do, and why the refusals matter
///
/// `crate::text::status::edit_declined_by_engine`'s documentation named two
/// shortcuts and forbade both:
///
/// - **matching on `EditError`'s variants** — a second copy of `pdfcer-core`'s
///   taxonomy living in this crate, which drifts and then tells the operator
///   the *wrong* reason, strictly worse than the silence it replaced;
/// - **grepping its `Display` string** — prose that is theirs to reword, which
///   `check-ui-strings.sh`'s exclusion 3 rules out in as many words.
///
/// It also said the generic sentence was *"written to be deleted"* the day
/// `EditError` gained a coarse kind. **It has one.**
/// `pdfcer_core::text_edit::RefusalKind` shipped at `b1033ab` in direct answer
/// to this project's 2026-09-04 request, and is deliberately **not**
/// `#[non_exhaustive]` so the match is proved complete by the compiler. It had
/// never been consumed here, because both engine-watching gates are keyed on
/// `EditSession`'s **verbs** and a new *type* is invisible to
/// `check-verb-coverage.sh` and `check-engine-backlog.sh` alike.
///
/// # `one_operator` is the fact the engine cannot have
///
/// [`crate::text::textedit::EditRefusal::of`] owns the joining rule and
/// [`crate::canvas::textedit::Plan::one_operator`] owns the measurement. In one
/// line: `RefusalKind::NotFound` is the engine's honest answer for a request
/// naming text that no single editable run contains, and on the operator's own
/// document that is true because **the producer wrote the line one glyph per
/// show operator** — not because anything is missing or has moved. Only this
/// shell can know that, because only this shell rebuilt the `find` from a run
/// it had segmented itself.
///
/// # Two audiences, two lines, one event
///
/// The trace line here carries the category; `funnel::vector_edit`'s error arm
/// carries the engine's own diagnostic prose unchanged, one line later. Whoever
/// is reading `PDFCER_DIAG` wants the clause number; the operator wants to know
/// their document is intact.
///
/// `said=` is traced beside `kind=`, and it is not redundant: a build that read
/// the category correctly and then chose the wrong sentence is the regression
/// that matters most here, and `kind=` alone cannot distinguish it.
/// `tools/ui-verify`'s `a_refused_typo_fix_says_why_it_was_refused` cross-checks
/// `said` against the independent `edit-text-pin` measurement and fails when the
/// two disagree.
pub(crate) fn record_edit_text_refusal(
    page: usize,
    run: usize,
    one_operator: bool,
    occurrences: Option<usize>,
    error: &pdfcer_core::text_edit::EditError,
) {
    use pdfcer_core::text_edit::RefusalClass;

    let kind = error.refusal_kind();
    let missing = missing_character(error);
    // ★★ The character itself, not a predicate over it — `EditRefusal::of`
    // took a `bool` until 2026-09-05, which meant the classification knew that
    // *a* character had been refused and the sentence could not say which. The
    // datum was already in hand here; only the status bar's return type stopped
    // it reaching the operator, and that is now a `Cow`.
    // ★★ Pass 256.1: the CASE, not just the character. `refused_char_kind`
    // reads `Refusal::trigger` to tell "the font has no such letter" from "the
    // font has two of them", which arrive as the same `RefusalKind` and need
    // opposite sentences. `missing` is still what the offer is keyed on, because
    // the remedy — change the face — is the same for both.
    let why = crate::text::textedit::EditRefusal::of(
        kind,
        one_operator,
        refused_char_kind(error),
        occurrences,
    );
    crate::diag::trace(|| {
        // ★★★ **FLAT FIELDS, NOT `{:?}` ON A TUPLE — corrected 2026-09-05, by
        // the first driven run this line was ever subjected to.**
        //
        // It used to emit `character={missing:?}`, which for
        // `Option<(char, String)>` renders as `character=Some(('q',` followed
        // by a space, the quoted base-font name, and two closing brackets.
        // Three things are wrong with that in a `key=value` trace:
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
        // ★★ `said` is `EditRefusal::name`, NOT `{why:?}` — see that function.
        // A payload added to a variant changed this field's spelling once, and
        // two driven checks went red against a build that was working.
        let said = why.name();
        let (character, character_font) = match &missing {
            Some((c, font)) => (format!("'{c}'"), font.clone()),
            None => ("none".to_owned(), "none".to_owned()),
        };
        // ★ A bare number, or `none` — never a debug-formatted `Option`, for
        // the three reasons numbered above. Same field name and same spelling
        // as `edit-text-pin`'s, so a check reading both lines compares like
        // with like rather than parsing two dialects of one datum.
        let occurrences = occurrences.map_or_else(|| "none".to_owned(), |n| n.to_string());
        format!(
            "edit-text-classified page={page} run={run} kind={kind:?} \
             one_operator={one_operator} occurrences={occurrences} character={character} \
             character_font={character_font} said={said}"
        )
    });
    record_edit_text(why);
    // ★★★ **O141 — the offer, raised in the same breath as the sentence.**
    //
    // One event, two surfaces: the bar says *what stopped* and names where the
    // answer is; `panels::properties::refusedchar` names the character and
    // holds the control. They are written together here so a build cannot say
    // one without the other — which is the state O141 was filed about, where
    // the engine, the refusal, the character and the chooser all existed and
    // nothing joined them.
    if let Some((character, base_font)) = missing {
        // ★★★ **The words the operator typed travel with the refusal** — O141's
        // second half, 2026-09-05. Without them the offer can change the face
        // and cannot finish the job: `Ctrl+Enter` calls `commit_into` and then
        // `abandon` whether or not the engine accepted, so the draft is gone by
        // the time the block draws, and taking the offer used to leave the
        // operator to click back into the text and type the character a second
        // time.
        //
        // ★ Read from `canvas::textedit::last_commit` rather than passed in,
        // and that module's [`Committing`] doc carries the whole argument: the
        // one call site of this function is inside `vector_edit`'s closure in
        // `app::actions::apply`, where nothing but the session and the error is
        // in scope — and *what this edit is trying to write* is a fact about the
        // edit, owned by the function that planned it, not by the router that
        // dispatched it.
        //
        // [`Committing`]: crate::canvas::textedit::Committing
        let typed =
            crate::canvas::textedit::last_commit().filter(|c| c.page == page && c.run == run);
        crate::panels::properties::refusedchar::record(page, run, character, base_font, typed);
    }
}

/// **The character the engine could not encode, and the face it was refused
/// against** — or `None` when the refusal was not about one character.
///
/// # ★★★ Why reading a variant is licensed here, when this module's own header
/// forbids it
///
/// The header forbids *"matching on `EditError`'s variants"* to derive **the
/// operator's reason** — *"a second copy of `pdfcer-core`'s taxonomy living in
/// this crate, which drifts and then tells the operator the wrong reason"*. That
/// prohibition stands and is honoured above: the category still comes from
/// `RefusalKind`, exhaustively, and [`crate::text::textedit::EditRefusal::of`]
/// owns the joining rule.
///
/// What happens here is a different act. `RefusalKind` is deliberately coarse —
/// four buckets, *"stable enough to match exhaustively"* — and `Refusal` carries
/// **fields the bucket structurally cannot hold**. This reads two of them and
/// derives nothing: no trigger id is inspected, no `Display` prose is grepped
/// (exclusion 3 of `check-ui-strings.sh` rules that out in as many words), and
/// no sentence is chosen from what is read. It is the identical licence
/// `one_operator` already has — the sentence needs a fact the category does not
/// carry — with the difference that this fact is the engine's own and is simply
/// being passed through.
///
/// ★ The `_` arm is not laziness: `EditError` is `#[non_exhaustive]`, so an
/// error the engine adds later must land somewhere honest, and *"this refusal
/// was not about one character"* is true of every error that is not
/// `Refused`. Getting that wrong in the other direction — inventing a character
/// — would put a chooser in front of an operator whose font cannot be
/// re-encoded at all, and every row in it would refuse.
///
/// # What `None` means, and it is a real answer
///
/// `Refusal::character` is `None` for the font-classification triggers —
/// R-INV-2 (symbolic, built-in cmap), R-INV-3 (`/ToUnicode` only) and R-INV-4.
/// Those are refusals about the font's whole code↔glyph relation rather than
/// about one scalar, **no other face makes this run re-encodable**, and the
/// existing `EditRefusal::UnsupportedFont` sentence is the true one for them.
/// That is why the offer is keyed on the field being `Some` rather than on the
/// kind being `UnsupportedFont`.
fn missing_character(error: &pdfcer_core::text_edit::EditError) -> Option<(char, String)> {
    match error {
        pdfcer_core::text_edit::EditError::Refused(refusal) => {
            Some((refusal.character?, refusal.base_font.clone()))
        }
        _ => None,
    }
}

/// **Which of the two character-level refusals this is** — `Pass 256.1`,
/// consumed 2026-09-06.
///
/// # ★★★ Why the shell reads `trigger` here when it reads no other trigger id
///
/// `app::status::decline::textedit`'s own header forbids building a second copy
/// of the engine's taxonomy, and grepping `Display` prose for a cause is exactly
/// what it forbids. This is neither. It reads **one field of a structured
/// refusal** — the same licence `Refusal::character` is read on, and for the
/// same reason: the coarse `RefusalKind` structurally cannot carry it, and
/// without it the shell says something **false** to the operator.
///
/// `RefusalKind` maps `EditError::Refused(_)` to `UnsupportedFont` wholesale, so
/// these two arrive identically:
///
/// | trigger | what is true of the document | what the operator can do |
/// |---|---|---|
/// | `TargetAbsent` (R-INV-1) | the character is **not in the font** | change the face |
/// | `Ambiguous` (R-INV-5, composite) | the character is in the font **twice** — two CIDs map to it | change the face |
///
/// The remedy is the same, which is why the face offer is raised for both. **The
/// sentence is not.** [`crate::text::textedit::font_lacks_the_character`] says
/// *"The font here was built with only the letters your page already prints"*,
/// and on an ambiguous character that is simply untrue — the letter is there,
/// twice over, and pdfcer is declining to guess which glyph he meant. An
/// operator told the false version would go looking for a missing letter that is
/// on his page in front of him.
///
/// ★ `Pass 256.1`'s own reply names the sentence it wants: *"this letter has two
/// glyphs in this font; pick another font for it"*. That is what
/// [`crate::text::textedit::font_has_two_glyphs_for`] says.
///
/// ⚠ **Test the trigger, not `is_hard()`.** The engine's rustdoc is explicit:
/// on a composite font `Ambiguous` arrives inside a `Refusal`, and a `Refusal`
/// is hard by construction, so `is_hard()` answers the simple-font disposition
/// and is the wrong question here.
fn refused_char_kind(
    error: &pdfcer_core::text_edit::EditError,
) -> Option<crate::text::textedit::RefusedCharacter> {
    use crate::text::textedit::RefusedCharacter;
    use pdfcer_core::text_edit::RInvTrigger;

    let pdfcer_core::text_edit::EditError::Refused(refusal) = error else {
        return None;
    };
    let c = refusal.character?;
    Some(if refusal.trigger == RInvTrigger::Ambiguous {
        RefusedCharacter::TwoGlyphsFor(c)
    } else {
        RefusedCharacter::NotInTheFont(c)
    })
}
