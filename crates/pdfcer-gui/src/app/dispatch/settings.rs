//! # `app::dispatch::settings` — the two commands that open the Settings window
//!
//! Split out of [`super`] under **R2** on 2026-08-28, when that file crossed
//! 1,500 lines for the **third time in one session** — a rate that is itself the
//! argument for cutting rather than trimming prose again.
//!
//! ## ★★ The seam, and why two ids share one arm
//!
//! `file.settings` and `tools.font_folders` open **one window, one draft, one
//! Save**. They differ in one thing: *where the window lands*.
//!
//! | id | the question it asks | where it lands |
//! |---|---|---|
//! | `file.settings` | *"show me the settings"* | the top, correctly |
//! | `tools.font_folders` | *"where do font folders live"* | the Fonts group, opened and scrolled to |
//!
//! ★★★ **`tools.font_folders` was a pure route until 2026-08-28** — it raised
//! `Action::Command("file.settings")` from [`super::routes`], on the stated rule
//! that *a second route to an existing command must not become a second
//! implementation of it*. That rule is intact and this does not break it: there
//! is still one implementation. What changed is that the two ids stopped being
//! the same **request**.
//!
//! ⇒ **A route stops being a pure route the moment the target needs to know WHY
//! it was reached.** `routes`' shape cannot express that — its `target` returns
//! a bare id and has nowhere to put an operand — so the arm moved here rather
//! than the mechanism growing a parameter for one caller.
//!
//! ## ★★★ What it cost to leave it a pure route, which is the whole reason
//!
//! `OPERATOR_REQUESTS.md` **O50** opens with the operator asking for a
//! font-folder setting **that had shipped the day before**. The route named
//! after his question opened the window at the top of ten *collapsed* headings
//! and left the finding to him.
//!
//! A driven run with the landing removed shows the state he was in: three
//! headings visible — `presets`, `appearance`, `colour` — and `settings.fonts`
//! not declared at all. **Opening the right window is not the same as answering
//! the question the command's own name asks.**

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
/// ★ One function, so [`handles`] and [`dispatch`] cannot answer differently
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
/// ★ **Application-scoped**, like About: these are choices about pdfcer, and an
/// operator who has just launched the program and wants a dark window should not
/// have to open a document first.
///
/// Two things a reader will ask, both answered on [`Draft`] rather than repeated
/// here. **The draft opens on the LIVE configuration** — the session's
/// `Settings`, not a re-read of the file — because a session honouring a choice
/// the disk does not have must show what pdfcer is *doing* rather than what it
/// wished it had written. **Re-opening does not reset a draft in progress**,
/// which is `DialogsState::open_print`'s guard and matters more here, because
/// four of these settings change saved bytes and the window's whole promise is
/// that nothing takes effect until Save.
///
/// ★★ The guard is also what makes the landing safe to re-fire: pressing Tools ▸
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

    /// ★★ **Only the font route asks for a group, and it asks for one that
    /// exists.**
    ///
    /// The second half is the load-bearing one: the group key is a string
    /// matched against `widgets::group_focused`'s `key` in the dialog, so a typo
    /// produces a window that opens at the top with no error anywhere — which is
    /// precisely the behaviour this whole change exists to remove, restored
    /// silently.
    #[test]
    fn the_font_route_lands_on_a_group_the_dialog_draws() {
        assert_eq!(focus("file.settings"), None);
        assert_eq!(focus("tools.font_folders"), Some("fonts"));
        assert!(handles("file.settings"));
        assert!(handles("tools.font_folders"));
        assert!(!handles("file.print"));

        // ★ The key, checked against the dialog's own source rather than
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
