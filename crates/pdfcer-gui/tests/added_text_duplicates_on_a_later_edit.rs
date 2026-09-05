//! # `added_text_duplicates_on_a_later_edit` — the reproduction for
//! # `OPERATOR_REQUESTS.md` **O127**, defect 1
//!
//! ## The report, verbatim
//!
//! > *"there's a bug I've come across where if you add text once it works, but
//! > if you add text a second or third time it will make duplicates of you try
//! > to move the instances after and make a duplicate for every new text box
//! > that you added regardless of which one you move, with the exception that
//! > if you make a text box, switch tools and make another one, then the first
//! > one doesn't start making duplicates."*
//!
//! ## ★★★ The cause, and it is not in this repository
//!
//! `D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs`, `text_edit_command`, the
//! line reading `if first_edit {`:
//!
//! ```text
//! let content_before = self.state.get(&content_id).cloned();
//! let first_edit = content_before.is_none();
//! …
//! if first_edit {
//!     for id in page.contents.iter().skip(1) { …empty it… }
//! }
//! ```
//!
//! Two engine facts have to be held at once, and neither is wrong on its own:
//!
//! | verb | what it does to `/Contents` |
//! |---|---|
//! | `add_text` (also `add_image`, `paste_objects`, `flatten_fields`) | **appends a NEW stream object** and leaves `contents[0]` byte-verbatim |
//! | every content surgery — `move_objects`, `transform_objects`, `edit_text`, `format_text`, `delete_object`, `reflow_block` | reads the **whole `/Contents` list concatenated**, splices, and writes the entire result back into **`contents[0]`** |
//!
//! ⇒ The second verb therefore **has** to empty `contents[1..]`, or the added
//! run is on the page twice: once folded into `contents[0]`, once still in its
//! own entry. It does empty them — but only `if first_edit`, i.e. only the very
//! first time that session rewrites `contents[0]`.
//!
//! The premise behind that gate is stated in the engine's own doc comment —
//! *"on a subsequent edit the extras are already emptied"* — and **`add_text`
//! falsifies it**, because it appends a new, non-empty extra *after* the
//! sweep has already happened.
//!
//! ## ★★ Why the operator's ordering is exactly the one that trips it
//!
//! | step | `/Contents` | `contents[0]` rewritten yet? | `first_edit` | result |
//! |---|---|---|---|---|
//! | add T1 | `[C0, A1]` | no | — | A1 holds T1 |
//! | **drag it** | `[C0, A1]` | no | **true** | `C0 := C0+A1`, A1 emptied — **correct**, and this is his *"if you add text once it works"* |
//! | add T2 | `[C0, A1ᵉ, A2]` | yes | — | A2 holds T2 |
//! | **drag anything** | same | yes | **false** | `C0 := C0+A2` **and A2 is left as it was** → T2 is on the page **twice** |
//! | add T3, drag again | `[C0, A1ᵉ, A2, A3]` | yes | false | T2 ×3, T3 ×2 — *"a duplicate for every new text box that you added regardless of which one you move"* |
//!
//! ★ And the exception he noticed is the same table read differently:
//! *"if you make a text box, switch tools and make another one, then the first
//! one doesn't start making duplicates."* Switching tools is how he gets to the
//! Select tool to drag the first box — and that drag is the `first_edit == true`
//! row, which sweeps the first box into `contents[0]` for good. It is the
//! **only** box that can never duplicate afterwards.
//!
//! ⇒ So the accumulating container really is a list that is appended to on each
//! placement and never emptied on commit. It is `/Contents`, it lives in
//! `pdfcer-core`, and what drains it is not a tool change — it is the session's
//! *first* content surgery, once, ever.
//!
//! ## ★★★ Why this is a test and not a paragraph in a request
//!
//! `D:\Dev\pdfcer` is READ-ONLY to this project, so the fix is not ours to
//! make. `engine_overlay_skew.rs` — the file beside this one — established the
//! shape and then proved its worth: **a test written to pass on the broken
//! engine and fail on the fixed one, carrying its own instruction to whoever
//! sees it go red.** All three of that file's tripwires fired on the day the
//! engine landed `Pass 186.0`, and inverting them took minutes.
//!
//! A test asserting the *correct* behaviour would be a red test in a green
//! repository for as long as the request stays open, and would be muted inside
//! a week. So [`text_added_after_an_earlier_edit_is_duplicated_by_the_next_one`]
//! asserts the **defect**, and says in its failure message what to do when it
//! stops reproducing.
//!
//! Beside it, [`three_placements_then_one_move_leave_three_runs`] asserts the
//! **count** on the ordering that is correct today. That is the assertion
//! `OPERATOR_REQUESTS.md` O127 asks for by name — *"three placements then one
//! move must produce three objects, not six"* — and it is the guard that stops
//! a future engine change from breaking the half that works.
//!
//! ## ★★ What was never tested, which is the more valuable finding
//!
//! A sweep of `D:\Dev\pdfcer\crates\pdfcer-core\tests` finds:
//!
//! * **no file that mentions both `add_text` and `move_object`/`transform_objects`**;
//! * `session_overlay_skew.rs` does one `add_image` then one `edit_text`, and
//!   one `add_image` then one `transform_objects` — both are the
//!   `first_edit == true` branch, the branch that works;
//! * `contents_append_shapes.rs` does two appends in one session and asserts
//!   only that the `/Contents` **array shape** stays flat — no surgery follows
//!   it and nothing looks at the page's words;
//! * `add_text.rs` does one add and an undo.
//!
//! ⇒ **Every existing test places once, or edits once.** The defect survives
//! the first placement and appears on the second, which is precisely what a
//! fixture exercising one of anything cannot see. That is the same shape as
//! `HANDOFF.md`'s redaction verifier, which was only ever run on synthetic
//! pages with no embedded font.

use pdfcer_core::edit::EditSession;
use pdfcer_core::text_edit::AddTextRequest;
use pdfcer_core::vector::{Matrix, TransformOptions};

/// A real one-page drawing that already ships with this repository.
///
/// `a1-titleblock.pdf` rather than a synthetic blank, for the reason
/// `engine_overlay_skew.rs` gives about the same choice: a page with zero
/// existing objects makes an index assertion trivially true, and a real page is
/// what the operator has.
fn session() -> EditSession {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/a1-titleblock.pdf"
    );
    let doc = pdfcer_core::document::Document::load(std::path::Path::new(path))
        .expect("fixture a1-titleblock.pdf must load");
    EditSession::new(doc)
}

/// Place one line of new page text, exactly as `Action::CommitAddText` does.
///
/// The marker strings are deliberately unlike anything on the title block, so
/// [`copies_of`] counts this test's own writing and nothing else.
fn add(session: &mut EditSession, marker: &str, y: f64) {
    let req = AddTextRequest::new(0, (120.0, y), marker);
    session
        .add_text(&req)
        .unwrap_or_else(|e| panic!("placing {marker:?} must succeed: {e}"));
}

/// **How many times `marker` appears in the page's own words.**
///
/// The operator-meaningful oracle, and chosen over counting decomposed objects
/// on purpose: the defect is *"the words are on my drawing twice"*, and the
/// count of extracted runs is the same number he can see. A `q`/`cm`/`Q`
/// wrapper — which is all a transform adds — changes no run's text, so this
/// number moves only when content is genuinely duplicated or lost.
fn copies_of(session: &EditSession, marker: &str) -> usize {
    let pages = session.pages().expect("the session has a page tree");
    let page = pages.first().expect("the fixture has a first page");
    let text = pdfcer_core::text_extract::extract_page_view(
        &session.view(),
        page,
        0,
        &pdfcer_core::text_extract::ExtractOptions::default(),
    )
    .expect("the page extracts");
    text.runs.iter().filter(|r| r.text.contains(marker)).count()
}

/// Nudge one object, which is what a drag of placed text reaches.
///
/// `transform_objects` and not `move_objects`, because that is the verb the
/// shell actually sends for a text object: `canvas::moving::eligible` forks on
/// `non_path`, and a show operator carries no coordinate operands for
/// `move_objects` to rewrite. Either verb would reproduce this — both go
/// through `vector_surgery_inner` and therefore through the gate — but the test
/// should send what the gesture sends.
fn nudge(session: &mut EditSession, object: usize) {
    session
        .transform_objects(
            0,
            &[object],
            Matrix::translate(6.0, 0.0),
            TransformOptions::default(),
        )
        .expect("nudging one object must succeed");
}

/// The index of the object the last `add_text` produced.
///
/// The added run is appended to the page's content, so it is the **last**
/// object in paint order. Taken from the model rather than assumed, because a
/// wrong index here would make the test move something else and prove nothing.
fn last_object(session: &mut EditSession) -> usize {
    session
        .page_objects(0)
        .expect("the page decomposes")
        .objects
        .len()
        - 1
}

/// ★★★ **Three placements and one move leave three runs, not six.**
///
/// `OPERATOR_REQUESTS.md` O127's own words for what this file had to assert.
/// This is the ordering that is **correct today** — every add happens before
/// the session's first content surgery, so that surgery's `first_edit` branch
/// sweeps all three appended streams into `contents[0]` and empties them.
///
/// It is here as the guard rather than as the reproduction: it is the half that
/// works, it is the half a fix to the gate must not break, and a count
/// assertion is the only thing that can tell "swept correctly" from "swept
/// twice".
#[test]
fn three_placements_then_one_move_leave_three_runs() {
    let mut s = session();
    add(&mut s, "ZZALPHA", 700.0);
    add(&mut s, "ZZBETA", 680.0);
    add(&mut s, "ZZGAMMA", 660.0);
    for marker in ["ZZALPHA", "ZZBETA", "ZZGAMMA"] {
        assert_eq!(
            copies_of(&s, marker),
            1,
            "{marker} must be on the page once before anything is moved"
        );
    }

    let object = last_object(&mut s);
    nudge(&mut s, object);

    for marker in ["ZZALPHA", "ZZBETA", "ZZGAMMA"] {
        assert_eq!(
            copies_of(&s, marker),
            1,
            "moving one placed run put {marker} on the page more than once — three \
             placements then one move must produce three runs, not six"
        );
    }
}

/// ★★★ **THE REPRODUCTION.** Text added *after* the session's first content
/// edit is duplicated by the next content edit.
///
/// # This test asserts the DEFECT, and here is what to do when it fails
///
/// ★★★ **INVERTED 2026-09-05, and this paragraph is the record of why.**
///
/// It *was* written to pass on the broken engine and fail on the fixed one. It
/// went red the moment the lock moved to `pdfcer-core` `b1033ab`, which is
/// exactly what it was for — **a test asserting somebody else's limitation is
/// the only thing that notices when the limitation ends.** Confirmed at source
/// before inverting, as the instructions below required: `edit.rs`'s
/// `text_edit_command` now sweeps on **every** surgery (`Pass 251.0`), reading
/// `self.value` — the overlay-or-base current payload — and emptying every
/// non-empty entry of `page.contents[1..]` rather than only on the first
/// rewrite.
///
/// The three expectations are now `1`, `1`, `1`. What they assert has not
/// changed shape: one placement, one copy; a second placement after an edit,
/// still one copy; **and a further edit does not add another** — that last is
/// the half of his report that said *"a duplicate for every new text box you
/// added"*, and a fix that emptied the extras only sometimes would still fail
/// it. The two assertions mean different things and both are kept.
///
/// ⚠ The `ZZFIRST` expectation was **already `1` and is unchanged**. It is the
/// operator's own exception — *"if you make a text box, switch tools and make
/// another one, the first one doesn't start making duplicates"* — and it is
/// what localised the defect to the `first_edit` gate in the first place. It
/// asserted correct behaviour before the fix and asserts it after, which is
/// why it did not move.
///
/// ---
///
/// The original instructions, kept verbatim because the next person to invert a
/// tripwire will want the shape of it:
///
/// It was written to **pass on the broken engine and fail on the fixed one**,
/// which is the shape `engine_overlay_skew.rs` established for a claim about a
/// crate this project may not change. If you are reading this because the test
/// went red:
///
/// 1. **The engine has been fixed.** Confirm against
///    `D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs`, `text_edit_command` — the
///    `if first_edit {` gate should now empty every non-empty entry of
///    `page.contents[1..]`, not only on the first rewrite.
/// 2. **Invert this test**: change the two expectations from `2` to `1` and
///    rewrite this comment to describe the fixed behaviour, exactly as
///    `engine_overlay_skew.rs` was inverted on 2026-08-31.
/// 3. Close the row in `ENGINE_BACKLOG.md`.
///
/// Do **not** merely delete it. The count is the only oracle that separates
/// "the extras were swept" from "the extras were folded in and left behind",
/// and both look identical to every other test in either repository.
///
/// # ★ The sequence is the operator's, step for step
///
/// Place a box, move it (this is his *"if you add text once it works"*), place
/// a second box, move something. The second box is now on the drawing twice —
/// and it is invisible until the move, because the two copies sit exactly on
/// top of one another until one of them is nudged.
#[test]
fn text_added_after_an_earlier_edit_is_duplicated_by_the_next_one() {
    let mut s = session();

    // Placement one, and the drag that follows it. `first_edit` is true here,
    // so this surgery folds the appended stream into `contents[0]` and empties
    // it — the branch that works.
    add(&mut s, "ZZFIRST", 700.0);
    let first = last_object(&mut s);
    nudge(&mut s, first);
    assert_eq!(
        copies_of(&s, "ZZFIRST"),
        1,
        "the FIRST placement must survive its own move exactly once — if this fails the \
         defect is wider than O127 describes and the whole diagnosis needs re-deriving"
    );

    // Placement two. `contents[0]` is already in the session's overlay, so
    // `first_edit` will be false for every surgery from here on and this
    // appended stream will never be emptied.
    add(&mut s, "ZZSECOND", 680.0);
    assert_eq!(
        copies_of(&s, "ZZSECOND"),
        1,
        "an added run is on the page once until something else is edited"
    );

    // Any content edit at all. It does not have to be the run that was just
    // placed — *"regardless of which one you move"* — so this deliberately
    // nudges the FIRST box, which by now lives inside `contents[0]`.
    let object = last_object(&mut s);
    nudge(&mut s, object);

    assert_eq!(
        copies_of(&s, "ZZSECOND"),
        1,
        "★ text added AFTER the session's first content edit must be folded into \
         contents[0] exactly once. Two copies is the operator's original report — the \
         defect this file was written to reproduce, fixed at the engine in Pass 251.0"
    );
    assert_eq!(
        copies_of(&s, "ZZFIRST"),
        1,
        "the first box was already swept into contents[0], so it must NOT gain a copy — \
         this is the operator's own exception, and it is what makes the diagnosis specific"
    );
}

/// ★★ **It compounds**: a third placement adds a third copy of the second.
///
/// The half of the report that says *"a duplicate for every new text box that
/// you added"*. Each surgery re-folds the still-live extras into `contents[0]`,
/// so a run placed after the first edit gains one copy per subsequent edit
/// rather than settling at two.
///
/// Asserting the growth as well as its existence is what makes this a
/// reproduction rather than a snapshot: a fix that emptied the extras only
/// *sometimes* would leave this red while the test above went green, and the
/// two failures mean different things.
///
/// Inverted with its neighbour when the engine is fixed — every expectation
/// here becomes `1`.
#[test]
fn each_further_edit_adds_another_copy() {
    let mut s = session();

    add(&mut s, "ZZFIRST", 700.0);
    let first = last_object(&mut s);
    nudge(&mut s, first);

    add(&mut s, "ZZSECOND", 680.0);
    let object = last_object(&mut s);
    nudge(&mut s, object);
    assert_eq!(
        copies_of(&s, "ZZSECOND"),
        1,
        "one edit, and still one copy — the sweep now runs on every surgery"
    );

    nudge(&mut s, first);
    assert_eq!(
        copies_of(&s, "ZZSECOND"),
        1,
        "★ AND IT MUST NOT COMPOUND. This is the half of his report that said `a duplicate \
         for every new text box you added` — a fix that emptied the extras only SOMETIMES \
         would leave this red while its neighbour went green, and the two failures would \
         mean different things"
    );
}
