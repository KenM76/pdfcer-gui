//! **Photographing a gesture while it is still happening.**
//!
//! Every other verb in [`super`] presses and releases inside one call. That
//! makes a whole class of behaviour invisible to this harness — the rubber
//! band, the move ghost, the travelling copy of a selection's own pixels, the
//! snap indicator, the resize handles. All of them exist only between the
//! press and the release, and by the time a caller regains control the button
//! is up and the affordance is gone.
//!
//! [`Driver::drag_observed`] holds the frame open: press, walk to the
//! destination, rest there long enough for a frame to run with the pointer
//! where it will be photographed, then hand control out with the button still
//! down.
//!
//! # Contract on the observer
//!
//! It may read the screen. It **must not move the pointer** — capture is
//! passive, and anything that sets the cursor position mid-hold changes the
//! gesture being photographed rather than recording it.
//!
//! It may fail, and it may panic. The release is in a guard's `Drop` for that
//! reason: a `?` on the observer's result is enough to leave the physical
//! button down, and a harness that does that has handed the operator a machine
//! that rubber-bands his desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/input/observed.md`.

use super::{CLICK_HOLD, DWELL_NUDGE_TICKS, Driver, MOVE_SETTLE, OBSERVE_DWELL};
use crate::coords::ScreenPoint;
use crate::error::Result;
use crate::sys;

/// **The primary button, down for as long as this value lives.**
struct ButtonHeld;

impl ButtonHeld {
    /// Press, and settle for the same interval every other press in this
    /// module settles for.
    fn press() -> Self {
        sys::mouse_button(true);
        std::thread::sleep(CLICK_HOLD);
        Self
    }
}

impl Drop for ButtonHeld {
    fn drop(&mut self) {
        sys::mouse_button(false);
        std::thread::sleep(MOVE_SETTLE);
    }
}

impl Driver {
    /// **Drag from `from` to `to`, and run `observe` at the destination with
    /// the button still down.**
    pub fn drag_observed<T>(
        &self,
        from: ScreenPoint,
        to: ScreenPoint,
        observe: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        self.raise_and_confirm()?;
        self.confirm_uncovered(from)?;
        self.confirm_uncovered(to)?;
        sys::set_cursor_position(from.x(), from.y())?;
        std::thread::sleep(MOVE_SETTLE);

        let _held = ButtonHeld::press();
        self.walk(from, to)?;
        // The dwell, as a handful of one-pixel jiggles rather than one sleep —
        // the argument is [`Self::drag_via`]'s and it applies with more force
        // here, because the frame this produces is the measurement.
        let per = OBSERVE_DWELL / DWELL_NUDGE_TICKS;
        for tick in 0..DWELL_NUDGE_TICKS {
            let nudge = i32::from(tick % 2 == 0);
            sys::set_cursor_position(to.x() + nudge, to.y())?;
            std::thread::sleep(per);
        }
        observe()
    }
}
