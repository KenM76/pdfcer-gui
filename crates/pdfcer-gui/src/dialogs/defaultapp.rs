//! # `dialogs::defaultapp` — the offer, made once
//!
//! `OPERATOR_REQUESTS.md` **O173**: *"Ask once with a don't show me again check
//! box option."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/defaultapp.md`.

use egui::Ui;

use crate::app::assoc;
use crate::text::assoc as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "defaultapp.body";
/// The button that registers and hands over to Windows.
// ui-text-exempt: trace region name, never displayed
pub const REGION_ACTION: &str = "defaultapp.action";
/// The checkbox O173 asks for by name.
// ui-text-exempt: trace region name, never displayed
pub const REGION_DONT_ASK: &str = "defaultapp.dont-ask";
/// The button that declines this time — the one route out of this window that
/// changes nothing outside pdfcer.
///
/// Declared for the same reason [`REGION_ACTION`] is, and the operator's
/// sentence of 2026-09-10 is plural: *"Those **buttons** should always be
/// available"*. A harness that could see one of the two would report the answer
/// row as reachable on a build where half of it had been clipped away.
// ui-text-exempt: trace region name, never displayed
pub const REGION_LATER: &str = "defaultapp.later";

/// **Whether the offer is owed on this launch.**
///
/// Three conditions, and the middle one is the reason this reads the machine
/// rather than a stored flag:
///
/// 1. The operator has not switched the question off — [`ask_default_app`].
/// 2. Windows does not already open PDFs with pdfcer. A remembered answer here
///    would go stale the moment they installed anything else that opens PDFs,
///    and pdfcer would then be silent in exactly the case the offer exists for.
/// 3. …and that is all. In particular it is **not** conditional on pdfcer being
///    unregistered: a build that is in Windows' list but has never been chosen
///    is the commonest state after somebody presses the button and then closes
///    the Windows page without picking.
///
/// [`ask_default_app`]: crate::app::prefs::Prefs::ask_default_app
///
/// ⚠ Costs two `reg.exe` processes, so it is called **once per launch**, from
/// [`crate::dialogs::DialogsState`]'s one-shot arm — never from a paint. The
/// same rule [`crate::app::assoc::Status`] states at length.
#[must_use]
pub fn should_offer(prefs: &crate::app::prefs::Prefs) -> bool {
    prefs.ask_default_app && !assoc::probe().is_default()
}

/// The offer.
pub struct DefaultAppDialog {
    /// The checkbox's state. See [`DefaultAppDialog::settle`].
    dont_ask: bool,
    /// What happened when the button was pressed, if it has been.
    ///
    /// It replaces the explanatory body rather than joining it: once the act
    /// has happened, the sentence describing what *would* happen is history,
    /// and leaving both would make the operator read two paragraphs to find the
    /// one that is current.
    note: Option<String>,
    /// Set by the button, read once by [`DefaultAppDialog::show`].
    acted: bool,
    /// Set by *Not now*, read once by [`DefaultAppDialog::show`].
    close_requested: bool,
}

impl Default for DefaultAppDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultAppDialog {
    /// Open it.
    ///
    /// The checkbox starts **unticked**. A pre-ticked *"don't ask me again"*
    /// is a dialog that suppresses itself if the operator dismisses it without
    /// reading, which is the same defect as a pre-ticked consent box and is
    /// worse here because there is no way to notice it happened.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dont_ask: false,
            note: None,
            acted: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns whether it stays open.
    ///
    /// `prefs` is written and saved when the window settles — see
    /// [`Self::settle`].
    pub fn show(&mut self, ctx: &egui::Context, prefs: &mut crate::app::prefs::Prefs) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "default-app", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(480.0, 250.0),
            egui::vec2(360.0, 190.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            crate::dialogs::host::Host::scrolled(
                ui,
                self,
                |ui, this| this.body(ui),
                |ui, this| this.footer(ui),
            );
        });

        // The window's own close button counts as *Not now* with whatever the
        // box says. Somebody who ticks the box and then presses the X has
        // answered the question, and losing that because they did not use one
        // of ours would make the checkbox unreliable in exactly the way that
        // makes people stop trusting checkboxes.
        if frame.closed || self.close_requested || self.acted {
            self.settle(prefs);
            return false;
        }
        true
    }

    /// **Write the operator's answer, and save it.**
    fn settle(&self, prefs: &mut crate::app::prefs::Prefs) {
        if !self.dont_ask && !self.acted {
            return;
        }
        if !prefs.ask_default_app {
            return;
        }
        prefs.ask_default_app = false;
        let saved = prefs.save();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                // Stable tokens, never `{:?}` on anything carrying a payload:
                // this project's standing lesson is that a Debug-formatted
                // field makes a driven check report the opposite of the truth
                // while quoting the truth in its own message.
                "default-app-settled acted={} dont_ask={} saved={}",
                self.acted,
                self.dont_ask,
                saved.is_ok()
            )
        });
    }

    /// The explanation, or — once the button has been pressed — what happened.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(self.note.clone().unwrap_or_else(t::body));
        ui.add_space(10.0);
        let check = ui.checkbox(&mut self.dont_ask, t::dont_ask());
        crate::diag::ui_rect_visible(REGION_DONT_ASK, check.rect, ui.clip_rect());
        check.on_hover_text(t::dont_ask_hover());
    }

    /// The answer row, which [`crate::dialogs::host::Host::scrolled`] allocates
    /// before the body so it can never be pushed off the bottom.
    fn footer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // R9: never greyed. Every state this window can be in is one the
            // operator may legitimately proceed from.
            let act = ui.button(t::action());
            crate::diag::ui_rect_visible(REGION_ACTION, act.rect, ui.clip_rect());
            if act.on_hover_text(t::action_hover()).clicked() {
                self.note = Some(act_now());
                // The window closes on the NEXT frame rather than this one,
                // so `note` is drawn at least once. Closing immediately would
                // make a refusal — the one case where the sentence is the only
                // thing that helps — appear and vanish inside a single frame.
                self.acted = true;
            }
            let later = ui.button(t::later());
            crate::diag::ui_rect_visible(REGION_LATER, later.rect, ui.clip_rect());
            if later.clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// Register, then hand over to Windows.
///
/// The same two steps, in the same order and for the same reason, as
/// [`crate::dialogs::settings::defaultapp`]: opening Windows' page having
/// failed to register would deep-link the operator to a list pdfcer is not in.
#[must_use]
fn act_now() -> String {
    if let Err(said) = assoc::register() {
        return said;
    }
    match assoc::open_settings_page() {
        Ok(()) => t::handed_over(),
        Err(said) => said,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::prefs::Prefs;

    /// **Dismissing without ticking leaves the question alive.**
    ///
    /// *Not this time* is not *never*, and the conventional behaviour of every
    /// browser on this platform is to ask again. See [`DefaultAppDialog::settle`].
    #[test]
    fn not_now_alone_does_not_silence_the_offer() {
        let mut prefs = Prefs::default();
        let d = DefaultAppDialog::new();
        d.settle(&mut prefs);
        assert!(prefs.ask_default_app);
    }

    /// **Ticking the box silences it**, which is what the box says.
    #[test]
    fn the_checkbox_silences_the_offer() {
        let mut prefs = Prefs::default();
        let mut d = DefaultAppDialog::new();
        d.dont_ask = true;
        d.settle(&mut prefs);
        assert!(!prefs.ask_default_app);
    }

    /// **Pressing the button silences it too**, box or no box.
    #[test]
    fn engaging_with_the_offer_silences_it() {
        let mut prefs = Prefs::default();
        let mut d = DefaultAppDialog::new();
        d.acted = true;
        d.settle(&mut prefs);
        assert!(!prefs.ask_default_app);
    }

    /// **The checkbox starts unticked.**
    #[test]
    fn the_box_is_not_pre_ticked() {
        assert!(!DefaultAppDialog::new().dont_ask);
    }

    /// **A machine that already opens PDFs with pdfcer is not asked**, and the
    /// preference is not what decides it.
    #[test]
    fn a_silenced_preference_is_never_asked() {
        let prefs = Prefs {
            ask_default_app: false,
            ..Prefs::default()
        };
        assert!(!should_offer(&prefs));
    }
}
