//! # `panels::properties::refusedchar` — the refusal, the character it names,
//! and the face that can type it
//!
//! `OPERATOR_REQUESTS.md` **O141**, 2026-09-05. The operator, while trying to
//! fix a typo:
//!
//! > *"if the character isn't available in a pdf are we able to change to a
//! > different font?"*
//!
//! **Yes**, and on the day he asked it every piece already existed:
//!
//! | piece | where it already was |
//! |---|---|
//! | the engine refuses by name, before touching bytes | `text_edit::encoding` — `CompositeEncoding::encode_str`, and the simple-font arm's four `Refusal` sites |
//! | the refusal carries **which character** | `Refusal::character`, `Option<char>`, public |
//! | a face chooser that offers the standard fourteen | [`super::face`], shipped `Pass 162.0`, drawn in two surfaces since 2026-08-29 |
//! | `set_font` accepting a face the page does not carry | measured on his own file: `set_font=AAAAAA+Arimo-Bold->Helvetica-Bold`, then the `€` went in |
//!
//! **Nothing connected the refusal to the chooser.** This module is that
//! connection, and it is the whole of what O141 asked to be built.
//!
//! ## ★★★ Why a PANEL block and not a sentence in the status bar
//!
//! [`crate::app::status::decline`] already words the refusal, and it cannot
//! carry this one. `Declined::line` returns `&'static str` — so the slot cannot
//! interpolate the character the engine named — and `disclosure_line` truncates
//! what it draws to 45 % of the bar and hangs the rest on hover. A route that
//! ends in a hover is a route the operator does not find.
//!
//! [`super::disclose`]'s header settled this exact question for the text tools'
//! other refusals and its answer transfers word for word: *"A dock panel's
//! width is the dock's, decided before the body draws, so text wrapped inside it
//! cannot drive a width and R128 does not apply"*, and *"a refusal is a fact
//! about the last thing the operator tried to do to the document in front of
//! them. Properties is where this application already puts facts of that
//! shape."* The bar keeps its elided line — reworded to name the obstacle and
//! point here — and the readable copy, the character, and the control live in
//! the panel.
//!
//! ## ★★ Rule 4, and the half of it that binds here
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 is *fuzzy,
//! never sneaky*: pdfcer may make an inference and may never make one silently.
//! It **forbids marking the drawing** and **requires** the report off the
//! canvas.
//!
//! Swapping the face is the operator's own instruction, so the changed
//! letterforms are not pdfcer marking its own uncertainty: nothing here badges,
//! tints or outlines a substituted letter, and the editing canvas goes on
//! looking exactly like the saved file.
//!
//! But a standard-14 face is a **name**, not a font program — the engine says so
//! itself, *"no font program is embedded and no bytes of glyph outline were
//! added"* — so the client's reader supplies the letterforms. **That is the part
//! the operator cannot see on his own screen**, and it is exactly the case rule
//! 4's surviving half exists for. It is discharged twice, off the canvas, both
//! times before he can act on it:
//!
//! 1. [`crate::text::panels::face::face_addable_disclosure`], drawn as a visible
//!    label in this block, above the chooser — `REVIEW_TRIAGE.md`'s rule that a
//!    disclosure sits above the thing it qualifies, because *"a caveat below a
//!    list arrives after the operator has already drawn a conclusion"*;
//! 2. the same sentence again inside the chooser's popup, which
//!    [`super::face::popup_body`] has drawn since the day the fourteen were
//!    offered.
//!
//! And a third time after the fact, without a line of code here: the engine's
//! own `format_text` disclosure — *"'Helvetica-Bold' was NOT a font resource
//! here, so pdfcer ADDED one as `/pdfceF6` … no font program is embedded"* —
//! arrives verbatim in [`super::disclose`], four lines above this block.
//!
//! ## ★★ The offer is UNTESTED against the character, and says so
//!
//! `preview_font_resources` coverage-tests the characters **already in** the
//! run, not the one the operator is about to type, so a row here can be a face
//! that then refuses their `€`. [`super::face`]'s header carries the reason this
//! is not silently filtered — `FontPreflight`'s `R221` forbids this crate
//! re-deriving the encoding rule, and a second copy would drift from the commit
//! path — and the standing ruling on this surface is the Bold button's: *offer
//! it, and surface the disclosure*. That is
//! [`crate::text::panels::face::refused_char_untested`], drawn beside the
//! chooser.
//!
//! ## The state machine, and why it is three states rather than one
//!
//! ```text
//!   (nothing)  --refusal naming a character-->  Offer
//!   Offer      --the operator picks a face--->  Swapped     (the edit epoch moved)
//!   Offer      --any other document change-->   (nothing)
//!   Swapped    --any document change-------->   (nothing)
//! ```
//!
//! **`Swapped` is not a courtesy.** The face swap is itself an edit, so it
//! retires the offer — and a block that vanished at that moment would leave the
//! operator at a caret with nothing telling them to type the character again,
//! which is a route that ends one gesture short of what it promised. It is also
//! the state in which the operator most needs a sentence, because on a
//! metric-compatible swap (his own file moved by **0.005 pt**) the page looks
//! exactly as it did and nothing on screen says the font changed.
//!
//! ★ Retirement is `doc.edit_epoch`, the number every other stamped read in this
//! panel uses, and it is honest in both directions: an undo moves it, a save does
//! not, and any edit the operator makes instead of taking the offer ends the
//! offer — which is right, because the refusal it reports is then two gestures
//! ago.
//!
//! ## Where the two halves are written and read
//!
//! The refusal is recorded by `crate::app::status::decline::textedit`, from
//! inside `vector_edit`'s closure, at the same moment it writes the status bar's
//! sentence — one event, two surfaces, one classification. It travels through
//! [`PENDING`], a thread-local, for the reason
//! [`crate::app::status::decline`]'s own store is one: the writer is the
//! dispatcher and the reader is a body that is handed `&OpenDoc` **shared**, so
//! there is no `&mut` path between them and inventing one would trade
//! `panels`' founding invariant for a parameter.

use std::cell::RefCell;

use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::text::panels::face as t;

/// The block itself, published on the frames it draws.
///
/// ★ Published only when it draws, so its **absence** is the evidence that no
/// character was refused — which is the distinction a driven check about this
/// feature is actually asking about, and the one a region declared
/// unconditionally could never provide.
pub const REGION: &str = "properties.refusedchar"; // ui-text-exempt: trace region name, never displayed

/// The chooser's combo — the control that opens the face list.
pub const FACE_REGION: &str = "properties.refusedchar.face"; // ui-text-exempt: trace region name, never displayed

/// The rule-4 disclosure, drawn above the chooser.
///
/// ★ Its own region rather than a clause of [`REGION`], because *"the sentence
/// reached a rectangle"* is the one thing about this feature that no unit test
/// in the workspace can observe: the string is catalogued and asserted, and a
/// build that drew it off the bottom of the panel would pass every one of those
/// assertions.
pub const DISCLOSURE_REGION: &str = "properties.refusedchar.disclosure"; // ui-text-exempt: trace region name, never displayed

/// What the engine refused, and what pdfcer did about it.
///
/// `character` and `base_font` are the engine's own — `Refusal::character` and
/// `Refusal::base_font`, read verbatim. Nothing here re-derives either.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefusedCharacter {
    /// The page the refused edit named.
    page: usize,
    /// The run the caret was pinned in.
    run: usize,
    /// **The character the engine could not encode.** `Refusal::character`.
    character: char,
    /// The `/BaseFont` the edit was refused against, subset tag and all.
    /// `Refusal::base_font`.
    base_font: String,
}

thread_local! {
    /// The refusal waiting to be adopted by the panel, written by the
    /// dispatcher and taken by the first body that draws after it.
    ///
    /// ★ A `take`, not a peek: adopting it stamps it against the epoch on
    /// screen, and a slot that kept handing the same refusal back would re-stamp
    /// it after every edit and never retire.
    static PENDING: RefCell<Option<RefusedCharacter>> = const { RefCell::new(None) };
}

/// **Record that an edit was refused because the run's font has no code for one
/// character** — the one entry point, called from
/// `crate::app::status::decline::textedit::record_edit_text_refusal`.
///
/// # ★★ Why the caller reads a field off `EditError` and this is not a second
/// taxonomy
///
/// `crate::app::status::decline::textedit`'s header forbids two shortcuts:
/// matching on `EditError`'s variants to *derive the operator's reason*, and
/// grepping its `Display` prose. Neither happens. The category still comes from
/// `RefusalKind`, which the engine made non-`#[non_exhaustive]` so a front end
/// can match it and have the compiler prove the sentences complete; what is read
/// here is **one datum that the coarse kind structurally cannot carry**, on the
/// identical licence `one_operator` already has.
pub(crate) fn record(page: usize, run: usize, character: char, base_font: String) {
    PENDING.with_borrow_mut(|slot| {
        *slot = Some(RefusedCharacter {
            page,
            run,
            character,
            base_font,
        });
    });
}

/// **Drop a refusal that has not been adopted yet**, called by
/// `PanelsState::forget_document`.
///
/// ★★ `PanelsState::forget_document` resets that struct whole, which clears
/// [`RefusedCharUi`] — and [`PENDING`] lives outside it, so without this a
/// refusal recorded on the frame a document was closed would be adopted by the
/// **next** document's first panel draw. `(page, run)` names different text
/// there, and the offer would aim a face swap at a run nobody asked about. The
/// same hazard `PanelsState::bookmarks`' own note records for an `ObjId`,
/// arriving through the one field that reset does not reach.
pub(crate) fn forget_document() {
    PENDING.with_borrow_mut(|slot| *slot = None);
}

/// The panel-side state: which refusal is live, the revision it was live for,
/// and whether the operator has taken the offer.
///
/// Held on `PanelsState` for that struct's own stated rule — a panel body is
/// handed `&OpenDoc`, shared, and this is the operator's state rather than a
/// derived cache of the document's. It is reset with the document by
/// `PanelsState::forget_document`, which matters here for the reason a bookmark
/// parent does: a `(page, run)` pair names different text in a different file,
/// so an offer carried across would restyle a run nobody asked about.
#[derive(Default)]
pub struct RefusedCharUi {
    /// The refusal being reported, or `None` when there is nothing to say.
    shown: Option<RefusedCharacter>,
    /// The `doc.edit_epoch` [`Self::shown`] was adopted at, or last advanced to.
    epoch: u64,
    /// The face the operator picked **from this block**, held from the frame
    /// they clicked until the edit lands.
    ///
    /// ★ It is what tells the *offer's* retirement from the *swap's*. Both look
    /// identical from the outside — the epoch moved — and only this block knows
    /// which of the two it caused.
    taken: Option<String>,
    /// The face now in force, once the swap has landed. `Some` is the `Swapped`
    /// state; see the module header's table.
    swapped_to: Option<String>,
    /// The `(page, run, epoch)` [`Self::faces`] was read at.
    ///
    /// ★★ The stamp is not an optimisation here, it is the difference between a
    /// usable application and an unusable one. Filling the list costs
    /// `pin::inspect` plus `preview_font_resources` — **392 ms** for the first
    /// alone on the operator's benchmark sheet — and a block that re-read it
    /// every frame would hold the whole program under three frames a second for
    /// as long as the refusal was on screen.
    faces_stamp: Option<(usize, usize, u64)>,
    /// The faces `set_font` would accept for this run, plus the fourteen pdfcer
    /// would author. [`super::face::choices`] builds it; nothing here filters it.
    faces: Vec<super::face::FaceChoice>,
}

impl RefusedCharUi {
    /// Advance the state machine for this frame, and answer what to draw.
    ///
    /// Split from [`section`] so the transitions can be read — and tested —
    /// without a frame. Every arm is one of the four rows in the module header's
    /// table.
    fn advance(&mut self, epoch: u64) -> bool {
        if let Some(next) = PENDING.with_borrow_mut(Option::take) {
            self.shown = Some(next);
            self.epoch = epoch;
            self.taken = None;
            self.swapped_to = None;
            self.faces_stamp = None;
            return true;
        }
        if self.shown.is_none() {
            return false;
        }
        if self.epoch == epoch {
            return true;
        }
        // The document changed under a live block. Exactly one such change is
        // ours — the face swap this block offered — and every other one ends the
        // report, because a refusal two gestures ago is not a fact about what
        // the operator just did.
        match self.taken.take() {
            Some(face) => {
                self.epoch = epoch;
                self.swapped_to = Some(face);
                true
            }
            None => {
                self.clear();
                false
            }
        }
    }

    /// Forget the whole report.
    fn clear(&mut self) {
        self.shown = None;
        self.taken = None;
        self.swapped_to = None;
        self.faces_stamp = None;
        self.faces.clear();
    }
}

/// Draw the offer, and say whether it drew.
///
/// Returns `false` on every frame where no character has been refused, which is
/// nearly all of them — and it renders **nothing at all** in that case, heading
/// included, on [`super::disclose`]'s rule: *"a heading present on every frame
/// trains an operator to stop reading the region under it, which would waste the
/// one surface a disclosure has."*
///
/// ★ Its answer is deliberately **not** folded into `body_sections`'
/// `something_drew`. That predicate is O75's and asks whether a
/// **selection**-scoped section has spoken; a refusal from the last edit is not
/// a description of the current selection, and letting it collapse the object
/// section would make the panel change shape for a reason unconnected to what is
/// picked. [`super::disclose`]'s call site records the identical exclusion for
/// the identical reason.
pub(super) fn section(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    state: &mut RefusedCharUi,
    actions: &mut Vec<Action>,
) -> bool {
    if !state.advance(doc.edit_epoch) {
        return false;
    }
    let Some(refused) = state.shown.clone() else {
        return false;
    };
    let font = super::text::shorten(&refused.base_font).to_owned();

    ui.label(t::refused_char_heading());
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());

    // ★ The face list is filled BEFORE the trace, not after, and the ordering is
    // load-bearing rather than tidy. `sync_faces` runs once per
    // `(page, run, epoch)`, so on the frame a refusal is adopted a trace written
    // first reports `faces=0` — which is exactly what a build whose pre-flight
    // returned nothing would report, on every frame. Two states that are not the
    // same must not print the same line.
    if state.swapped_to.is_none() {
        sync_faces(doc, state, &refused);
    }

    // ★★★ THE TRACE LINE. The harness cannot read rendered text — there is no
    // accessibility reader and no OCR — so the region above says the block drew
    // and this says *what it drew about*. Without it a check could not tell a
    // build that names the character from one that draws the heading over an
    // empty offer, which is the exact difference O141 is about.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "refused-char page={} run={} character={:?} font={font} faces={} state={}",
            refused.page,
            refused.run,
            refused.character,
            state.faces.len(),
            if state.swapped_to.is_some() {
                "swapped"
            } else {
                "offer"
            }
        )
    });

    if let Some(face) = state.swapped_to.clone() {
        // The follow-up. No chooser: the face is already changed, and a second
        // list here would invite the operator to change it again instead of
        // doing the one thing left to do.
        ui.label(t::refused_char_swapped(refused.character, &face));
        ui.separator();
        return true;
    }

    ui.label(t::refused_char_named(refused.character, &font));

    // ★★★ RULE 4's off-canvas report, ABOVE the control it qualifies. See the
    // module header: the letterforms of an added standard-14 face come from the
    // reader's own copy, which is the one consequence the operator cannot see by
    // looking at their own screen.
    //
    // ★ It is drawn unconditionally rather than gated on the list containing an
    // addable row, and that differs from `face::popup_body` deliberately. There
    // the fourteen are one of two groups and the sentence is untrue of the other;
    // here the block exists **because** the page's own faces could not take this
    // character, so the fourteen are the answer in every case that reaches this
    // line — and a disclosure that appeared only sometimes would be one the
    // operator learns to skip.
    let note = ui.label(egui::RichText::new(t::face_addable_disclosure()).small());
    crate::diag::ui_rect_visible(DISCLOSURE_REGION, note.rect, ui.clip_rect());
    ui.label(egui::RichText::new(t::refused_char_untested(refused.character)).small());

    ui.label(t::refused_char_offer(refused.character));
    let mut chosen = None;
    let combo = egui::ComboBox::from_id_salt("properties-refusedchar-face")
        .selected_text(font.clone())
        .show_ui(ui, |ui| {
            // ★★ The SAME popup body the Properties panel's *This text* section
            // and the ribbon's Format ▸ Font group draw, prefix and all. A third
            // copy of that loop is how a face gets offered in one surface and not
            // another, which is the divergence `super::face` exists to end — and
            // it would be a third place the rule-4 disclosure could go missing.
            chosen = super::face::popup_body(ui, FACE_REGION, &state.faces, &font);
        });
    crate::diag::ui_rect_visible(FACE_REGION, combo.response.rect, ui.clip_rect());

    if let Some(selector) = chosen {
        // ★ Raised outside the popup closure, because nothing mutates from a
        // widget — `app::actions`' founding invariant, and the rule
        // `properties::text::face_row` already follows.
        //
        // ★★ `runs: vec![refused.run]` — the run the REFUSAL named, not the
        // current selection. The caret is gone by now: `Ctrl+Enter` calls
        // `commit_into` and then `abandon`, whether or not the engine accepted,
        // so by the time this block is on screen there is nothing selected to
        // borrow a run index from. Carrying the pair through the report is what
        // lets the offer work at all.
        state.taken = Some(super::text::shorten(&selector).to_owned());
        actions.push(Action::TextStyle {
            page: refused.page,
            runs: vec![refused.run],
            change: StyleChange::Face(selector),
        });
    }
    ui.separator();
    true
}

/// Fill [`RefusedCharUi::faces`] when the stamp has moved, and otherwise keep
/// what is there.
///
/// # ★★ Why this block asks for its own pre-flight rather than borrowing the
/// panel's
///
/// `properties::text::TextStyleDraft` already holds a face list, and it is the
/// wrong one twice over. It is stamped on the **selection**, which is empty here
/// — the caret was abandoned by the commit that got refused — and even where a
/// selection survives, it need not be the run the refusal named. Borrowing it
/// would make the offer's rows depend on what happens to be selected, which is a
/// list that is right most of the time and silently wrong the rest.
///
/// The cost is one extraction with provenance capture plus one pre-flight, paid
/// once per `(page, run, epoch)` and only while a refusal is on screen.
fn sync_faces(doc: &OpenDoc, state: &mut RefusedCharUi, refused: &RefusedCharacter) {
    let stamp = (refused.page, refused.run, doc.edit_epoch);
    if state.faces_stamp == Some(stamp) {
        return;
    }
    state.faces_stamp = Some(stamp);
    state.faces = crate::canvas::textedit::pin::inspect(doc, refused.page, refused.run)
        .and_then(|read| crate::canvas::textedit::pin::font_preflight(doc, refused.page, &read))
        .as_ref()
        .map_or_else(Vec::new, |preflight| super::face::choices(Some(preflight)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refusal() -> RefusedCharacter {
        RefusedCharacter {
            page: 1,
            run: 4,
            character: '€',
            base_font: "AAAAAA+Arimo-Bold".to_owned(),
        }
    }

    /// Clear the thread-local between cases, so one test's leftover is never
    /// another's subject.
    fn drain() {
        PENDING.with_borrow_mut(|slot| *slot = None);
    }

    /// ★★ **A recorded refusal is adopted once and stamped against the
    /// revision on screen.**
    ///
    /// The `take` is the property: a slot that kept handing the same refusal
    /// back would re-stamp it after every edit, so the block would never retire
    /// and the operator would read a sentence about a gesture from ten minutes
    /// ago.
    #[test]
    fn a_recorded_refusal_is_adopted_exactly_once() {
        drain();
        record(1, 4, '€', "AAAAAA+Arimo-Bold".to_owned());
        let mut ui = RefusedCharUi::default();
        assert!(ui.advance(7), "the refusal must be adopted");
        assert_eq!(ui.shown, Some(refusal()));
        assert_eq!(ui.epoch, 7);
        assert!(
            PENDING.with_borrow(Option::is_none),
            "the slot must be emptied, or the next edit re-adopts it"
        );
    }

    /// ★★★ **An edit the operator makes INSTEAD of taking the offer ends the
    /// offer.**
    ///
    /// The refusal it reports is then two gestures ago, and
    /// `app::status::decline`'s own retirement rule is the precedent: a sentence
    /// about an earlier press, read after a later one, is a small lie told
    /// confidently.
    #[test]
    fn an_unrelated_edit_retires_the_offer() {
        drain();
        record(1, 4, '€', "AAAAAA+Arimo-Bold".to_owned());
        let mut ui = RefusedCharUi::default();
        assert!(ui.advance(7));
        assert!(!ui.advance(8), "the epoch moved and nothing here caused it");
        assert_eq!(ui.shown, None);
    }

    /// ★★★ **Taking the offer moves to the follow-up rather than retiring**,
    /// and this is the transition the whole route rests on.
    ///
    /// The face swap is itself an edit, so from the outside it is
    /// indistinguishable from the case above — the epoch moved. Only this block
    /// knows it caused it. Without `taken` the offer would vanish at exactly the
    /// moment the operator needs to be told to type the character again, and the
    /// route would end one gesture short of the thing it promised.
    #[test]
    fn taking_the_offer_leaves_the_follow_up_behind() {
        drain();
        record(1, 4, '€', "AAAAAA+Arimo-Bold".to_owned());
        let mut ui = RefusedCharUi::default();
        assert!(ui.advance(7));
        ui.taken = Some("Helvetica-Bold".to_owned());
        assert!(ui.advance(8), "the swap must leave the block on screen");
        assert_eq!(ui.swapped_to.as_deref(), Some("Helvetica-Bold"));
        assert_eq!(ui.epoch, 8);
        assert!(
            ui.taken.is_none(),
            "the swap is consumed, so the NEXT edit retires the block"
        );
    }

    /// ★★★ **And the successful re-type retires it** — the second half of the
    /// same property, and the one that gives the block dynamic range.
    ///
    /// A follow-up that survived every subsequent edit would be a permanent
    /// *"type it again"* under a document where it had already gone in, which is
    /// the same defect class as a decline that never retires.
    #[test]
    fn the_edit_that_lands_retires_the_follow_up() {
        drain();
        record(1, 4, '€', "AAAAAA+Arimo-Bold".to_owned());
        let mut ui = RefusedCharUi::default();
        assert!(ui.advance(7));
        ui.taken = Some("Helvetica-Bold".to_owned());
        assert!(ui.advance(8));
        assert!(
            !ui.advance(9),
            "the character went in; there is nothing left to say"
        );
        assert_eq!(ui.shown, None);
        assert_eq!(ui.swapped_to, None);
    }

    /// ★ **A second refusal replaces the first**, rather than queuing behind it.
    ///
    /// `app::status::decline`'s slot rule, and it matters more here: an operator
    /// who meets two different missing characters in a row must be offered a face
    /// for the second one, and a block still naming the first would send them to
    /// a chooser aimed at the wrong run.
    #[test]
    fn a_second_refusal_replaces_the_first() {
        drain();
        record(1, 4, '€', "AAAAAA+Arimo-Bold".to_owned());
        let mut ui = RefusedCharUi::default();
        assert!(ui.advance(7));
        record(2, 9, 'q', "AAAAAA+Arimo-Bold".to_owned());
        assert!(ui.advance(7));
        let shown = ui.shown.clone().expect("the second refusal is live");
        assert_eq!(shown.character, 'q');
        assert_eq!((shown.page, shown.run), (2, 9));
        assert!(
            ui.faces_stamp.is_none(),
            "the face list belongs to the run it was read for, and this is a different run"
        );
    }

    /// **Nothing recorded, nothing drawn.** R9 and [`super::disclose`]'s rule:
    /// no heading, no empty state, no region.
    #[test]
    fn nothing_is_shown_when_nothing_was_refused() {
        drain();
        let mut ui = RefusedCharUi::default();
        assert!(!ui.advance(0));
        assert!(!ui.advance(1));
    }

    /// ★★★ **THE WHOLE CHAIN, DRAWN** — the refusal recorded, the panel run for
    /// real, and a face that can type the character offered at the end of it.
    ///
    /// # Why this is not another state-machine test
    ///
    /// Every test above it exercises [`RefusedCharUi::advance`], which is the
    /// part that is easy to get right and easy to test. **None of them would
    /// fail on a build where `body_sections` never calls [`section`]**, where
    /// the pre-flight is never asked, or where `choices` answers an empty list —
    /// and this project's standing lesson is exactly that: *eight green unit
    /// tests once passed while the feature performed one of fourteen steps.*
    ///
    /// So this runs the **real** `panels::properties::body` through
    /// `Context::run_ui`, on the real fixture, and reads what the frame left
    /// behind. `panels::comments::tests` established the technique and its
    /// header carries the argument in full.
    ///
    /// # What each assertion would catch
    ///
    /// | assertion | the build it fails on |
    /// |---|---|
    /// | the refusal was adopted | `body_sections` does not call [`section`] at all — the capability built and unreached, which is O141's own subject |
    /// | the face list is non-empty | [`sync_faces`] asked the engine and got nothing: a pin that did not resolve, or a pre-flight that refused |
    /// | at least fourteen rows are `PdfcerWouldAdd` | the offer is built from **the page's own fonts**, which is the offer that cannot work: this block only exists because those faces refused the character |
    ///
    /// ★ The third is the one that matters most and it is a real risk rather
    /// than a hypothetical: `preview_font_resources` enumerates the page's own
    /// `/Font` resources, and a chooser built from `accepted()` alone was
    /// precisely the state the shell shipped in until 2026-08-29 — the engine's
    /// standard-14 authoring present, released, and unreachable from any
    /// surface.
    ///
    /// # ★★ Two frames, not one
    ///
    /// `panels::dimension_groups`' reason, which `panels::comments::tests`
    /// repeats: an immediate-mode layout's first pass is a guess and the scroll
    /// area's size settles on the second. This panel is one `ScrollArea` and
    /// [`section`] publishes through `ui_rect_visible`, which answers nothing
    /// for a rect outside a clip that has not settled.
    #[test]
    fn the_offer_reaches_the_panel_and_carries_faces_the_page_does_not_have() {
        drain();
        let doc = crate::app::state::open_local_fixture("subset-font-floor.pdf");
        let mut state = crate::panels::PanelsState::default();
        let mut actions = Vec::new();
        // The refusal the engine raises for this exact fixture, measured with
        // `pdfcer.exe` before this test was written and recorded in
        // `fixtures/subset-font-floor.PROVENANCE.md`: `R-INV-1 (embedded-subset
        // floor): character U+0071 'q' … which font 'SUBSET+pdfceSubsetDemo'
        // (an embedded SUBSET) does not already carry on this page`.
        record(0, 0, 'q', "SUBSET+pdfceSubsetDemo".to_owned());

        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(360.0, 900.0),
            )),
            ..Default::default()
        };
        for _ in 0..2 {
            let _ = ctx.run_ui(input.clone(), |ui| {
                crate::panels::properties::body(ui, &doc, &mut state, &mut actions);
            });
        }

        let ui = state.refused_char_mut();
        let shown = ui.shown.clone().expect(
            "the panel drew and did not adopt the refusal — `body_sections` is not calling \
             `refusedchar::section`, which is the capability-built-and-unreached shape O141 is \
             itself about",
        );
        assert_eq!(shown.character, 'q');
        assert!(
            !ui.faces.is_empty(),
            "the offer named the character and had no face to offer, so the route ends in a \
             sentence: `sync_faces` asked `preview_font_resources` and got nothing"
        );
        let addable = ui
            .faces
            .iter()
            .filter(|face| {
                face.origin == crate::panels::properties::face::FaceOrigin::PdfcerWouldAdd
            })
            .count();
        assert!(
            addable >= 14,
            "the offer holds {addable} face(s) pdfcer would ADD, and this page carries none of \
             the standard fourteen — so a list built from the page's own resources is a list \
             whose every row already refused this character. Rows: {:?}",
            ui.faces
        );
    }

    /// ★ **The three regions are distinct and none of them is another
    /// surface's.**
    ///
    /// Worth asserting because the face chooser's regions are namespaced by a
    /// prefix passed in at the call site, and a prefix copied from
    /// `properties::text` would make this block's popup indistinguishable from
    /// the *This text* section's in a trace — so a driven check would read the
    /// wrong control's rectangle and click it.
    #[test]
    fn the_regions_name_this_block_and_not_the_text_section() {
        assert_eq!(REGION, "properties.refusedchar");
        assert!(FACE_REGION.starts_with(REGION));
        assert!(DISCLOSURE_REGION.starts_with(REGION));
        assert_ne!(FACE_REGION, super::super::text::FACE_REGION);
        assert!(!FACE_REGION.starts_with("properties.text"));
    }
}
