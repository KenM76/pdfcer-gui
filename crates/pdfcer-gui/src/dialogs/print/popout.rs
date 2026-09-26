//! `dialogs::print::popout` — the print preview in a window of its own.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/print/popout.md`.

use crate::app::state::OpenDoc;
use crate::dialogs::print::spooler::Job;
use crate::text::print as t;

use super::{PrintDialog, preview, verdicts};

/// The popped window's viewport key, and the string a driven check names.
///
/// Stable and distinct from `"print"`. `Host::new` turns it into a
/// `ViewportId` by hashing, and two dialogs sharing one id would be two
/// dialogs sharing one OS window — so this string is as load-bearing as any
/// code in the file.
// ui-text-exempt: a viewport key, never displayed.
pub(super) const HOST_ID: &str = "print-preview";

/// The popped window's body rect, for the driven harness.
pub(super) const REGION_POPPED_BODY: &str = "print.preview.window";

/// The size the pop-out window opens at, in egui points.
const DEFAULT_SIZE_PTS: egui::Vec2 = egui::vec2(560.0, 760.0);

/// The smallest the pop-out window may be dragged to, in egui points.
const MIN_SIZE_PTS: egui::Vec2 = egui::vec2(320.0, 320.0);

impl PrintDialog {
    /// **Draw the preview in its own window, if it is out there.**
    pub(super) fn popped_preview(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        job: Option<&Job>,
        page_sizes: &[(f64, f64)],
        context: Option<&verdicts::Context>,
    ) {
        if !self.preview_popped {
            return;
        }
        let (frame, ()) = crate::dialogs::host::Host::new(
            HOST_ID,
            t::preview_window_title(),
            DEFAULT_SIZE_PTS,
            MIN_SIZE_PTS,
        )
        // A view, not a transaction: he maximises it to read fine print.
        .maximizable()
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_POPPED_BODY, ui.max_rect());
            match job.zip(context) {
                Some((job, context)) => {
                    // The available space MINUS nothing, handed straight to
                    // the column — and the direction bound in this module's
                    // header is what makes that safe. `column` allocates
                    // exactly what it is given and clamps the canvas at both
                    // ends, so the content is never larger than the window and
                    // `Host::fit`, which only grows, has nothing to chase.
                    //
                    // The width is read once, before the height, because
                    // `available_height` is affected by anything already laid
                    // out in this `Ui` and nothing has been. Reading them in
                    // the other order would work today and would silently stop
                    // working the first time a sentence was added above the
                    // canvas.
                    let width = ui.available_width();
                    let height = ui.available_height();
                    preview::column(
                        ui,
                        &preview::Inputs {
                            doc,
                            job,
                            page_sizes,
                            context,
                        },
                        self,
                        height,
                        width,
                        preview::Placement::PoppedOut,
                    );
                }
                // The same sentence the column would have shown, for the same
                // reason: everything a preview draws comes from the device's
                // own description of itself, and a guessed rectangle is the
                // confidently wrong preview this feature exists to prevent.
                //
                // It is a sentence rather than an empty window, and that is
                // not a placeholder: the operator asked for this window and it
                // owes them an answer, and *"the printer would not describe
                // itself"* is one. R9 forbids a stub standing in for a feature,
                // not a surface stating why there is nothing to show.
                None => {
                    ui.label(t::device_unavailable());
                }
            }
        });

        // THE RETURN PATH, AND IT IS ONE LINE BECAUSE IT WAS ALREADY BUILT.
        //
        // `Frame::closed` is the OS close button **and** Escape, together,
        // because G4 says those are one gesture and a caller that told them
        // apart would give one route out a different meaning from the other.
        // The operator's *"closing the window pops it back into place"* is
        // therefore satisfied by both, and by dragging the window shut from the
        // taskbar, without any of the three being written down here.
        if frame.closed {
            self.preview_popped = false;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-preview-popped state=in".to_owned()
            });
        }
    }
}
