//! # `dialogs::redact` — the Apply-redactions transaction
//!
//! The body of `edit.redact_apply`, and the **irreversible** half of the
//! redaction feature. Its reversible twin is [`crate::panels::redact`], and the
//! split between them is the distinction
//! `crate::text::commands::edit_redact`'s shipped tooltip already draws:
//! *"Marking is reversible; applying is not."*
//!
//! This is the only surface in pdfcer-gui that commits an operation nothing can
//! take back, and its whole shape follows from that.
//!
//! ## The four states
//!
//! | state | what the operator sees | what exists |
//! |---|---|---|
//! | **prepared** | the measured report, two checkboxes, and a control whose label is the consequence | the finished redacted bytes, **in memory** |
//! | **refused** | a named refusal, and nothing to confirm | nothing |
//! | **written** | where the file went, and what is still in it | a file |
//! | **write failed** | why no file appeared | nothing |
//!
//! There is deliberately no *ready* state. Opening this dialog **runs the whole
//! removal** — see §2 — so by the time anything is drawn the numbers on screen
//! are measurements of the exact bytes that will be written, not predictions
//! about bytes that do not exist yet.
//!
//! ## ★ 1. Why the report comes BEFORE the write, and the write asks for a
//! destination
//!
//! `crate::dialogs::ocr`'s argument, one operation further along the scale of
//! consequence. That dialog recognises, discloses what it inferred, and only
//! then offers to save — so *"the operator reads the disclosure while holding
//! the one thing that gives it force: the ability to not save."*
//!
//! Here the disclosure is not about inference, it is about **what will be
//! destroyed and what pdfcer could not destroy**, and the residual half of it is
//! the whole reason the feature can be trusted. A surface that redacted and
//! dropped a file picker in front of the operator would be technically
//! disclosive and practically a program that quietly shipped a partially
//! redacted document.
//!
//! ## ★ 2. Why the removal runs synchronously, on open
//!
//! It is the salvage source's shape and it is kept, with the trade stated
//! rather than inherited.
//!
//! The alternative is `crate::ocr::Job`'s: a worker thread, a spinner and a
//! poll. Everything needed for it is available — `OpenDoc::session` is an
//! `Arc<EditSession>` and every field of [`crate::redact::PreparedRedaction`]
//! is `Send`. It is not done, for one reason that decides it: **a report
//! computed on another thread is a report about a document that may have
//! changed by the time it is read.** OCR can tolerate that because it refuses
//! outright when `edit_epoch != 0`; a redaction cannot, because the marks the
//! operator is applying are the ones they have just made and the epoch is
//! moving by construction.
//!
//! Running it inside the dispatch that opens the dialog gives the report and
//! the bytes one consistent snapshot, taken at a moment the operator caused. The
//! cost is a frame that takes as long as a full rewrite of the document —
//! visible on a large sheet, and paid once, on a deliberate click, for the one
//! operation in the program where a stale answer would be a security defect.
//!
//! ## ★ 3. What confirmation actually consists of, and why it is not one click
//!
//! Three gates, and each closes a different failure:
//!
//! 1. **[`crate::text::redact::confirm_checkbox`]** — always present. Its
//!    wording targets the exact misunderstanding the feature exists to prevent:
//!    that applying removes the *marks* rather than the *content*.
//! 2. **[`crate::text::redact::residual_acknowledgement_checkbox`]** — present
//!    **only when the report has residuals**. Showing it always would make it a
//!    box operators tick without reading, which is how every acknowledgement in
//!    a program becomes worthless. It is also enforced below the UI, at
//!    [`crate::redact::PreparedRedaction::write_to`], because a greyed control
//!    is a drawing decision and not a mechanism.
//! 3. **A control whose label is the consequence** —
//!    *"Permanently remove & save as…"*, never "OK", never "Apply". The ellipsis
//!    is a promise that a further question is coming, and one is: the file
//!    picker.
//!
//! And a fourth thing that is an absence: **no keyboard shortcut, and no Enter
//! binding.** The footer says so in words rather than leaving it to be noticed.
//! Every other destructive verb in this shell is chorded and reversible; this
//! one is neither, and the asymmetry is deliberate.
//!
//! ## ★ 4. The `ready` flag is read one frame late, on purpose
//!
//! [`RedactDialog::show`] computes whether the confirm control may be enabled
//! **before** the checkboxes are drawn, so a checkbox ticked on this frame does
//! not enable the button until the next one. A fast double-click on the box
//! would otherwise land its second press on a control that became enabled
//! between the two — which on this dialog means an irreversible operation
//! reached by a gesture the operator made at a disabled control.
//!
//! ## 5. Why this dialog does not push an `Action`
//!
//! [`super`]'s rule: a dialog uses the action funnel when it edits **this**
//! document, and this one never does. Applying produces a *new file*; the open
//! document keeps its marks, its undo log and its epoch, and
//! `crate::text::redact::permanence_statement` says so on screen. What the
//! funnel's reasoning does still demand is that irreversible work not run
//! part-way through a layout pass — and it does not: the confirm control sets a
//! flag, and the picker and the write happen after the window's closure
//! returns.
//!
//! ## 6. It is document-scoped, and closing the document discards the bytes
//!
//! `crate::dialogs::ocr`'s ruling, and it matters more here: a redaction is of
//! *these marks* on *this file*, and writing prepared bytes after the operator
//! has put the document away would produce a redacted file derived from a
//! document nobody is looking at any more.

use std::path::{Path, PathBuf};

use egui_shell::theme::Theme;

use crate::app::state::{OpenDoc, Status};
use crate::redact::{
    PreparedRedaction, RedactApplyRefusal, ResidualAcknowledgement, WriteRefusal,
    prepare_redaction_apply,
};
use crate::text::redact as t;

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

/// Height kept clear below the report for the checkbox and button rows.
const FOOTER_RESERVE: f32 = 150.0;

/// The least height the report may be given.
///
/// Without a floor, a small window produces a scroll area that draws **nothing
/// at all** — `available_height()` minus a reservation goes negative, and a
/// negative `max_height` is a silently empty area rather than an error. On this
/// dialog that would be a confirmation with no report above it, which is the
/// one shape it must never take. The About and OCR dialogs record the same
/// trap.
const REPORT_FLOOR: f32 = 120.0;

/// Where one apply transaction has got to.
///
/// A state machine rather than several `Option`s, because the states are
/// mutually exclusive and an `Option` quadruple has combinations that would all
/// compile and none of which means anything.
#[derive(Debug)]
enum Phase {
    /// The removal ran, the proof passed, and the bytes are waiting for a
    /// confirmation. `Box`ed because this variant is far larger than its
    /// siblings and a `match` on the enum would otherwise move the whole
    /// document around.
    Prepared(Box<PreparedRedaction>),
    /// The apply was refused before anything was written.
    Refused(RedactApplyRefusal),
    /// The bytes reached this path.
    ///
    /// ★ It carries the three numbers the outcome sentence needs rather than
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
    /// Set by the confirm control, consumed by [`Self::show`] after the
    /// window's closure returns.
    ///
    /// The two-step every dialog here uses, for a stronger reason than most:
    /// this is the irreversible half, and an `rfd` modal opened from inside an
    /// `egui::Window` closure blocks the frame it is being drawn in.
    confirm_requested: bool,
    /// Set by the Close control; same two-step, because a widget drawn from the
    /// state cannot drop the state it is being drawn from.
    close_requested: bool,
}

impl RedactDialog {
    /// **Prepare the redaction and build the dialog around the answer.**
    ///
    /// The whole removal runs here — see §2 — so this call is as expensive as a
    /// full rewrite of the document, once, on a deliberate click.
    fn open(doc: &OpenDoc) -> Self {
        let phase = match prepare_redaction_apply(&doc.session) {
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
                        prepared.verification.raw_byte_residuals.len(),
                        prepared.verification.is_clean(),
                        prepared.byte_len(),
                    )
                });
                Phase::Prepared(Box::new(prepared))
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
            confirm_requested: false,
            close_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when the dialog should close.
    pub(super) fn show(&mut self, ctx: &egui::Context, _doc: &OpenDoc) -> bool {
        // ★ §4 — read BEFORE the body draws its checkboxes, so a box ticked on
        // this frame does not enable the confirm control until the next one.
        let ready = self.ready_to_confirm();

        // ★ ITS OWN OS WINDOW as of 2026-08-21, and of every dialog in this
        // directory this is the one where being able to move it off the
        // document matters most: the report lists what will be REMOVED, and
        // checking it against the page underneath was impossible while the
        // window covered that page.
        //
        // ★ The dialog region is published from inside the callback now — the
        // window response it used to come from no longer exists, and
        // `dialogs::host` tags what is published with this viewport so the
        // harness can convert it.
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
            self.commit();
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// Whether the confirm control may be enabled.
    ///
    /// Pure, and the whole of the gate's rule — so every property of it is
    /// asserted headlessly, which is `crate::viewer`'s standing split applied
    /// to the one control in the program that must not be enabled early.
    fn ready_to_confirm(&self) -> bool {
        let Phase::Prepared(prepared) = &self.phase else {
            return false;
        };
        self.acknowledged && (residual_lines(prepared).is_empty() || self.residuals_acknowledged)
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui, ready: bool) {
        let theme = Theme::of(ui.ctx());
        match &self.phase {
            Phase::Prepared(prepared) => {
                let residuals = residual_lines(prepared);
                Self::report(ui, &theme, prepared, &residuals);
                ui.add_space(8.0);
                ui.separator();
                self.gates(ui, &residuals, ready);
            }
            Phase::Refused(refusal) => {
                ui.label(t::report_heading());
                ui.add_space(6.0);
                ui.label(t::refusal_message(refusal));
            }
            Phase::Written {
                path,
                regions,
                pages,
                residuals,
            } => {
                ui.label(outcome_line(path, *regions, *pages, *residuals));
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
    ///
    /// Every optional line is drawn **only when its count is non-zero**. A
    /// report that listed "0 annotations removed" beside four real findings
    /// would train the operator to skim it, and the skim is what this whole
    /// surface exists to prevent.
    fn report(
        ui: &mut egui::Ui,
        theme: &Theme,
        prepared: &PreparedRedaction,
        residuals: &[String],
    ) {
        ui.label(t::report_heading());
        ui.add_space(6.0);
        // ★ The permanence statement is FIRST in the body and in the warning
        // role — never fine print, never below the counts. It is the one
        // sentence a reader who takes in nothing else must take in.
        ui.label(egui::RichText::new(t::permanence_statement()).color(theme.palette.danger));
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
                // ★★★ What happened to the raster images, stated even though it
                // is a success. `pdfcer-core` v0.26.0 destroys the covered
                // samples and re-encodes; before 2026-09-03 it refused the
                // document instead. A report that lists glyphs removed and says
                // nothing about an overwritten photograph has quietly picked
                // which irreversible act is worth mentioning.
                if report.images_cleared > 0 || report.images_removed > 0 {
                    ui.add_space(4.0);
                    ui.label(t::images_destroyed(
                        report.images_cleared,
                        report.images_removed,
                        report.images_overcovered,
                    ));
                }
                // ★ Separate, because it is a different claim: the same picture
                // is still on the other pages, and "I redacted the logo" and
                // "the logo is gone from this file" are not the same sentence.
                if report.images_cloned_shared > 0 {
                    ui.add_space(4.0);
                    ui.label(t::images_shared_copied(report.images_cloned_shared));
                }
                // ★★ The drawn geometry that was cut out. New in `pdfcer-core`
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
            });
    }

    /// The two checkboxes, the confirm control, and the no-shortcut note.
    fn gates(&mut self, ui: &mut egui::Ui, residuals: &[String], ready: bool) {
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
        crate::diag::ui_rect(REGION_ACK, ack.rect);
        ui.add_space(8.0);

        let confirm = ui.add_enabled(ready, egui::Button::new(t::confirm_button()));
        // Declared only while it is live, so its absence from a trace is
        // evidence the gates are closed rather than evidence a click missed.
        if ready {
            crate::diag::ui_rect(REGION_CONFIRM, confirm.rect);
        }
        let clicked = confirm.clicked();
        // ★★★ **A greyed Confirm with no explanation at all** — O77's sweep,
        // and the most consequential of the seven: this is the last control
        // before content is destroyed, and an operator who cannot press it had
        // no way to find out why.
        //
        // ★ It names WHICH box is unticked rather than refusing generically.
        // Two checkboxes gate this button and they appear at different times —
        // the residual one only when the engine reported residuals — so
        // *"tick the box"* would be ambiguous exactly when it matters.
        //
        // ★★ The `if !ready` shape, and the borrow order, are copied from
        // `dialogs::formfield` and `dialogs::textannot`:
        // `on_disabled_hover_text` CONSUMES the response, so `.rect` and
        // `.clicked()` are read first.
        if !ready {
            confirm.on_disabled_hover_text(t::confirm_disabled(
                self.acknowledged,
                self.residuals_acknowledged,
            ));
        }
        if clicked {
            self.confirm_requested = true;
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new(t::no_shortcut_note()).small().weak());
    }

    /// **Ask where the redacted copy goes, and write it there.**
    ///
    /// ★ It asks, every time, and the suggestion is never the file that was
    /// opened — see [`suggested_path`]. There is no "save over the original"
    /// branch to find, because there is none to write, and on this operation
    /// that is the difference between a copy and the destruction of the only
    /// remaining source of the content being removed.
    fn commit(&mut self) {
        let Phase::Prepared(prepared) = &self.phase else {
            return;
        };
        let acknowledgement = if self.residuals_acknowledged {
            ResidualAcknowledgement::Given
        } else {
            ResidualAcknowledgement::Withheld
        };
        let residuals = residual_lines(prepared).len();
        let regions = prepared.report.marks_applied;
        let pages = prepared.report.pages_redacted;
        let suggested = suggested_path(&self.source);
        let crate::app::files::Picked::Path(target) =
            crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
        else {
            // Cancelled, or a build with no picker. The prepared bytes are
            // still in hand and the control is still there: nothing is lost and
            // nothing is said, because a cancelled save is a complete and
            // uninteresting outcome. The marks are untouched either way.
            return;
        };
        self.phase = match prepared.write_to(&target, acknowledgement) {
            Ok(_) => Phase::Written {
                path: target,
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

/// **The sentence shown once bytes are on disk.**
///
/// Free rather than a method so the catalog's rule 1 — *a residual is named in
/// the same sentence as the success* — is decided by a pure function a test can
/// drive, rather than inside a `match` on a window's state.
///
/// The branch is on `residuals`, and the two sentences are genuinely different
/// copy rather than one with a number in it. An operator who acknowledged a
/// residual in this dialog and then closed it is owed a standing record of what
/// remains, and *"…and verified absent from the saved file"* would be a lie in
/// that case rather than merely an omission.
///
/// The **file name** rather than the whole path, because the sentence is read
/// in a window that is about 700 pt wide and a Windows path is routinely longer
/// than that. The full destination is on the trace line
/// `PreparedRedaction::write_to` emits, which is where a reader who needs it
/// will look.
#[must_use]
fn outcome_line(path: &Path, regions: u64, pages: usize, residuals: usize) -> String {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    if residuals == 0 {
        t::applied_clean(&name, regions, pages)
    } else {
        t::applied_with_residuals(&name, regions, residuals)
    }
}

/// **Every item the report discloses as NOT removed.**
///
/// The single expression that both gates the extra acknowledgement and prints
/// the section — one derivation, so a residual can never be listed without
/// being acknowledgeable or acknowledged without being listed. Three sources,
/// in this order:
///
/// 1. **carriers the engine could not scrub** — `CarrierAction::
///    DisclosedNotScrubbed`, the cardinal-rule-honest outcome for a carrier
///    this build cannot fully redact;
/// 2. **raw-byte residuals** — [`crate::redact::proof`]'s middle verdict, a
///    byte run that survives outside every decoded stream and that pdfcer
///    genuinely cannot classify;
/// 3. **retained marks** — regions where nothing was removed because the image
///    under them could not be decoded. The engine names this as the number to
///    read before saying "redacted", and it is the strongest kind of residual
///    on this list: the content is still there, under a rectangle that says it
///    is not;
/// 4. **vector geometry that could not be cut**, and **clips whose outline had
///    to be kept** — an outline on a drawing can be as identifying as the text
///    it surrounded;
/// 5. **objects promoted out of a compressed container** by materialising the
///    operator's unsaved edits (engine rule R38).
///
/// The last is the mildest and is listed anyway. Page content cannot live in
/// an object stream at all (ISO 32000-1 §7.5.7), so it cannot hold redacted
/// text — but it is a leftover of the operator's own edits, and a report that
/// silently drops the findings it judges harmless is a report whose judgement
/// the operator has no way to audit.
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
    // ★★★ RETAINED MARKS, and the engine names this as the one number to read
    // before the word "redacted" is used. A retained mark is a region where
    // NOTHING was removed — the image under it could not be decoded, so the
    // engine applied every other mark and left that one standing rather than
    // refusing the document. The result is a half-redacted file that looks
    // finished, which is precisely what this list exists to prevent.
    if prepared.report.marks_retained > 0 {
        out.push(t::marks_retained_line(prepared.report.marks_retained));
    }
    // ★★ Vector geometry crossing a region that could NOT be cut — a malformed
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
    // ★ A clip whose ink was cut and whose ORIGINAL outline had to stay: ISO
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
            .raw_byte_residuals
            .iter()
            .map(|text| t::raw_residual_line(text)),
    );
    if !prepared.promoted_by_materialisation.is_empty() {
        out.push(t::promotion_line(
            prepared.promoted_by_materialisation.len(),
        ));
    }
    out
}

/// **The name to suggest for the redacted copy.**
///
/// ★ **Never the file that was opened.** The suffix is what makes the default
/// answer a new document, so an operator who accepts the suggestion without
/// reading it cannot overwrite the one file that still contains the content
/// they are removing. That is the standing rule expressed as a default rather
/// than as a warning — a warning is something to click past.
///
/// The same shape and the same argument as `crate::app::save::suggested_path`
/// and `crate::dialogs::ocr::suggested_path`, with a different suffix, and the
/// extension is forced to `.pdf` for their reason: the bytes are a PDF whatever
/// the source was called.
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
///
/// The dispatch target for `edit.redact_apply`. Lives here rather than in
/// [`super::DialogsState`] only because it needs [`RedactDialog::open`]'s
/// private constructor; the guard it applies is the one `open_print` documents
/// — the ribbon control is gated on `doc.pages`, a chord bound to the same id is
/// not, and both are fixed by refusing here at the one place the dialog is
/// built.
pub(super) fn open_for(status: &Status) -> Option<RedactDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(RedactDialog::open(doc))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **The suggested name is never the file that was opened.**
    ///
    /// The standing rule as a default, and the single most consequential
    /// assertion in this module: the source file is the only remaining copy of
    /// the content being removed, so a default that pointed at it would make
    /// the safety of the operation depend on the operator reading a pre-filled
    /// field before pressing Enter.
    #[test]
    fn the_suggested_name_is_never_the_source_file() {
        let source = PathBuf::from("D:\\jobs\\4471\\Sheet 1.pdf");
        let suggested = suggested_path(&source);
        assert_ne!(suggested, source);
        assert_eq!(
            suggested,
            PathBuf::from("D:\\jobs\\4471\\Sheet 1-redacted.pdf")
        );
        assert_eq!(
            suggested.parent(),
            source.parent(),
            "the copy should land beside the original, where the operator will look for it"
        );
    }

    /// A capitalised extension still produces a `.pdf`, and a bare filename
    /// still produces a usable name.
    #[test]
    fn the_suggestion_is_always_a_usable_pdf_name() {
        for name in ["scan.PDF", "scan.pdf", "scan", "D:\\a.b.pdf"] {
            let suggested = suggested_path(Path::new(name));
            assert!(
                suggested.to_string_lossy().ends_with(".pdf"),
                "{name} suggested {suggested:?}"
            );
            assert_ne!(suggested, PathBuf::from(name));
        }
    }

    /// A dialog opened with nothing loaded is not built at all.
    ///
    /// The guard matters more here than for print: [`RedactDialog::open`] runs
    /// the whole removal, so one built against an empty shell would be a window
    /// that had done a full rewrite of nothing in order to refuse.
    #[test]
    fn no_document_means_no_dialog() {
        assert!(open_for(&Status::Empty).is_none());
    }

    /// ★★ **The confirm control is not enabled until both gates are answered.**
    ///
    /// §3, asserted over the state machine rather than over pixels. The
    /// interesting direction is the residual one: an operator who ticks only
    /// the permanence box on a report with residuals must **not** be able to
    /// commit, because the two boxes answer different questions and treating
    /// one as both is how a partially-redacted file gets handed over as a
    /// complete one.
    ///
    /// It is asserted here as well as at
    /// `crate::redact::PreparedRedaction::write_to` deliberately: this is the
    /// drawing decision and that is the mechanism, and a test for only one of
    /// them would leave the other free to drift.
    #[test]
    fn the_confirm_control_needs_every_gate_that_applies() {
        let mut dialog = RedactDialog {
            source: PathBuf::from("x.pdf"),
            phase: Phase::Refused(RedactApplyRefusal::NothingToApply),
            acknowledged: false,
            residuals_acknowledged: false,
            confirm_requested: false,
            close_requested: false,
        };
        assert!(
            !dialog.ready_to_confirm(),
            "a refusal has nothing to confirm"
        );

        // A prepared, clean redaction: one box.
        let session = clean_session();
        let prepared = prepare_redaction_apply(&session).expect("the fixture applies");
        assert!(prepared.verification.is_clean());
        assert!(
            residual_lines(&prepared).is_empty(),
            "the fixture must have nothing to disclose, or the two cases below \
             are the same case"
        );
        dialog.phase = Phase::Prepared(Box::new(prepared));
        assert!(!dialog.ready_to_confirm(), "nothing acknowledged yet");
        dialog.acknowledged = true;
        assert!(
            dialog.ready_to_confirm(),
            "a clean report must not demand a tick nobody can give"
        );

        // …and the same value with a residual: two boxes.
        let session = clean_session();
        let mut prepared = prepare_redaction_apply(&session).expect("the fixture applies");
        prepared
            .verification
            .raw_byte_residuals
            .push("MARGARETHALE".to_owned());
        assert_eq!(residual_lines(&prepared).len(), 1);
        dialog.phase = Phase::Prepared(Box::new(prepared));
        dialog.acknowledged = true;
        dialog.residuals_acknowledged = false;
        assert!(
            !dialog.ready_to_confirm(),
            "★ the permanence box alone must not commit a report with residuals \
             — the two boxes answer different questions, and treating one as \
             both hands over a partially-redacted file as a complete one"
        );
        dialog.residuals_acknowledged = true;
        assert!(dialog.ready_to_confirm());
    }

    /// ★ **Every kind of residual reaches the list, and the list is what gates
    /// the checkbox.**
    ///
    /// One derivation for both, so a residual cannot be listed without being
    /// acknowledgeable or acknowledged without being listed. The promotion
    /// source is the one a tidying edit would drop, because it is the mildest —
    /// and a report that silently drops the findings it judges harmless is one
    /// whose judgement nobody can audit.
    #[test]
    fn every_source_of_a_residual_reaches_the_disclosed_list() {
        let session = clean_session();
        let mut prepared = prepare_redaction_apply(&session).expect("the fixture applies");
        assert!(residual_lines(&prepared).is_empty());

        prepared
            .verification
            .raw_byte_residuals
            .push("MARGARETHALE".to_owned());
        assert_eq!(residual_lines(&prepared).len(), 1);

        prepared
            .promoted_by_materialisation
            .push(pdfcer_core::object::ObjId {
                num: 7,
                generation: 0,
            });
        let lines = residual_lines(&prepared);
        assert_eq!(
            lines.len(),
            2,
            "a promotion is a disclosed residual too: {lines:?}"
        );
        assert!(
            lines.iter().any(|l| l.contains("compressed container")),
            "{lines:?}"
        );
    }

    /// ★ **A written outcome with residuals does not borrow the clean
    /// sentence.**
    ///
    /// The catalog's rule 1, at the one call site that chooses between the two.
    /// A build that always used the clean form would produce a window saying a
    /// file was *"verified absent"* over a report the operator had just
    /// acknowledged as incomplete — which is worse than saying nothing, because
    /// it contradicts the thing they read a moment earlier.
    #[test]
    fn the_written_sentence_follows_the_residual_count() {
        let path = Path::new("D:\\jobs\\Sheet 1-redacted.pdf");
        let clean = outcome_line(path, 4, 2, 0);
        let dirty = outcome_line(path, 4, 2, 1);
        assert_ne!(clean, dirty);
        assert!(clean.contains("verified"), "{clean}");
        assert!(
            !dirty.contains("verified"),
            "a file with an acknowledged residual is not verified absent: {dirty}"
        );
        assert!(dirty.contains("NOT be removed"), "{dirty}");
        // The file name, not the path — see `outcome_line`.
        assert!(clean.contains("Sheet 1-redacted.pdf"), "{clean}");
        assert!(!clean.contains("D:\\jobs"), "{clean}");
    }

    /// A session with one mark over a distinctive secret, applying cleanly.
    ///
    /// Built from `crate::redact`'s own fixture shape rather than from a file,
    /// so every byte in it is one this suite put there — which is what makes
    /// "the report has no residuals" a fact about the fixture rather than a
    /// property of somebody's producer.
    fn clean_session() -> pdfcer_core::edit::EditSession {
        const SECRET: &str = "CONFIDENTIALWITNESSNAME";
        let content = format!("BT /F1 12 Tf 20 100 Td ({SECRET}) Tj ( KEEPTHIS) Tj ET");
        let stream = format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        );
        let bodies: [&str; 5] = [
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
            &stream,
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        ];
        let mut buf = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in bodies.iter().enumerate() {
            offsets.push(buf.len());
            buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref_at = buf.len();
        let n = bodies.len() + 1;
        buf.extend_from_slice(format!("xref\n0 {n}\n0000000000 65535 f \n").as_bytes());
        for off in &offsets {
            buf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        buf.extend_from_slice(
            format!("trailer\n<< /Size {n} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n")
                .as_bytes(),
        );
        let doc = pdfcer_core::document::Document::from_bytes(buf).expect("the fixture parses");
        let mut session = pdfcer_core::edit::EditSession::new(doc);
        session
            .mark_redactions_by_search(SECRET, false)
            .expect("the fixture's text is extractable");
        session
    }
}
