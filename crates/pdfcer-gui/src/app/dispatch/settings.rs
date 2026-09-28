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
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(id, "file.settings" | "tools.font_folders")
}

/// Which settings group an id asks to land on, or `None` for the whole window.
#[must_use]
fn focus(id: &str) -> Option<&'static str> {
    // ui-text-exempt: a command id and a settings group key, never displayed.
    match id {
        "tools.font_folders" => Some("fonts"),
        _ => None,
    }
}

/// Open the Settings window, at the group the id names.
pub(crate) fn dispatch(id: &str, draft: &mut Option<Draft>, settings: &Settings, prefs: &Prefs) {
    if draft.is_none() {
        *draft = Some(Draft::focused_on(settings, prefs, focus(id)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Only the font route asks for a page, and it asks for one that
    /// exists.**
    #[test]
    fn the_font_route_lands_on_a_group_the_dialog_draws() {
        assert_eq!(focus("file.settings"), None);
        assert_eq!(focus("tools.font_folders"), Some("fonts"));
        assert!(handles("file.settings"));
        assert!(handles("tools.font_folders"));
        assert!(!handles("file.print"));

        assert!(
            crate::dialogs::settings::nav::has_page("fonts"),
            "the Settings window no longer has a page keyed `fonts`, so the Tools route lands nowhere and does so silently"
        );
    }
}
