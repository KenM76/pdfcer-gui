//! One wait a frame for the display, in place of the swap's.
//!
//! eframe's glow back end waits for the display at every window's swap, and
//! a dialog window (`show_viewport_immediate`) swaps inside the main window's
//! frame: with one open, each frame waited twice and ran at half the refresh
//! rate or less, the 3D viewer included. [`crate::run`] turns the swap's wait
//! off; [`frame`] holds the frames to [`SHORTEST`] apart instead, once a
//! frame however many windows are open.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// The shortest time from one frame's start to the next: 120 a second, at or
/// above the refresh rate of the displays it runs on.
pub const SHORTEST: Duration = Duration::from_micros(8_333);

static ON: AtomicBool = AtomicBool::new(false);

thread_local! {
    static LAST: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// Pace every frame from here on; set by a caller that turned the swap's
/// wait off.
pub fn enable() {
    ON.store(true, Ordering::Relaxed);
}

/// Wait out what is left of [`SHORTEST`] since the last frame began. Called
/// first in the main window's frame, before anything is timed.
pub fn frame() {
    if !ON.load(Ordering::Relaxed) {
        return;
    }
    if let Some(last) = LAST.get() {
        let since = last.elapsed();
        if since < SHORTEST {
            std::thread::sleep(SHORTEST - since);
        }
    }
    LAST.set(Some(Instant::now()));
}
