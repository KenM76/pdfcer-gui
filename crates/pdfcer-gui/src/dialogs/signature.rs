//! # `dialogs::signature` — the question Save has never asked about a signed
//! document
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/signature.md`.

use egui::Ui;

use pdfcer_core::signature::{ImpactBasis, SaveMode, SignatureImpact};

use crate::app::state::Status;
use crate::text::signature as t;

/// The region the window body publishes.
pub const REGION_BODY: &str = "dialog:signature"; // ui-text-exempt: trace region name, never displayed
/// The region the *proceed* button publishes.
pub const REGION_PROCEED: &str = "signature.proceed"; // ui-text-exempt: trace region name, never displayed
/// The region the Cancel button publishes.
pub const REGION_CANCEL: &str = "signature.cancel"; // ui-text-exempt: trace region name, never displayed

/// **Which save is waiting on the answer.**
///
/// Two variants because this shell has two writers that append a revision, and
/// they differ in the one way that matters to somebody deciding: whether the
/// file they already have is the one being written over.
///
/// It carries no operand. A save has nothing to re-derive after the frame —
/// unlike `crate::dialogs::unsaved::PendingIntent::Open`, which carries the
/// picked path because that path *is* the operand — so the variant is the whole
/// of what has to survive until the answer comes back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingSave {
    /// `Action::Save` — [`crate::app::save::save_in_place`]. Writes over the
    /// document's own file.
    InPlace,
    /// `Action::SaveCopy` — [`crate::app::save::save_copy`]. Asks for a
    /// destination and writes a new file; the original is untouched.
    Copy,
}

impl PendingSave {
    /// The sentence saying what this save does to the file the signature is
    /// in.
    ///
    /// `name` is the document's own file name, used only by
    /// [`Self::InPlace`]; a copy has no name to give yet, because the picker
    /// has not opened.
    #[must_use]
    pub fn target_sentence(self, name: &str) -> String {
        match self {
            Self::InPlace => t::target_in_place(name),
            Self::Copy => t::target_copy().to_owned(),
        }
    }
}

/// **What surface an impact earns.**
///
/// The type [`disclosure_for`] returns, and the reason that function can be
/// unit-tested without a `Ui`, a `Context`, an `OpenDoc` or an `EditSession`.
///
/// Three variants and not an `Option`, because *"say nothing"* and *"say it
/// afterwards"* are different answers that a two-state type would collapse —
/// and the collapse would go in the dangerous direction, since the cheapest
/// way to make an `Option<Dialog>` compile is to return `None` for both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disclosure {
    /// Nothing is said and nothing is drawn. The engine's instruction for
    /// `SignatureImpact::None`: *"a front end should add no friction at all."*
    Silent,
    /// One sentence on the status bar's disclosure row, **after** the write.
    /// There is nothing to consent to.
    NoteAfterSaving,
    /// A window, **before** the write, worded for this footing.
    WarnBeforeSaving(ImpactBasis),
}

/// **Which surface a given impact earns — the whole decision, as a pure
/// function.**
///
/// Takes the two engine enums rather than a session or a census, so that every
/// row of §2's table is asserted headlessly. That is not merely convenient: the
/// alternative is a decision made inline inside a `show` method, where the only
/// way to exercise it is to drive a window, and where a fourth case added to
/// the engine would be absorbed by a `_ =>` arm nobody re-read.
///
/// `basis` is ignored for every variant but `Invalidated`, and is *supplied*
/// for all of them because `SignatureImpact::documentation_basis` is total —
/// it answers `ImpactBasis::NotApplicable` for `None`. Requiring the caller to
/// compute it unconditionally keeps the one call to the engine in one place.
///
/// # Both enums are `#[non_exhaustive]`
///
/// So the wildcard arms are mandatory rather than lazy, and their answers are
/// chosen rather than defaulted: an impact this build does not recognise is
/// **not silent**. It gets the note, which discloses that something was said
/// about the signature without asserting what — the honest answer for a verdict
/// this shell cannot read. Choosing `Silent` there would let a future engine
/// variant, added precisely because it mattered, ship as nothing at all.
#[must_use]
pub fn disclosure_for(impact: SignatureImpact, basis: ImpactBasis) -> Disclosure {
    match impact {
        // The engine's own words, and the reason this arm is first: most
        // documents this operator opens are unsigned, so this is the hot path
        // and it must cost the operator nothing.
        SignatureImpact::None => Disclosure::Silent,
        SignatureImpact::ByteRangePreserved => Disclosure::NoteAfterSaving,
        SignatureImpact::Invalidated => Disclosure::WarnBeforeSaving(basis),
        // See the section above. Not `Silent`.
        _ => Disclosure::NoteAfterSaving,
    }
}

/// **Ask the engine what this save would do, and decide what to show.**
///
/// The one place `EditSession::signature_impact_of_save` is called, and the
/// one place `SignatureImpact::documentation_basis` is. Returns the surface
/// together with the census's signature count, because every sentence in
/// [`crate::text::signature`] that is not a button label needs the count and
/// re-taking a census to get it would be a second walk of the field tree for a
/// number the first walk already had.
///
/// `SaveMode::Incremental` is not a parameter, and that is a statement about
/// this shell rather than a simplification. `crate::app::save`'s §1 records
/// that the save mode was *"decided by a shipped promise rather than by this
/// module"* — `file.save_copy`'s tooltip has promised an appended update since
/// the day the command was registered — and that the honest response to an
/// input where incremental is impossible is **to refuse and say so**, never to
/// fall back to a rewrite. So there is no route through this function on which
/// a full rewrite could arrive, and accepting a mode would invite one.
/// `file.save_compacted`, which genuinely rewrites, does not come through here
/// at all; see the header's §5.
#[must_use]
pub fn impact_of_saving(doc: &crate::app::state::OpenDoc) -> (Disclosure, usize) {
    let census = doc.session.signature_census();
    let impact = doc.session.signature_impact_of_save(SaveMode::Incremental);
    let basis = impact.documentation_basis(&census);
    (disclosure_for(impact, basis), census.signatures)
}

/// The window's live state.
///
/// Existence is the "open" state, as everywhere in [`super`]. Everything it
/// needs was computed when the question was raised: the footing, the count and
/// the file name are all facts about the moment the operator was asked, which
/// is what a confirmation's text is for — `crate::dialogs::unsaved::UnsavedDialog`
/// captures its edit count at open time for the same reason and says so.
pub struct SignatureDialog {
    /// Which save is waiting.
    pending: PendingSave,
    /// On what footing the verdict rests. See the header's §3.
    basis: ImpactBasis,
    /// How many signature dictionaries the document carries.
    count: usize,
    /// The document's own file name, for [`PendingSave::InPlace`]'s sentence.
    ///
    /// The **name**, not the full path: the sentence is a reminder of which of
    /// several open documents is about to be written over, and a full Windows
    /// path in a dialog body wraps to three lines and buries the verb.
    name: String,
    /// Set by the proceed button, drained by the owner.
    confirmed: bool,
    /// Set by Cancel and by the window's ✕.
    cancelled: bool,
}

impl SignatureDialog {
    /// Ask about `pending`, on `basis`, for a document with `count`
    /// signatures called `name`.
    #[must_use]
    pub fn new(pending: PendingSave, basis: ImpactBasis, count: usize, name: String) -> Self {
        Self {
            pending,
            basis,
            count,
            name,
            confirmed: false,
            cancelled: false,
        }
    }

    /// Whether the copy should assert the outcome or attribute it.
    const fn spec_sourced(&self) -> bool {
        matches!(self.basis, ImpactBasis::SpecSourced)
    }

    /// Take the operator's answer, if they have given one.
    ///
    /// Returns the pending save **with** the confirmation, for
    /// `crate::dialogs::unsaved::UnsavedDialog::take_outcome`'s reason: the
    /// owner needs both, and holding them apart would let a future edit drain
    /// one without the other and resume the wrong save.
    ///
    /// One-shot. The second call answers `None`, which is what stops the owner
    /// performing the save on every frame after one press.
    pub fn take_confirmation(&mut self) -> Option<PendingSave> {
        if std::mem::take(&mut self.confirmed) {
            Some(self.pending)
        } else {
            None
        }
    }

    /// **Whether a confirmation is parked here and has not been drained.**
    ///
    /// # Why this predicate exists, and it is a defect fix
    ///
    /// [`Self::show`] answers `false` on the very frame the proceed button is
    /// pressed — that is what closes the window, and it is correct. Its owner
    /// read that `false` as *"this dialog is finished"* and dropped the whole
    /// dialog out of `DialogsState`:
    ///
    /// ```ignore
    /// if self.signature.as_mut().map(|d| d.show(ctx)) == Some(false) {
    ///     self.signature = None;      // <- with the answer still inside it
    /// }
    /// ```
    ///
    ///
    /// It is invisible to every test that does not run a whole frame. The
    /// dialog is correct in isolation (`take_confirmation` returns the answer),
    /// the drain is correct in isolation (it acts on whatever it is given), and
    /// the defect lives entirely in the *lifetime* between them.
    ///
    /// So the retirement rule is now [`crate::dialogs::retire`]: a window that
    /// closed **because it was answered** stays in its slot until the answer
    /// has been taken out of it.
    #[must_use]
    pub const fn answered(&self) -> bool {
        self.confirmed
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        // Its own OS window, with a taskbar entry, exactly as
        // `dialogs::unsaved` is and for the same reason: it appears in answer
        // to a keystroke (`Ctrl+S`) that an operator fires and then looks away
        // from, so a modal question with no entry anywhere is the classic "the
        // program has frozen" report.
        //
        // No `ScrollArea`, and the reason is `dialogs::unsaved`'s verbatim:
        // the content is bounded by construction — three sentences, two
        // buttons and one footnote — so the family of reach defects cannot
        // arise, and adding a scroll region "for safety" would create the
        // condition it was meant to prevent. Taller than the unsaved window
        // because the sentences are longer; the floor equals the opening size
        // for the same reason it does there.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "signature", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 280.0),
            egui::vec2(460.0, 280.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;
        // The ✕ is a Cancel — the NON-destructive answer — because it is the
        // control an operator presses reflexively to make a surprise go away.
        // `dialogs::unsaved` states the rule; this is the second surface it
        // governs.
        open && !self.cancelled && !self.confirmed
    }

    /// The body.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(if self.spec_sourced() {
            t::headline_certified(self.count)
        } else {
            t::headline_approval(self.count)
        });
        ui.add_space(6.0);
        ui.label(if self.spec_sourced() {
            t::basis_certified()
        } else {
            t::basis_approval()
        });
        ui.add_space(6.0);
        ui.label(self.pending.target_sentence(&self.name));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            // Cancel FIRST, which inverts `dialogs::unsaved`'s order, and
            // the inversion is the point rather than a slip.
            //
            // There, the reading order runs from the answer that loses nothing
            // (*Save a copy…*) to the answer that loses everything (*Close
            // without saving*), and the leftmost button is the safe one. Here
            // there are only two answers and the destructive one is the
            // *proceed*, so the same rule — safest first — puts Cancel on the
            // left. The rule is "the destructive button is not the one your
            // hand lands on", not "the affirmative button is on the left".
            let cancel = ui.button(t::cancel_button());
            crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
            if cancel.clicked() {
                self.cancelled = true;
            }
            let proceed = ui.button(if self.spec_sourced() {
                t::proceed_certified()
            } else {
                t::proceed_approval()
            });
            crate::diag::ui_rect(REGION_PROCEED, proceed.rect);
            if proceed.clicked() {
                self.confirmed = true;
            }
        });

        ui.add_space(8.0);
        ui.label(egui::RichText::new(t::verifies_nothing()).small().weak());
    }
}

/// **Raise the question if this save would invalidate a signature.**
///
/// Returns `None` when there is nothing to ask about, and the caller then
/// proceeds unchanged. That shape is `crate::dialogs::unsaved::ask_for`'s
/// deliberately: the guard is **one call at the top of an arm** whose `None`
/// answer is the unchanged path, so adding it to a third save route later is
/// one line rather than a new rule.
///
/// `None` covers three genuinely different situations and it is worth naming
/// them, because a future reader will want to split them and there is no
/// caller that could use the distinction:
///
/// * **no document** — there is nothing to save, and the arms that reach here
///   trace their own decline;
/// * **no signature** — `SignatureImpact::None`, the overwhelmingly common
///   case, and the one the engine says must cost the operator nothing;
/// * **`ByteRangePreserved`** — real, disclosed, and disclosed *after* the
///   write by [`crate::app::save`], because there is no decision to make.
#[must_use]
pub fn ask_for(status: &Status, pending: PendingSave) -> Option<SignatureDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    let (disclosure, count) = impact_of_saving(doc);
    let Disclosure::WarnBeforeSaving(basis) = disclosure else {
        return None;
    };
    // The file NAME. See the field's own note for why not the path.
    let name = doc.path.file_name().map_or_else(
        || doc.path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    Some(SignatureDialog::new(pending, basis, count, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **An unsigned document adds no friction at all.**
    #[test]
    fn an_unsigned_document_is_never_interrupted() {
        for basis in [
            ImpactBasis::NotApplicable,
            ImpactBasis::SpecSourced,
            ImpactBasis::ConservativeReport,
        ] {
            assert_eq!(
                disclosure_for(SignatureImpact::None, basis),
                Disclosure::Silent,
                "the engine says a front end should add no friction at all for {basis:?}"
            );
        }
    }

    /// **A preserved byte range is disclosed, and it is disclosed
    /// afterwards.**
    #[test]
    fn a_preserved_byte_range_is_a_note_and_not_a_window() {
        assert_eq!(
            disclosure_for(
                SignatureImpact::ByteRangePreserved,
                ImpactBasis::SpecSourced
            ),
            Disclosure::NoteAfterSaving
        );
    }

    /// **An invalidating save asks first, and the question knows which
    /// footing it is on.**
    #[test]
    fn an_invalidating_save_asks_first_and_carries_its_footing() {
        assert_eq!(
            disclosure_for(SignatureImpact::Invalidated, ImpactBasis::SpecSourced),
            Disclosure::WarnBeforeSaving(ImpactBasis::SpecSourced)
        );
        assert_eq!(
            disclosure_for(
                SignatureImpact::Invalidated,
                ImpactBasis::ConservativeReport
            ),
            Disclosure::WarnBeforeSaving(ImpactBasis::ConservativeReport)
        );
        assert_ne!(
            disclosure_for(SignatureImpact::Invalidated, ImpactBasis::SpecSourced),
            disclosure_for(
                SignatureImpact::Invalidated,
                ImpactBasis::ConservativeReport
            ),
            "the two footings must not collapse into one surface — that is the whole reason \
             `documentation_basis` exists"
        );
    }

    /// **The window's wording follows the footing, all three strings
    /// together.**
    #[test]
    fn the_window_words_the_two_footings_apart() {
        let certified = SignatureDialog::new(
            PendingSave::InPlace,
            ImpactBasis::SpecSourced,
            1,
            "sheet.pdf".to_owned(),
        );
        let approval = SignatureDialog::new(
            PendingSave::InPlace,
            ImpactBasis::ConservativeReport,
            1,
            "sheet.pdf".to_owned(),
        );
        assert!(certified.spec_sourced());
        assert!(!approval.spec_sourced());
    }

    /// **An unreadable footing gets the cautious wording.**
    #[test]
    fn a_footing_this_build_cannot_read_asserts_less() {
        let unknown = SignatureDialog::new(
            PendingSave::Copy,
            ImpactBasis::NotApplicable,
            1,
            "sheet.pdf".to_owned(),
        );
        assert!(
            !unknown.spec_sourced(),
            "an unrecognised footing must not be worded as a spec citation"
        );
    }

    /// **The two save routes say different things about the operator's
    /// file, and only one of them names it.**
    #[test]
    fn the_two_save_routes_describe_different_risks() {
        let in_place = PendingSave::InPlace.target_sentence("Sheet 1.pdf");
        let copy = PendingSave::Copy.target_sentence("Sheet 1.pdf");
        assert_ne!(in_place, copy);
        assert!(in_place.contains("Sheet 1.pdf"));
        assert!(
            !copy.contains("Sheet 1.pdf"),
            "a copy has no destination yet — the picker has not opened — so naming the source \
             file in it would point at the wrong file"
        );
    }

    /// **The answer fires once and carries its save.**
    #[test]
    fn the_confirmation_fires_once() {
        let mut d = SignatureDialog::new(
            PendingSave::Copy,
            ImpactBasis::SpecSourced,
            1,
            "a.pdf".to_owned(),
        );
        assert_eq!(d.take_confirmation(), None);
        d.confirmed = true;
        assert_eq!(d.take_confirmation(), Some(PendingSave::Copy));
        assert_eq!(d.take_confirmation(), None, "it must not repeat");
    }

    /// **A parked answer is visible to the owner for exactly as long as it
    /// is undrained — which is what keeps the window alive long enough to hand
    /// it over.**
    #[test]
    fn an_answer_is_visible_until_it_is_taken_and_not_after() {
        let mut d = SignatureDialog::new(
            PendingSave::InPlace,
            ImpactBasis::ConservativeReport,
            2,
            "drawing.pdf".to_owned(),
        );
        assert!(
            !d.answered(),
            "a warning nobody has answered is holding nothing"
        );
        d.confirmed = true;
        assert!(d.answered(), "the proceed button parked an answer");
        assert_eq!(d.take_confirmation(), Some(PendingSave::InPlace));
        assert!(
            !d.answered(),
            "the drain emptied it, so the next frame may retire the window"
        );
    }

    /// **A cancelled window is holding nothing**, which is what lets it be
    /// retired on the frame it closes.
    #[test]
    fn a_cancelled_window_is_holding_nothing() {
        let mut d = SignatureDialog::new(
            PendingSave::Copy,
            ImpactBasis::SpecSourced,
            1,
            "a.pdf".to_owned(),
        );
        d.cancelled = true;
        assert!(!d.answered());
    }

    /// **Cancelling answers nothing.**
    #[test]
    fn cancelling_does_not_save() {
        let mut d = SignatureDialog::new(
            PendingSave::InPlace,
            ImpactBasis::SpecSourced,
            1,
            "a.pdf".to_owned(),
        );
        d.cancelled = true;
        assert_eq!(d.take_confirmation(), None);
    }

    /// **The fixture really is signed, really is an approval signature,
    /// and really does move between the two surfaces when a page goes.**
    #[test]
    fn the_signed_fixture_moves_from_a_note_to_a_window_when_a_page_goes() {
        use crate::app::state::{SIGNED_TWO_PAGES, open_local_fixture};

        let mut doc = open_local_fixture(SIGNED_TWO_PAGES);
        let census = doc.session.signature_census();
        assert_eq!(
            census.signatures, 1,
            "the fixture must carry exactly one signature dictionary; a `/SigFlags` declaration \
             is not one and the census does not count it"
        );
        assert_eq!(
            census.certifications, 0,
            "the fixture must be an APPROVAL signature — no `/Reference` — so the cautious \
             wording is the one under test"
        );
        assert_eq!(doc.pages.len(), 2, "a page has to be spare to delete");

        // Unedited: the save appends nothing structural, so the byte range
        // survives and there is nothing to consent to.
        let (before, count) = impact_of_saving(&doc);
        assert_eq!(before, Disclosure::NoteAfterSaving);
        assert_eq!(count, 1);

        // …and now the structural change the engine says can only be seen here.
        let session = std::sync::Arc::get_mut(&mut doc.session)
            .expect("nothing else holds the session in a test");
        session
            .delete_pages(&[1])
            .expect("deleting the second of two pages must be expressible");

        let (after, count) = impact_of_saving(&doc);
        assert_eq!(
            after,
            Disclosure::WarnBeforeSaving(ImpactBasis::ConservativeReport),
            "a page removed from a signed document must raise the question, on the footing that \
             says the verdict is pdfcer's rather than the standard's"
        );
        assert_eq!(count, 1);

        // And the guard really raises a window for it — the join between the
        // decision and the surface, asserted through the same call the
        // `Action::Save` arm makes. Every step above could be right with
        // `ask_for` still answering `None`, and the operator would see nothing.
        let status = Status::Open(Box::new(doc));
        let raised = ask_for(&status, PendingSave::InPlace)
            .expect("an invalidating save must raise the question");
        assert!(
            !raised.spec_sourced(),
            "the fixture's approval signature must reach the cautious wording"
        );
        assert!(
            raised
                .pending
                .target_sentence("signed-two-pages.pdf")
                .contains("signed-two-pages.pdf"),
            "an in-place save names the file it is about to write over"
        );
    }

    /// **A document with no session cannot be asked about.**
    #[test]
    fn nothing_is_asked_about_an_empty_shell() {
        assert!(ask_for(&Status::Empty, PendingSave::InPlace).is_none());
        assert!(ask_for(&Status::Empty, PendingSave::Copy).is_none());
    }
}
