//! **Which window a dialog's OS window belongs to.**
//!
//! A dialog is owned by the application window for as long as it is open, so
//! it stays in front of that window and keeps the keyboard. An *independent*
//! window, such as the 3D viewer, is owned only for [`HOLD_SECS`] after it
//! opens and then released, so it minimises, restores and goes behind on its
//! own. The hold covers the opening: a window that is never owned is handed the
//! keyboard and then, a third of a second later, loses the foreground back to
//! the application window (measured; see `docs/modules/pdfcer-gui-base/dialoghost.md`).
//!
//! Both calls are idempotent and run every frame; a window the platform has
//! not created yet answers "not yet" and is tried again next frame.

use egui::ViewportId;

/// How long, in seconds of the toolkit's clock, an independent window stays
/// owned after it opens.
const HOLD_SECS: f64 = 1.5;

/// Own or release the window titled `title`. `fresh` is true on the pass the
/// dialog opened on, which starts an independent window's hold.
pub(super) fn hold(
    ctx: &egui::Context,
    id: ViewportId,
    title: &str,
    owner: Option<isize>,
    (independent, fresh): (bool, bool),
) {
    let Some(owner) = owner else {
        return;
    };
    if !independent {
        let owned = native_window::own_window(owner, title);
        crate::diag::trace_on_change("dialog-owned", || {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("title={title:?} owned={owned}")
        });
        return;
    }
    // ui-text-exempt: a memory key, never displayed.
    let key = egui::Id::new(("dialog-host-opened-time", id));
    let now = ctx.input(|i| i.time);
    if fresh {
        ctx.data_mut(|d| d.insert_temp(key, now));
    }
    let opened = ctx.data(|d| d.get_temp::<f64>(key)).unwrap_or(now);
    let holding = now - opened < HOLD_SECS;
    if holding {
        // The release must happen even if nothing else repaints the window.
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(
            HOLD_SECS - (now - opened),
        ));
    }
    let done = if holding {
        native_window::own_window(owner, title)
    } else {
        native_window::disown_window(title)
    };
    crate::diag::trace_on_change("dialog-independent", || {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("title={title:?} holding={holding} done={done}")
    });
}
