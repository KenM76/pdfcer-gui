//! **Why the print window is closing, and what that does to the settings** —
//! `OPERATOR_REQUESTS.md` **O185**, the operator's words of 2026-09-14: *"I set
//! the printer up, close the window to go check something, and it's all
//! gone."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/print/dismissal.md`.

use super::PrintDialog;

/// **Why the print window is closing**, and therefore what happens to the
/// settings on the way out. `OPERATOR_REQUESTS.md` **O185**.
///
/// # Three meanings, because the window owns state the document does not
///
/// Until O185 this was a bare `bool` and it could not have been anything else:
/// a window whose only product is a print job has one way to leave it that
/// matters and one that does not. The moment O166 gave the window a
/// **persistent** subset -- [`crate::app::prefs::PrintPrefs`] -- leaving it
/// became a decision about that subset, and a boolean cannot carry a decision.
///
/// # How each variant reaches this type
///
/// | variant | route |
/// |---|---|
/// | [`Self::Revert`] | the OS close button, Escape, **or** the Cancel button |
/// | [`Self::Keep`] | the *Keep and close* button, and nothing else |
/// | [`Self::Printed`] | a spool the driver accepted |
///
/// **The first row is `dialogs.md` G4 held to the letter.** That rule makes
/// the chrome, Escape and Cancel deliberately indistinguishable, so all three
/// must mean the same thing -- and the thing they mean has to be the SAFE one,
/// because the chrome is what an operator presses without deciding anything.
/// Inheriting three hundred copies and the wrong tray from a window you shut in
/// irritation is the hazard; re-entering four settings is not.
///
/// *Keep and close* is then a **fourth, positively-chosen** route, which G4
/// never contemplated and does not govern. See
/// [`crate::text::print::keep_and_close`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
// `pub(in crate::dialogs)` rather than `pub(super)`, so that `super`'s
// re-export can hand the same visibility on. A `pub(super)` here would stop at
// `dialogs::print` and the footer's `super::Dismissal` would not resolve.
pub(in crate::dialogs) enum Dismissal {
    /// Put the settings back to [`PrintDialog::opened_with`].
    Revert,
    /// Write the settings to the preferences file, and print nothing.
    Keep,
    /// The job is away. The settings were already written, **before** the
    /// spool and whether or not it succeeded -- see the commit block -- so
    /// this variant writes nothing and reverts nothing. It exists so the
    /// closing path can say which of the three happened without lying about
    /// the other two.
    Printed,
}

impl PrintDialog {
    /// **Decide what this window's closing does to the settings, and say so.**
    pub(super) fn dismiss(
        &mut self,
        prefs: &mut crate::app::prefs::Prefs,
        closed: bool,
        saved_on_commit: bool,
    ) -> bool {
        // G4 is intact and is now visible in the code rather than implied by
        // it: `closed` — the OS close button and Escape — maps to the
        // same [`Dismissal::Revert`] the Cancel button sets, so all three
        // routes the window chrome offers keep one meaning. What changed at
        // O185 is that there is a FOURTH route with a label on it, and a
        // labelled button the operator pressed on purpose is the one case G4
        // never contemplated. See [`Dismissal`].
        //
        // An explicit button wins over `closed` in the `or` below.
        // They cannot both be set by the same gesture, but a frame in which
        // egui reported a viewport close AND a footer press should honour the
        // press: it carries a decision and the other does not.
        let Some(reason) =
            std::mem::take(&mut self.dismissal).or(closed.then_some(Dismissal::Revert))
        else {
            return true;
        };
        let (saved, reverted) = match reason {
            // The one arm that does NOT call a writer, because the writing
            // already happened -- above, before the spool. Calling `remember`
            // again here would not double-write (it would find the value
            // unchanged and decline), but it would report the settings as
            // persisted even when the write it is standing in for FAILED: the
            // in-memory `prefs.print` was assigned before `save()` was asked,
            // so a second call compares equal and answers `true` about a disk
            // that refused. The local carries the real answer forward.
            Dismissal::Printed => (saved_on_commit, false),
            Dismissal::Keep => (self.remember(prefs), false),
            // Both fields of one call, which is the whole reason `restore`
            // is not a `bool`. `saved` is whether the preferences now hold what
            // the window opened with; `reverted` is whether anything had to be
            // put back to make that true. They are independent, and the arm
            // that reported a hard-coded `false` for the first of them is the
            // reason `restore`'s doc carries a paragraph about this one.
            Dismissal::Revert => {
                let put_back = self.restore(prefs);
                (put_back.stored, put_back.changed)
            }
        };
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-dismissed reason={} saved={saved} reverted={reverted}",
                match reason {
                    Dismissal::Printed => "print",
                    Dismissal::Keep => "keep",
                    Dismissal::Revert => "revert",
                }
            )
        });
        false
    }
}
