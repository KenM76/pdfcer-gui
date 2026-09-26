//! # `app::keyboard::scripted` — a keystroke for a window OS input cannot reach
//!
//! `PDFCER_DIAG_KEYS` names chords in [`super::parse_chord`]'s grammar and this
//! module delivers them into the frame's event stream, so that a window placed
//! off the desktop — which takes no OS input at all — can still be driven
//! through the *viewer* verbs.
//!
//! Those verbs are the reason a second seam exists at all. `PDFCER_DIAG_INVOKE`
//! rings a **registered command id**, and zoom-in, zoom-out, next-page and
//! previous-page have none: `RIBBON_IA.md` assigns them to the status bar and
//! the keyboard, and registering them so a harness could reach them would
//! change what the product offers in order to make it testable.
//!
//! It reports one line per rung, whether or not the chord could be spelled:
//!
//! ```text
//! diag-keys index=0 chord=Ctrl++ spelled=yes
//! ```
//!
//! ⚠ `spelled=yes` means the key was **pushed**, and nothing more. This module
//! runs before [`super::collect`] and cannot know what became of the press, so
//! a check must read the effect it wanted from the application's own trace —
//! `status … zoom=`, `render-spawn … scale=` — keyed on the `index=` above.
//! Counting rungs and inferring a result is the one reading this line forbids.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/keyboard/scripted.md`.

use egui::Context;

/// Pick the `n`th chord out of a comma-separated list, ignoring blanks.
fn nth_chord(list: &str, n: usize) -> Option<&str> {
    list.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .nth(n)
}

/// How many of the application's own frames separate one scripted chord from
/// the next, and precede the first.
const CHORD_GAP_FRAMES: u64 = 20;

/// **Deliver the chords named in the environment — the seam for a window that
/// OS input cannot reach.**
pub fn scripted_press(ctx: &Context) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    /// How many chords of the list have been delivered.
    static SENT: AtomicUsize = AtomicUsize::new(0);

    if !crate::diag::enabled() {
        return;
    }
    // ui-text-exempt: an environment variable name, never displayed.
    let Ok(list) = std::env::var("PDFCER_DIAG_KEYS") else {
        return;
    };
    let n = SENT.load(Ordering::Relaxed);
    let Some(spelling) = nth_chord(&list, n) else {
        return;
    };

    // There is at least one chord still to deliver, so there must be at least
    // one more frame. See the header — without this the gap below is a wait
    // for a frame an idle viewer will never draw.
    ctx.request_repaint();
    let due = u64::try_from(n).unwrap_or(u64::MAX).saturating_add(1) * CHORD_GAP_FRAMES;
    if ctx.cumulative_pass_nr() < due {
        return;
    }

    // Consumed before it is spelled, so an unspellable entry costs one rung
    // rather than wedging the list on it forever.
    SENT.store(n.saturating_add(1), Ordering::Relaxed);

    let Some((modifiers, key)) = super::parse_chord(spelling) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("diag-keys index={n} chord={spelling} spelled=no")
        });
        return;
    };
    ctx.input_mut(|i| {
        i.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
        i.modifiers = modifiers;
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("diag-keys index={n} chord={spelling} spelled=yes")
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scripted-keystroke seam takes one chord per frame, in order.
    #[test]
    fn the_scripted_key_list_is_taken_in_order_and_skips_blanks() {
        assert_eq!(nth_chord("Ctrl++,Ctrl+-", 0), Some("Ctrl++"));
        assert_eq!(nth_chord("Ctrl++,Ctrl+-", 1), Some("Ctrl+-"));
        assert_eq!(nth_chord("Ctrl++,Ctrl+-", 2), None);
        assert_eq!(nth_chord(" Ctrl++ , , Ctrl+- ,", 1), Some("Ctrl+-"));
        assert_eq!(nth_chord("", 0), None);
    }
}
