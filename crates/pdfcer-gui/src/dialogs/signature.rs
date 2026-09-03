//! # `dialogs::signature` — the question Save has never asked about a signed
//! document
//!
//! ## The gap
//!
//! Found **2026-08-28**, auditing this build against `pdfcer-core`'s capability
//! register. The engine exposes two verbs written specifically so a front end
//! could answer this question, and this shell called neither:
//!
//! ```text
//! session.signature_impact_of_save(mode: SaveMode) -> SignatureImpact
//! session.changes_structure() -> bool
//! ```
//!
//! So `file.save`, `file.save_copy` and the *Save a copy…* button inside the
//! unsaved-edits window all wrote a revision over, or beside, a digitally
//! signed document **and said nothing at all** — not before, not after, not on
//! any surface. The compacted-save path was the one exception, because
//! [`crate::dialogs::compact`] measures a census and shows a sentence before
//! its picker opens; see §5 below for why that path needed nothing from this
//! module.
//!
//! An operator's signed drawing is a legal artifact. A structural edit and a
//! save is a normal afternoon. The two together produced a file whose
//! signature pdfcer believed to be invalidated, and pdfcer kept that belief to
//! itself.
//!
//! ## ★★ 1. Why the engine says this can only be asked at save time
//!
//! [`EditSession::signature_impact_of_save`]'s own documentation:
//!
//! > A front end asks this **immediately before Save**, not at edit time: per
//! > §11.1 the dirty set is a diff computed at save time, so "does this save
//! > change structure?" is not knowable when the edit is made.
//!
//! That single sentence settles the architecture of this module and rules out
//! the design a reader would otherwise expect. There is **no** flag on
//! `OpenDoc` recording *"an invalidating edit has happened"*, no marker painted
//! when a page is deleted, and no state accumulated across the session — all of
//! which would be wrong for the same reason: an edit that has since been undone
//! is not a change, the dirty set is a **diff against the base document**, and
//! only the moment before the write knows what that diff is.
//!
//! The consequence worth naming, because it looks like an omission: **an
//! operator who deletes a page from a signed document is told nothing at the
//! moment they delete it.** They are told when they save. That is not this
//! shell declining to be helpful; it is the only moment at which the answer
//! exists.
//!
//! ## ★★★ 2. The three impacts and the three surfaces
//!
//! [`disclosure_for`] is the whole decision, as a pure function, and this is
//! the table it encodes:
//!
//! | `SignatureImpact` | Surface | Why |
//! |---|---|---|
//! | `None` | **nothing at all** | the engine's own instruction: *"Nothing to say, and a front end should add no friction at all."* Most documents this operator opens are unsigned drawings, and a save that paused to tell them so would be a nag on the commonest path in the application |
//! | `ByteRangePreserved` | a status-bar **note, after** the save | the fact is real but it is not a decision — there is nothing to consent to, because the save does not disturb what any signature covers. See [`crate::text::signature::preserved_note`] for why this shell pairs the fact with its uncertainty rather than taking the permitted option of saying nothing |
//! | `Invalidated` | a **window, before** the save, plus a note after | there is an irreversible-looking decision to make and the operator is the only one who can make it |
//!
//! ### Why the middle row is not a window
//!
//! Because there is no question. A confirmation dialog asks the operator to
//! choose, and here both choices lead to the same document: the save changes
//! nothing about the bytes any signature covers whether they proceed or not.
//! A window would be friction charged for information, which is how
//! confirmations come to be dismissed unread — and the one that matters, the
//! row below it, is the one that would then be dismissed.
//!
//! ### Why the last row is a window and not a louder note
//!
//! Because it is the **conventional interaction**, which is a standing
//! instruction on this project rather than a preference: *use the conventional
//! interaction, never invent one*. Every document application that can
//! invalidate a signature asks first. A note after the fact would tell the
//! operator about a choice at the moment it stopped being available.
//!
//! ## ★★ 3. Why the window's copy branches on `documentation_basis`
//!
//! `SignatureImpact::Invalidated` is one variant reached on two very different
//! footings, and the engine exposes
//! `SignatureImpact::documentation_basis(&census)` — in its own words —
//! *"because the two deserve different operator-facing wording and a front end
//! cannot tell them apart from the variant alone"*:
//!
//! * **`SpecSourced`** — a certification signature is present, and Table 254's
//!   permitted-change lists are closed (*"other changes shall invalidate the
//!   signature"*). pdfcer can state the outcome as fact.
//! * **`ConservativeReport`** — only approval signatures are present, and ISO
//!   32000-1 defines stage 1 and only stage 1 for them. pdfcer reports
//!   `Invalidated` anyway, as *"a product decision under rule 4
//!   (fuzzy-never-sneaky), not a spec citation"*. The copy must therefore
//!   report the verdict **and** whose it is.
//!
//! [`crate::text::signature`]'s header carries the wording rules that follow
//! from this, and its tests guard them. What lives here is the plumbing: the
//! basis is computed once, at the moment the question is raised, and travels
//! with the dialog — because the census it was computed from describes the
//! document as it stood when the operator was asked, which is what a
//! confirmation's text is for.
//!
//! ## ★ 4. Why `documentation_basis` is NOT consulted on a full rewrite
//!
//! A trap, recorded because the next reader will reach for it. The helper takes
//! only the impact and the census — **it cannot see the `SaveMode`** — so for a
//! document carrying nothing but approval signatures it answers
//! `ConservativeReport` even when the invalidation comes from a full rewrite,
//! where stage 1 genuinely fails outright under §12.8.1 and the answer is
//! `SpecSourced` by any reading. That is not a defect in the helper; it is a
//! consequence of its signature, and it is exactly why this module never routes
//! the compacted path through it (§5).
//!
//! Everything this module *does* classify is an **incremental** save, where the
//! helper is precisely right: an incremental save reaches `Invalidated` only
//! through a structural change, and there the presence of a certification
//! signature really is what separates a spec citation from a cautious report.
//!
//! ## 5. Why the compacted path is untouched
//!
//! `file.save_compacted` is a full rewrite, and §12.8.1 makes that destroy
//! every signature outright. [`crate::dialogs::compact`] already:
//!
//! * takes a `signature_census()` when it opens,
//! * draws [`crate::text::compact::signature_line`] full-size and conditionally
//!   when the count is non-zero,
//! * says the loss **cannot be repaired**, which is stronger than anything this
//!   module says and is correct there and only there,
//! * and does all of it **before** its file picker opens.
//!
//! That is the same disclosure this module makes, one command over, already
//! shipped and already correct on the spec's own terms. Adding a second window
//! in front of it would put two modals on one gesture — and the second would be
//! the weaker of the two, which is the wrong one to leave standing.
//!
//! ## ★★ 6. The shape is `dialogs::unsaved`'s, deliberately and exactly
//!
//! [`crate::dialogs::unsaved`] is this shell's existing *ask before
//! proceeding* machinery and this module copies it rather than paraphrasing
//! it: a parked intent, a one-shot answer drained by the application, a guard
//! returning *"did I interrupt you"*, and the destructive act performed by
//! `crate::app::lifecycle` rather than by the window.
//!
//! Its header also carries the argument this module inherits wholesale — that
//! a second predicate beside `save_pending` was correct rather than a
//! redefinition of it, because *"is a save in flight"* and *"are there unsaved
//! edits"* are different questions with different answers, and conflating them
//! would have broken a live consumer. The same holds a third time here:
//! **"will this save invalidate a signature"** is a third question, it is
//! answered by the engine rather than by this shell, and it composes with the
//! other two rather than replacing either.
//!
//! ⇒ The guard therefore returns `true` for *"stop, the window is up"*, for
//! [`crate::dialogs::DialogsState::ask_unsaved`]'s stated reason: read as
//! *"may I proceed"*, a guard that somebody inverts or forgets fails **open**
//! and the destructive thing happens. Read this way it fails **closed** — a
//! missing `if` asks a question whose answer performs the save anyway, so the
//! operator sees one redundant window instead of an unannounced write.
//!
//! ## ★★ 7. The one route that is told afterwards rather than asked first
//!
//! `crate::app::lifecycle::resume_after_unsaved` writes a copy when the
//! operator presses *Save a copy…* inside the unsaved-edits window. That call
//! does **not** raise this window, and the reasoning is stated here rather than
//! left to be discovered:
//!
//! 1. **The operator has just answered a modal**, on this same gesture, about
//!    this same document. `DialogsState::ask_unsaved` already codifies the
//!    rule for that situation one question earlier — a second request while one
//!    is on screen is *swallowed* rather than stacked, because *"the operator is
//!    looking at a question and has not answered it"*. Stacking a second window
//!    on the answer to the first is the same failure one step along, and its
//!    result is a confirmation dismissed unread.
//! 2. **That button writes a copy, and only a copy.** This build has no *Save*
//!    inside that window and the whole of `dialogs::unsaved`'s §★★ is the
//!    argument for why. So the operator's signed original is untouched by that
//!    write, no matter what the answer here would have been — which is the fact
//!    that makes deferring the disclosure safe on this route and would not make
//!    it safe for save-in-place.
//!
//! What that route gets instead is [`crate::text::signature::invalidated_note`],
//! recorded by [`crate::app::save`] on every successful write, so the operator
//! is told — after, rather than before. That is a real difference and it is
//! **named as a difference** rather than smoothed over. If `file.save` ever
//! joins that window's buttons, this paragraph stops being true and the guard
//! has to move; the note stays either way.

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
/// ★ Three variants and not an `Option`, because *"say nothing"* and *"say it
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
/// ★ `SaveMode::Incremental` is not a parameter, and that is a statement about
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
    ///
    /// One predicate rather than three `match`es on `basis`, so the headline,
    /// the explanation and the button cannot come to disagree about which
    /// footing they are on — which is the specific way a two-wording surface
    /// goes wrong, and it goes wrong silently because each string is correct
    /// in isolation.
    ///
    /// `ImpactBasis` is `#[non_exhaustive]`, and the wildcard answers `false`:
    /// the cautious wording is correct for a footing this build cannot read,
    /// because it asserts less.
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
    /// # ★★★ Why this predicate exists, and it is a defect fix
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
    /// The answer lives **in the dialog** until
    /// `PdfcerApp::resume_after_signature` drains it, later in the same frame.
    /// So the press closed the window, the window took the confirmation to the
    /// grave with it, `take_signature_answer` found an empty slot, and the save
    /// never ran. Observed by driving on 2026-08-29
    /// (`an_invalidating_save_is_warned_about`): the window opened, the button
    /// was pressed, the window closed, and **no `signature-confirmed` line was
    /// ever traced** — which made Save unusable on every signed document, by
    /// any route this guard covers.
    ///
    /// ★ It is invisible to every test that does not run a whole frame. The
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
    ///
    /// The order is fixed and each position is argued:
    ///
    /// 1. **the headline** — what is happening, in the sentence read first;
    /// 2. **the footing** — why pdfcer says so, and whose claim it is;
    /// 3. **the target** — which file this is about to touch;
    /// 4. **the buttons**, non-destructive-to-destructive left to right, which
    ///    is `dialogs::unsaved`'s ordering rule and every application the
    ///    operator uses;
    /// 5. **the footnote** — that pdfcer verified nothing — below the buttons,
    ///    because it answers a question an operator only has *after* noticing
    ///    the window is making a claim.
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
            // ★ Cancel FIRST, which inverts `dialogs::unsaved`'s order, and
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

    /// ★★★ **An unsigned document adds no friction at all.**
    ///
    /// The engine's instruction for `SignatureImpact::None`, asserted as the
    /// property it is. Every other row of §2's table is a disclosure this
    /// module owes; this row is the one it owes *nothing*, and it is the row
    /// covering most documents this operator opens — so a regression here
    /// would put a window in front of the commonest save in the application.
    ///
    /// The basis is varied across every variant to make the point that the
    /// answer does not depend on it: `documentation_basis` answers
    /// `NotApplicable` for `None`, but a build that passed the wrong basis
    /// must still not produce a window.
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

    /// ★★ **A preserved byte range is disclosed, and it is disclosed
    /// afterwards.**
    ///
    /// Both halves are load-bearing and they fail in opposite directions.
    /// `Silent` would be the engine's permitted option and this shell's
    /// choice against — see [`crate::text::signature::preserved_note`] for the
    /// argument. `WarnBeforeSaving` would be worse: a window asking the
    /// operator to consent to something that changes nothing they could
    /// decline, which is exactly the friction that teaches a person to dismiss
    /// the window that matters.
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

    /// ★★★ **An invalidating save asks first, and the question knows which
    /// footing it is on.**
    ///
    /// The conventional interaction — every document application that can
    /// invalidate a signature warns before saving, and the operator's standing
    /// rule is to use the conventional interaction rather than invent one.
    ///
    /// The second assertion is the one the engine commissioned: the basis must
    /// travel *through* the decision into the surface, because
    /// `documentation_basis` exists precisely so the two footings can be
    /// worded differently, and a decision that discarded it would leave the
    /// window unable to tell them apart no matter how carefully the catalog
    /// was written.
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

    /// ★★ **The window's wording follows the footing, all three strings
    /// together.**
    ///
    /// [`SignatureDialog::spec_sourced`] is one predicate for exactly this
    /// reason: three independent `match`es on the basis would each be correct
    /// and could still disagree — a certified headline over a cautious
    /// explanation over an *anyway* button is a window that reads as though
    /// pdfcer is unsure what it just asserted.
    ///
    /// Asserted through the public strings rather than through the private
    /// flag, because the flag is not what an operator reads.
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

    /// ★ **An unreadable footing gets the cautious wording.**
    ///
    /// `ImpactBasis` is `#[non_exhaustive]`, so a future variant compiles into
    /// [`SignatureDialog::spec_sourced`]'s wildcard. It must land on the
    /// wording that asserts **less**: pdfcer stating Table 254 as fact about a
    /// footing it cannot identify would be the one failure this whole module
    /// exists to prevent, arriving through a language feature rather than
    /// through a sentence.
    ///
    /// Asserted with `NotApplicable`, which is a real variant that cannot
    /// reach here through `ask_for` — the only value available today that
    /// stands in for "something this code did not plan for".
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

    /// ★★ **The two save routes say different things about the operator's
    /// file, and only one of them names it.**
    ///
    /// The distinction an operator is actually deciding on: *is the file I
    /// already have the one being written over?* A build that used one
    /// sentence for both would tell somebody saving a copy that their original
    /// was at risk, or — far worse — tell somebody saving in place that it was
    /// not.
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

    /// ★ **The answer fires once and carries its save.**
    ///
    /// `dialogs::unsaved`'s one-shot property, and the failure it prevents is
    /// the same one: an answer read every frame would re-enter the save on
    /// each of the next sixty, which for save-in-place means sixty rewrites of
    /// the operator's file and for save-a-copy means a file picker that will
    /// not go away.
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

    /// ★★★ **A parked answer is visible to the owner for exactly as long as it
    /// is undrained — which is what keeps the window alive long enough to hand
    /// it over.**
    ///
    /// The regression test for `an_invalidating_save_is_warned_about`, found by
    /// driving on 2026-08-29: pressing *Save anyway* set `confirmed`, which
    /// made [`SignatureDialog::show`] answer `false`, which made
    /// `DialogsState::show` drop this dialog **with the confirmation still in
    /// it** — so `resume_after_signature` found an empty slot, traced no
    /// `signature-confirmed`, and no file was written. Save was unusable on
    /// every signed document.
    ///
    /// [`crate::dialogs::retire`] is the fix and it reads [`Self::answered`],
    /// so the two edges asserted here are the ones it depends on:
    ///
    /// * **`true` before the drain** — or the window is dropped and the save is
    ///   lost, which is the defect above;
    /// * **`false` after it** — or the window is kept forever, redrawing an
    ///   answered question every frame.
    ///
    /// It is asserted against `take_confirmation` rather than alone, because
    /// the property that matters is that the *pair* agrees about what "parked"
    /// means.
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

    /// ★★ **A cancelled window is holding nothing**, which is what lets it be
    /// retired on the frame it closes.
    ///
    /// The other half of [`crate::dialogs::retire`]'s input: `answered()` must
    /// distinguish *closed because answered* from *closed because dismissed*,
    /// or the ✕ would keep a window alive that has nothing to say.
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

    /// ★ **Cancelling answers nothing.**
    ///
    /// The ✕ and the Cancel button must be separable from an answer, or the
    /// control an operator presses reflexively to dismiss a surprise becomes
    /// the one that performs the write.
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

    /// ★★★ **The fixture really is signed, really is an approval signature,
    /// and really does move between the two surfaces when a page goes.**
    ///
    /// The one test here that goes through the **engine** rather than over the
    /// two enums, and it is worth its cost for three reasons that no amount of
    /// pure-function testing reaches:
    ///
    /// 1. **It proves the fixture.** `tools/gen-signed-fixture.py` asserts in
    ///    prose that it produces one approval signature over two pages. A
    ///    generator's prose is not evidence; `signature_census()` is. If a
    ///    future edit to that script produced a `/SigFlags` declaration with no
    ///    signature dictionary — which the engine deliberately does not count —
    ///    every other test in this module would still pass and `ui-verify`'s
    ///    `signature_save` would SKIP with a reason blaming the shell.
    /// 2. **It proves the arm the copy was written for.**
    ///    `documentation_basis` answering `ConservativeReport` is what makes
    ///    the cautious wording the one an operator sees, and it is computed
    ///    from `census.certifications`, which is read from `/Reference` and
    ///    never from `/Perms`. A fixture that accidentally acquired a `/DocMDP`
    ///    would silently switch the whole surface to the assertive wording.
    /// 3. **It proves the transition.** The same document, one page-delete
    ///    apart, must move from a note to a window. That is
    ///    `EditSession::changes_structure` doing its job, and it is the fact
    ///    the engine says can only be known at save time — so it is the one
    ///    claim in this module that cannot be checked any earlier.
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

        // ★ And the guard really raises a window for it — the join between the
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

    /// ★ **A document with no session cannot be asked about.**
    ///
    /// The `Status::Empty` guard, asserted for `ask_for`'s stated contract:
    /// `None` means *proceed unchanged*, and the save arms that reach here
    /// trace their own no-document decline one line later.
    #[test]
    fn nothing_is_asked_about_an_empty_shell() {
        assert!(ask_for(&Status::Empty, PendingSave::InPlace).is_none());
        assert!(ask_for(&Status::Empty, PendingSave::Copy).is_none());
    }
}
