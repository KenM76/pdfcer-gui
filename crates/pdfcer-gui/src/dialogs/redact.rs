//! # `dialogs::redact` — the Apply-redactions transaction
//!
//! The body of `edit.redact_apply`, and the **irreversible** half of the
//! redaction feature. Its reversible twin is [`crate::panels::redact`], and the
//! split between them is the distinction
//! `crate::text::commands::edit_redact`'s shipped tooltip already draws:
//! *"Marking is reversible; applying is not."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/redact.md`.

mod staged;

mod disclosures;

use std::path::{Path, PathBuf};

use egui_shell::theme::Theme;

use crate::app::prefs::RedactionReach;
use crate::app::state::{OpenDoc, Status};
use crate::redact::{
    PreparedRedaction, RedactApplyRefusal, ResidualAcknowledgement, WriteRefusal,
    prepare_redaction_apply,
};
use crate::text::redact as t;

use destination::{DEFAULT_DESTINATION, Destination};
use pdfcer_gui_base::redactdestination as destination;

// ---------------------------------------------------------------------------
// Named regions
//
// Matched LITERALLY by `tools/ui-verify/src/checks/redaction.rs`, so renaming
// one silently un-aims the check that measures it. See `crate::dialogs::ocr`'s
// equivalent block for why a dialog needs these when a ribbon control gets its
// rect for free.
// ---------------------------------------------------------------------------

/// The whole window.
const REGION_DIALOG: &str = "redact-apply-dialog"; // ui-text-exempt: trace region name, never displayed

/// The mandatory confirmation checkbox.
const REGION_ACK: &str = "redact-apply-ack"; // ui-text-exempt: trace region name, never displayed

/// The extra acknowledgement, declared **only while it exists** — which is
/// itself the assertion a harness wants, since its presence is evidence that
/// the report disclosed a residual.
const REGION_RESIDUAL_ACK: &str = "redact-apply-residual-ack"; // ui-text-exempt: trace region name, never displayed

/// The control that commits.
const REGION_CONFIRM: &str = "redact-apply-confirm"; // ui-text-exempt: trace region name, never displayed

/// The *replace the original* destination choice, declared **only while the
/// document has an original to replace** — so its absence from a trace is
/// evidence about the document rather than about the build.
const REGION_DESTINATION_REPLACE: &str = "redact-apply-destination-replace"; // ui-text-exempt: trace region name, never displayed

/// The third acknowledgement, declared **only while it is being asked for** —
/// i.e. only while the operator has chosen to replace the original.
const REGION_OVERWRITE_ACK: &str = "redact-apply-overwrite-ack"; // ui-text-exempt: trace region name, never displayed

/// The *this document* destination choice — the default since 2026-09-04.
const REGION_DESTINATION_INTO_DOCUMENT: &str = "redact-apply-destination-into-document"; // ui-text-exempt: trace region name, never displayed
/// The *this document, now* radio, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
const REGION_DESTINATION_INTO_DOCUMENT_NOW: &str = "redact-apply-destination-into-document-now";

/// The *a new file* destination choice, also declared unconditionally.
const REGION_DESTINATION_NEW_FILE: &str = "redact-apply-destination-new-file"; // ui-text-exempt: trace region name, never displayed

/// The staging disclosure, declared **only while it is on screen** — i.e. only
/// while the deferred destination is selected.
const REGION_STAGING_NOTE: &str = "redact-apply-staging-note"; // ui-text-exempt: trace region name, never displayed

/// Height kept clear below the report for the checkbox and button rows.
const FOOTER_RESERVE: f32 = 150.0;

/// The least height the report may be given.
const REPORT_FLOOR: f32 = 120.0;

/// Where one apply transaction has got to.
#[derive(Debug)]
enum Phase {
    /// The removal ran, the proof passed, and the bytes are waiting for a
    /// confirmation. `Box`ed because this variant is far larger than its
    /// siblings and a `match` on the enum would otherwise move the whole
    /// document around.
    Prepared(Box<PreparedRedaction>),
    /// The apply was refused before anything was written.
    Refused(RedactApplyRefusal),
    /// **A removal is already armed on this document** (`Pass 250.2`, new
    /// 2026-09-05).
    ///
    /// A phase of its own rather than a [`Self::Refused`] carrying
    /// [`RedactApplyRefusal::AlreadyStaged`], and the two questions are why: an
    /// un-staged document asks *"shall I?"* and a staged one asks *"what did I
    /// already decide, and can I change my mind?"*. A refusal answers the first
    /// question badly instead of the second one well, and — critically — a
    /// refusal has no control on it, which would leave the operator staring at
    /// the reason he cannot save with nothing to press.
    ///
    /// It carries no data. Everything the phase says is true of any staged
    /// document, and the one thing it might have carried — the preview report —
    /// is a **stale measurement** by the time this phase is drawn: the removal
    /// re-runs at the save over whatever the document says then. Quoting it
    /// here would present yesterday's numbers as today's, which is precisely
    /// what `crate::redact::StagedRedaction`'s own doc comment warns about.
    Staged,
    /// The bytes reached this path.
    ///
    /// It carries the three numbers the outcome sentence needs rather than
    /// the [`PreparedRedaction`] they came from. Keeping the prepared value
    /// alive after the write would mean holding a second copy of a redacted
    /// document in memory for as long as the operator leaves the window open,
    /// for no purpose — the bytes are on disk and cannot be written twice from
    /// here. The counts are what the sentence is about.
    ///
    /// `residuals` is the field that decides **which** sentence: the catalog's
    /// rule 1 is that a leftover is named in the same sentence as the success,
    /// so a zero and a non-zero here are two different pieces of copy rather
    /// than one with a number in it.
    Written {
        /// Where the operator put it.
        path: PathBuf,
        /// Whether that path was the document that is open — i.e. whether the
        /// source file was replaced rather than a copy written beside it.
        ///
        /// Carried rather than re-derived by comparing `path` to `source`,
        /// because the outcome sentence must describe **what happened**, and a
        /// comparison performed later answers a question about the paths as
        /// they are now. It is also the difference between two sentences that
        /// say opposite things about the window the operator is looking at.
        replaced: bool,
        /// `RedactionReport::marks_applied`.
        regions: u64,
        /// `RedactionReport::pages_redacted`.
        pages: usize,
        /// How many items the report disclosed as NOT removed.
        residuals: usize,
    },
    /// A destination was named and no file appeared.
    WriteFailed(WriteRefusal),
}

/// The Apply-redactions dialog.
#[derive(Debug)]
pub struct RedactDialog {
    /// The document's own path, for suggesting a name to save under.
    ///
    /// Captured on construction rather than read per frame, for
    /// `crate::dialogs::ocr`'s reason applied to the file rather than to the
    /// page: nothing can change it while the dialog is open, and reading it
    /// from a `&OpenDoc` at save time would make the suggestion depend on a
    /// borrow the write path does not otherwise need.
    source: PathBuf,
    /// The transaction's state.
    phase: Phase,
    /// The mandatory acknowledgement.
    acknowledged: bool,
    /// The extra acknowledgement, meaningful only when the report has
    /// residuals.
    ///
    /// Two flags rather than one, deliberately: they answer different
    /// questions, and a single flag would let an operator who understood the
    /// permanence be treated as having read a residual list they were never
    /// shown.
    residuals_acknowledged: bool,
    /// Where the redacted document goes. [`Destination::OpenDocument`] until
    /// the operator says otherwise — see that type for the whole argument, and
    /// for why the default moved on 2026-09-04.
    destination: Destination,
    /// The **third** acknowledgement: that replacing the original destroys the
    /// last copy of the content being removed.
    ///
    /// A third flag rather than folding it into
    /// [`Self::acknowledged`], on this dialog's own standing reason for keeping
    /// the first two apart: they answer different questions, and a shared flag
    /// would let an operator who ticked one be treated as having read the
    /// other. Here the asymmetry is sharper still — the permanence box is about
    /// the *content*, and this one is about the *file*. A person can perfectly
    /// well understand that the text is going for good and not have noticed
    /// that the document they opened is going with it.
    ///
    /// It is only *asked for* while [`Destination::ReplaceOriginal`] is
    /// selected, and only *required* then. Left ticked from an earlier
    /// selection it is harmless, because the destination it applies to is read
    /// at the same instant.
    overwrite_acknowledged: bool,
    /// Set by the confirm control, consumed by [`Self::show`] after the
    /// window's closure returns.
    ///
    /// The two-step every dialog here uses, for a stronger reason than most:
    /// this is the irreversible half, and an `rfd` modal opened from inside an
    /// `egui::Window` closure blocks the frame it is being drawn in.
    confirm_requested: bool,
    /// Set by the *Call the removal off* control in [`Phase::Staged`], consumed
    /// by [`Self::show`] after the window's closure returns.
    ///
    /// A second flag rather than a reuse of [`Self::confirm_requested`], for
    /// the reason this file keeps its acknowledgement flags apart: the two
    /// presses are opposite acts on the one operation that cannot be undone
    /// once it reaches a file, and a shared flag with a phase test would make
    /// *arm* and *disarm* one code path distinguished by state.
    cancel_requested: bool,
    /// Set by the Close control; same two-step, because a widget drawn from the
    /// state cannot drop the state it is being drawn from.
    close_requested: bool,
}

impl RedactDialog {
    /// **Prepare the redaction and build the dialog around the answer.**
    fn open(doc: &OpenDoc, reach: RedactionReach) -> Self {
        let phase = match prepare_redaction_apply(&doc.session, reach) {
            Ok(prepared) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // Emitted at PREPARE rather than only at write, so a
                        // harness can tell "the removal ran and the operator
                        // did not confirm" from "the removal never ran". The
                        // two look identical from the file system.
                        "redact-prepared marks={} pages={} glyphs={} streams={} checked={} \
                         short={} residuals={} verified={} bytes={}",
                        prepared.report.marks_applied,
                        prepared.report.pages_redacted,
                        prepared.report.glyphs_removed,
                        prepared.report.content_streams_rewritten,
                        prepared.verification.strings_checked,
                        prepared.verification.strings_too_short_for_raw_check,
                        prepared.verification.residuals.len(),
                        prepared.verification.is_clean(),
                        prepared.byte_len(),
                    )
                });
                Phase::Prepared(Box::new(prepared))
            }
            // The staged state arrives as a REFUSAL from the pipeline and
            // is turned into a phase here, rather than being detected by asking
            // `doc.session.has_pending_redaction()` before the call.
            //
            // That is deliberate and it is this project's standing preference
            // for a mechanism over a condition. `prepare_redaction_apply`
            // refuses `AlreadyStaged` by name because it must — while a removal
            // is armed the engine declines `to_full_bytes`, and without the
            // named refusal the operator would read *"this document cannot be
            // rewritten in full"* — so the fact is established in the pipeline
            // whatever this dialog does. Asking the session again here would be
            // a second, independent test of the same condition, and the day the
            // two disagreed the dialog would be the one that was wrong.
            Err(RedactApplyRefusal::AlreadyStaged) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "redact-already-staged".to_owned()
                });
                Phase::Staged
            }
            Err(refusal) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("redact-refused reason={refusal:?}")
                });
                Phase::Refused(refusal)
            }
        };
        Self {
            source: doc.path.clone(),
            phase,
            acknowledged: false,
            residuals_acknowledged: false,
            destination: DEFAULT_DESTINATION,
            overwrite_acknowledged: false,
            confirm_requested: false,
            cancel_requested: false,
            close_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when the dialog should close.
    pub(super) fn show(
        &mut self,
        ctx: &egui::Context,
        _doc: &OpenDoc,
        actions: &mut Vec<crate::app::actions::Action>,
    ) -> bool {
        // §4 — read BEFORE the body draws its checkboxes, so a box ticked on
        // this frame does not enable the confirm control until the next one.
        let ready = self.ready_to_confirm();

        //
        let (frame, ()) = crate::dialogs::host::Host::new(
            "redact-apply", // ui-text-exempt: a viewport key, never displayed.
            t::apply_title(),
            egui::vec2(760.0, 560.0),
            egui::vec2(480.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_DIALOG, ui.max_rect());
            self.body(ui, ready);
        });
        let open = !frame.closed;

        // The irreversible half, after the closure. See `confirm_requested`.
        if std::mem::take(&mut self.confirm_requested) {
            self.commit(actions);
        }
        // …and the reversible one, which still waits for the closure to
        // return. See [`Self::take_cancel`].
        self.take_cancel(actions);
        open && !std::mem::take(&mut self.close_requested)
    }

    /// **Consume the *call the removal off* press, if there was one.**
    fn take_cancel(&mut self, actions: &mut Vec<crate::app::actions::Action>) {
        if !std::mem::take(&mut self.cancel_requested) {
            return;
        }
        actions.push(crate::app::actions::Action::Redact(
            crate::app::actions::RedactAction::Pending(crate::redact::Staging::Cancel),
        ));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "redact-cancel-requested".to_owned()
        });
        self.close_requested = true;
    }

    /// Whether the confirm control may be enabled.
    fn ready_to_confirm(&self) -> bool {
        let Phase::Prepared(prepared) = &self.phase else {
            return false;
        };
        self.acknowledged
            && (residual_lines(prepared).is_empty() || self.residuals_acknowledged)
            // The overwrite acknowledgement is owed by ONE destination, and
            // it is spelled as that destination rather than as "not NewFile".
            // The negative form was correct while there were two choices and
            // became wrong the moment there were three — it would have demanded
            // an overwrite acknowledgement from the destination that overwrites
            // nothing, which is a control the operator cannot satisfy because it
            // is not on screen.
            && (self.destination != Destination::ReplaceOriginal || self.overwrite_acknowledged)
    }

    /// **The staging disclosure, or nothing** — the sentence drawn between
    /// the destination choice and the confirm control.
    fn staging_disclosure(&self) -> Option<&'static str> {
        // Asked as *"does this write a file?"* rather than by naming the
        // variant: the disclosure belongs to the destination that defers, which
        // is precisely the one that does not write, and a fourth deferred
        // destination would inherit it rather than have to be remembered here.
        (!self.destination.writes_now()).then(t::removal_happens_at_save)
    }

    /// Whether replacing the open document is an option at all.
    fn can_replace_original(&self) -> bool {
        self.source.is_file()
    }

    /// **Take the destination choice, and retire the acknowledgement that was
    /// given about the previous one.**
    fn choose_destination(&mut self, choice: Destination) {
        if choice != self.destination {
            self.overwrite_acknowledged = false;
            self.destination = choice;
        }
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui, ready: bool) {
        let theme = Theme::of(ui.ctx());
        match &self.phase {
            Phase::Prepared(prepared) => {
                let residuals = residual_lines(prepared);
                Self::report(ui, &theme, prepared, &residuals, self.destination);
                ui.add_space(8.0);
                ui.separator();
                self.gates(ui, &residuals, ready);
            }
            Phase::Refused(refusal) => {
                ui.label(t::report_heading());
                ui.add_space(6.0);
                ui.label(t::refusal_message(refusal));
            }
            // The armed-removal phase. Its whole body is in `staged`, which
            // owns the sentences and the one control, so that this `match`
            // stays a list of states rather than becoming a place where one of
            // them is drawn and the others are dispatched.
            Phase::Staged => {
                self.cancel_requested |= staged::body(ui, &theme);
            }
            Phase::Written {
                path,
                replaced,
                regions,
                pages,
                residuals,
            } => {
                ui.label(outcome_line(path, *regions, *pages, *residuals, *replaced));
            }
            Phase::WriteFailed(reason) => {
                ui.label(t::write_failed(reason));
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The measured report: what will be removed, what was verified, and what
    /// could not be.
    fn report(
        ui: &mut egui::Ui,
        theme: &Theme,
        prepared: &PreparedRedaction,
        residuals: &[String],
        destination: Destination,
    ) {
        ui.label(t::report_heading());
        ui.add_space(6.0);
        // The permanence statement is FIRST in the body and in the warning
        // role — never fine print, never below the counts. It is the one
        // sentence a reader who takes in nothing else must take in.
        //
        let permanence = match destination {
            Destination::OpenDocument => t::permanence_statement_deferred(),
            // Content goes, no file moves — a combination neither neighbour
            // describes. See `permanence_statement_now`.
            Destination::OpenDocumentNow => t::permanence_statement_now(),
            Destination::NewFile => t::permanence_statement(false),
            Destination::ReplaceOriginal => t::permanence_statement(true),
        };
        ui.label(egui::RichText::new(permanence).color(theme.palette.danger));
        ui.add_space(6.0);
        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt(REGION_DIALOG)
            .auto_shrink([false, true])
            .max_height((ui.available_height() - FOOTER_RESERVE).max(REPORT_FLOOR))
            .show(ui, |ui| {
                let report = &prepared.report;
                ui.label(t::will_remove_heading());
                ui.add_space(4.0);
                ui.label(t::removal_summary(
                    report.marks_applied,
                    report.pages_redacted,
                    report.glyphs_removed,
                    report.content_streams_rewritten,
                ));
                if report.annotations_removed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::annotations_removed(report.annotations_removed));
                }
                if report.info_strings_scrubbed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::info_scrubbed(report.info_strings_scrubbed));
                }
                // …and what the SAME sweep found in the rest of the file.
                // Directly beneath the line above because they are two halves
                // of one question, and the engine counts them apart precisely
                // so a shell can say which half a number came from.
                disclosures::sweep(ui, report);
                if report.images_cleared > 0 || report.images_removed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::images_destroyed(
                        report.images_cleared,
                        report.images_removed,
                        report.images_overcovered,
                    ));
                }
                // Separate, because it is a different claim: the same picture
                // is still on the other pages, and "I redacted the logo" and
                // "the logo is gone from this file" are not the same sentence.
                if report.images_cloned_shared > 0 {
                    ui.add_space(4.0);
                    ui.label(t::images_shared_copied(report.images_cloned_shared));
                }
                // The drawn geometry that was cut out. New in `pdfcer-core`
                // v0.27.0 and worth a line of its own on a CAD sheet: before
                // it, lines ran straight through a redacted rectangle and
                // nothing said so. This is the count that makes "the drawing
                // under the box is gone" a statement rather than an assumption.
                if report.vector_paths_cut > 0 {
                    ui.add_space(4.0);
                    ui.label(t::vector_paths_cut_line(
                        report.vector_paths_cut,
                        report.vector_paths_dropped,
                    ));
                }
                if report.containers_decomposed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::containers_decomposed(
                        report.containers_decomposed,
                        report.objects_promoted,
                    ));
                }
                // …and now WHAT, rather than how much. `OPERATOR_REQUESTS.md`
                // O217's fourth bullet: every line above is a count, and a count
                // cannot be checked against an intention. Last in the removal
                // block because it is the only part of it that can run to
                // hundreds of lines, and the counts must stay above the fold.
                disclosures::removed_text(ui, theme, report);
                ui.add_space(4.0);
                ui.label(t::single_revision_note());

                // --- the proof -------------------------------------------
                //
                // "Verified" only from a clean verification that actually
                // checked something — the catalog's rule 2, enforced at the
                // one call site entitled to the word.
                let verification = &prepared.verification;
                if verification.is_clean() && verification.strings_checked > 0 {
                    ui.add_space(8.0);
                    ui.label(t::verified_line(verification.strings_checked));
                }
                if verification.strings_too_short_for_raw_check > 0 {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(t::verification_limit_line(
                            verification.strings_too_short_for_raw_check,
                        ))
                        .color(theme.palette.text_muted),
                    );
                }
                // The carriers the engine looked inside and found clean.
                // With the proof rather than with the counts, because it is the
                // same kind of statement — evidence that a check ran — and it
                // is the difference between "nothing to do" and "checked,
                // clean". See `disclosures::checked_clean`.
                disclosures::checked_clean(ui, theme, report);

                // --- what will be left, because he asked for it -----------
                //
                // Above the residual section so the danger colour stays last.
                // Notice weight, never danger: the engine keeps "told not to"
                // apart from "could not" so a deliberate scope does not read as
                // a fault, and this shell keeps that distinction visible.
                disclosures::left_by_choice(ui, theme, report);

                // --- what could not be removed ----------------------------
                if !residuals.is_empty() {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label(
                        egui::RichText::new(t::residual_heading()).color(theme.palette.danger),
                    );
                    ui.add_space(4.0);
                    for line in residuals {
                        ui.label(egui::RichText::new(line).color(theme.palette.danger));
                        ui.add_space(4.0);
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label(egui::RichText::new(t::scope_reminder()).color(theme.palette.text_muted));

                // Last, and collapsed. `residual_sweep_line` promises the
                // operator that the object numbers are "at the foot of this
                // report"; this is the foot, and this is where they are.
                disclosures::engine_notes(ui, theme, report);
            });
    }

    /// The two checkboxes, the confirm control, and the no-shortcut note.
    fn gates(&mut self, ui: &mut egui::Ui, residuals: &[String], ready: bool) {
        // The destination, ABOVE the acknowledgements and above the
        // confirm control, because it changes what two of them say. An
        // operator who ticked "I understand this is permanent" and then chose
        // to replace the original would have acknowledged a sentence that was
        // not yet about the thing they went on to do.
        //
        // Radio buttons rather than two confirm controls: the choice is one
        // state with two values, it is read back at the write, and a pair of
        // buttons would put two irreversible verbs side by side where a
        // mis-aimed click lands on the wrong one. Drawn only when there is an
        // original to replace — see `can_replace_original`.
        //
        ui.label(t::destination_heading());
        ui.add_space(2.0);
        let mut choice = self.destination;
        let into = ui.radio_value(
            &mut choice,
            Destination::OpenDocument,
            t::destination_open_document(),
        );
        crate::diag::ui_rect(REGION_DESTINATION_INTO_DOCUMENT, into.rect);
        into.on_hover_text(t::destination_open_document_tooltip());
        let now = ui.radio_value(
            &mut choice,
            Destination::OpenDocumentNow,
            t::destination_open_document_now(),
        );
        crate::diag::ui_rect(REGION_DESTINATION_INTO_DOCUMENT_NOW, now.rect);
        now.on_hover_text(t::destination_open_document_now_tooltip());
        let new_file = ui.radio_value(&mut choice, Destination::NewFile, t::destination_new_file());
        crate::diag::ui_rect(REGION_DESTINATION_NEW_FILE, new_file.rect);
        new_file.on_hover_text(t::destination_new_file_tooltip());
        if self.can_replace_original() {
            let name = file_name_of(&self.source);
            let replace = ui.radio_value(
                &mut choice,
                Destination::ReplaceOriginal,
                t::destination_replace(&name),
            );
            crate::diag::ui_rect(REGION_DESTINATION_REPLACE, replace.rect);
            replace.on_hover_text(t::destination_replace_tooltip());
        }
        self.choose_destination(choice);
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // **The staging disclosure, ABOVE the confirm control.**
        //
        //
        // What is drawn instead is the fact that replaces it, and it is more
        // surprising rather than less: **the page does not change**. He presses
        // a control about permanent removal and the marks and the content stay
        // exactly where they are, because the removal happens at the save. The
        // two readings available to him without this sentence are *"it did not
        // work"* and *"it worked and the marks are just still drawn"*, and the
        // second is the one that ships a marked file.
        //
        // A sentence and not a fourth checkbox, deliberately, and that part
        // of the old argument is untouched. This dialog's §3 argues that a box
        // which is always there is a box that is always ticked, and the same
        // erosion applies to boxes that multiply: four acknowledgements is a
        // form, and a form is filled in rather than read. The operator is
        // already gated on the permanence box; what he is owed here is the
        // FACT, before the click, which is what rule 4 asks for and what
        // "told, not asked" means.
        //
        // Drawn only for the destination it is true of. The two write-now
        // destinations really do produce a file at the click.
        if let Some(sentence) = self.staging_disclosure() {
            // `danger`, not `notice`, and the palette's own split decides it:
            // notice is *"worth knowing and nothing is broken"*, and a
            // permanent removal an operator has just armed without seeing
            // anything happen is the sharpest end of this dialog.
            let danger = Theme::of(ui.ctx()).palette.danger;
            let note = ui.label(egui::RichText::new(sentence).color(danger));
            crate::diag::ui_rect(REGION_STAGING_NOTE, note.rect);
            ui.add_space(6.0);
        }
        // Shown only when the operator has actually asked to replace the
        // original, for the same reason the residual box is conditional: a
        // permanent checkbox is a permanent reflex.
        if self.destination == Destination::ReplaceOriginal {
            let name = file_name_of(&self.source);
            let box_ = ui.checkbox(
                &mut self.overwrite_acknowledged,
                t::overwrite_acknowledgement_checkbox(&name),
            );
            crate::diag::ui_rect(REGION_OVERWRITE_ACK, box_.rect);
            ui.add_space(4.0);
        }
        // Shown only when there is something to acknowledge — §3 item 2.
        if !residuals.is_empty() {
            let box_ = ui.checkbox(
                &mut self.residuals_acknowledged,
                t::residual_acknowledgement_checkbox(),
            );
            crate::diag::ui_rect(REGION_RESIDUAL_ACK, box_.rect);
            ui.add_space(4.0);
        }
        let ack = ui.checkbox(&mut self.acknowledged, t::confirm_checkbox());
        //
        // Published unconditionally it was a **fossil**: the region appeared in
        // the trace while the control was scrolled out of view, so a driven
        // check clicked a phantom, the acknowledgement never took, and the
        // confirm stayed disabled — reported as *"the acknowledgement was
        // clicked and the confirm control is still not offered"*, which sent
        // the reader looking at the gates rather than at the scroll position.
        //
        // ⇒ The rule this file already states elsewhere: `ui_rect_visible` for
        // a control a check will CLICK; `ui_rect` for a section it scrolls TO.
        // An absence now means *not reachable*, which is the truth and is
        // actionable.
        crate::diag::ui_rect_visible(REGION_ACK, ack.rect, ui.clip_rect());
        ui.add_space(8.0);

        // The label IS the consequence, and the consequence now depends on
        // the destination: an ellipsis promises the picker, and naming the file
        // promises there will be no further question before it is replaced.
        let label = match self.destination {
            // No ellipsis and no file name: nothing is written, so there is no
            // further question and no file to name.
            Destination::OpenDocument => t::confirm_button_into_document().to_owned(),
            Destination::OpenDocumentNow => t::confirm_button_into_document_now().to_owned(),
            Destination::NewFile => t::confirm_button().to_owned(),
            Destination::ReplaceOriginal => t::confirm_button_replace(&file_name_of(&self.source)),
        };
        let confirm = ui.add_enabled(ready, egui::Button::new(label));
        // Declared only while it is live, so its absence from a trace is
        // evidence the gates are closed rather than evidence a click missed.
        if ready {
            crate::diag::ui_rect(REGION_CONFIRM, confirm.rect);
        }
        let clicked = confirm.clicked();
        // **A greyed Confirm with no explanation at all** — O77's sweep,
        // and the most consequential of the seven: this is the last control
        // before content is destroyed, and an operator who cannot press it had
        // no way to find out why.
        //
        // It names WHICH box is unticked rather than refusing generically.
        // Two checkboxes gate this button and they appear at different times —
        // the residual one only when the engine reported residuals — so
        // *"tick the box"* would be ambiguous exactly when it matters.
        //
        // The `if !ready` shape, and the borrow order, are copied from
        // `dialogs::formfield` and `dialogs::textannot`:
        // `on_disabled_hover_text` CONSUMES the response, so `.rect` and
        // `.clicked()` are read first.
        if !ready {
            // Three OUTSTANDING flags, not three "acknowledged" ones. A box
            // that was never drawn is not owed, and sending the operator to
            // look for it would be the vague refusal this sentence exists to
            // prevent — so the conditions that decide whether each box appears
            // are the same expressions used here.
            confirm.on_disabled_hover_text(t::confirm_disabled(
                !self.acknowledged,
                !residuals.is_empty() && !self.residuals_acknowledged,
                self.destination == Destination::ReplaceOriginal && !self.overwrite_acknowledged,
            ));
        }
        if clicked {
            self.confirm_requested = true;
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new(t::no_shortcut_note()).small().weak());
    }

    /// **Send the redacted bytes to the destination the operator chose.**
    fn commit(&mut self, actions: &mut Vec<crate::app::actions::Action>) {
        let Phase::Prepared(prepared) = &self.phase else {
            return;
        };
        //
        // It leaves through the ACTION FUNNEL rather than being performed here,
        // and that reverses §5 of this file's header for one destination — the
        // section is corrected in place. The reason §5 gave for staying out of
        // the funnel was that applying *"changes no document, so it has nothing
        // to order against and no epoch to bump"*. On this destination it
        // changes the open document, so it has both: `vector_edit` cancels the
        // render worker, takes the session, bumps `edit_epoch`, invalidates the
        // page textures and resyncs the page set. Performing that from inside a
        // dialog's draw is exactly what the funnel exists to prevent.
        //
        // The dialog CLOSES rather than moving to an outcome phase, and the
        // outcome is reported by the funnel's edit disclosure like any other
        // edit. Two accounts of one event is worse than one: the action runs
        // after this frame, so a sentence written here would be a prediction
        // — and on the one path where the action failed, a false one.
        if self.destination.stages() {
            actions.push(crate::app::actions::Action::Redact(
                crate::app::actions::RedactAction::Pending(crate::redact::Staging::Stage),
            ));
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `marks=` is the number the operator was shown at the moment
                // of consent. The funnel's own `redact-staged` line records
                // what the engine's staging preview then measured; a build in
                // which the report on screen and the removal that gets armed
                // had drifted apart would show the two disagreeing.
                format!(
                    "redact-apply-deferred marks={}",
                    prepared.report.marks_applied
                )
            });
            self.close_requested = true;
            return;
        }
        //
        // It leaves through the action funnel for the same reason the staging
        // branch above does, and more strongly: this one replaces the open
        // session outright, so it needs the render worker cancelled, the
        // textures dropped and the page set resynced — every one of which is
        // `vector_edit`'s job and none of which a dialog's draw may do.
        //
        // The prepared BYTES are carried rather than re-derived. They are the
        // same bytes `NewFile` would write and they have already been through
        // `proof::prove` — re-preparing on the far side would be a second
        // removal, verified separately, and the two could disagree.
        //
        // ⚠ Handled here rather than in the destination match below, which
        // chooses a FILE PATH. This destination writes no file; falling into
        // that match would have made it ask for one.
        if self.destination == Destination::OpenDocumentNow {
            let acknowledgement = if self.residuals_acknowledged {
                ResidualAcknowledgement::Given
            } else {
                ResidualAcknowledgement::Withheld
            };
            let marks = prepared.report.marks_applied;
            let pages = prepared.report.pages_redacted;
            let size = prepared.byte_len();
            //
            // `into_verified_document` is the second sanctioned exit, shaped
            // like `write_to`: same two gates, same order, and it hands back a
            // parsed `Document` rather than a buffer anyone could write.
            match prepared.to_verified_document(acknowledgement) {
                Ok(document) => crate::redact::park_applied_document(document),
                Err(refusal) => {
                    // The same two refusals `write_to` raises, on the same
                    // gates in the same order — so an unacknowledged residual
                    // behaves identically whether the operator chose a file or
                    // the open document.
                    //
                    // `Phase::WriteFailed` although nothing was written, and
                    // the name is the only thing about it that does not fit:
                    // the phase means *the removal did not land and here is
                    // why*, which is exactly this. A parallel phase would be a
                    // second state with one meaning.
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("redact-apply-now-failed detail={refusal}")
                    });
                    self.phase = Phase::WriteFailed(refusal);
                    return;
                }
            }
            actions.push(crate::app::actions::Action::Redact(
                crate::app::actions::RedactAction::ApplyNow {
                    marks: marks as usize,
                    pages,
                },
            ));
            crate::diag::trace(move || {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `bytes=` is a LENGTH. `PreparedRedaction`'s hand-written
                // `Debug` exists so that `{:?}` cannot emit a redacted document
                // into a log, and a trace line that formatted the buffer would
                // defeat it from the other side.
                format!("redact-apply-now marks={marks} pages={pages} bytes={size}")
            });
            self.close_requested = true;
            return;
        }
        let acknowledgement = if self.residuals_acknowledged {
            ResidualAcknowledgement::Given
        } else {
            ResidualAcknowledgement::Withheld
        };
        let residuals = residual_lines(prepared).len();
        let regions = prepared.report.marks_applied;
        let pages = prepared.report.pages_redacted;
        let target = match self.destination {
            // Answered above and returned; spelled rather than left to a `_`
            // arm so that a future fourth destination is a compile error here
            // rather than a file written to the wrong place.
            // Both answered above and returned; spelled rather than left to a
            // `_` arm so that a FIFTH destination is a compile error here
            // rather than a file written to the wrong place.
            Destination::OpenDocument | Destination::OpenDocumentNow => return,
            // No picker: the consent for this path was taken in words, at the
            // radio and the third checkbox, before the click. See the table
            // above for why a pre-filled picker would be worse rather than
            // safer.
            Destination::ReplaceOriginal => self.source.clone(),
            Destination::NewFile => {
                let suggested = suggested_path(&self.source);
                let crate::app::files::Picked::Path(chosen) =
                    crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
                else {
                    // Cancelled, or a build with no picker. The prepared bytes
                    // are still in hand and the control is still there: nothing
                    // is lost and nothing is said, because a cancelled save is a
                    // complete and uninteresting outcome. The marks are
                    // untouched either way.
                    return;
                };
                chosen
            }
        };
        self.phase = match prepared.write_to(&target, acknowledgement) {
            Ok(_) => Phase::Written {
                path: target,
                replaced: self.destination == Destination::ReplaceOriginal,
                regions,
                pages,
                residuals,
            },
            Err(refusal) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("redact-write-failed path={target:?} detail={refusal}")
                });
                Phase::WriteFailed(refusal)
            }
        };
    }
}

/// **The file name a sentence should use for `path`.**
#[must_use]
fn file_name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// **The sentence shown once bytes are on disk.**
#[must_use]
fn outcome_line(
    path: &Path,
    regions: u64,
    pages: usize,
    residuals: usize,
    replaced: bool,
) -> String {
    let name = file_name_of(path);
    if residuals == 0 {
        t::applied_clean(&name, regions, pages, replaced)
    } else {
        t::applied_with_residuals(&name, regions, residuals, replaced)
    }
}

/// **Every item the report discloses as NOT removed.**
#[must_use]
fn residual_lines(prepared: &PreparedRedaction) -> Vec<String> {
    use pdfcer_core::redact::CarrierAction;
    let mut out: Vec<String> = prepared
        .report
        .carriers
        .iter()
        .filter(|c| c.action == CarrierAction::DisclosedNotScrubbed)
        .map(|c| t::residual_carrier_line(c.carrier))
        .collect();
    // RETAINED MARKS, and the engine names this as the one number to read
    // before the word "redacted" is used. A retained mark is a region where
    // NOTHING was removed — the image under it could not be decoded, so the
    // engine applied every other mark and left that one standing rather than
    // refusing the document. The result is a half-redacted file that looks
    // finished, which is precisely what this list exists to prevent.
    if prepared.report.marks_retained > 0 {
        out.push(t::marks_retained_line(prepared.report.marks_retained));
    }
    // Vector geometry crossing a region that could NOT be cut — a malformed
    // path object the engine cannot rewrite as a unit. Zero on every
    // well-formed page since `pdfcer-core` v0.27.0, which cuts paths at the
    // region boundary; a non-zero value here is therefore rare and is a real
    // residual, not the ordinary case.
    //
    // On a drawing this is the residual that matters most and the one nobody
    // asks about: a title-block border or a view's geometry running through a
    // redacted rectangle is a shape, and a shape can be as identifying as the
    // text it surrounded.
    if prepared.report.vector_paths_intersecting > 0 {
        out.push(t::vector_paths_residual_line(
            prepared.report.vector_paths_intersecting,
        ));
    }
    // A clip whose ink was cut and whose ORIGINAL outline had to stay: ISO
    // 32000-1 §8.5.4 applies a clip after painting, so shrinking it would hide
    // later, unmarked content. Nothing of it is visible and it is still a shape
    // in the file — exactly the finding rule 1 forbids judging harmless on the
    // operator's behalf.
    if prepared.report.vector_clips_kept > 0 {
        out.push(t::vector_clips_kept_line(prepared.report.vector_clips_kept));
    }
    out.extend(
        prepared
            .verification
            .residuals
            .iter()
            .map(|r| t::raw_residual_line(&r.text, r.site)),
    );
    if !prepared.promoted_by_materialisation.is_empty() {
        out.push(t::promotion_line(
            prepared.promoted_by_materialisation.len(),
        ));
    }
    out
}

/// **The name to suggest for the redacted copy.**
#[must_use]
pub fn suggested_path(source: &Path) -> PathBuf {
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy. Both sibling suggestion functions make the same one.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let name = format!("{stem}{}.pdf", t::suggested_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

/// Open the dialog for the document in `status`, if there is one.
pub(super) fn open_for(status: &Status, reach: RedactionReach) -> Option<RedactDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(RedactDialog::open(doc, reach))
}

/// The headless assertions for this dialog's state machine, in their own file
/// since 2026-09-04 — see [`tests`]'s header for the seam.
#[cfg(test)]
mod tests;
