//! `dialogs::print::popout` — the print preview in a window of its own.
//!
//! # What this is, in the operator's words
//!
//! > *"also the preview should be adjustable size, and even better if it has
//! > the option to pop out into its own resizeable window - closing the window
//! > pops it back into place on the print window."* — 2026-09-03,
//! > `OPERATOR_REQUESTS.md` **O112**.
//!
//! Ask 1 — the draggable splitter — shipped the same day. This file is ask 2.
//!
//! # ★★★ Why it is thirty lines of code and four hundred of reasoning
//!
//! Because almost all of it already existed and the value of this file is
//! knowing that. Every dialog in this shell is a real OS window through
//! [`crate::dialogs::host::Host`], which is keyed on a name string, remembers
//! where the operator dragged it, and reports its own close as
//! [`Frame::closed`](crate::dialogs::host::Frame::closed). A popped-out preview
//! is therefore:
//!
//! | the ask | what answers it | written here? |
//! |---|---|---|
//! | *"its own window"* | a second `Host`, keyed `print-preview` | one call |
//! | *"resizeable"* | `Host` windows are resizable, with a floor | nothing |
//! | *"closing the window pops it back"* | `Frame::closed` → `preview_popped = false` | one line |
//! | it survives being dragged to a second monitor | `Host` remembers position | nothing |
//! | it is findable when it falls behind | `with_taskbar(true)` in `Host` | nothing |
//!
//! ★ **The close is already the return path.** That is the sentence worth
//! carrying out of here: the request reads like a feature with two halves —
//! pop out, and put back — and the second half is the default behaviour of a
//! window. Building a "put it back" control would have been building a second,
//! worse route to something the title bar already does.
//!
//! # ★★★ What the print dialog draws while the preview is out: NOTHING
//!
//! Not a greyed rectangle. Not a *"the preview is in another window"* card. Not
//! an outline where it used to be. The column collapses and the options take
//! the room — see `layout::Columns::split`, whose popped arm returns a preview
//! width of exactly zero and hands the whole content width to the options, and
//! `layout::tests::popping_the_preview_out_collapses_its_column_and_gives_the_
//! room_away`, which asserts all three parts of that.
//!
//! R9 is what forbids the stub, and the reason is not tidiness: a placeholder
//! occupying the space teaches the operator that this dialog has a dead region
//! in it, and the next time something *is* broken there they will not report
//! it.
//!
//! # ★★ The geometry trap this file had to be written around
//!
//! `Host::fit` grows a dialog whose content is bigger than its window — a
//! measurement fed into a size, which is R128's shape and has bitten this
//! project three times, once for this very dialog's footer. The preview's own
//! `fit` (the sheet-into-canvas ratio) is recomputed from the current rect
//! **every frame**, on purpose, so that a taller window shows a bigger sheet.
//! Putting that loop inside a window whose size is itself derived from its
//! content would close the circle.
//!
//! It does not close, and the reason is a direction bound rather than a guard:
//! [`preview::column`] allocates its canvas at *exactly* the width and height
//! it is handed, and it is handed `ui.available_*` minus a constant. The
//! content is therefore never larger than the window, `fit_target` only ever
//! grows, and a function that only grows cannot be driven by content that is
//! by construction no bigger than what it is measured against. The one case
//! that can grow the window once is a canvas clamped up to
//! `CANVAS_MIN_HEIGHT_PTS` on a window shorter than that — bounded by the
//! clamp, not by a budget, and it settles in one round trip.
//!
//! ★ `Host`'s three-step growth budget is still behind it, and the trace line
//! `dialog-fit-runaway title="Print preview"` is what a future mistake here
//! would announce itself as. Read that line before reading this file.
//!
//! # What is NOT claimed
//!
//! **No window was rendered while this was written.** The operator was at the
//! machine, so `ui-verify` could not be run and no screenshot exists. The
//! arithmetic of the collapse is unit-tested and falsified; that a popped-out
//! preview *looks* right, that the window opens at a sensible size on his
//! monitors, and that the column visibly disappears are asserted by
//! `ui-verify`'s `the_print_preview_pops_into_its_own_window` — **which has
//! been written and registered and has not been run.**

use crate::app::state::OpenDoc;
use crate::dialogs::print::spooler::Job;
use crate::text::print as t;

use super::{PrintDialog, preview, verdicts};

/// The popped window's viewport key, and the string a driven check names.
///
/// ★ Stable and distinct from `"print"`. `Host::new` turns it into a
/// `ViewportId` by hashing, and two dialogs sharing one id would be two
/// dialogs sharing one OS window — so this string is as load-bearing as any
/// code in the file.
// ui-text-exempt: a viewport key, never displayed.
pub(super) const HOST_ID: &str = "print-preview";

/// The popped window's body rect, for the driven harness.
///
/// Published inside the child viewport, so its `ui-rect` line carries the
/// `viewport=` suffix `diag::ViewportScope` adds — which is how a check tells
/// *"the preview drew in its own window"* from *"the preview drew"*.
pub(super) const REGION_POPPED_BODY: &str = "print.preview.window";

/// The size the pop-out window opens at, in egui points.
///
/// Taller than it is wide, because a print preview is a sheet and every sheet
/// an operator of this program prints is either portrait or, rotated, still
/// better served by height than by width — the fit takes the smaller of the two
/// ratios either way. Larger than the 340 pt column it came out of, because
/// making the preview bigger is the entire reason to pop it out; opening at the
/// same size as the column would answer the request with a window that changed
/// nothing.
const DEFAULT_SIZE_PTS: egui::Vec2 = egui::vec2(560.0, 760.0);

/// The smallest the pop-out window may be dragged to, in egui points.
///
/// A floor and not a preference, for `Host`'s stated reason: a resizable window
/// with no minimum can be dragged down to a title bar, which is a state with no
/// way back except closing it. Here that is milder than for a form — closing is
/// the intended exit — but a preview squeezed under
/// `CANVAS_MIN_HEIGHT_PTS + STRIP_HEIGHT_PTS` would show a smudge and a
/// scrollbar, which is not a preview.
const MIN_SIZE_PTS: egui::Vec2 = egui::vec2(320.0, 320.0);

impl PrintDialog {
    /// **Draw the preview in its own window, if it is out there.**
    ///
    /// Called from [`PrintDialog::show`] *before* the print dialog's own host,
    /// and the order is deliberate rather than incidental.
    ///
    /// # ★★ Why before, and not after
    ///
    /// The commit button's label carries how many sheets will lose content, and
    /// that number is corrected by what the preview has actually **examined** —
    /// `verdicts::Verdicts::claim` reads verdicts the preview records while it
    /// paints. The footer reads the claim after the body has drawn, so that the
    /// button and the picture beside it describe the same frame.
    ///
    /// With the preview in another window that ordering has to be restated
    /// here, because "the body" no longer contains it. Drawing the popped
    /// window first keeps the invariant exactly as it was: everything the
    /// preview learned this frame is in the cache before anything reads it.
    /// Drawing it afterwards would make the button lag the picture by one
    /// frame — a contradiction that flickers rather than one that persists,
    /// which this dialog's footer already records as the worse of the two.
    ///
    /// # The arguments are the frame's, not the dialog's
    ///
    /// `job`, `page_sizes` and `context` are computed once per frame in
    /// [`PrintDialog::show`] and passed down to both homes of the preview, so
    /// the two cannot be drawn from different plans. `context` is `Some`
    /// exactly when `job` is, and they are zipped rather than unwrapped
    /// separately for the reason `body` gives: a job drawn against a context
    /// from a different device is the staleness `verdicts::Context` exists to
    /// prevent.
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
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_POPPED_BODY, ui.max_rect());
            match job.zip(context) {
                Some((job, context)) => {
                    // ★ The available space MINUS nothing, handed straight to
                    // the column — and the direction bound in this module's
                    // header is what makes that safe. `column` allocates
                    // exactly what it is given and clamps the canvas at both
                    // ends, so the content is never larger than the window and
                    // `Host::fit`, which only grows, has nothing to chase.
                    //
                    // ★★ The width is read once, before the height, because
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
                // ★ It is a sentence rather than an empty window, and that is
                // not a placeholder: the operator asked for this window and it
                // owes them an answer, and *"the printer would not describe
                // itself"* is one. R9 forbids a stub standing in for a feature,
                // not a surface stating why there is nothing to show.
                None => {
                    ui.label(t::device_unavailable());
                }
            }
        });

        // ★★★ THE RETURN PATH, AND IT IS ONE LINE BECAUSE IT WAS ALREADY BUILT.
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
