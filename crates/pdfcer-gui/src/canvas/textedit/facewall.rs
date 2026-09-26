//! # `canvas::textedit::facewall` — one session, two verbs, and the second one refuses
//!
//! **The experiment that decides whose defect O141's last step is**, kept in the
//! tree rather than run once and reported, because the answer it gives is a
//! statement about `pdfcer-core` and this project has learnt that a paragraph
//! about what the engine cannot do has a shelf life measured in hours
//! (`RESUME.md`, *"Where the claim can be an assertion, make it one"*).
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/facewall.md`.

#![cfg(test)]
// ---------------------------------------------------------------------------
// A deliberate duplicate of the `#![cfg(test)]` above, for
// `tools/gates/check-ui-strings.sh` rather than for rustc — the same device
// `proof.rs` uses and for the same reason: the gate reads modules line by line
// and cannot see an inner attribute at the top of a file it is scanning for
// bare string literals. Removing this line does not change what rustc builds.
// ---------------------------------------------------------------------------
#![cfg(test)]

use std::path::PathBuf;

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::text_edit::{
    EditOptions, EditRequest, FontSelector, FormatOptions, FormatRequest,
};

/// The text the fixture's single run prints, and the character the subset font
/// cannot carry.
///
/// `q` rather than `€` deliberately: the driven check types `q`, and O141's
/// whole point to the operator is that the wall is **not** about accents or
/// symbols — the fixture's subset face
/// `SUBSET+pdfceSubsetDemo` (old-name-exempt: a font NAME inside
/// `fixtures/subset-font-floor.pdf`'s own bytes, verified with a byte grep; it
/// belongs to the engine's synthetic fixture and did not rename when this
/// project did, so spelling it the new way would name a font no document
/// contains) carries `A`, `B` and `C` because those are the letters the page
/// prints, and nothing else, a plain lowercase `q` included.
const RUN_TEXT: &str = "ABC";
const TYPED: &str = "ABCq";

/// The face the offer swaps to. One of the fourteen standard PDF fonts, so the
/// swap adds a *name* to the document and no font program — which is why it
/// raises none of O47's licence question, and why it owes an off-canvas
/// sentence instead of a mark on the page (R8b rule 4).
const FACE: &str = "Helvetica";

/// A second standard-14 face, used only to build a document that carries two of
/// them. See [`a_face_the_page_already_carries_can_be_typed_into_at_once`].
const OTHER_FACE: &str = "Times-Roman";

/// The fixture, and the reason no other one in this repository can stand in for
/// it is written out in `fixtures/subset-font-floor.PROVENANCE.md`: every other
/// document here is either a non-embedded standard-14 face (whose
/// `WinAnsiEncoding` accepts the edit), a fully embedded non-subset face (the
/// floor never fires), or a symbolic face that refuses for an unrelated reason
/// and offers no remedy. A check driven against any of those would be unable to
/// fail.
fn fixture() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/subset-font-floor.pdf");
    assert!(
        p.exists(),
        "the fixture is missing at {}. It is a byte copy of the engine's \
         fixtures/synthetic/text/subset-simple-embedded.pdf; see \
         fixtures/subset-font-floor.PROVENANCE.md",
        p.display()
    );
    p
}

/// Open the fixture as a fresh session.
fn session() -> EditSession {
    EditSession::new(Document::load(&fixture()).expect("the fixture loads"))
}

/// The face swap, exactly as the offer performs it, located by `find` alone.
fn swap_face(session: &mut EditSession) {
    let req = FormatRequest::new(0, RUN_TEXT).font(FontSelector::new(FACE));
    session
        .format_text(&req, &FormatOptions::default())
        .expect("a standard-14 face swap on a three-letter run is accepted");
}

/// Type the character the subset font refused, into the same run, located by
/// `find` alone.
fn type_the_character(session: &mut EditSession) -> Result<(), String> {
    let req = EditRequest::find_replace(0, RUN_TEXT, TYPED);
    session
        .edit_text(&req, &EditOptions::default())
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Type it the way **the shell** does: a pinned whole-operator request, with
/// the pin measured **now**, from a fresh provenance-carrying extraction over
/// the session's current view.
fn type_the_character_pinned(session: &mut EditSession) -> Result<(), String> {
    use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel};

    let (span, target) = {
        let view = session.view();
        let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
        let text = pdfcer_core::text_extract::extract_page_view(
            &view,
            &pages[0],
            0,
            &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
        )
        .expect("the page's text extracts");
        let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
        let p = super::pin::of_run(&model, 0).expect("run 0 carries provenance");
        (p.span, p.target)
    };
    let mut req = EditRequest::whole_operator(0, span, TYPED);
    req.target = target;
    session
        .edit_text(&req, &EditOptions::default())
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The text the page currently prints, through the session's own view — so it
/// reads the overlay, not the file on disk.
fn page_text(session: &EditSession) -> String {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
    let text = pdfcer_core::text_extract::extract_page_view(
        &view,
        &pages[0],
        0,
        &pdfcer_core::text_extract::ExtractOptions::default(),
    )
    .expect("the page's text extracts");
    text.runs.iter().map(|r| r.text.as_str()).collect()
}

/// **The measurement, INVERTED on 2026-09-06 because the engine shipped the
/// fix.** One session, `format_text` then `edit_text`, and the second call now
/// **succeeds** — in both request shapes, with no save and no reopen between
/// them.
#[test]
fn the_engine_types_into_a_face_it_just_swapped_in() {
    // ── The control, and it is not decoration ──────────────────────────────
    // It establishes that this fixture still reaches the wall O141 is about, so
    // the success asserted below cannot pass on a document that never had the
    // problem. If the engine ever taught the subset floor to widen a font
    // automatically, THIS is the assertion that would go red and tell us the
    // remedy under test had become unreachable rather than unnecessary.
    let before = type_the_character(&mut session())
        .expect_err("the subset font has no code for 'q'; the floor must fire");
    assert!(
        before.contains("R-INV-1") || before.contains("SUBSET"),
        "the FIRST refusal must be the embedded-subset floor — the wall the face \
         swap is the remedy for. Got: {before}"
    );

    // ── Shape 1: pinned, which is what the shell itself builds ─────────────
    let mut pinned_session = session();
    swap_face(&mut pinned_session);
    type_the_character_pinned(&mut pinned_session).expect(
        "SHIPPED in engine v0.41.0 (Pass 257.0): a face swapped in THIS session is \
         resolvable by the next edit_text, because the planner reads the session view \
         rather than the base revision. If this refuses again, the class has returned \
         and `panels::properties::refusedchar`'s blocked arm is load-bearing once more",
    );
    assert!(
        page_text(&pinned_session).contains(TYPED),
        "the character must be IN the page, not merely un-refused: a verb that returns \
         Ok and leaves the run unchanged is the failure this assertion exists to catch"
    );

    // ── Shape 2: by `find` alone, the quieter door ─────────────────────────
    let mut find_session = session();
    swap_face(&mut find_session);
    type_the_character(&mut find_session).expect(
        "the unpinned request must succeed too. Its old refusal was the locational \
         NoMatch — the one that claimed the operator's text was absent from a page \
         printing it — so a regression here would read as a bad search rather than as \
         a broken session, which is why it is asserted separately",
    );
    assert!(
        page_text(&find_session).contains(TYPED),
        "and the find-located edit must reach the page as well"
    );
}

/// **How wide the defect is: a swap to a face the page ALREADY carries
/// works in the same session.**
///
/// This is the measurement that decides whether the shell has a remedy or only
/// a sentence, so it is made rather than reasoned about.
///
/// The document is built by the engine itself, in two saved passes, so nothing
/// here hand-authors a PDF and the fixture on disk stays the one its provenance
/// note describes:
///
/// 1. the fixture, swapped to `Times-Roman` and saved — the file now carries a
///    `/Font` resource for it;
/// 2. reopened, swapped to `Helvetica` and saved — a second authored resource,
///    with the `Times-Roman` one still in the dictionary and no longer painted.
///
/// Then, in **one** fresh session, the run is swapped back to `Times-Roman` — a
/// name the base revision already holds, so `plan.created_font` is `None` and no
/// object is allocated — and the character the original subset font refused is
/// typed straight into it. It lands.
///
/// ⇒ **The trigger was the newly-created object, not the face swap.** A restyle
/// that resolves to a resource the file already had left nothing for
/// `edit_text` to fail to find. It was only the standard-14 offer — the one that
/// has to author a resource, which is the only remedy a single-font document has
/// — that could not be typed into until the file was saved and reopened.
///
/// ⚠ **Past tense throughout, since engine v0.41.0.** `Pass 257.0` fixed the
/// newly-created-object case, so this test no longer *bounds* a live defect. It
/// is kept because it still holds down the half that was never broken, and
/// because it is the control that would tell a future session whether a
/// returning refusal was the whole class coming back or only the authored-object
/// corner of it. It was also the measurement behind
/// `text::panels::face::refused_char_blocked`'s original wording — the wording
/// that has now been struck; that function's doc comment records why.
///
/// The swap **back to the original subset face** was the first shape of this
/// test and it is not usable, which is worth recording so it is not tried again:
/// `format_text` refuses it with `CoverageFailure … is an embedded SUBSET that
/// does not already carry code 65 for … 'A'`, naming the face
/// `SUBSET+pdfceSubsetDemo`. (old-name-exempt: see `RUN_TEXT`.) The
/// subset's own three letters are not addressable by the codes the standard-14
/// run now uses, so the page's own original face is not a face the page can go
/// back to.
#[test]
fn a_face_the_page_already_carries_can_be_typed_into_at_once() {
    /// Swap the run to `face`, save, and answer the saved bytes.
    fn swapped_and_saved(mut session: EditSession, face: &str) -> Vec<u8> {
        let req = FormatRequest::new(0, RUN_TEXT).font(FontSelector::new(face));
        session
            .format_text(&req, &FormatOptions::default())
            .expect("a standard-14 face covers A, B and C");
        session
            .to_incremental_bytes(&pdfcer_core::writer::SaveOptions::identity())
            .expect("the session saves incrementally")
            .0
    }

    let first = swapped_and_saved(session(), OTHER_FACE);
    let second = swapped_and_saved(
        EditSession::new(Document::from_bytes(first).expect("the one-face document reopens")),
        FACE,
    );
    let mut session =
        EditSession::new(Document::from_bytes(second).expect("the two-face document reopens"));

    // Back to a face the reopened file already holds. No object is allocated,
    // so there is nothing that exists only in the overlay.
    let back = FormatRequest::new(0, RUN_TEXT).font(FontSelector::new(OTHER_FACE));
    session
        .format_text(&back, &FormatOptions::default())
        .expect("the page's own Times-Roman resource covers A, B and C");
    type_the_character(&mut session).expect(
        "MEASURED 2026-09-05: a swap to a face the page already carries is editable at once. \
         If this fails, the defect is wider than 'the newly-created resource is invisible' \
         and the engine request must be re-worded before it is believed",
    );
    assert!(
        page_text(&session).contains(TYPED),
        "and the character is in the page's text afterwards, not merely un-refused"
    );
}

/// **The control that makes the measurement evidence.** The same two verbs,
/// with a save and a reopen between them — which is exactly what two runs of
/// `pdfcer.exe` do — and the character lands.
#[test]
fn two_sessions_do_what_one_session_will_not() {
    let mut first = session();
    swap_face(&mut first);
    let bytes = first
        .to_incremental_bytes(&pdfcer_core::writer::SaveOptions::identity())
        .expect("the session saves incrementally")
        .0;
    drop(first);

    let mut second = EditSession::new(
        Document::from_bytes(bytes).expect("the saved bytes reopen as a document"),
    );
    type_the_character(&mut second).expect(
        "REOPENED, the swapped face resolves and the character is accepted. If this \
         fails, the fixture or the engine's face swap has changed and the experiment \
         above is measuring something else",
    );
    assert!(
        page_text(&second).contains(TYPED),
        "and the character is in the page's text afterwards, not merely un-refused"
    );
}

/// **The words the operator typed survive the refusal that threw the draft
/// away** — the round trip [`super::Committing`] promises, asserted rather than
/// argued.
#[test]
fn last_commit_is_the_one_just_planned() {
    let doc = crate::app::state::open_local_fixture("subset-font-floor.pdf");
    let planned = super::plan(&doc, 0, 0, RUN_TEXT, TYPED);
    assert!(
        planned.one_operator,
        "the fixture's run is one show operator; if it were not, the pin path would not \
         be the one the offer's retype takes and this test would be measuring the other \
         branch"
    );
    let carried = super::last_commit().expect("planning a commit records what it will write");
    assert_eq!(carried.page, 0);
    assert_eq!(carried.run, 0);
    assert_eq!(
        carried.original, RUN_TEXT,
        "the `find` operand of a re-raised CommitTextEdit"
    );
    assert_eq!(
        carried.replacement, TYPED,
        "★ THE OPERAND THAT EXISTS NOWHERE ELSE once `Ctrl+Enter` has abandoned the draft"
    );
}
