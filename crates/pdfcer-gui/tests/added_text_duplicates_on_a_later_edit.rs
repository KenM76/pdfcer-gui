//! # `added_text_duplicates_on_a_later_edit` — page text added after an
//! # earlier edit is folded in exactly ONCE
//!
//! The reproduction for `OPERATOR_REQUESTS.md` **O127**, defect 1, and now this
//! project's standing guard on the engine contract it named.
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
//! ## The mechanism, and it lives in `pdfcer-core` rather than here
//!
//! Two engine facts have to be held at once, and neither is wrong on its own:
//!
//! * `add_text` — and equally `add_image`, `paste_objects`, `flatten_fields` —
//!   **appends a NEW stream object** to the page's `/Contents` and leaves
//!   `contents[0]` byte-verbatim.
//! * Every content surgery — `move_objects`, `transform_objects`, `edit_text`,
//!   `format_text`, `delete_object`, `reflow_block`, all of them through
//!   `vector_surgery_inner` — reads the **whole `/Contents` list concatenated**,
//!   splices, and writes the entire result back into **`contents[0]`**.
//!
//! So the second verb **has** to empty `contents[1..]`, or the added run is on
//! the page twice: once folded into `contents[0]`, once still in its own entry.
//! `text_edit_command` empties them, and the predicate it uses is the contract
//! these tests hold it to: **every extra whose CURRENT payload is non-empty, on
//! EVERY surgery.**
//!
//! Sweeping once per session is the near miss, and it is near enough to look
//! right: it rests on the premise that a later edit finds the extras already
//! emptied, and `add_text` falsifies that premise by appending a fresh
//! non-empty extra *after* the sweep has happened.
//!
//! ## Why the operator's ordering is exactly the one that finds it
//!
//! Place a box and drag it: `contents[0]` is rewritten for the first time and
//! the appended stream is emptied with it — his *"if you add text once it
//! works"*. Place a second box, and a new non-empty extra now sits beside an
//! **already-rewritten** `contents[0]`. Drag anything at all — *"regardless of
//! which one you move"* — and that extra is folded in; leave it unemptied and
//! the second box is on the page twice, three times after the next edit, and so
//! on. The defect grows, which is why the count and not the presence is the
//! assertion.
//!
//! His own exception is the same rule read from the other end: *"if you make a
//! text box, switch tools and make another one, then the first one doesn't
//! start making duplicates."* Switching tools is how he reaches the Select tool
//! to drag the first box, and that drag sweeps the first box into `contents[0]`
//! for good. It is the one box that can never duplicate afterwards, whatever
//! the sweep predicate is — which is what separates a fault in the sweep from a
//! fault in the fold.
//!
//! ## Why this is a test and not a paragraph in a request
//!
//! `D:\Dev\pdfcer` is READ-ONLY to this project, so the fix is not ours to
//! make. `engine_overlay_skew.rs` — the file beside this one — established the
//! shape for a claim about a crate this project may not change: **a test
//! written to pass on the broken engine and fail on the fixed one, carrying its
//! own instruction to whoever sees it go red.** A test asserting the *correct*
//! behaviour instead would be a red test in a green repository for as long as
//! the request stays open, and would be muted inside a week.
//!
//! The engine now sweeps on every surgery, so every expectation here reads `1`
//! and the file has changed job: it stopped being a claim about somebody else's
//! crate and became this project's regression net under an engine it does not
//! control and updates weekly. What the assertions mean did not change shape —
//! one placement, one copy; a second placement after an edit, still one copy;
//! **and a further edit does not add another**. A sweep that emptied the extras
//! only sometimes would still satisfy the first two.
//!
//! ## The blind spot this file exists to cover
//!
//! **A fixture that places once, or edits once, cannot see this class of
//! defect.** It survives the first placement and appears on the second, so any
//! session that does one of anything is running the branch that works — which
//! is how a page-content invariant can be well tested and still have nothing
//! standing between it and this failure. Both orderings are driven here, in one
//! file, for that reason.

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

/// **Three placements and one move leave three runs, not six.**
///
/// The ordering in which every add happens **before** the session's first
/// content surgery, so that one surgery folds all three appended streams into
/// `contents[0]` and empties them in the same command.
///
/// It is here as the guard rather than as the reproduction: it is the half that
/// works even under a once-per-session sweep, it is therefore the half a change
/// to the sweep predicate must not break, and a count assertion is the only
/// thing that can tell "swept correctly" from "swept twice".
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

/// **THE REPRODUCTION.** Text added *after* the session's first content edit
/// is folded into `contents[0]` exactly once, not twice.
///
/// # The sequence is the operator's, step for step
///
/// Place a box, move it — this is his *"if you add text once it works"*. Place
/// a second box, move something. Under a sweep that runs only on the first
/// rewrite of `contents[0]` the second box is now on the drawing twice, and
/// invisibly so: the two copies sit exactly on top of one another until one of
/// them is nudged, which is why the report arrived as *"duplicates when you try
/// to move the instances after"* rather than as *"my text is doubled"*.
///
/// # The `ZZFIRST` expectation is not redundant
///
/// It asserts the operator's own exception — *"if you make a text box, switch
/// tools and make another one, the first one doesn't start making
/// duplicates"* — and it is what makes a failure here specific. The first box
/// already lives inside `contents[0]`, so it must **not** gain a copy whatever
/// the extras do; `ZZFIRST` going red means the fold itself is wrong, while
/// `ZZSECOND` alone going red means the extras were folded in and left behind.
///
/// # If this test goes red
///
/// Read `pdfcer-core`'s `edit.rs`, `text_edit_command`, and check the predicate
/// that empties `page.contents[1..]`: it must empty every extra whose CURRENT
/// payload is non-empty — the overlay-or-base value, read per surgery — rather
/// than only on the session's first rewrite. A once-per-session predicate
/// passes [`three_placements_then_one_move_leave_three_runs`] and fails this
/// one, and that pair of verdicts is itself the diagnosis.
///
/// Do **not** merely delete it. The count is the only oracle that separates
/// "the extras were swept" from "the extras were folded in and left behind",
/// and both look identical to every other test in either repository.
#[test]
fn text_added_after_an_earlier_edit_is_duplicated_by_the_next_one() {
    let mut s = session();

    // Placement one, and the drag that follows it. This is the session's
    // first rewrite of `contents[0]`, so the appended stream is folded in and
    // emptied — the branch that works under any sweep predicate.
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
    // every surgery from here on is a LATER rewrite — and this freshly
    // appended stream is the extra a once-per-session sweep would never reach.
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

/// **AND IT MUST NOT COMPOUND**: a further edit adds no further copy.
///
/// The half of the report that says *"a duplicate for every new text box that
/// you added"*. Each surgery re-folds whatever the extras currently hold into
/// `contents[0]`, so a run placed after the first edit would gain one copy per
/// subsequent edit rather than settling at two — growth, not a one-off.
///
/// Asserting the absence of growth as well as the absence of the duplicate is
/// what makes this a reproduction rather than a snapshot: a sweep that emptied
/// the extras only *sometimes* leaves this red while its neighbour goes green,
/// and the two failures mean different things.
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
