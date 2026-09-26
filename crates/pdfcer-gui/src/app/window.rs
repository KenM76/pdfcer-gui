//! # `app::window` — the two verbs in View ▸ Window that change the *shape of
//! the application* rather than anything about the document
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/window.md`.

use egui::{Id, ViewportCommand};

/// `egui::Memory` key for "the chrome is hidden".
const READ_MODE_ID: &str = "pdfcer-read-mode"; // ui-text-exempt: widget id, never displayed

/// The command id whose chord the two exit statements name.
const READ_MODE_COMMAND: &str = "view.read_mode"; // ui-text-exempt: command id, never displayed

/// The full-screen command id, named here for the same reason as its sibling.
///
/// Used only for the combined state — see the module header's *Full screen is
/// NOT the same trap* section.
const FULLSCREEN_COMMAND: &str = "view.fullscreen"; // ui-text-exempt: command id, never displayed

/// `egui::Memory` key for **the chord that turns read mode off**, as the live
/// keymap holds it this session.
const EXIT_CHORD_ID: &str = "pdfcer-read-mode-exit-chord"; // ui-text-exempt: memory key, never displayed

/// `egui::Memory` key for the full-screen chord. See [`fullscreen_chord`].
const FULLSCREEN_CHORD_ID: &str = "pdfcer-fullscreen-chord"; // ui-text-exempt: memory key, never displayed

/// The chord a keymap binds to a command, choosing exactly as a menu chooses.
#[must_use]
pub fn chord_for<'a>(
    keymap: Option<&'a egui_shell::manifest::Keymap>,
    command: &str,
) -> Option<&'a str> {
    let keymap = keymap?;
    let mut best: Option<&str> = None;
    for (chord, bound) in keymap.iter() {
        if bound != command {
            continue;
        }
        match best {
            Some(incumbent)
                if egui_shell::menu::shortcut::prefer(chord, incumbent)
                    != std::cmp::Ordering::Less => {}
            _ => best = Some(chord),
        }
    }
    best
}

/// **Publish this frame's exit chords**, before anything that states them
/// draws.
pub fn publish_exit_chord(ctx: &egui::Context, shell: Option<&egui_shell::manifest::Shell>) {
    let keymap = shell.and_then(|s| s.keymap.as_ref());
    let put = |id: &str, command: &str| {
        let chord = chord_for(keymap, command).map(str::to_owned);
        ctx.data_mut(|d| d.insert_temp(Id::new(id), chord));
    };
    put(EXIT_CHORD_ID, READ_MODE_COMMAND);
    put(FULLSCREEN_CHORD_ID, FULLSCREEN_COMMAND);
}

/// The chord that turns read mode off, as published this session.
#[must_use]
pub fn exit_chord(ctx: &egui::Context) -> Option<String> {
    ctx.data(|d| d.get_temp::<Option<String>>(Id::new(EXIT_CHORD_ID)))
        .flatten()
}

/// The chord that leaves full screen, as published this session.
#[must_use]
pub fn fullscreen_chord(ctx: &egui::Context) -> Option<String> {
    ctx.data(|d| d.get_temp::<Option<String>>(Id::new(FULLSCREEN_CHORD_ID)))
        .flatten()
}

/// Whether the ribbon and the docks are drawn this frame.
#[must_use]
pub fn draws_chrome(ctx: &egui::Context) -> bool {
    !read_mode(ctx)
}

/// Whether read mode is on.
#[must_use]
pub fn read_mode(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(Id::new(READ_MODE_ID)))
        .unwrap_or(false)
}

/// Flip read mode, and report the state it landed in.
pub fn toggle_read_mode(ctx: &egui::Context) -> bool {
    let next = !read_mode(ctx);
    ctx.data_mut(|d| d.insert_temp(Id::new(READ_MODE_ID), next));
    // The chrome appearing or disappearing changes the space left for the
    // canvas, which an active `FitMode` recomputes its zoom from. Requesting
    // the repaint here rather than relying on the click's own is what makes
    // the **chord** behave identically: egui wakes on input, and a chord
    // pressed while nothing else is happening would otherwise leave the new
    // composition undrawn until the next unrelated event.
    ctx.request_repaint();
    next
}

/// Whether the window is in full screen, as the **windowing system** reports
/// it.
#[must_use]
pub fn fullscreen(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().fullscreen).unwrap_or(false)
}

/// The `egui::Memory` key the last full-screen **request** is parked under,
/// with the frame it was made on.
const PENDING_FULLSCREEN: &str = "pdfcer.window.fullscreen-asked"; // ui-text-exempt: memory key.

/// How many frames a full-screen request is believed over the viewport's own
/// report before the report wins again.
const PENDING_FRAMES: u64 = 4;

/// The value a press of `view.fullscreen` should ask the viewport for.
#[must_use]
pub fn next_fullscreen(reported: Option<bool>, pending: Option<(u64, bool)>, now: u64) -> bool {
    let current = match pending {
        // A request this shell made, recently, that the report has not caught
        // up with. Ours is the truth for now.
        Some((then, asked))
            if now.saturating_sub(then) <= PENDING_FRAMES && reported != Some(asked) =>
        {
            asked
        }
        // Either there is no outstanding request, or the report has confirmed
        // it, or it has been outstanding too long to believe. In all three the
        // windowing system's answer is the one to use — and an unreported state
        // counts as windowed, which is the honest default: it is what a window
        // that has never been asked to fill the display is.
        _ => reported.unwrap_or(false),
    };
    !current
}

/// Flip full screen, and report the state that was asked for.
pub fn toggle_fullscreen(ctx: &egui::Context) -> bool {
    let id = egui::Id::new(PENDING_FULLSCREEN);
    let now = ctx.cumulative_pass_nr();
    let pending: Option<(u64, bool)> = ctx.data(|d| d.get_temp(id));
    let reported = ctx.input(|i| i.viewport().fullscreen);
    let next = next_fullscreen(reported, pending, now);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // It carries BOTH the report and the outstanding request, because the
        // failure is the two disagreeing. A line saying only `asked=true` is
        // identical for the build that works and the build that asks for the
        // same thing twice.
        format!("fullscreen-toggle reported={reported:?} pending={pending:?} asked={next}")
    });
    ctx.data_mut(|d| d.insert_temp(id, (now, next)));
    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(next));
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Read mode starts off, flips, and flips back.**
    #[test]
    fn read_mode_starts_off_and_toggles_both_ways() {
        let ctx = egui::Context::default();
        assert!(
            !read_mode(&ctx),
            "a shell that opened with no ribbon and no way to say how to get \
             one back would look broken"
        );
        assert!(draws_chrome(&ctx));

        assert!(toggle_read_mode(&ctx), "the first press turns it on");
        assert!(read_mode(&ctx));
        assert!(
            !draws_chrome(&ctx),
            "the whole behaviour: the ribbon and the docks are not drawn"
        );

        assert!(!toggle_read_mode(&ctx), "the second press turns it off");
        assert!(draws_chrome(&ctx));
    }

    /// **`draws_chrome` is exactly the negation of `read_mode`**, asserted so
    /// that a future third thing (a presentation mode, a kiosk switch) has to
    /// change this test rather than silently widening one and not the other.
    #[test]
    fn the_frame_draws_chrome_exactly_when_read_mode_is_off() {
        let ctx = egui::Context::default();
        for _ in 0..3 {
            assert_eq!(draws_chrome(&ctx), !read_mode(&ctx));
            toggle_read_mode(&ctx);
        }
    }

    /// **An unreported viewport state counts as windowed.**
    #[test]
    fn an_unreported_fullscreen_state_is_read_as_windowed() {
        assert!(
            next_fullscreen(None, None, 0),
            "the first press must fill the screen"
        );
        assert!(next_fullscreen(Some(false), None, 0));
        assert!(!next_fullscreen(Some(true), None, 0));
    }

    /// **A second press while the report still lags turns full screen OFF**,
    /// which is the whole reason [`next_fullscreen`] takes three arguments.
    #[test]
    fn a_second_press_before_the_backend_answers_still_toggles_off() {
        // Press one, on frame 10.
        assert!(next_fullscreen(Some(false), None, 10), "press one fills it");
        // Press two, on frame 11. The report has not caught up.
        assert!(
            !next_fullscreen(Some(false), Some((10, true)), 11),
            "★ the second press must ask for WINDOWED. Reading the lagging report \
             instead asks for full screen a second time, and the display never comes back"
        );
    }

    /// …and once the report agrees, the request is spent and the report wins.
    #[test]
    fn a_confirmed_request_hands_authority_back_to_the_report() {
        // We asked for `true` on frame 10 and the report now agrees.
        assert!(
            !next_fullscreen(Some(true), Some((10, true)), 12),
            "a confirmed request must not be believed over the report"
        );
        // The window manager took us out of full screen behind our back; the
        // next press must fill it again rather than "toggling off" a state we
        // are no longer in.
        assert!(
            next_fullscreen(Some(false), Some((10, true)), 20),
            "a stale request must not outlive its window"
        );
    }

    /// **A request the platform never answers expires**, so a shell cannot be
    /// left permanently convinced of a state its window is not in.
    #[test]
    fn an_unanswered_request_expires_rather_than_latching() {
        // Asked on frame 10; it is now well past the window and the report has
        // never agreed. The report wins.
        assert!(
            next_fullscreen(Some(false), Some((10, true)), 10 + PENDING_FRAMES + 1),
            "an unanswered request must stop being believed"
        );
    }

    /// **The chord the operator is told to press is the chord the manifest
    /// binds** — asserted against the real manifest, not against a literal.
    #[test]
    fn the_published_chord_is_the_one_the_manifest_binds() {
        let shell = crate::shell::manifest::built_in();
        let keymap = shell
            .keymap
            .as_ref()
            .expect("the built-in manifest has a keymap");
        let chord = chord_for(Some(keymap), READ_MODE_COMMAND).expect("read mode has a chord");
        assert_eq!(
            keymap.get(chord),
            Some(READ_MODE_COMMAND),
            "the reverse lookup must land on the same binding the dispatcher resolves"
        );

        let ctx = egui::Context::default();
        publish_exit_chord(&ctx, Some(&shell));
        assert_eq!(exit_chord(&ctx).as_deref(), Some(chord));

        let full = chord_for(Some(keymap), FULLSCREEN_COMMAND).expect("full screen has a chord");
        assert_eq!(fullscreen_chord(&ctx).as_deref(), Some(full));
        assert_ne!(chord, full, "two commands, two keys");
    }

    /// **An unbound command yields no chord, and no default is invented.**
    #[test]
    fn an_unbound_command_yields_no_chord_and_no_guess() {
        let empty = egui_shell::manifest::Keymap::default();
        assert_eq!(chord_for(Some(&empty), READ_MODE_COMMAND), None);
        assert_eq!(chord_for(None, READ_MODE_COMMAND), None);

        let ctx = egui::Context::default();
        publish_exit_chord(&ctx, None);
        assert_eq!(exit_chord(&ctx), None);
        assert_eq!(fullscreen_chord(&ctx), None);
    }

    /// **One command bound twice advertises the same chord a menu would show.**
    #[test]
    fn a_command_bound_twice_advertises_what_a_menu_advertises() {
        let mut map = std::collections::BTreeMap::new();
        map.insert("Ctrl+Shift+H".to_owned(), READ_MODE_COMMAND.to_owned());
        map.insert("F9".to_owned(), READ_MODE_COMMAND.to_owned());
        let keymap = egui_shell::manifest::Keymap(map);
        assert_eq!(chord_for(Some(&keymap), READ_MODE_COMMAND), Some("F9"));
        assert_eq!(
            egui_shell::Shortcuts::from_keymap(&keymap).get(READ_MODE_COMMAND),
            chord_for(Some(&keymap), READ_MODE_COMMAND),
            "the two derivations must agree, or the menu and the bar teach different keys"
        );
    }

    /// The headless context reports no viewport full-screen flag, which is the
    /// precondition the test above is about.
    #[test]
    fn a_headless_context_reports_no_fullscreen_state() {
        let ctx = egui::Context::default();
        assert!(!fullscreen(&ctx));
    }
}
