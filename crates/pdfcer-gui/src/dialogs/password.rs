//! # `dialogs::password` — the box that lets an encrypted document be opened
//!
//! ## ★★★ The defect this closes, and it is the shape this project keeps finding
//!
//! Found 2026-09-03 by `tools/security-coverage.py`, while auditing this shell
//! against `pdfcer-core`'s API for `OPERATOR_REQUESTS.md` O108.
//!
//! **An encrypted PDF could not be opened at all.** The shell detected the case
//! perfectly: `Document::load` returns `DocError::PasswordRequired`,
//! [`crate::app::lifecycle::PdfcerApp::open_path`] branches on it by structured
//! error data rather than by message, and produces
//! [`crate::app::state::Status::NeedsPassword`] — a tab whose tooltip reads, in
//! this build, *"This document is encrypted and pdfcer has not been given the
//! password."*
//!
//! And then **nothing could give it one.** `Document::load_with_password` and
//! `from_bytes_with_password` were named in exactly one place in this crate: a
//! doc comment in `crate::app::blank` listing the four loading entry points.
//! Nothing called either.
//!
//! ★★ **That doc comment is why the coverage tool now strips comment-only lines
//! before it searches.** Its first run reported `load_with_password` as
//! *reached*, on the strength of that one sentence — which would have recorded
//! the single most important missing capability in the whole area as already
//! built, in the instrument written to find exactly this.
//!
//! ⇒ The general form, and it is a third instance: **a mention is not a call, a
//! backlog row is not evidence, and a sentence about a limit is a dated
//! citation.** Every one of the three is a case of prose being mistaken for
//! mechanism.
//!
//! ## Why a real OS window
//!
//! Every dialog in this shell has been one since 2026-08-21, and this one earns
//! it twice over: it appears in answer to an **Open**, which is a gesture an
//! operator makes and then looks away from, and a modal question hidden behind
//! the application window with no taskbar entry is the classic *"the program has
//! frozen"* report. [`crate::dialogs::host`] gives it the entry.
//!
//! ## ★★ What is deliberately NOT here
//!
//! - **No "remember this password".** It would have to be stored, and the only
//!   places to store it are a settings file in plain text or an OS keychain this
//!   project has no binding to. A checkbox that wrote a document password into
//!   `settings.txt` would be a security defect authored on purpose.
//! - **No attempt limit and no delay.** pdfcer is reading a local file the
//!   operator already has. Rate-limiting their guesses at their own document is
//!   theatre that costs them time and stops nobody.
//! - **No "show password" eye.** It is one line of code and it is a real
//!   shoulder-surfing surface in an office; the value of it here is low because
//!   the field is short-lived and the failure is cheap to retry.
//!
//! ## ★★★ The password never reaches a log
//!
//! It travels in [`crate::secret::Secret`], whose entire purpose is a `Debug`
//! that cannot print the value. See that module: this crate traces liberally to
//! stderr under `PDFCER_DIAG`, and `tools/ui-verify` **captures that stderr to a
//! file it keeps as evidence** — so one `format!("{action:?}")` on the action
//! queue would write the operator's password into `target/ui-verify/`, in plain
//! text, in a directory whose purpose is to be kept and read.
//!
//! The trace lines below say the length and the outcome and never the value.

use egui::Ui;

use crate::app::actions::Action;
use crate::secret::Secret;
use crate::text::security as t;

/// The region the body publishes.
pub const REGION_BODY: &str = "dialog:password"; // ui-text-exempt: trace region name, never displayed
/// The password field's own rect.
pub const REGION_FIELD: &str = "password.field"; // ui-text-exempt: trace region name, never displayed
/// The Open button.
pub const REGION_OPEN: &str = "password.open"; // ui-text-exempt: trace region name, never displayed
/// The Cancel button.
pub const REGION_CANCEL: &str = "password.cancel"; // ui-text-exempt: trace region name, never displayed

/// Why the last attempt did not open the document.
///
/// ★ Two variants, not one, because `pdfcer-core` reports two errors and its own
/// doc comment says why: `PasswordRequiresNormalisation` exists *"so that
/// failure does not masquerade as `PasswordRequired`'s 'you typed it wrong',
/// which would send the operator to re-check a password that was correct."*
/// Collapsing them here would undo that on the last step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// Nothing authenticated. The ordinary case: a wrong password.
    Wrong,
    /// The password is non-ASCII and this document's revision specifies a
    /// normalisation pdfcer does not implement. **The password may be correct.**
    NeedsNormalisation,
}

/// The dialog.
///
/// `password` is a plain `String` here rather than a [`Secret`] because that is
/// what `egui::TextEdit` binds to; it becomes a `Secret` at the moment it leaves
/// this struct, which is the boundary that matters — the value never enters an
/// `Action`, a queue or a trace unwrapped.
pub struct PasswordDialog {
    /// The file being opened. Carried so the retry knows what to re-open, and so
    /// the prompt can name it.
    path: std::path::PathBuf,
    /// What the operator has typed.
    password: String,
    /// How many passwords have been tried and refused.
    attempts: u32,
    /// Why the last one failed, if there was one.
    rejection: Option<Rejection>,
    /// Set when the operator asked for an empty password, which is refused
    /// locally — see [`t::password_empty`].
    empty_refused: bool,
    /// The operator gave up.
    cancelled: bool,
    /// An attempt is being submitted this frame.
    submitted: bool,
}

impl PasswordDialog {
    /// Open the prompt for `path`.
    #[must_use]
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            password: String::new(),
            attempts: 0,
            rejection: None,
            empty_refused: false,
            cancelled: false,
            submitted: false,
        }
    }

    /// The document this prompt is for.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// **Report that the password just submitted did not work**, and stay open.
    ///
    /// Called by the application when its retry comes back refused. Clears the
    /// field, because a wrong password left in the box is one an operator
    /// re-submits by reflex, and increments the attempt count so the message can
    /// say which try this was — without that, a second rejection produces a
    /// dialog identical to the first and the operator cannot tell whether their
    /// press registered.
    pub fn reject(&mut self, why: Rejection) {
        self.attempts += 1;
        self.rejection = Some(why);
        self.password.clear();
        self.submitted = false;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            //
            // ★ The REASON and the attempt number, never the password. See the
            // module header and `crate::secret`.
            format!(
                "password-rejected attempt={} reason={}",
                self.attempts,
                match why {
                    Rejection::Wrong => "wrong",
                    Rejection::NeedsNormalisation => "needs-normalisation",
                }
            )
        });
    }

    /// Draw it, raising an action when a password is submitted.
    ///
    /// Returns `false` when the dialog should close — cancelled, or dismissed by
    /// the window's own ✕.
    ///
    /// ★ The ✕ is a **Cancel**. The window's close control must mean the
    /// non-destructive answer, which is the rule `dialogs::unsaved` states: it is
    /// the control an operator presses reflexively to make a surprise go away.
    /// Here nothing is destroyed either way, and the tab stays in the document
    /// list saying why it did not open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "password", // ui-text-exempt: a viewport key, never displayed.
            t::password_title(),
            egui::vec2(460.0, 210.0),
            egui::vec2(460.0, 210.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui, actions);
        });
        !frame.closed && !self.cancelled
    }

    /// The body.
    fn body(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let name = self.path.file_name().map_or_else(
            || self.path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        ui.label(t::password_prompt(&name));
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(t::password_label());
            // ★ `password(true)` — the field renders dots. Not a nicety: this
            // window is opened in an office and read over a shoulder, and the
            // one thing a password field must do is not display the password.
            let field = ui.add(
                egui::TextEdit::singleline(&mut self.password)
                    .password(true)
                    .desired_width(ui.available_width() - 8.0),
            );
            crate::diag::ui_rect(REGION_FIELD, field.rect);
            // ★★ Focus on the first frame, so the operator can type straight
            // away. The prompt is modal in intent and there is exactly one
            // thing to do in it; making them click the box first is a step that
            // exists only because nobody asked for the focus.
            if self.attempts == 0 && !field.has_focus() && !self.submitted {
                field.request_focus();
            }
            // Enter submits, which is what every password box in the operating
            // system does.
            if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.submit(actions);
            }
        });

        ui.add_space(6.0);

        // ★ The reason the last attempt failed, if there was one. Drawn ABOVE
        // the buttons so it is between the field and the control that repeats
        // the mistake, rather than below where a short window can hide it.
        if self.empty_refused {
            ui.label(egui::RichText::new(t::password_empty()).small());
        } else {
            match self.rejection {
                Some(Rejection::Wrong) => {
                    ui.label(egui::RichText::new(t::password_rejected(self.attempts)).small());
                }
                Some(Rejection::NeedsNormalisation) => {
                    ui.label(egui::RichText::new(t::password_needs_normalisation()).small());
                }
                None => {}
            }
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let open = ui.button(t::password_open());
            crate::diag::ui_rect(REGION_OPEN, open.rect);
            if open.clicked() {
                self.submit(actions);
            }
            let cancel = ui.button(t::password_cancel());
            crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
            if cancel.clicked() {
                self.cancelled = true;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("password-cancelled attempts={}", self.attempts)
                });
            }
        });
    }

    /// Hand the typed password to the application, or refuse an empty one.
    ///
    /// ★★ The empty case is refused **here** rather than sent on, and the reason
    /// is [`Secret::is_empty`]'s: `Document::load(path)` already tried the empty
    /// password before this prompt existed — every conforming reader does that
    /// silently — so submitting it again asks the engine a question it has
    /// answered and returns an identical rejection, which the operator reads as
    /// *"my password was wrong"* about a password they never supplied.
    fn submit(&mut self, actions: &mut Vec<Action>) {
        let secret = Secret::new(std::mem::take(&mut self.password));
        if secret.is_empty() {
            self.empty_refused = true;
            return;
        }
        self.empty_refused = false;
        self.submitted = true;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            //
            // ★★★ The LENGTH and whether it is ASCII, never the value. Those two
            // facts are what a trace reader needs — "a password of 11 characters
            // was supplied and it was non-ASCII" explains a
            // `needs-normalisation` rejection completely — and neither carries
            // the password. See `crate::secret`.
            format!(
                "password-submitted chars={} non_ascii={}",
                secret.len(),
                u8::from(secret.has_non_ascii())
            )
        });
        actions.push(Action::OpenWithPassword {
            path: self.path.clone(),
            password: secret,
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "a test that cannot unwrap has failed")] // ui-text-exempt: clippy lint justification, never displayed
mod tests {
    use super::*;

    /// ★★★ **An empty box raises no action.**
    ///
    /// The whole of [`PasswordDialog::submit`]'s argument, asserted: pdfcer has
    /// already tried the empty password, so sending it again would produce a
    /// rejection the operator reads as *"my password was wrong"* about a
    /// password they never typed.
    #[test]
    fn pressing_open_with_nothing_typed_raises_no_action() {
        let mut d = PasswordDialog::new("x.pdf".into());
        let mut actions = Vec::new();
        d.submit(&mut actions);
        assert!(actions.is_empty(), "{actions:?}");
        assert!(d.empty_refused, "and the operator is told why");
    }

    /// A typed password becomes exactly one action, carrying the path it was
    /// typed for.
    #[test]
    fn a_typed_password_raises_one_open_action_for_that_path() {
        let mut d = PasswordDialog::new("drawing.pdf".into());
        d.password = "hunter2".to_owned();
        let mut actions = Vec::new();
        d.submit(&mut actions);
        assert_eq!(actions.len(), 1);
        let Action::OpenWithPassword { path, password } = &actions[0] else {
            panic!("expected an OpenWithPassword, got {:?}", actions[0])
        };
        assert_eq!(path, std::path::Path::new("drawing.pdf"));
        assert_eq!(password.len(), 7);
    }

    /// ★★★ **The action carrying the password cannot print it**, asserted on
    /// the real `Action` rather than on `Secret` alone.
    ///
    /// `crate::secret` proves the type is safe; this proves the type is the one
    /// actually used on this path. A variant that took a bare `String` would
    /// pass every test in that module and write the password into the evidence
    /// file on the first `{:?}`.
    #[test]
    fn the_action_that_carries_a_password_never_formats_it() {
        let mut d = PasswordDialog::new("drawing.pdf".into());
        d.password = "correct horse battery".to_owned();
        let mut actions = Vec::new();
        d.submit(&mut actions);
        let rendered = format!("{actions:?}");
        assert!(
            !rendered.contains("correct horse battery"),
            "the action queue rendered as `{rendered}`, which carries the operator's \
             password — and `tools/ui-verify` keeps the trace file as evidence"
        );
    }

    /// ★★ **The field is cleared on a rejection**, and the attempt count rises.
    ///
    /// A wrong password left in the box is one an operator re-submits by
    /// reflex; and without the count, a second rejection is indistinguishable
    /// from a press that did not register.
    #[test]
    fn a_rejection_clears_the_field_and_counts_the_attempt() {
        let mut d = PasswordDialog::new("x.pdf".into());
        d.password = "wrong".to_owned();
        d.reject(Rejection::Wrong);
        assert!(d.password.is_empty());
        assert_eq!(d.attempts, 1);
        assert_eq!(d.rejection, Some(Rejection::Wrong));
    }

    /// ★ **The two rejection reasons stay distinct**, because the engine went
    /// to the trouble of separating them so an operator is not sent to re-check
    /// a password that was correct.
    #[test]
    fn the_normalisation_refusal_is_not_the_wrong_password_refusal() {
        assert_ne!(Rejection::Wrong, Rejection::NeedsNormalisation);
        assert_ne!(
            t::password_rejected(1),
            t::password_needs_normalisation().to_owned(),
            "the two failures must not read the same: one means try again, the other \
             means pdfcer cannot open this file however correct the password is"
        );
    }
}
