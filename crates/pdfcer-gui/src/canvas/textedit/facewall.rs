//! # `canvas::textedit::facewall` — one session, two verbs, and the second one refuses
//!
//! **The experiment that decides whose defect O141's last step is**, kept in the
//! tree rather than run once and reported, because the answer it gives is a
//! statement about `pdfcer-core` and this project has learnt that a paragraph
//! about what the engine cannot do has a shelf life measured in hours
//! (`RESUME.md`, *"Where the claim can be an assertion, make it one"*).
//!
//! ## The question
//!
//! `tools/ui-verify`'s `a_refused_character_offers_a_face_that_can_type_it`
//! drives the whole of O141 and fails at the last step. The operator types a
//! character the run's subset font cannot carry; the refusal names it; the
//! Properties panel offers faces that can; the offer is taken and **reaches the
//! document** (`text-style-applied … change=face applied=1 runs=1`); and then
//! the same character, committed again into the same run, is refused a second
//! time with a *different* sentence:
//!
//! ```text
//! this run cannot be edited in the first cut:
//!   the run's font resource is unresolvable in the target stream's resources
//! ```
//!
//! The identical pair **succeeds on disk**, measured with `pdfcer.exe` 0.40.0 on
//! this very fixture: `format-text --set-font Helvetica` then `edit-text
//! --replace ABCq` lands, and `extract-text` reads `ABCq` back. The only
//! difference between the working case and the broken one is **one process
//! versus two** — the command line reopens the file between the verbs, the shell
//! holds one live [`EditSession`] across both. So the defect is in session
//! state, and the two candidates are:
//!
//! * **the shell**, handing the second `edit_text` an operand pinned before the
//!   restyle rewrote the content stream; or
//! * **the engine**, resolving the run's `Tf` name against a document revision
//!   that predates the resource the restyle just created.
//!
//! ## The experiment, and why it is decisive
//!
//! [`the_engine_cannot_type_into_a_face_it_just_swapped_in`] makes **one**
//! [`EditSession`], calls `format_text` and then `edit_text` on it, and locates
//! both by `find` text alone — no pin, no span, no
//! [`super::pin::Pinned`], nothing this shell computes. Every input the shell
//! could have got wrong is absent from it. If it still refuses, no arrangement
//! of shell operands can make the route work and the defect is the engine's;
//! if it succeeds, the engine is fine and the shell is handing it something
//! stale.
//!
//! [`two_sessions_do_what_one_session_will_not`] is the control that makes the
//! first one evidence. It runs the identical pair of calls with a **save and
//! reopen** between them — which is exactly what the command line does — and
//! asserts the second `edit_text` succeeds and the character is in the extracted
//! text afterwards. Without it, a failure above could be a fixture that cannot
//! take the edit at all, and the report would name the wrong subject.
//!
//! ## ★★★ What it measured, 2026-09-05
//!
//! **The engine's.** One session refuses; two sessions succeed; the fixture,
//! the request, the face and the character are the same in both.
//!
//! The mechanism, read afterwards and stated here because the assertion alone
//! does not explain it: [`EditSession::format_text`] allocates the new `/Font`
//! object and puts it in the same undo command
//! (`pdfcer-core/src/edit.rs:9556`, `font_resource_writes` — which correctly
//! binds against `self.graph()`, the overlay, *"not `self.base`"*, and says so).
//! [`EditSession::edit_text`] then plans with `plan_edit(&self.base, …)`
//! (`edit.rs:9265`), and `resolve_font_dict` dereferences the `/Font` name
//! through **that** document (`text_edit/edit.rs:3582-3595`,
//! `o.map(|o| doc.resolve(o))`). The name the restyle wrote into the stream
//! names an object that exists only in the overlay, so the deref against the
//! base answers `None`, and `None` is reported as *"unresolvable in the target
//! stream's resources"* (`edit.rs:1796`). The stream read is the session's
//! (`current_page_content`, which is what makes sequential edits accumulate);
//! only the object graph the names are resolved through is the base's.
//!
//! ⇒ **Any session-path verb that creates an indirect object and then resolves
//! a name that points at it will do this**, which is why it is filed as a class
//! rather than as "the font swap is broken":
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\request_edit_text_resolves_font_names_against_the_base_revision_so_a_face_swapped_in_this_session_is_unresolvable.md`.
//!
//! ## ★★ What the shell does about it, and what it deliberately does NOT do
//!
//! **It does not work around it.** The only in-process route that works is to
//! throw the `EditSession` away and rebuild it from the saved bytes, and that
//! silently destroys the operator's undo history — a document-level act, taken
//! without asking, to paper over a limit the engine itself prescribes the remedy
//! for (`EditSession::reflow_block`'s own documentation says *"Save and reopen
//! to reflow after an in-session edit of the same page"* about the same class of
//! staleness).
//!
//! So `panels::properties::refusedchar` did the two honest things instead: it
//! **re-applies the operator's edit itself** the moment the face lands, so the
//! route is one gesture and lands outright on any document carrying a second
//! usable face; and when the retype is refused it says so
//! (`text::panels::face::refused_char_blocked`).
//!
//! ⚠ **Both halves survive the fix, and only one of them changed.** The
//! re-apply is now the whole route — it lands on the fixture this module drives,
//! not just on a document that happened to carry a second usable face. The
//! refusal sentence kept its state and lost its *cause*: it used to name the
//! save-and-reopen remedy measured below, and that remedy is now a remedy for
//! nothing. See the section at the foot of this header.
//!
//! ## ★★★ THE ENGINE SHIPPED THE FIX — 2026-09-06, engine v0.41.0
//!
//! [`the_engine_types_into_a_face_it_just_swapped_in`] is the same test with its
//! assertion inverted. It went red on the first run after the pin moved to
//! v0.41.0, from its own `expect_err` message, which is the entire design
//! working: this project's standing failure mode is a sentence recording a
//! limitation that stopped being true and nothing noticing, and the fix for it
//! is to make the claim executable. `Pass 257.0` (`5e95805`) is the answer —
//! every text-edit planner and helper takes `&DocumentView<'_>`, every
//! `EditSession` verb passes `self.view()`, and with no `&Document →
//! &DocumentView` coercion the class is now a **compile error**.
//!
//! ### The three-step plan this header used to carry, and the one step of it
//! that was WRONG
//!
//! It said: (1) delete the test; (2) delete
//! `text::panels::face::refused_char_blocked` and the `retried` arm of
//! `refusedchar::section` that shows it, *"the state becomes unreachable,
//! because the retype will land and the block will retire on the next frame"*;
//! (3) tighten the driven check to *"the character went in"*.
//!
//! **Steps 1 and 3 were right and are done. Step 2 was wrong**, and the way it
//! was wrong is worth more than the instruction was:
//!
//! > The `retried` arm is **not** reached by recognising this cause. It is
//! > reached by arithmetic — the retype was raised and `doc.edit_epoch` did not
//! > move — and that condition is agnostic about **why**. Other whys exist: an
//! > offered face that does not cover the character after all, a run whose
//! > operators a pinned request cannot span, a refusal nothing here has met.
//! > Deleting the arm converts every one of them into **silence**, which is a
//! > defect already on this project's own list.
//!
//! ⇒ ★★ **A state and its explanation have different lifetimes, and a plan
//! written from one cause will happily delete the handling for all of them.**
//! The explanation expired; the state did not. So the cause was struck from the
//! sentence and the state kept its voice. `refused_char_blocked`'s own doc
//! comment carries the full reasoning and its test now asserts that the sentence
//! names **no** cause — including, by name, the two the old wording used, so
//! that a later session improving the prose cannot reinstate a falsehood from
//! this file's git history.
//!
//! Step 1 became an inversion rather than a deletion for the same family of
//! reason: the test still buys something. It holds the fix down in **both**
//! request shapes, and the `find` shape is the one whose regression would come
//! back quietly, as a "no match" that reads like a bad search.

#![cfg(test)]
// ---------------------------------------------------------------------------
// ★ A deliberate duplicate of the `#![cfg(test)]` above, for
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
///
/// No pin and no target: the point of this module is that **nothing the shell
/// computes** is in the request, so a refusal downstream cannot be blamed on a
/// stale span.
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
///
/// This is the shape `canvas::textedit::plan` builds and `Action::CommitTextEdit`
/// applies, reduced to its operands. The pin being re-measured after the restyle
/// is the point: it is what removes "the shell handed the engine a span pinned
/// before the stream was rewritten" from the list of explanations, by
/// construction rather than by argument.
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

/// ★★★ **The measurement, INVERTED on 2026-09-06 because the engine shipped the
/// fix.** One session, `format_text` then `edit_text`, and the second call now
/// **succeeds** — in both request shapes, with no save and no reopen between
/// them.
///
/// # What this test asserted until 2026-09-06, and why the inversion is the
/// design working rather than the design failing
///
/// It asserted the **refusal**: that the `/Font` object `format_text` had just
/// allocated lived only in the session overlay, that `edit_text` planned with
/// `plan_edit(&self.base, …)`, and that `resolve_font_dict` therefore
/// dereferenced the run's `Tf` name through a revision that predated the
/// resource — answering `None`, which surfaced as *"the run's font resource is
/// unresolvable in the target stream's resources"*.
///
/// That assertion was written **so that it would go red**, and on the engine
/// bump to v0.41.0 it did, on the first run, with its own `expect_err` message
/// naming the response. `Pass 257.0` (`5e95805`) made every text-edit planner
/// and helper take `&DocumentView<'_>` and every `EditSession` verb pass
/// `self.view()`; there is no `&Document → &DocumentView` coercion, so handing
/// a planner the base revision is now a **compile error** rather than a latent
/// wrong answer, and the class cannot return by the route it arrived.
///
/// ⇒ The lesson worth keeping is the one `RESUME.md` states: **a sentence about
/// what the engine cannot do is a dated citation with a shelf life measured in
/// hours, and where the claim can be an assertion it must be one.** This file
/// cost one test and returned the day the limit stopped being true, which is
/// what a paragraph cannot do.
///
/// # Why BOTH shapes are still asserted, now that both pass
///
/// They failed in **different voices** and the difference was the finding: the
/// pinned form reached `resolve_font_dict` and named the resource, while the
/// `find` form never got that far — locating a run by its text means decoding
/// every show operator, decoding needs the font, and the font was the thing
/// that would not resolve, so the answer was the locational `NoMatch`, *"text
/// to edit was not found in an editable run on the page"*, which told the
/// operator their text was absent from a page that was plainly printing it.
///
/// A regression could come back through either door, and the `find` door is the
/// one that would come back **quietly** — as a "no match", which reads like a
/// bad search rather than like a broken session. So both are driven, each in
/// its **own session**, because a successful edit rewrites the run and the
/// second shape would otherwise be searching a page the first one changed.
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

/// ★★★ **How wide the defect is: a swap to a face the page ALREADY carries
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
/// ★ The swap **back to the original subset face** was the first shape of this
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

/// ★★★ **The control that makes the measurement evidence.** The same two verbs,
/// with a save and a reopen between them — which is exactly what two runs of
/// `pdfcer.exe` do — and the character lands.
///
/// Without this, the test above would be equally consistent with "this fixture
/// cannot take that edit at all", and the request filed against the engine would
/// name the wrong subject. This project has filed two such requests in one week
/// on diagnoses that did not survive re-measurement.
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

/// ★★★ **The words the operator typed survive the refusal that threw the draft
/// away** — the round trip [`super::Committing`] promises, asserted rather than
/// argued.
///
/// `super::plan` is the one function every text commit passes through, and it
/// writes the slot; `panels::properties::refusedchar::record` reads it back. The
/// hazard the assertion covers is not exotic — it is a slot written before the
/// operands are known, or written for the wrong run, either of which would make
/// the offer re-apply *something else* into the operator's document, silently
/// and with an undo entry. That is a worse outcome than the two-gesture route
/// this replaced.
///
/// ★ Driven through the real `plan`, on this repository's own fixture, rather
/// than by poking the thread-local: the claim is about what `plan` does, and a
/// test that set the slot itself would pass on a build where `plan` had stopped
/// setting it.
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
