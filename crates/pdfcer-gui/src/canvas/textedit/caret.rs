//! # `canvas::textedit::caret` — **where the caret is, and what a key does to
//! the text around it**
//!
//! ## Why this is its own file
//!
//! R2, on 2026-08-20, when the caret pushed `canvas::textedit` past 1,500
//! lines. It is a real seam rather than a convenient cut, and the test for that
//! is the one this project uses everywhere: **everything here is a pure
//! function of a `&str` and an index.** No `egui::Context`, no document, no
//! page, no PDF. The rest of `textedit` is about *placing* a caret on a page
//! and *committing* what was typed into it; this file is about the string.
//!
//! That split is worth having for a second reason. A caret is the most
//! convention-bound object in any editor — every operator alive already knows
//! what Home does, what Ctrl+Left does, and that Delete eats forwards while
//! Backspace eats backwards. Conventions are testable as arithmetic, and a file
//! with no window in it can assert every one of them in microseconds.
//!
//! ## The defect this file is the answer to
//!
//! Until 2026-08-20 there was **no caret index at all**. `insert` extended the
//! end of the draft and `backspace` popped its last character, so the painter
//! drew its line at the right edge of the run's glyph box — because that is the
//! only position an append-only draft has. The operator:
//!
//! > *"the cursor just sits at the end of a text line. It can't be moved to the
//! > center of an existing text block."*
//!
//! Exactly right, and it made editing existing page text nearly useless: a
//! title-block cell reading `SHEET 1 OF 4` could only be changed by deleting it
//! back to `SHEET ` and retyping the rest.
//!
//! ## The invariant every function here maintains
//!
//! ```text
//! caret <= text.chars().count()
//! ```
//!
//! Held by **clamping on entry** rather than by asserting, so a caller that
//! hands over a stale index — one taken before the text was replaced — gets the
//! nearest sensible position instead of a panic. A panic in a caret would take
//! the whole window down over a keystroke, which is a spectacularly bad trade
//! for a class of bug whose worst honest outcome is a cursor one character from
//! where it should be.
//!
//! ## Characters, never bytes
//!
//! Every operation here is expressed in keystrokes and one keystroke is one
//! `char`. A byte index would make Left-arrow over `é` — two bytes — either
//! move half a character or need a decode at every use, and a byte truncation
//! of a multi-byte character is a **panic** in Rust rather than mojibake.
//!
//! The cost is that every operation is O(n) in the draft's length. A draft is
//! one show operator — a table cell, a label, one line of a note — so n is tens
//! of characters, and the alternative is a byte index plus a boundary check at
//! every call site.
//!
//! ## What is deliberately NOT here
//!
//! **A selection.** There is no anchor, no range, no Shift+arrow, no Ctrl+A.
//! That is a second feature with its own drawing, its own replace-the-range
//! semantics in every edit below, and its own interaction with the clipboard —
//! and folding it in half-done would give the operator a highlight that some
//! keys respect and others silently ignore. It is recorded as its own row in
//! `OPERATOR_REQUESTS.md` rather than left as an implied gap.
//!
//! ## conventions: text-caret
//!
//! Corpus: `ui-conventions/text-caret.md`.
//!
//! - T1 live-preview: **GAP, and it is the operator's open complaint** — *"I can
//!   edit text now, but there is no live preview of that either."* The page
//!   renders committed glyphs; the draft lives beside it and nothing draws it,
//!   so the operator sees the old text and a blinking caret. The corpus is
//!   explicit that the approximation is acceptable and the absence is not:
//!   drawing the draft in the shell's own font, scaled to the run, shifts
//!   slightly on commit and is still a preview.
//! - T2 caret-has-a-position: a click lands it at the nearest character
//!   boundary; arrows, Ctrl+arrows, Home and End move it; Backspace and Delete
//!   act either side of it. Added 2026-08-20 — before that there was no index at
//!   all, and the painter drew its line at the right edge because that is the
//!   only position an append-only draft has.
//! - T3 graphemes-not-bytes: **PARTIAL** — characters, not bytes, so `é` takes
//!   one keystroke. Not grapheme clusters, so a combining mark or an emoji
//!   sequence still takes two. `unicode-segmentation` is already in the tree.
//! - T4 clamp-never-assert: every operation clamps on entry. A panic in a caret
//!   would take the whole window down over a keystroke.
//! - T5 composer-owns-the-keyboard: `composing` is the one predicate, and
//!   `tools/gates/check-typing-guard.sh` fails the build on a second copy.
//!   **This row exists because it failed twice** — Delete after a canvas click,
//!   then the space bar, which the pan tool took because this caret is not an
//!   `egui::TextEdit` and egui's own predicate cannot see it.
//! - T6 enter-commits-escape-abandons: both, and a draft identical to what it
//!   replaces raises no action.
//! - T7 no-control-characters: `insert` filters them; Enter and Escape arrive as
//!   key events and mean something.
//! - T8 selection: **GAP** — no Shift+arrow, no Ctrl+A, no drag-select within a
//!   draft. Named rather than left implied, because a highlight that some keys
//!   respect and others silently ignore is worse than none.

/// **Insert `s` at the end of the draft.** The one mutation typing performs.
///
/// A free function over `&mut String` rather than a method, because it is the
/// single point both the keyboard and the diagnostic seam pass through and a
/// method would invite a second caller that skipped it. Control characters are
/// dropped: `egui` delivers Enter and Escape as `Key` events, so a control
/// character arriving in a `Text` event is something this shell has no meaning
/// for, and putting it in a PDF show string would be authoring a byte the
/// operator cannot see.
pub fn insert(text: &mut String, caret: usize, s: &str) -> usize {
    let typed: Vec<char> = s.chars().filter(|c| !c.is_control()).collect();
    if typed.is_empty() {
        return caret;
    }
    splice(text, caret, typed)
}

/// **Insert a line break**, and the only way to get a control character into a
/// draft.
///
/// # ★★★ Why this is its own function rather than `insert(text, caret, "\n")`
///
/// Because [`insert`] **drops control characters**, and it is right to. Its own
/// doc says why: *"`egui` delivers Enter and Escape as `Key` events, so a
/// control character arriving in a `Text` event is something this shell has no
/// meaning for, and putting it in a PDF show string would be authoring a byte
/// the operator cannot see."* That is still true of typed text.
///
/// It stopped being true of the whole draft on 2026-08-21, when a box gained a
/// paragraph break — and the guard silently ate it. **The Enter arrived, the
/// branch was right, `insert` was called, and the newline was filtered out one
/// call deeper.** The driven check reported *"the paragraph was authored as 1
/// line"*; the trace showed the key arriving and the length not moving; and the
/// answer was a filter written for a different question.
///
/// ★ So the filter stays and the newline gets a door of its own. Relaxing
/// `insert` to permit `\n` would have permitted every other control character
/// with it — a stray `\t` or `\r` from a paste would land in a show string —
/// and it would have made *"can a control character be in a draft?"* a question
/// with two answers depending on which caller you asked.
///
/// **This is the fifth guard in two days to expire the week it was written**,
/// and the shape is always the same: a well-argued restriction reads as
/// permanent precisely because it is well argued. See
/// `C:\personal_rag\claude_code\lesson_20260820_a_refusal_is_a_claim_with_a_date_on_it.md`.
pub fn newline(text: &mut String, caret: usize) -> usize {
    splice(text, caret, vec!['\n'])
}

/// Put `chars` into `text` at `caret`, and answer the caret after them.
///
/// The shared body of [`insert`] and [`newline`], so there is one statement of
/// *"caret indices are CHARACTERS, not bytes"* rather than two. `é` is one
/// keystroke and two bytes, and a byte-indexed splice would panic on the next
/// one.
fn splice(text: &mut String, caret: usize, chars_to_add: Vec<char>) -> usize {
    let mut chars: Vec<char> = text.chars().collect();
    let at = caret.min(chars.len());
    let n = chars_to_add.len();
    chars.splice(at..at, chars_to_add);
    *text = chars.into_iter().collect();
    at + n
}

/// **Remove the last character.** Backspace, and the whole of it.
///
/// By `char` and not by byte: a draft holding `é` must lose one keystroke's
/// worth of text per Backspace, and truncating a byte would leave an invalid
/// `String` — which in Rust is a panic rather than mojibake.
pub fn backspace(text: &mut String, caret: usize) -> usize {
    let mut chars: Vec<char> = text.chars().collect();
    let at = caret.min(chars.len());
    if at == 0 {
        // Nothing before the caret. Not an error and not a beep: an operator
        // holding Backspace at the start of a draft means "stop", and the
        // honest response is to stop.
        return 0;
    }
    chars.remove(at - 1);
    *text = chars.into_iter().collect();
    at - 1
}

/// **Remove the character AFTER the caret.** The Delete key.
///
/// The caret does not move, which is what makes Delete different from
/// Backspace rather than a mirror of it: the text to the left of the caret is
/// untouched, so the operator's position in the word is preserved while what
/// follows is eaten.
pub fn delete_forward(text: &mut String, caret: usize) -> usize {
    let mut chars: Vec<char> = text.chars().collect();
    let at = caret.min(chars.len());
    if at >= chars.len() {
        return at;
    }
    chars.remove(at);
    *text = chars.into_iter().collect();
    at
}

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------
//
// ★★ Added 2026-08-21 for `OPERATOR_REQUESTS.md` O14 item 11: *"no selection
// inside a draft — no Shift+arrow, no Ctrl+A, no drag-select."*
//
// The whole of a selection is TWO INDICES AND FOUR RULES, and every one of the
// rules is here rather than in the keystroke handler, because a rule stated at
// a call site is a rule the next call site does not know about:
//
// 1. A selection is the range between the mark and the caret, in either order.
// 2. Typing anything **replaces** it.
// 3. Backspace and Delete **remove** it, and remove nothing else.
// 4. Any movement without Shift **drops** it.
//
// Rule 4 is the one that looks like a detail and is not. Without it a
// highlight stays on screen after the caret has walked out of it, and the next
// keystroke deletes text the operator is no longer looking at.

/// **The selected range**, as normalised character indices, or `None` when
/// nothing is selected.
///
/// An empty range answers `None` rather than `Some((n, n))`: a mark sitting
/// exactly on the caret is *not a selection*, it is the state after
/// Shift+Right followed by Shift+Left, and every caller would otherwise need
/// its own emptiness check before deciding whether Backspace deletes a
/// selection or a character.
#[must_use]
pub fn range(mark: Option<usize>, caret: usize) -> Option<(usize, usize)> {
    let mark = mark?;
    if mark == caret {
        return None;
    }
    Some((mark.min(caret), mark.max(caret)))
}

/// **Remove `from..to`** and answer the caret, which lands where the removed
/// text began.
///
/// Character indices, clamped, like everything else in this module. A caller
/// that passes a reversed pair gets nothing removed rather than a panic —
/// [`range`] is the intended source and never produces one, and a panic here
/// would be a crash in the middle of typing.
pub fn delete_range(text: &mut String, from: usize, to: usize) -> usize {
    let mut chars: Vec<char> = text.chars().collect();
    let from = from.min(chars.len());
    let to = to.min(chars.len());
    if from >= to {
        return from;
    }
    chars.drain(from..to);
    *text = chars.into_iter().collect();
    from
}

/// **Was Shift held for this movement?** — asked of BOTH places egui keeps the
/// answer, because on this toolkit they disagree.
///
/// # ★★★ The measurement, 2026-08-21
///
/// The first driven run of `shift_arrows_select_text` failed with the caret
/// moving and nothing selected. A trace of both values, on the same frame, on
/// three consecutive presses with Shift physically held down throughout:
///
/// ```text
/// TMPDIAG right ev=Modifiers::NONE frame=Modifiers { shift: true }
/// TMPDIAG right ev=Modifiers::NONE frame=Modifiers { shift: true }
/// TMPDIAG right ev=Modifiers::NONE frame=Modifiers { shift: true }
/// ```
///
/// `egui-winit` builds a `Key` event with `modifiers: self.egui_input.modifiers`
/// — the state as of the moment the key was *translated* — and updates that
/// field from a separate `ModifiersChanged` window event. When the two arrive in
/// one batch with the key first, the event carries `NONE` and the frame ends
/// holding `shift: true`. Every arrow arrived; not one of them carried the
/// modifier that gives it its meaning.
///
/// # ★★ Why the answer is OR and not "pick the better one"
///
/// `app::keyboard` argues at length for the opposite preference — *"read the
/// modifiers CARRIED BY THE KEY EVENT, not the frame's"* — and it is right for
/// what it does. A **chord** is a command: `Ctrl+S` read from a stale frame
/// saves a document nobody asked to save, so a chord must be certain the
/// modifier was down *at the keystroke*.
///
/// A **selection modifier is not a command**, and its two failure modes are not
/// the same size:
///
/// | wrong answer | cost |
/// |---|---|
/// | shift read as held when it had just been released | the selection extends by ONE character, and the next press fixes it |
/// | shift read as absent when it was held | **the selection is destroyed** — rule 4 drops the mark, and the range the operator was building is gone |
///
/// The second is unrecoverable by pressing the key again, so the union is the
/// answer that fails cheaply. This is the same asymmetry argument
/// `canvas::constrain` makes about a snap that is offered versus one that is
/// silently taken.
#[must_use]
pub fn shifted(event: bool, frame: bool) -> bool {
    event || frame
}

/// **What the mark becomes after a movement**, given whether Shift was held.
///
/// The single statement of rule 4. `was` is the mark before the movement and
/// `from` is the caret before it.
///
/// - **Shift held, no mark yet** — the mark is planted where the caret *was*,
///   which is what makes the first Shift+Right select one character rather
///   than none.
/// - **Shift held, mark already set** — it stays, so the selection grows and
///   shrinks from the same fixed end.
/// - **No Shift** — dropped.
#[must_use]
pub fn moved(was: Option<usize>, from: usize, shift: bool) -> Option<usize> {
    if shift {
        Some(was.unwrap_or(from))
    } else {
        None
    }
}

/// The caret one **word** to the left of `caret` - `Ctrl+Left`.
///
/// Skips any run of spaces immediately behind the caret, then the run of
/// non-spaces behind that. This is the behaviour of every text field the
/// operator uses, and the reason it is here rather than deferred is that a
/// caret which can only move one character at a time is a caret nobody uses
/// twice on a line of any length.
#[must_use]
pub fn word_left(text: &str, caret: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut at = caret.min(chars.len());
    while at > 0 && chars[at - 1].is_whitespace() {
        at -= 1;
    }
    while at > 0 && !chars[at - 1].is_whitespace() {
        at -= 1;
    }
    at
}

/// The caret one **word** to the right of `caret` - `Ctrl+Right`.
///
/// The mirror of [`word_left`], and deliberately not symmetric in its order:
/// it skips the non-spaces first and then the spaces, so one press lands the
/// caret at the start of the next word rather than at the end of this one.
/// That is what the same key does everywhere else.
#[must_use]
pub fn word_right(text: &str, caret: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut at = caret.min(n);
    while at < n && !chars[at].is_whitespace() {
        at += 1;
    }
    while at < n && chars[at].is_whitespace() {
        at += 1;
    }
    at
}

#[cfg(test)]
mod tests {

    // ---------------------------------------------------------------------
    // Selection
    // ---------------------------------------------------------------------

    /// ★★ **A mark sitting on the caret is not a selection.**
    ///
    /// The state after Shift+Right then Shift+Left, and it is reached by every
    /// operator who changes their mind. If it answered `Some((n, n))` then
    /// Backspace would take the "delete the selection" branch, delete nothing,
    /// and leave the character it was supposed to remove — a key that stopped
    /// working, silently, at one specific caret position.
    #[test]
    fn an_empty_selection_is_no_selection() {
        assert_eq!(super::range(Some(3), 3), None);
        assert_eq!(super::range(None, 3), None);
    }

    /// It normalises, because the operator may have dragged either way.
    #[test]
    fn a_selection_reads_the_same_from_either_end() {
        assert_eq!(super::range(Some(2), 7), Some((2, 7)));
        assert_eq!(super::range(Some(7), 2), Some((2, 7)));
    }

    /// ★ **Removing a selection is by CHARACTER**, like everything else here.
    ///
    /// Asserted on a string with an accent in it, because a byte-indexed
    /// `drain` compiles, passes on ASCII, and panics on the first document with
    /// a `café` in it — which is the failure this module has now avoided in
    /// four separate functions for the same reason.
    #[test]
    fn removing_a_selection_counts_characters_not_bytes() {
        let mut text = String::from("café au lait");
        let caret = super::delete_range(&mut text, 0, 5);
        assert_eq!(text, "au lait");
        assert_eq!(caret, 0, "the caret lands where the removed text began");
    }

    /// A reversed or out-of-range pair removes nothing rather than panicking.
    ///
    /// ★ Not defensive programming for its own sake: this runs inside the
    /// keystroke path, and a panic there is a crash in the middle of typing —
    /// the one place a program must not crash, because the operator's work is
    /// in the thing that died.
    #[test]
    fn a_nonsense_range_removes_nothing() {
        let mut text = String::from("abc");
        assert_eq!(super::delete_range(&mut text, 2, 1), 2);
        assert_eq!(text, "abc");
        let mut text = String::from("abc");
        assert_eq!(super::delete_range(&mut text, 9, 99), 3);
        assert_eq!(text, "abc");
    }

    /// ★★★ **The first Shift+Right selects one character**, which is the whole
    /// reason [`super::moved`] takes the caret's position BEFORE the movement.
    ///
    /// Planting the mark where the caret *ends up* would select nothing on the
    /// first press and one character on the second — an off-by-one that looks
    /// like the key being ignored.
    #[test]
    fn the_first_shifted_move_plants_the_mark_where_the_caret_was() {
        assert_eq!(super::moved(None, 4, true), Some(4));
    }

    /// A mark already planted stays put, so the selection grows and shrinks
    /// from one fixed end.
    #[test]
    fn a_planted_mark_survives_further_shifted_moves() {
        assert_eq!(super::moved(Some(2), 6, true), Some(2));
    }

    /// ★★ **Any movement without Shift drops it.** Rule 4, and the one whose
    /// absence leaves a highlight on screen after the caret has walked out of
    /// it — so the next keystroke deletes text the operator is no longer
    /// looking at.
    #[test]
    fn an_unshifted_move_drops_the_selection() {
        assert_eq!(super::moved(Some(2), 6, false), None);
    }

    use super::*;

    /// **Typing inserts; Backspace removes one character, not one byte.**
    #[test]
    fn the_draft_edits_by_character_and_not_by_byte() {
        let mut s = String::new();
        let caret = insert(&mut s, 0, "café");
        assert_eq!(s, "café");
        assert_eq!(caret, 4, "the caret follows what was typed");
        let caret = backspace(&mut s, caret);
        assert_eq!(s, "caf", "a multi-byte char must go in one Backspace");
        assert_eq!(caret, 3);
    }

    /// **Control characters never reach the draft.**
    ///
    /// Enter and Escape arrive as `Key` events and mean something; a control
    /// byte arriving as text means nothing this shell can author, and putting
    /// it in a PDF show string would be authoring a glyph the operator cannot
    /// see.
    #[test]
    fn a_control_character_is_not_typed_into_the_page() {
        let mut s = String::new();
        insert(&mut s, 0, "a\u{1b}b\u{7}c");
        assert_eq!(s, "abc");
    }

    /// ★★ **The whole of the operator's 2026-08-20 report, as one test.**
    ///
    /// > *"the cursor just sits at the end of a text line. It can't be moved to
    /// > the center of an existing text block."*
    ///
    /// A title-block cell, edited the way he would edit one: put the caret in
    /// the middle, remove the character before it, type a different one. Before
    /// the caret existed this was impossible — the only reachable edit was to
    /// Backspace from the end and retype everything after the change, which on
    /// a drawing sheet full of `SHEET n OF m` cells is why the feature was
    /// reported as not working at all.
    #[test]
    fn a_character_in_the_middle_can_be_changed_without_retyping_the_tail() {
        let mut s = String::from("SHEET 1 OF 4");
        let caret = backspace(&mut s, 7);
        assert_eq!(s, "SHEET  OF 4");
        let caret = insert(&mut s, caret, "2");
        assert_eq!(s, "SHEET 2 OF 4");
        assert_eq!(caret, 7, "and the caret stays where the operator was");
    }

    /// Delete eats forwards and leaves the caret alone — which is the whole
    /// difference between it and Backspace, and the reason both exist.
    #[test]
    fn delete_eats_forwards_and_backspace_eats_backwards() {
        let mut s = String::from("abcd");
        assert_eq!(delete_forward(&mut s, 2), 2);
        assert_eq!(s, "abd");
        assert_eq!(backspace(&mut s, 2), 1);
        assert_eq!(s, "ad");
    }

    /// Both ends refuse to run off, rather than panicking or wrapping.
    ///
    /// The last assertion is the one that matters most: a caret index taken
    /// before the text was replaced can legitimately be past the end, and the
    /// answer is to clamp rather than to panic. See the module header.
    #[test]
    fn the_caret_cannot_be_pushed_past_either_end() {
        let mut s = String::from("ab");
        assert_eq!(
            backspace(&mut s, 0),
            0,
            "Backspace at the start does nothing"
        );
        assert_eq!(s, "ab");
        assert_eq!(
            delete_forward(&mut s, 2),
            2,
            "Delete at the end does nothing"
        );
        assert_eq!(s, "ab");
        assert_eq!(insert(&mut s, 99, "!"), 3, "a stale index is clamped");
        assert_eq!(s, "ab!");
    }

    /// Word movement lands where every other text field lands: to the LEFT, at
    /// the start of the word behind; to the RIGHT, at the start of the word
    /// ahead. The asymmetry is the convention, not an oversight — one press of
    /// Ctrl+Right puts the caret ready to type the next word, which is what an
    /// operator is doing when they press it.
    #[test]
    fn word_movement_follows_the_convention_every_text_field_uses() {
        let s = "SHEET 1 OF 4";
        assert_eq!(word_left(s, 12), 11, "from the end, back to the `4`");
        assert_eq!(word_left(s, 10), 8, "again, back to `OF`");
        assert_eq!(word_left(s, 0), 0, "and it stops at the start");
        assert_eq!(word_right(s, 0), 6, "from the start, on to the `1`");
        assert_eq!(word_right(s, 6), 8, "then on to `OF`");
        assert_eq!(word_right(s, 12), 12, "and it stops at the end");
    }
}
