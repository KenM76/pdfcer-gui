//! # `app::dispatch::settings` — the commands that open the Settings window
//!
//! ## The seam, and why two ids share one arm
//!
//! `file.settings` and `tools.font_folders` open **one window, one draft, one
//! Save**. They differ in one thing: *where the window lands*.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/settings.md`.

use crate::app::prefs::Prefs;
use crate::dialogs::settings::Draft;
use pdfcer_core::settings::Settings;

/// Whether this file owns `id`.
///
/// `pub(crate)` for [`super::routes::handles`]' reason: `shell::commands::reach`'s
/// reachability checker must be able to evaluate every guard arm it finds, and a
/// guard it cannot evaluate is a place commands could hide from the check that
/// exists to find them.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(id, "file.settings" | "tools.font_folders")
}

/// Which settings group an id asks to land on, or `None` for the whole window.
///
/// One function, so [`handles`] and [`dispatch`] cannot answer differently
/// about the same id — `routes::target` states that rule and it applies here for
/// the same reason.
#[must_use]
fn focus(id: &str) -> Option<&'static str> {
    // ui-text-exempt: a command id and a settings group key, never displayed.
    match id {
        "tools.font_folders" => Some("fonts"),
        _ => None,
    }
}

/// Open the Settings window, at the group the id names.
///
/// **Application-scoped**, like About: these are choices about pdfcer, and an
/// operator who has just launched the program and wants a dark window should not
/// have to open a document first.
///
/// Two things a reader will ask, both answered on [`Draft`] rather than repeated
/// here. **The draft opens on the LIVE configuration** — the session's
/// `Settings`, not a re-read of the file — because a session honouring a choice
/// the disk does not have must show what pdfcer is *doing* rather than what it
/// wished it had written. **Re-opening does not reset a draft in progress**,
/// which is `DialogsState::open_print`'s guard and matters more here, because
/// some of these settings change saved bytes and the window's whole promise is
/// that nothing takes effect until Save.
///
/// The guard is also what makes the landing safe to re-fire: pressing Tools ▸
/// Font folders while the window is already open does **nothing**, rather than
/// scrolling a window the operator has since scrolled somewhere else.
pub(crate) fn dispatch(id: &str, draft: &mut Option<Draft>, settings: &Settings, prefs: &Prefs) {
    if draft.is_none() {
        *draft = Some(Draft::focused_on(settings, prefs, focus(id)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Only the font route asks for a group, and it asks for one that
    /// exists.**
    ///
    /// The second half is the load-bearing one: the group key is a string
    /// matched against `widgets::group_focused`'s `key` in the dialog, so a typo
    /// produces a window that opens at the top with no error anywhere — the
    /// exact failure the landing exists to prevent, restored silently.
    #[test]
    fn the_font_route_lands_on_a_group_the_dialog_draws() {
        assert_eq!(focus("file.settings"), None);
        assert_eq!(focus("tools.font_folders"), Some("fonts"));
        assert!(handles("file.settings"));
        assert!(handles("tools.font_folders"));
        assert!(!handles("file.print"));

        // The key, checked against the dialog's own source rather than
        // against a second copy of the string. A test asserting
        // `focus(..) == Some("fonts")` against a constant this module also owns
        // would pass on a rename that broke the landing.
        let dialog = include_str!("../../dialogs/settings/mod.rs");
        assert!(
            dialog.contains(r#""fonts", // ui-text-exempt: a group key, never displayed."#),
            "the Settings window no longer draws a group keyed `fonts`, so the Tools route \
             lands nowhere and does so silently"
        );
    }
}
