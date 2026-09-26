//! # `dialogs::open_in_acrobat` — the question that comes before pdfcer lets
//! go of the file
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/open_in_acrobat.md`.
//!
//! ## conventions: dialogs
//!
//! Corpus: `ui-conventions/dialogs.md`. Answered row by row — and most of the
//! answers are [`super::scale`]'s, because this window is built out of the
//! same [`super::host::Host`] every dialog in this directory uses. That is the
//! point of having one idiom, and it also means the idiom's gaps are inherited
//! rather than re-argued here.
//!
//! - G1 is-an-os-window: **YES.** `Host` opens a real
//!   `show_viewport_immediate` window with a taskbar entry, so this dialog
//!   drags outside the application and onto a second monitor. The taskbar
//!   entry earns its keep here specifically: this window appears in answer to
//!   a click on a control at the extreme right of the ribbon, which is where a
//!   pointer is on its way somewhere else, and a question hidden behind the
//!   main window with no entry anywhere is the classic *"the program has
//!   frozen"* report.
//! - G2 use-the-os-dialog: **N/A for this window.** The question is *shall I
//!   hand this document to Acrobat and close it here*, and no operating system
//!   has a dialog for that — it carries choices only pdfcer has, which is the
//!   corollary's stated right reason to draw one. The one place O122 does
//!   reach for the system's own dialog is Settings' Browse button, which is
//!   `rfd`'s native file picker (`crate::app::files::pick_acrobat`).
//! - G3 owned-by-the-app: **YES**, through `Host`. An unowned dialog falls
//!   behind its parent and the operator concludes the button did nothing.
//! - G4 enter-accepts-escape-cancels: **PARTIAL, and inherited.** Escape and
//!   the ✕ both mean Cancel — the non-destructive answer, which is the half
//!   that matters most on a window whose other button closes a document. Enter
//!   is **not** wired as the affirmative default and no button is drawn as the
//!   default: `egui` 0.35 has no default-button concept for a `Window`, and
//!   faking one with a focus request would make Enter destructive with no
//!   visible cue that it would be. [`super::scale`] records the same gap.
//! - G5 keyboard-reachable: **PARTIAL, and inherited.** Every control here is
//!   a `Button`, so Tab reaches all of them in reading order; what is not
//!   asserted anywhere is that focus starts on the safe one or that it is
//!   trapped.
//! - G6 remembers-position: **GAP, and inherited** — `Host` opens centred
//!   every time.
//! - G7 destructive-verbs-named: **YES**, and it is the row this window was
//!   written around. The buttons read *Save and open in Acrobat* and *Close
//!   and open in Acrobat*, never *OK* — even though the operator's own words
//!   were *"with and ok button to continue"*. The unsaved sentence names the
//!   **file**; the heading names what is about to happen to it. See
//!   [`crate::text::acrobat`] for the argument, and for why there is no third
//!   button.
//! - G8 cancel-is-silent: **YES.** A cancel writes one trace line and nothing
//!   else — no status sentence, no warning, no error. That line exists only
//!   because a cancel and a question nobody answered leave the screen
//!   identical, so a driven check cannot otherwise tell them apart; it is not
//!   a report of a fault.
//! - G9 nothing-blocks-silently: **YES.** Nothing runs behind this window: it
//!   parks an answer and returns, and the save, the launch and the close all
//!   happen after it has closed. The launch is a `spawn` rather than a wait —
//!   see [`crate::acrobat::windows`] — so pdfcer never stops repainting.

use egui::Ui;

use crate::acrobat::{Prompt, Viewer};
use crate::text::acrobat as t;

/// The region the dialog body publishes.
pub const REGION_BODY: &str = "dialog:open-in-acrobat"; // ui-text-exempt: trace region name, never displayed

/// The region the **proceed** button publishes — *Save and open* or *Close and
/// open*, whichever this shape carries.
pub const REGION_PROCEED: &str = "open-in-acrobat.proceed"; // ui-text-exempt: trace region name, never displayed

/// Published **only on the frames the unsaved shape is drawn**.
pub const REGION_SAVE_FIRST: &str = "open-in-acrobat.save_first"; // ui-text-exempt: trace region name, never displayed

/// Published only on the frames the never-saved refusal is drawn.
///
/// Same reasoning as [`REGION_SAVE_FIRST`], for the state that is easiest to
/// mistake for a missing Acrobat.
pub const REGION_NO_FILE: &str = "open-in-acrobat.no_file"; // ui-text-exempt: trace region name, never displayed

/// The region the Cancel button publishes.
pub const REGION_CANCEL: &str = "open-in-acrobat.cancel"; // ui-text-exempt: trace region name, never displayed

/// What the operator chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Save if there is anything to save, close the document, hand the file
    /// to Acrobat. The application does all three — see
    /// [`crate::app::actions`] — because a window that could close a document
    /// would be a second route to the most destructive operation this shell
    /// has.
    Proceed,
}

/// The dialog's live state.
///
/// Existence is the "open" state, as everywhere in [`super`].
pub struct OpenInAcrobatDialog {
    /// Which of the three shapes.
    prompt: Prompt,
    /// The Acrobat that will be started, so the sentences can name it.
    viewer: Viewer,
    /// How many edits are at stake, for the sentence that says so.
    ///
    /// Captured at **open** time rather than read per frame, exactly as
    /// [`crate::dialogs::unsaved`] captures its own: this window is the only
    /// thing on screen that could change the document (it cannot), so a live
    /// read could only return the same number — but capturing makes the
    /// sentence a statement about the moment the operator was asked, which is
    /// what a confirmation's text is for.
    edits: u64,
    /// The file's name, for the sentence that names it.
    ///
    /// The **file name**, not the full path: the sentence is about which of
    /// the operator's documents this is, and a full path in the middle of a
    /// sentence is a line-break problem rather than information.
    file: String,
    /// Set by the proceed button, drained by the owner.
    outcome: Option<Outcome>,
    /// Set by Cancel and by the window's ✕.
    cancelled: bool,
}

impl OpenInAcrobatDialog {
    /// Ask about handing `viewer` the open document.
    #[must_use]
    pub fn new(prompt: Prompt, viewer: Viewer, edits: u64, file: String) -> Self {
        Self {
            prompt,
            viewer,
            edits,
            file,
            outcome: None,
            cancelled: false,
        }
    }

    /// **Did the operator answer Cancel?**
    #[must_use]
    pub const fn was_cancelled(&self) -> bool {
        self.cancelled
    }

    /// **Whether an answer is parked here and has not been drained.**
    #[must_use]
    pub const fn answered(&self) -> bool {
        self.outcome.is_some()
    }

    /// Take the operator's answer, with the viewer it was about.
    pub fn take_outcome(&mut self) -> Option<(Outcome, Viewer)> {
        let outcome = self.outcome.take()?;
        Some((outcome, self.viewer.clone()))
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        // Its own OS window with a taskbar entry, like every dialog in this
        // crate — and this one needs the entry: it appears in answer to a
        // click on a control at the extreme right of the ribbon, which is
        // where an operator's pointer is on its way somewhere else.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "open-in-acrobat", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(460.0, 200.0),
            egui::vec2(460.0, 200.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;
        // The ✕ is a Cancel — the non-destructive answer, for the reason the
        // conventions block above states.
        open && !self.cancelled && self.outcome.is_none()
    }

    /// The body.
    fn body(&mut self, ui: &mut Ui) {
        let (heading, body) = match self.prompt {
            Prompt::NoFileOnDisk => (t::no_file_heading(), t::no_file_body()),
            Prompt::SaveFirst => (
                t::save_first_heading(),
                t::save_first_body(self.edits, &self.file, &self.viewer),
            ),
            Prompt::ConfirmClose => (
                t::confirm_close_heading(),
                t::confirm_close_body(&self.viewer),
            ),
        };

        // The heading carries its weight through SIZE and position rather
        // than through `RichText::strong()`, which resolves its own colour out
        // of egui's active-widget channel and comes back pale on a panel —
        // defect D2's shape, and what `tools/gates/check-strong-text.sh`
        // exists to catch. `heading()` is a text style, so the theme's own ink
        // applies.
        ui.label(egui::RichText::new(heading).heading());
        ui.add_space(6.0);
        ui.label(body);
        ui.add_space(10.0);

        // The reading order runs from the answer that acts to the answer
        // that does not, which is where every application the operator uses
        // puts them — and neither of them loses anything, which is the whole
        // point of this window having two buttons rather than three.
        ui.horizontal(|ui| match self.prompt {
            Prompt::NoFileOnDisk => {
                // ONE button, published under its own region and NOT also
                // under `REGION_CANCEL`. This shape has no Cancel — there is
                // nothing to cancel — and publishing the dismiss control under
                // both names would let a driven check assert "the operator was
                // offered a way out of a decision" on a window that offered no
                // decision. The two states have almost the same screenshot,
                // which is exactly when a region name has to be exact.
                let button = ui.button(t::dismiss_button());
                crate::diag::ui_rect_visible(REGION_NO_FILE, button.rect, ui.clip_rect());
                if button.clicked() {
                    // Recorded as a cancellation because that is what it is to
                    // everything downstream: nothing was saved, nothing closed,
                    // nothing launched. The trace line reads `acrobat-cancelled`
                    // for a refusal the operator merely acknowledged, and the
                    // `prompt=NoFileOnDisk` on the earlier `acrobat-asked` line
                    // is what tells the two apart in a log.
                    self.cancelled = true;
                }
            }
            Prompt::SaveFirst => {
                let proceed = ui.button(t::save_and_open_button());
                crate::diag::ui_rect_visible(REGION_PROCEED, proceed.rect, ui.clip_rect());
                crate::diag::ui_rect_visible(REGION_SAVE_FIRST, proceed.rect, ui.clip_rect());
                if proceed.clicked() {
                    self.outcome = Some(Outcome::Proceed);
                }
                self.cancel(ui);
            }
            Prompt::ConfirmClose => {
                let proceed = ui.button(t::close_and_open_button());
                crate::diag::ui_rect_visible(REGION_PROCEED, proceed.rect, ui.clip_rect());
                if proceed.clicked() {
                    self.outcome = Some(Outcome::Proceed);
                }
                self.cancel(ui);
            }
        });
    }

    /// The Cancel button, on the two shapes that have one.
    fn cancel(&mut self, ui: &mut Ui) {
        let button = ui.button(t::cancel_button());
        crate::diag::ui_rect_visible(REGION_CANCEL, button.rect, ui.clip_rect());
        if button.clicked() {
            self.cancelled = true;
        }
    }
}
