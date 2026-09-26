//! # `dialogs::unsaved` — the question `file.close` has been promising to ask
//! since it shipped
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/unsaved.md`.

use egui::Ui;

use crate::app::state::Status;
use crate::text::unsaved as t;

/// The region the dialog body publishes.
pub const REGION_BODY: &str = "dialog:unsaved"; // ui-text-exempt: trace region name, never displayed
/// The region the *Save a copy…* button publishes.
pub const REGION_SAVE: &str = "unsaved.save_copy"; // ui-text-exempt: trace region name, never displayed

/// The **Save all** control, published only while it is drawn.
pub const REGION_SAVE_ALL: &str = "unsaved.save_all"; // ui-text-exempt: trace region name, never displayed

/// The **Save-over-the-open-file** button's region — `OPERATOR_REQUESTS.md` O65.
pub const REGION_SAVE_IN_PLACE: &str = "unsaved.save_in_place"; // ui-text-exempt: trace region name, never displayed
/// The region the discard button publishes.
pub const REGION_DISCARD: &str = "unsaved.discard"; // ui-text-exempt: trace region name, never displayed
/// The region the Cancel button publishes.
pub const REGION_CANCEL: &str = "unsaved.cancel"; // ui-text-exempt: trace region name, never displayed

/// What the operator asked for, held until they have answered the question.
#[derive(Debug, Clone, PartialEq)]
pub enum PendingIntent {
    /// `Action::Close`, and `Action::CloseDocument` once it has brought the
    /// tab it is closing to the front.
    ///
    Close,
    /// `Action::Open(path)`.
    ///
    /// ⚠ **Never constructed today.** See the note under this enum.
    Open(std::path::PathBuf),
    /// `Action::New`.
    ///
    /// ⚠ **Never constructed today.** See the note under this enum.
    New,
    /// `Action::NewSized`, with its page box in points.
    ///
    /// ⚠ **Never constructed today.** See the note under this enum.
    NewSized {
        /// Page width in points.
        width_pt: f64,
        /// Page height in points.
        height_pt: f64,
    },
    /// `Action::RereadWithDuplicateKeys` — **the fifth variant, and the
    /// second one anything constructs.**
    ///
    /// It is the odd one out in this enum and worth saying how: the four above
    /// are about *which document is on screen*, and this one is about **the
    /// same document, read differently**. The tab that comes back has the same
    /// path, the same name and the same pages. What it does not have is the
    /// edits, because a re-read is a fresh parse of the bytes on disk with an
    /// empty undo stack — which is precisely why it belongs here rather than
    /// beside the harmless commands.
    ///
    /// It carries the [`pdfcer_core::document::LoadOptions`] across the
    /// dialog rather than re-deriving them on the other side. The operator
    /// chose a reading and then answered a question about saving; the answer
    /// must not be able to change which reading they get.
    Reread {
        /// The reading the operator asked for, held until they have answered.
        options: pdfcer_core::document::LoadOptions,
    },
}

// ---------------------------------------------------------------------------
// THREE OF THE FOUR VARIANTS ARE CURRENTLY UNCONSTRUCTED, AND THAT IS
//    RECORDED RATHER THAN DELETED.
//
//
// They are kept, with their sentences, for one specific reason that is a real
// gap rather than a hedge: **pdfcer still asks nothing when the window is
// closed with unsaved edits.** That was true before this change and is now
// true across N documents instead of one. A quit guard is exactly this
// machinery with a fourth intent, and the three sentences here — *"Open
// another document?"*, *"Make a new document?"* — are the shape the fourth
// would take.
//
// If a quit guard lands and does not use them, delete them then. An enum arm
// nothing can reach is dead code wearing a design pattern, and this crate's
// standing preference is to make unreachable states unrepresentable — the
// exception is bought here by a named, dated, still-open gap.
// ---------------------------------------------------------------------------

impl PendingIntent {
    /// The sentence naming what is about to happen to the open document.
    #[must_use]
    pub fn question(&self) -> &'static str {
        match self {
            Self::Close => t::question_close(),
            Self::Open(_) => t::question_open(),
            Self::New | Self::NewSized { .. } => t::question_new(),
            // Its own sentence, not `question_close`'s. An operator who
            // pressed *use the first value* and is asked about **closing** will
            // read the prompt as being about a control they did not touch.
            Self::Reread { .. } => t::question_reread(),
        }
    }

    /// The label of the button that goes ahead without saving.
    #[must_use]
    pub fn discard_label(&self) -> &'static str {
        match self {
            Self::Close => t::discard_close(),
            Self::Open(_) => t::discard_open(),
            Self::New | Self::NewSized { .. } => t::discard_new(),
            Self::Reread { .. } => t::discard_reread(),
        }
    }
}

/// What the operator chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// **Write the file the operator opened, then resume** —
    /// `OPERATOR_REQUESTS.md` O65.
    ///
    /// Offered only when the document has a file to be written over. Resumes
    /// on a successful write and on nothing else, exactly as [`Self::SaveCopy`]
    /// does and for the identical reason: a save that did not happen must
    /// never be a route to discarding the work it was supposed to preserve.
    SaveInPlace,
    /// Write a copy first; resume only if a file was actually written.
    SaveCopy,
    /// Go ahead and lose the edits.
    Discard,
    /// **Save every document with unsaved work, in place** —
    /// `OPERATOR_REQUESTS.md` O102.
    ///
    /// The operator: *"have a save all button that saves all changed
    /// documents."* It is what makes the quit cycle bearable — without it,
    /// somebody with six dirty documents answers six questions.
    ///
    /// **Offered only when more than one document is dirty.** With one, it
    /// is the same act as [`Self::SaveInPlace`], and a second button meaning
    /// the same thing is one the operator has to stop and think about on a
    /// modal that is standing between them and their work.
    ///
    /// It saves **in place**, so it reaches only documents that have a file.
    /// A never-saved document needs a destination, which is a question only the
    /// operator can answer — the cycle asks about those individually
    /// afterwards, which is Word's behaviour and the only honest one.
    SaveAll,
}

/// The dialog's live state.
pub struct UnsavedDialog {
    /// What the operator asked for before this question interrupted them.
    intent: PendingIntent,
    /// How many edits are at stake, for the sentence that says so.
    ///
    /// Captured at **open** time rather than read per frame, and the reason
    /// is the dialog's own honesty: this window is the only thing on screen
    /// that can change the document (it cannot), so a live read could only ever
    /// return the same number — but capturing it makes the sentence a statement
    /// about the moment the operator was asked, which is what a confirmation
    /// dialog's text is *for*.
    edits: u64,
    /// **Whether this document has a file to save over**, captured at open
    /// time from `app::save::has_a_file`.
    ///
    /// Decides whether the Save button is drawn at all. R9: a never-saved
    /// document renders **nothing** rather than a greyed Save, because "this
    /// has never been written anywhere" is a standing property of the
    /// document, not a temporary condition a hover sentence could resolve —
    /// and the operator already has the control that fixes it, one button to
    /// the right.
    has_file: bool,
    /// **How many documents are dirty**, so the *Save all* button knows
    /// whether it would do more than *Save*.
    ///
    /// Captured at open time, like [`Self::edits`] and for the same reason: it
    /// is a statement about the moment the operator was asked. `1` is the
    /// single-document case and is what every caller outside the quit cycle
    /// passes — a tab close is about one document however many are open.
    dirty_documents: usize,
    /// Set by a button, drained by the owner.
    outcome: Option<Outcome>,
    /// Set by Cancel and by the window's ✕.
    cancelled: bool,
}

impl UnsavedDialog {
    /// Ask about `intent`, with `edits` unsaved changes at stake.
    #[must_use]
    pub fn new(intent: PendingIntent, edits: u64, has_file: bool) -> Self {
        Self::for_cycle(intent, edits, has_file, 1)
    }

    /// The same, told how many documents are dirty — the quit cycle's
    /// constructor.
    #[must_use]
    pub fn for_cycle(
        intent: PendingIntent,
        edits: u64,
        has_file: bool,
        dirty_documents: usize,
    ) -> Self {
        Self {
            intent,
            edits,
            has_file,
            dirty_documents,
            outcome: None,
            cancelled: false,
        }
    }

    /// **Did the operator answer Cancel?**
    #[must_use]
    pub const fn was_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Take the operator's answer, if they have given one.
    pub fn take_outcome(&mut self) -> Option<(PendingIntent, Outcome)> {
        let outcome = self.outcome.take()?;
        Some((self.intent.clone(), outcome))
    }

    /// **Whether an answer is parked here and has not been drained.**
    #[must_use]
    pub const fn answered(&self) -> bool {
        self.outcome.is_some()
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        //
        // Still no `ScrollArea`, and the note that said so stands: this is
        // the one dialog whose content is bounded by construction — three
        // buttons and at most four sentences — so the family of reach defects
        // cannot arise here, and adding a scroll region "for safety" would
        // create the condition it was meant to prevent. The floor equals the
        // opening size for the same reason.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "unsaved", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(420.0, 190.0),
            egui::vec2(420.0, 190.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;
        // The ✕ is a Cancel. That is not a convenience: the window's close
        // control must mean the NON-destructive answer, because it is the one
        // an operator presses reflexively to make a surprise go away.
        open && !self.cancelled && self.outcome.is_none()
    }

    /// The body.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(self.intent.question());
        ui.add_space(4.0);
        ui.label(t::edits_at_stake(self.edits));
        ui.add_space(8.0);

        // The buttons are in a fixed left-to-right order and the destructive
        // one is NOT first. Save-a-copy, then discard, then cancel: the reading
        // order runs from the answer that loses nothing to the answer that
        // loses everything, which is the order every application the operator
        // uses puts them in.
        ui.horizontal(|ui| {
            // **Save, when there is a file to save over** — O65.
            //
            // First, because it is the answer that loses nothing AND changes
            // nothing about where the operator's work lives. The order of this
            // row runs from least to most destructive and this is the new
            // least.
            //
            // Absent, not greyed, on a never-saved document (R9). Its own
            // region is published either way a reader might expect — no: the
            // region is published only when the control is drawn, so a driven
            // check can tell "the build has no Save button" from "the Save
            // button is off screen", which two adjacent recorded findings say
            // is otherwise the same screenshot.
            if self.has_file {
                let save_now = ui.button(t::save_button());
                crate::diag::ui_rect(REGION_SAVE_IN_PLACE, save_now.rect);
                if save_now.clicked() {
                    self.outcome = Some(Outcome::SaveInPlace);
                }
            }
            // **Save all** — `OPERATOR_REQUESTS.md` O102, and it is drawn
            // only when it would do more than the button to its left.
            //
            // Second, immediately after Save, because it is the same act at a
            // larger scope and the row's order is least-destructive-first. It
            // loses nothing, exactly as Save does.
            //
            // Absent rather than greyed on a single dirty document (R9): with
            // one document the two buttons are the same act, and this is not a
            // *temporarily* unavailable control that a hover sentence could
            // explain — there is simply nothing else to save.
            if self.dirty_documents > 1 {
                let all = ui.button(t::save_all_button(self.dirty_documents));
                crate::diag::ui_rect(REGION_SAVE_ALL, all.rect);
                if all.clicked() {
                    self.outcome = Some(Outcome::SaveAll);
                }
            }
            let save = ui.button(t::save_copy_button());
            crate::diag::ui_rect(REGION_SAVE, save.rect);
            if save.clicked() {
                self.outcome = Some(Outcome::SaveCopy);
            }
            let discard = ui.button(self.intent.discard_label());
            crate::diag::ui_rect(REGION_DISCARD, discard.rect);
            if discard.clicked() {
                self.outcome = Some(Outcome::Discard);
            }
            let cancel = ui.button(t::cancel_button());
            crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
            if cancel.clicked() {
                self.cancelled = true;
            }
        });

        ui.add_space(8.0);
        // The disclosure that makes the first button honest, and it is
        // BELOW the buttons rather than above them on purpose: it is what an
        // operator needs after they have noticed the button says "a copy" and
        // wondered why, and putting it above would make three sentences stand
        // between the question and the answer.
        // **Which sentence, and it is decided by which BUTTONS are on
        // screen** — `OPERATOR_REQUESTS.md` O65.
        //
        // With one writing button the note explains what "a copy" means for
        // the file they came from, which is the surprising half.
        //
        // With two, the surprising half changes: the buttons now differ only
        // in DESTINATION, four words apiece, and nothing on the surface says
        // which one touches the original. `save_choice_note` says it. Drawing
        // the copy note instead would be worse than saying nothing — it would
        // describe one of the two buttons as though it were the whole choice.
        let note = if self.has_file {
            t::save_choice_note()
        } else {
            t::save_copy_note()
        };
        ui.label(egui::RichText::new(note).small().weak());
    }
}

/// Ask about `intent` if the document in `status` has unsaved edits.
#[must_use]
pub fn ask_for_cycle(
    status: &Status,
    intent: PendingIntent,
    dirty: usize,
) -> Option<UnsavedDialog> {
    let mut dialog = ask_for(status, intent)?;
    dialog.dirty_documents = dirty;
    Some(dialog)
}

/// See [`ask_for_cycle`] for the quit cycle's form.
pub fn ask_for(status: &Status, intent: PendingIntent) -> Option<UnsavedDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    //
    // This line used to read `if doc.edit_epoch == 0`, which asks *"has
    // anything EVER been edited"* and is therefore permanently true after the
    // first edit. A document the operator had just saved was still asked
    // about, the prompt's only save button is "Save a copy…" — a picker — and
    // succeeding at that picker proceeds with the pending intent. So pressing
    // Save and then Close produced: a filename prompt, and then the document
    // closing. That is his report, and Save never closed anything.
    //
    // The old comment's argument was sound and is kept: two independent
    // notions of "edited" would eventually disagree. The correction is that
    // there were THREE, and the answer is one predicate rather than one
    // *expression* copied to three places.
    if !crate::app::save::has_unsaved_edits(doc) {
        return None;
    }
    // The count is measured from the last SAVE, not from zero. "You have 12
    // unsaved changes" after saving eleven of them was a true count of the
    // wrong thing.
    Some(UnsavedDialog::new(
        intent,
        doc.edit_epoch.saturating_sub(doc.saved_epoch),
        // Whether a Save button is drawn at all — O65. `has_a_file` is the
        // same predicate `file.save` itself consults before deciding between
        // an overwrite and a picker, so the button offered here and the verb
        // behind it cannot disagree about whether there is a file.
        crate::app::save::has_a_file(doc),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An unedited document is closed without a question.
    #[test]
    fn an_unedited_document_is_not_asked_about() {
        assert!(ask_for(&Status::Empty, PendingIntent::Close).is_none());
    }

    /// Each intent asks its own question and offers its own destructive label.
    #[test]
    fn the_four_intents_do_not_share_a_sentence() {
        let close = PendingIntent::Close;
        let open = PendingIntent::Open(std::path::PathBuf::from("a.pdf"));
        let new = PendingIntent::New;
        assert_ne!(close.question(), open.question());
        assert_ne!(close.question(), new.question());
        assert_ne!(open.question(), new.question());
        assert_ne!(close.discard_label(), open.discard_label());
        assert_ne!(close.discard_label(), new.discard_label());
        assert_ne!(open.discard_label(), new.discard_label());
    }

    /// A sized New asks the same question as a plain New.
    #[test]
    fn both_kinds_of_new_ask_one_question() {
        let sized = PendingIntent::NewSized {
            width_pt: 595.0,
            height_pt: 842.0,
        };
        assert_eq!(sized.question(), PendingIntent::New.question());
        assert_eq!(sized.discard_label(), PendingIntent::New.discard_label());
    }

    /// The answer is a one-shot, and it comes back with its own intent.
    #[test]
    fn the_answer_fires_once_and_carries_its_intent() {
        let mut d = UnsavedDialog::new(PendingIntent::Close, 3, true);
        assert_eq!(d.take_outcome(), None);
        d.outcome = Some(Outcome::Discard);
        assert_eq!(
            d.take_outcome(),
            Some((PendingIntent::Close, Outcome::Discard))
        );
        assert_eq!(d.take_outcome(), None, "it must not repeat");
    }

    /// **A parked answer is visible to the owner until it is drained, and
    /// not after.**
    #[test]
    fn an_answer_is_visible_until_it_is_taken_and_not_after() {
        let mut d = UnsavedDialog::new(PendingIntent::Open("a.pdf".into()), 7, true);
        assert!(!d.answered(), "nobody has answered it yet");
        d.outcome = Some(Outcome::SaveCopy);
        assert!(d.answered(), "a button parked an answer");
        assert!(d.take_outcome().is_some());
        assert!(
            !d.answered(),
            "the drain emptied it, so the next frame may retire the window"
        );
    }

    /// Cancelling closes the window and answers nothing.
    #[test]
    fn cancelling_answers_nothing() {
        let mut d = UnsavedDialog::new(PendingIntent::Close, 1, true);
        d.cancelled = true;
        assert_eq!(d.take_outcome(), None);
        assert!(
            !d.answered(),
            "so the window is retired on the frame it closes"
        );
    }
}
