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

use egui::Context;

/// Pick the `n`th chord out of a comma-separated list, ignoring blanks.
///
/// Split out of [`scripted_press`] so the selection can be tested without
/// touching the process environment — `std::env::set_var` is process-global
/// and the test harness is multi-threaded, so a test that sets it is a test
/// that can change another test's answer.
fn nth_chord(list: &str, n: usize) -> Option<&str> {
    list.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .nth(n)
}

/// How many of the application's own frames separate one scripted chord from
/// the next, and precede the first.
///
/// ★★★ **Measured, not chosen.** With the chords delivered on consecutive
/// frames, a six-rung zoom ladder against `fixtures/a1-titleblock.pdf` climbed
/// only five rungs: the first `Ctrl` `+` arrived before the canvas had laid
/// out, so `FitMode::Page` was still standing and the next frame's fit solve
/// overwrote the zoom it had just set. The trace said `spelled=yes` for that
/// rung, because that is all it can honestly say — the seam runs before
/// [`super::collect`] and cannot know what became of the press.
///
/// ⚠ **A swallowed rung is worse than a failed one.** It makes a ladder's end
/// state depend on how fast the machine got its first frame out, which is the
/// non-determinism a driven check exists to remove. A gap in front of the
/// first chord removes it structurally: by frame twenty the fit is long since
/// solved, on any machine.
///
/// ⚠ **Frames, not milliseconds, and the two are not interchangeable here.**
/// [`scripted_press`] requests a repaint while chords remain, so the frames
/// come as fast as the application can produce them and twenty of them is not
/// a settle. What the gap buys is ordering — the state each rung acts on is
/// the state the rung before it left — and separability in the trace. What it
/// does **not** buy is a completed raster, and a check that needs one must
/// wait for the application's own `render-async-done`.
const CHORD_GAP_FRAMES: u64 = 20;

/// **Deliver the chords named in the environment — the seam for a window that
/// OS input cannot reach.**
///
/// `PDFCER_DIAG_KEYS="Ctrl++,Ctrl++,Ctrl+-"`, spelled in [`super::parse_chord`]'s
/// grammar, one chord every [`CHORD_GAP_FRAMES`] frames, gated on
/// [`crate::diag::enabled`].
///
/// # ★★★ Why this exists when `PDFCER_DIAG_INVOKE` already does
///
/// That seam dispatches a **registered command id**, and the verbs this seam
/// is for have no command id. `view.zoom_in`, `view.zoom_out`, `view.next_page`
/// and `view.prev_page` are not registered — deliberately, because
/// `RIBBON_IA.md` assigns them to the status bar and the keyboard rather than
/// to the ribbon, and `app::dispatch::zoom`'s header states the position.
/// Registering them so a harness could reach them would change what the
/// product offers in order to make it testable, which is a ribbon decision
/// wearing a test's clothes.
///
/// ⇒ this seam substitutes for the **gesture the harness cannot make**, and
/// changes nothing about what the operator can reach.
///
/// # Where it enters, and why nowhere else would do
///
/// Into `RawInput`'s event stream, before [`super::collect`] reads it, so the
/// synthetic press goes through every line a real one does: the D1 typing
/// guard, the modifier match, the action push, the dispatcher. A seam that
/// called `Action::ZoomIn` directly would pass on a build where `Ctrl` `+`
/// was broken, which is the one thing it is here to detect.
///
/// # ⚠ The standing modifier state is set too, and it must be
///
/// `egui` carries the modifiers twice — on the event, and as the frame's
/// standing state fed from `RawInput::modifiers`. Any reader of the second
/// kind is blind to an event that carries `Ctrl` only on itself. Setting one
/// and not the other would make this seam a source of silent no-ops that look
/// exactly like a dead handler.
///
/// # ★★ It keeps the application awake until the list is finished
///
/// `egui` draws on demand. A quiescent viewer requests no repaint, so a seam
/// that waited for frame twenty in a window nobody is touching would wait
/// forever — and the symptom would be a harness that hangs and then reports
/// the *last* rung as the one that failed. While chords remain, this asks for
/// the next frame itself. The cost is a spinning viewer, which is exactly what
/// a driven run wants and is confined to `PDFCER_DIAG` runs.
///
/// # What is deliberately absent
///
/// No repeat count, no syntax, no state. A ladder of thirty rungs is thirty
/// commas, built by the harness; the argument `scripted_invoke` gives against
/// growing a grammar applies here word for word.
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
    ///
    /// Blanks are skipped rather than counted, so a trailing comma or a list
    /// assembled by joining an empty slot does not spend a frame delivering
    /// nothing — which would show up in a driven ladder as one rung silently
    /// missing and a ceiling one step lower than the run intended.
    ///
    /// ⚠ The selection is tested here and not through the environment on
    /// purpose: `std::env::set_var` is process-global, the test harness is
    /// multi-threaded, and a test that sets it can change another test's
    /// answer without either test being wrong.
    #[test]
    fn the_scripted_key_list_is_taken_in_order_and_skips_blanks() {
        assert_eq!(nth_chord("Ctrl++,Ctrl+-", 0), Some("Ctrl++"));
        assert_eq!(nth_chord("Ctrl++,Ctrl+-", 1), Some("Ctrl+-"));
        assert_eq!(nth_chord("Ctrl++,Ctrl+-", 2), None);
        assert_eq!(nth_chord(" Ctrl++ , , Ctrl+- ,", 1), Some("Ctrl+-"));
        assert_eq!(nth_chord("", 0), None);
    }
}
