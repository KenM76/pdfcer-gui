//! # `panels::properties::refusedchar` — the refusal, the character it names,
//! and the face that can type it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/refusedchar.md`.

use std::cell::RefCell;

use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::text::panels::face as t;

/// The block itself, published on the frames it draws.
pub const REGION: &str = "properties.refusedchar"; // ui-text-exempt: trace region name, never displayed

/// The chooser's combo — the control that opens the face list.
pub const FACE_REGION: &str = "properties.refusedchar.face"; // ui-text-exempt: trace region name, never displayed

/// The rule-4 disclosure, drawn above the chooser.
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
    /// The font maps the character from two codes, rather than lacking it.
    two_ways: bool,
    /// **The words the operator typed, which the refusal threw away.**
    typed: Option<crate::canvas::textedit::Committing>,
}

thread_local! {
    /// The refusal waiting to be adopted by the panel, written by the
    /// dispatcher and taken by the first body that draws after it.
    static PENDING: RefCell<Option<RefusedCharacter>> = const { RefCell::new(None) };
}

/// **Record that an edit was refused because the run's font has no code for one
/// character** — the one entry point, called from
/// `crate::app::status::decline::textedit::record_edit_text_refusal`.
pub(crate) fn record(
    page: usize,
    run: usize,
    character: char,
    base_font: String,
    two_ways: bool,
    typed: Option<crate::canvas::textedit::Committing>,
) {
    PENDING.with_borrow_mut(|slot| {
        *slot = Some(RefusedCharacter {
            page,
            run,
            character,
            base_font,
            two_ways,
            typed,
        });
    });
}

/// **Drop a refusal that has not been adopted yet**, called by
/// `PanelsState::forget_document`.
pub(crate) fn forget_document() {
    PENDING.with_borrow_mut(|slot| *slot = None);
}

/// The panel-side state: which refusal is live, the revision it was live for,
/// and whether the operator has taken the offer.
#[derive(Default)]
pub struct RefusedCharUi {
    /// The refusal being reported, or `None` when there is nothing to say.
    shown: Option<RefusedCharacter>,
    /// The `doc.edit_epoch` [`Self::shown`] was adopted at, or last advanced to.
    epoch: u64,
    /// The face the operator picked **from this block**, held from the frame
    /// they clicked until the edit lands.
    ///
    /// It is what tells the *offer's* retirement from the *swap's*. Both look
    /// identical from the outside — the epoch moved — and only this block knows
    /// which of the two it caused.
    taken: Option<String>,
    /// The face now in force, once the swap has landed. `Some` is the `Swapped`
    /// state; see the module header's table.
    swapped_to: Option<String>,
    /// **Whether the operator's own edit has been re-applied since the swap.**
    ///
    /// The one bit that tells the `Swapped` state from the `Blocked` one,
    /// and it does so **without reading the engine's refusal**. The retype is
    /// raised on the frame the swap is detected; from the next frame on there
    /// are only two possibilities, and `advance` has already sorted them: if the
    /// retype landed, the epoch moved a second time with nothing `taken`, the
    /// block cleared, and this state does not exist. So a block that is still
    /// drawing with this set is a block whose retype was refused — arithmetic
    /// over `doc.edit_epoch`, not a `Display` string matched against a substring.
    retried: bool,
    /// The `(page, run, epoch)` [`Self::faces`] was read at.
    ///
    /// The stamp is not an optimisation here, it is the difference between a
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
    fn advance(&mut self, epoch: u64) -> bool {
        if let Some(next) = PENDING.with_borrow_mut(Option::take) {
            self.shown = Some(next);
            self.epoch = epoch;
            self.taken = None;
            self.swapped_to = None;
            self.retried = false;
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
        self.retried = false;
        self.faces_stamp = None;
        self.faces.clear();
    }
}

/// Draw the offer, and say whether it drew.
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

    // The face list is filled BEFORE the trace, not after, and the ordering is
    // load-bearing rather than tidy. `sync_faces` runs once per
    // `(page, run, epoch)`, so on the frame a refusal is adopted a trace written
    // first reports `faces=0` — which is exactly what a build whose pre-flight
    // returned nothing would report, on every frame. Two states that are not the
    // same must not print the same line.
    if state.swapped_to.is_none() {
        sync_faces(doc, state, &refused);
    }

    // THE TRACE LINE. The harness cannot read rendered text — there is no
    // accessibility reader and no OCR — so the region above says the block drew
    // and this says *what it drew about*. Without it a check could not tell a
    // build that names the character from one that draws the heading over an
    // empty offer, which is the exact difference O141 is about.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "refused-char page={} run={} character={:?} font={font} faces={} state={} two_ways={}",
            refused.page,
            refused.run,
            refused.character,
            state.faces.len(),
            // THREE states on this field, not two, since the retype became
            // the block's own job. `swapped` is the frame the face landed and
            // the operator's edit was re-raised; `blocked` is a frame after that
            // with the block still drawing, which by `RefusedCharUi::retried`'s
            // argument is the retype having been refused. A check that could not
            // tell those apart would read a working build and a stuck one the
            // same way.
            match (state.swapped_to.is_some(), state.retried) {
                (true, true) => "blocked",
                (true, false) => "swapped",
                (false, _) => "offer",
            },
            u8::from(refused.two_ways)
        )
    });

    if let Some(face) = state.swapped_to.clone() {
        // The follow-up. No chooser: the face is already changed, and a second
        // list here would invite the operator to change it again instead of
        // letting the one thing left to do happen.
        //
        // The refusal arrived with no carried words, so there is nothing to
        // put back and the block says the one true thing left: type it again.
        // It does NOT enter the retry states — `retried` would then make the
        // next frame claim the engine refused a retype that was never made,
        // which is a confident wrong reason about a document that is fine.
        let Some(typed) = refused.typed.clone() else {
            ui.label(t::refused_char_swapped_type_again(refused.character, &face));
            ui.separator();
            return true;
        };
        if state.retried {
            // The retype was raised on the previous frame and the block is
            // still here, which — see [`RefusedCharUi::retried`] — is the retype
            // having been refused. The sentence names the measured cause and the
            // remedy that was actually run.
            ui.label(t::refused_char_blocked(refused.character, &face));
            ui.separator();
            return true;
        }
        ui.label(t::refused_char_swapped(refused.character, &face));
        // **O141's second half: the offer finishes the job.** The face is
        // in force as of this frame, so the edit the operator already typed is
        // re-applied now, with no second gesture from him.
        //
        // Raised as `Action::CommitTextEdit` — the same variant `Ctrl+Enter`
        // raises, with the same four operands — rather than through a new verb,
        // so the retype takes the identical route: `canvas::textedit::plan`
        // re-derives the pin and the follower disposition from the page **as it
        // is now**, which is the whole of `DEFECTS.md` D4b and is exactly what
        // must happen after a restyle rewrote the stream.
        //
        // Set on the same frame the action is pushed, and the two are one
        // act: from the next frame on, a block still drawing is a block whose
        // retype came back refused. That is the whole discriminator, and it is
        // arithmetic over `doc.edit_epoch` rather than a reading of the
        // engine's prose — see [`RefusedCharUi::retried`].
        state.retried = true;
        actions.push(Action::CommitTextEdit {
            page: typed.page,
            run: typed.run,
            original: typed.original,
            replacement: typed.replacement,
            reface: None,
            workarounds: false,
        });
        ui.separator();
        return true;
    }

    ui.label(if refused.two_ways {
        t::refused_char_drawn_two_ways(refused.character, &font)
    } else {
        t::refused_char_named(refused.character, &font)
    });

    // RULE 4's off-canvas report, ABOVE the control it qualifies. See the
    // module header: the letterforms of an added standard-14 face come from the
    // reader's own copy, which is the one consequence the operator cannot see by
    // looking at their own screen.
    //
    // It is drawn unconditionally rather than gated on the list containing an
    // addable row, and that differs from `face::popup_body` deliberately. There
    // the fourteen are one of two groups and the sentence is untrue of the other;
    // here the block exists **because** the page's own faces could not take this
    // character, so the fourteen are the answer in every case that reaches this
    // line — and a disclosure that appeared only sometimes would be one the
    // operator learns to skip.
    // AN EMPTY OFFER IS A SENTENCE — not a heading above a combo with nothing
    // in it.
    //
    // The list is coverage-tested against the refused character itself, so every
    // row in it is a face that WILL take the character — and for a character
    // outside `WinAnsiEncoding` there are none, because no standard-14 face can
    // encode one. That state is reachable, and drawing *"Pick a font that has
    // the 中"* above an empty control would be an instruction the operator
    // cannot follow.
    //
    // ⇒ R9, in its ordinary form: an unavailable capability renders nothing.
    // What still renders is the REPORT — the refusal was already announced two
    // lines up, and stopping without saying why would leave that hanging.
    if !state
        .faces
        .iter()
        .any(|f| f.origin == crate::panels::properties::face::FaceOrigin::PdfcerWouldAdd)
    {
        let dead = ui
            .label(egui::RichText::new(t::refused_char_no_face(refused.character)).small())
            .rect;
        crate::diag::ui_rect_visible(DISCLOSURE_REGION, dead, ui.clip_rect());
        installed_letter(ui, doc, &refused, actions);
        ui.separator();
        return true;
    }

    let note = ui.label(egui::RichText::new(t::face_addable_disclosure()).small());
    crate::diag::ui_rect_visible(DISCLOSURE_REGION, note.rect, ui.clip_rect());

    ui.label(t::refused_char_offer(refused.character));
    let mut chosen = None;
    let combo = egui::ComboBox::from_id_salt("properties-refusedchar-face")
        .selected_text(font.clone())
        .show_ui(ui, |ui| {
            // The SAME popup body the Properties panel's *This text* section
            // and the ribbon's Format ▸ Font group draw, prefix and all. A third
            // copy of that loop is how a face gets offered in one surface and not
            // another, which is the divergence `super::face` exists to end — and
            // it would be a third place the rule-4 disclosure could go missing.
            chosen = super::face::popup_body(ui, FACE_REGION, &state.faces, &font);
        });
    crate::diag::ui_rect_visible(FACE_REGION, combo.response.rect, ui.clip_rect());

    if let Some(selector) = chosen {
        // Raised outside the popup closure, because nothing mutates from a
        // widget — `app::actions`' founding invariant, and the rule
        // `properties::text::face_row` already follows.
        //
        // `runs: vec![refused.run]` — the run the REFUSAL named, not the
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
    installed_letter(ui, doc, &refused, actions);
    ui.separator();
    true
}

/// The installed-font offer, for a letter refused at the keystroke only: a
/// commit-time refusal has no live draft to type it into.
fn installed_letter(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    refused: &RefusedCharacter,
    actions: &mut Vec<Action>,
) {
    if refused.typed.is_none() {
        super::installedletter::offer(
            ui,
            doc,
            refused.page,
            refused.run,
            refused.character,
            actions,
        );
    }
}

/// Fill [`RefusedCharUi::faces`] when the stamp has moved, and otherwise keep
/// what is there.
fn sync_faces(doc: &OpenDoc, state: &mut RefusedCharUi, refused: &RefusedCharacter) {
    let stamp = (refused.page, refused.run, doc.edit_epoch);
    if state.faces_stamp == Some(stamp) {
        return;
    }
    state.faces_stamp = Some(stamp);
    state.faces = crate::canvas::textedit::pin::inspect(doc, refused.page, refused.run)
        .and_then(|read| {
            // THE REFUSED CHARACTER IS THE CANDIDATE.
            //
            // This surface exists to answer one question: *"this character will
            // not go in; which face WILL take it?"* Coverage-testing the run's
            // EXISTING characters answers a different one, and answers it
            // confidently — every face that can hold the words already there,
            // including the ones that cannot hold the character the operator is
            // trying to type.
            //
            // ⇒ So the offer carries no caveat about untested rows: there are
            // none to admit to. Every row here has been tested against the
            // character it is being offered for.
            crate::canvas::textedit::pin::font_preflight(
                doc,
                refused.page,
                &read,
                Some(&refused.character.to_string()),
            )
        })
        .as_ref()
        .map_or_else(Vec::new, |preflight| super::face::choices(Some(preflight)));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words the operator typed, as `plan` would have recorded them for the
    /// refusal [`refusal`] describes.
    fn typed() -> crate::canvas::textedit::Committing {
        crate::canvas::textedit::Committing {
            page: 1,
            run: 4,
            original: "Interior Door".to_owned(),
            replacement: "Interior Door €".to_owned(),
        }
    }

    /// The fixture's one font, as the engine spells it in its own refusal.
    ///
    /// A constant rather than two string literals, because the name carries the
    /// engine's pre-rename resource prefix and each literal would need its own
    /// exemption marker on `tools/gates/check-old-name-absent.sh`. One marked
    /// line is better than several, and the gate's own guidance is that a
    /// reference which cannot be explained in one sentence is a miss.
    const FIXTURE_FONT: &str = "SUBSET+pdfceSubsetDemo"; // old-name-exempt: a PDF resource / BaseFont name the ENGINE writes into the file, quoted verbatim from its own output. It is data in the file format, not prose about this project, and the engine deliberately stopped the rename at that boundary because changing it would alter the bytes of every document pdfcer has ever produced. Same ruling as O141's.

    fn refusal() -> RefusedCharacter {
        RefusedCharacter {
            page: 1,
            run: 4,
            character: '€',
            base_font: "AAAAAA+Arimo-Bold".to_owned(),
            two_ways: false,
            typed: Some(typed()),
        }
    }

    /// Clear the thread-local between cases, so one test's leftover is never
    /// another's subject.
    fn drain() {
        PENDING.with_borrow_mut(|slot| *slot = None);
    }

    /// **A recorded refusal is adopted once and stamped against the
    /// revision on screen.**
    #[test]
    fn a_recorded_refusal_is_adopted_exactly_once() {
        drain();
        record(
            1,
            4,
            '€',
            "AAAAAA+Arimo-Bold".to_owned(),
            false,
            Some(typed()),
        );
        let mut ui = RefusedCharUi::default();
        assert!(ui.advance(7), "the refusal must be adopted");
        assert_eq!(ui.shown, Some(refusal()));
        assert_eq!(ui.epoch, 7);
        assert!(
            PENDING.with_borrow(Option::is_none),
            "the slot must be emptied, or the next edit re-adopts it"
        );
    }

    /// **An edit the operator makes INSTEAD of taking the offer ends the
    /// offer.**
    #[test]
    fn an_unrelated_edit_retires_the_offer() {
        drain();
        record(
            1,
            4,
            '€',
            "AAAAAA+Arimo-Bold".to_owned(),
            false,
            Some(typed()),
        );
        let mut ui = RefusedCharUi::default();
        assert!(ui.advance(7));
        assert!(!ui.advance(8), "the epoch moved and nothing here caused it");
        assert_eq!(ui.shown, None);
    }

    /// **Taking the offer moves to the follow-up rather than retiring**,
    /// and this is the transition the whole route rests on.
    #[test]
    fn taking_the_offer_leaves_the_follow_up_behind() {
        drain();
        record(
            1,
            4,
            '€',
            "AAAAAA+Arimo-Bold".to_owned(),
            false,
            Some(typed()),
        );
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

    /// **And the successful re-type retires it** — the second half of the
    /// same property, and the one that gives the block dynamic range.
    #[test]
    fn the_edit_that_lands_retires_the_follow_up() {
        drain();
        record(
            1,
            4,
            '€',
            "AAAAAA+Arimo-Bold".to_owned(),
            false,
            Some(typed()),
        );
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

    /// **A second refusal replaces the first**, rather than queuing behind it.
    #[test]
    fn a_second_refusal_replaces_the_first() {
        drain();
        record(
            1,
            4,
            '€',
            "AAAAAA+Arimo-Bold".to_owned(),
            false,
            Some(typed()),
        );
        let mut ui = RefusedCharUi::default();
        assert!(ui.advance(7));
        record(2, 9, 'q', "AAAAAA+Arimo-Bold".to_owned(), false, None);
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

    /// **THE WHOLE CHAIN, DRAWN** — the refusal recorded, the panel run for
    /// real, and a face that can type the character offered at the end of it.
    #[test]
    fn the_offer_reaches_the_panel_and_carries_faces_the_page_does_not_have() {
        drain();
        let doc = crate::app::state::open_local_fixture("subset-font-floor.pdf");
        let mut state = crate::panels::PanelsState::default();
        let mut actions = Vec::new();
        // The refusal the engine raises for this exact fixture, measured with
        // `pdfcer.exe` before this test was written and recorded in
        // `fixtures/subset-font-floor.PROVENANCE.md`: `R-INV-1 (embedded-subset
        // floor): character U+0071 'q' … which font [`FIXTURE_FONT`] (an
        // embedded SUBSET) does not already carry on this page`.
        record(0, 0, 'q', FIXTURE_FONT.to_owned(), false, None);

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
        let addable: Vec<&str> = ui
            .faces
            .iter()
            .filter(|face| {
                face.origin == crate::panels::properties::face::FaceOrigin::PdfcerWouldAdd
            })
            .map(|face| face.label.as_str())
            .collect();
        assert!(
            addable.len() >= 12,
            "the offer holds {} face(s) pdfcer would ADD, and this page carries none of \
             the standard fourteen — so a list built from the page's own resources is a list \
             whose every row already refused this character. Rows: {:?}",
            addable.len(),
            ui.faces
        );

        // **12, NOT 14 — and the two that are missing are the point of the
        // bound rather than a shortfall in it.**
        //
        // `panels::properties::face::choices` reads the engine's `standard_14`
        // survey, so a face whose own encoding cannot hold the character never
        // reaches the list. A `>= 14` bound here would assert the opposite.
        //
        // ⇒ `Symbol` and `ZapfDingbats` carry built-in font-specific encodings
        // that map codes to symbol glyphs; **neither can hold a `q`**. Offering
        // them here was offering the operator a fix that would refuse — on the
        // one surface whose entire job is *"this character will not go in; here
        // is a face that will"*. That is the O141 surface, and a wrong row on it
        // is worse than a short list.
        //
        // ⚠ Asserted by NAME, not by an exact count. `assert_eq!(len, 12)`
        // would catch the same regression today and would break for the wrong
        // reason the day the standard 14 gains a member — and it would not say
        // which face came back.
        for cannot_hold_a_q in ["Symbol", "ZapfDingbats"] {
            assert!(
                !addable.contains(&cannot_hold_a_q),
                "{cannot_hold_a_q} cannot encode {:?} under its built-in font-specific encoding, \
                 so `set_font` would refuse it — offering it on the very panel that exists to \
                 name a face that WORKS is the defect this surface is about. Offered: {addable:?}",
                shown.character
            );
        }
    }

    /// **THE OFFER IS TESTED AGAINST THE CHARACTER THAT WAS REFUSED, NOT
    /// AGAINST THE WORDS ALREADY IN THE RUN.**
    #[test]
    fn the_offer_is_coverage_tested_for_the_refused_character_not_the_runs_own_text() {
        drain();
        let doc = crate::app::state::open_local_fixture("subset-font-floor.pdf");
        let mut state = crate::panels::PanelsState::default();
        let mut actions = Vec::new();

        // Same run, same fixture, same route as the sibling test — only the
        // character differs, which is what makes the comparison meaningful.
        record(0, 0, '中', FIXTURE_FONT.to_owned(), false, None);

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
        let shown = ui
            .shown
            .clone()
            .expect("the panel must adopt the refusal exactly as it does for 'q'");
        assert_eq!(shown.character, '中');

        let addable: Vec<&str> = ui
            .faces
            .iter()
            .filter(|face| {
                face.origin == crate::panels::properties::face::FaceOrigin::PdfcerWouldAdd
            })
            .map(|face| face.label.as_str())
            .collect();

        assert!(
            addable.is_empty(),
            "no face pdfcer can author encodes {:?}, so every one of these rows would refuse if \
             the operator pressed it. A non-empty list here means `sync_faces` is still asking \
             the pre-flight about the RUN'S OWN characters — the `candidate` argument is not \
             reaching `preview_font_resources_for`. Offered: {addable:?}",
            shown.character
        );
    }

    /// **Taking the offer swaps the face AND re-applies the operator's own
    /// edit** — O141's second half, asserted without a frame.
    #[test]
    fn taking_the_offer_re_applies_the_edit_the_operator_already_typed() {
        drain();
        // The document is opened FIRST and its own `edit_epoch` drives the
        // state machine, because `section` re-runs `advance` against that field.
        // A test that stepped the epoch with numbers of its own would have the
        // block retired on the frame it drew, which is not the state under test.
        let mut doc = crate::app::state::open_local_fixture("subset-font-floor.pdf");
        let mut ui = RefusedCharUi::default();
        PENDING.with_borrow_mut(|slot| *slot = Some(refusal()));
        assert!(ui.advance(doc.edit_epoch), "the refusal is adopted");
        // The operator picks a face. `section` does this inside the popup's
        // caller; here it is the one line of it that matters to the transition.
        ui.taken = Some("Helvetica-Bold".to_owned());

        // The swap lands: one edit-epoch step, caused by this block.
        doc.edit_epoch += 1;
        assert!(
            ui.advance(doc.edit_epoch),
            "the block survives the swap it asked for"
        );
        assert_eq!(ui.swapped_to.as_deref(), Some("Helvetica-Bold"));
        assert!(
            !ui.retried,
            "nothing has been re-applied yet — `section` raises it on the frame it \
             draws the swapped sentence, and a state that claimed otherwise before \
             drawing would skip the retype entirely"
        );

        let mut actions = Vec::new();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(360.0, 900.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |egui_ui| {
            section(egui_ui, &doc, &mut ui, &mut actions);
        });

        assert!(
            ui.retried,
            "the block has done everything it can and must say so"
        );
        let typed = typed();
        let raised = actions
            .iter()
            .find_map(|a| match a {
                Action::CommitTextEdit {
                    page,
                    run,
                    original,
                    replacement,
                    ..
                } => Some((*page, *run, original.clone(), replacement.clone())),
                _ => None,
            })
            .expect(
                "★★★ THE OFFER DID NOT FINISH THE JOB: the face was swapped and no \
                 `Action::CommitTextEdit` was raised, so the operator's edit was thrown \
                 away by the refusal and never put back",
            );
        assert_eq!(raised.0, typed.page);
        assert_eq!(raised.1, typed.run);
        assert_eq!(raised.2, typed.original, "the `find` operand");
        assert_eq!(
            raised.3, typed.replacement,
            "★ THE OPERAND THAT EXISTS NOWHERE ELSE — what he typed, not what the page \
             already says"
        );
    }

    /// **A refusal that arrived without the operator's words still swaps the
    /// face, and still leaves the block able to move on.**
    #[test]
    fn a_refusal_with_no_carried_words_still_leaves_the_swapped_state() {
        drain();
        let mut doc = crate::app::state::open_local_fixture("subset-font-floor.pdf");
        let mut ui = RefusedCharUi::default();
        PENDING.with_borrow_mut(|slot| {
            *slot = Some(RefusedCharacter {
                typed: None,
                ..refusal()
            });
        });
        assert!(ui.advance(doc.edit_epoch));
        ui.taken = Some("Helvetica".to_owned());
        doc.edit_epoch += 1;
        assert!(ui.advance(doc.edit_epoch));

        let mut actions = Vec::new();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(360.0, 900.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |egui_ui| {
            section(egui_ui, &doc, &mut ui, &mut actions);
        });

        assert!(
            !ui.retried,
            "with nothing to retype the block must NOT enter the retry states: \
             `retried` set with no action sent makes the next frame report an engine \
             refusal of a retype that was never made"
        );
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, Action::CommitTextEdit { .. })),
            "and it must NOT invent words to retype: a rebuilt replacement would be the \
             page's own text, so the 'retype' would land the ORIGINAL and report success"
        );
    }

    /// **The three regions are distinct and none of them is another
    /// surface's.**
    #[test]
    fn the_regions_name_this_block_and_not_the_text_section() {
        assert_eq!(REGION, "properties.refusedchar");
        assert!(FACE_REGION.starts_with(REGION));
        assert!(DISCLOSURE_REGION.starts_with(REGION));
        assert_ne!(FACE_REGION, super::super::text::FACE_REGION);
        assert!(!FACE_REGION.starts_with("properties.text"));
    }
}
