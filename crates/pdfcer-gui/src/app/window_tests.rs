//! Tests for `crate::app::window`, kept in the gui because they reach gui modules.

use crate::app::window::*;

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
