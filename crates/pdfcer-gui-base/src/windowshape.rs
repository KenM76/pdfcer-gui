//! # `windowshape` — the two verbs in View ▸ Window that change the *shape of
//! the application* rather than anything about the document
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/windowshape.md`.

use egui::{Id, ViewportCommand};

/// `egui::Memory` key for "the chrome is hidden".
const READ_MODE_ID: &str = "pdfcer-read-mode"; // ui-text-exempt: widget id, never displayed

/// The command id whose chord the two exit statements name.
#[doc(hidden)]
pub const READ_MODE_COMMAND: &str = "view.read_mode"; // ui-text-exempt: command id, never displayed

/// The full-screen command id, named here for the same reason as its sibling.
///
/// Used only for the combined state — see the module header's *Full screen is
/// NOT the same trap* section.
#[doc(hidden)]
pub const FULLSCREEN_COMMAND: &str = "view.fullscreen"; // ui-text-exempt: command id, never displayed

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
#[doc(hidden)]
pub const PENDING_FRAMES: u64 = 4;

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
