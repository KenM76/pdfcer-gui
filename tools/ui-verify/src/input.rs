//! Drive the pointer and the keyboard **through the operating system**.
//!
//! Design and rationale: `docs/modules/ui-verify/input.md`.

use std::time::Duration;

use crate::coords::ScreenPoint;
use crate::error::{Error, Result};
use crate::sys::{self, WindowHandle};

/// How long the primary button stays down during a synthetic click.
const CLICK_HOLD: Duration = Duration::from_millis(60);

/// How long to wait after moving the pointer before pressing.
const MOVE_SETTLE: Duration = Duration::from_millis(80);

/// How long between the RELEASE of one click and the press of the next, in a
/// double click.
const DOUBLE_CLICK_GAP: Duration = Duration::from_millis(40);

/// How many intermediate positions [`Driver::drag`] walks through.
pub(crate) const DRAG_STEPS: u32 = 8;

/// How long to pause between the intermediate positions of a drag.
const DRAG_STEP_SETTLE: Duration = Duration::from_millis(25);

/// How long [`Driver::carry`] rests on its destination before letting go.
const CARRY_SETTLE: Duration = Duration::from_millis(400);

/// How long the pointer rests at the destination of a [`Driver::drag_observed`]
/// before the observer runs.
const OBSERVE_DWELL: Duration = Duration::from_millis(400);

/// Ticks that dwell is split into, with a one-pixel nudge between them.
const DWELL_NUDGE_TICKS: u32 = 8;

/// The mid-gesture verb: hold a drag open and hand control to an observer.
mod observed;

/// The OS-level input driver.
///
/// Owns the operator's pointer position for its lifetime and returns it on
/// drop.
pub struct Driver {
    original_cursor: Option<(i32, i32)>,
    target: Option<WindowHandle>,
    /// **The window the last pointer action put the focus in.**
    ///
    ///
    /// It now has one per open dialog, and the failure without this field is
    /// specific: a check clicks a field inside a dialog (which raises the
    /// dialog, correctly), then presses a key — and `press` raises **the main
    /// window**, taking focus away from the dialog, so the characters go to the
    /// application. The check then reports that the dialog ignored the
    /// keyboard.
    ///
    /// `Cell` rather than `&mut self`, because every input method takes `&self`
    /// and threading mutability through them would change every call site to
    /// record a fact the driver can perfectly well remember itself.
    focus: std::cell::Cell<Option<WindowHandle>>,
}

impl Driver {
    /// Take the pointer, remembering where it was.
    ///
    #[must_use]
    pub fn new(target: Option<WindowHandle>) -> Self {
        Self {
            original_cursor: sys::cursor_position().ok(),
            target,
            focus: std::cell::Cell::new(None),
        }
    }

    /// Move the pointer and click the primary button.
    pub fn click_at(&self, p: ScreenPoint) -> Result<()> {
        self.raise_and_confirm_at(p)?;
        self.confirm_uncovered(p)?;
        sys::set_cursor_position(p.x(), p.y())?;
        std::thread::sleep(MOVE_SETTLE);
        sys::mouse_button(true);
        std::thread::sleep(CLICK_HOLD);
        sys::mouse_button(false);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Move the pointer and click the **secondary** button.
    pub fn right_click_at(&self, p: ScreenPoint) -> Result<()> {
        self.raise_and_confirm_at(p)?;
        self.confirm_uncovered(p)?;
        sys::set_cursor_position(p.x(), p.y())?;
        std::thread::sleep(MOVE_SETTLE);
        sys::mouse_button_secondary(true);
        std::thread::sleep(CLICK_HOLD);
        sys::mouse_button_secondary(false);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// **Press at `from`, travel to `to`, release** — a real primary-button
    /// drag.
    pub fn drag(&self, from: ScreenPoint, to: ScreenPoint) -> Result<()> {
        self.drag_unmodified(from, to)
    }

    /// [`Self::drag`] with `key` held down for the **whole** gesture.
    pub fn drag_with_modifier(&self, from: ScreenPoint, to: ScreenPoint, key: Key) -> Result<()> {
        self.raise_and_confirm()?;
        sys::with_modifiers(&[key.vk()], || self.drag_unmodified(from, to))
    }

    /// [`Self::drag`]'s body, with whatever modifier state the caller has
    /// already established.
    fn drag_unmodified(&self, from: ScreenPoint, to: ScreenPoint) -> Result<()> {
        self.raise_and_confirm()?;
        //
        // See [`Self::confirm_uncovered`] for the whole argument. It had been
        // wired into `click_at` and `right_click_at` and into no verb that
        // presses the button anywhere else — which left the single most
        // important gesture in the harness, the canvas drag, driving blind into
        // whatever happened to be lying on the desktop.
        //
        // Measured that day on `the_second_stamp_dialog_still_has_its_buttons`.
        // An Outlook *"Internet Email"* dialog (`#32770`, pid 30956) was open
        // over the right-hand half of the target window. The first drag missed
        // it and placed a stamp; the second was aimed at desktop (1408, 571),
        // which was inside the Outlook dialog, so pdfcer saw the pointer leave
        // the canvas (`canvas-gesture … pos=0`) and never saw a button at all.
        // The check then reported *"the second drag traced no further
        // `text-annot-open` line … either the tool did not re-arm or the drag
        // landed on the first stamp"* — an accusation against a feature that
        // works, naming two causes that were both disproved by the same trace.
        //
        // The sibling check `stamp_size_reaches_the_engine`, driven in the
        // same minute against the same desktop, SKIPPED with *"the point
        // (1212, 605) is owned by \"Internet Email — ken@toprops.com\""*. Same
        // machine, same obstruction, same second — and one of the two said so
        // while the other blamed the application. **The difference was entirely
        // which verb it happened to use.**
        //
        // ⇒ Every verb that presses at a coordinate asks the question now.
        self.confirm_uncovered(from)?;
        self.confirm_uncovered(to)?;
        sys::set_cursor_position(from.x(), from.y())?;
        std::thread::sleep(MOVE_SETTLE);
        sys::mouse_button(true);
        std::thread::sleep(CLICK_HOLD);
        for step in 1..=DRAG_STEPS {
            // Integer arithmetic in i64 rather than f64: the endpoints are
            // whole pixels and the intermediate points should be too, so the
            // application is never handed a coordinate a real mouse could not
            // produce.
            let lerp = |a: i32, b: i32| -> i32 {
                let n = i64::from(DRAG_STEPS);
                let (wide_a, wide_b) = (i64::from(a), i64::from(b));
                // The endpoint on overflow rather than a clamp to `i32::MAX`: a
                // coordinate that far out is not a screen position, and
                // finishing where the drag was aimed is the only answer that is
                // not silently somewhere else.
                i32::try_from(wide_a + (wide_b - wide_a) * i64::from(step) / n).unwrap_or(b)
            };
            sys::set_cursor_position(lerp(from.x(), to.x()), lerp(from.y(), to.y()))?;
            std::thread::sleep(DRAG_STEP_SETTLE);
        }
        sys::mouse_button(false);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// **Drag through a waypoint, resting on it.**
    pub fn drag_via(
        &self,
        from: ScreenPoint,
        via: ScreenPoint,
        dwell: std::time::Duration,
        to: ScreenPoint,
        modifier: Option<Key>,
    ) -> Result<()> {
        match modifier {
            Some(key) => sys::with_modifiers(&[key.vk()], || {
                self.drag_via_unmodified(from, via, dwell, to)
            }),
            None => self.drag_via_unmodified(from, via, dwell, to),
        }
    }

    /// [`Self::drag_via`]'s body, with whatever modifier state the caller has
    /// already established.
    fn drag_via_unmodified(
        &self,
        from: ScreenPoint,
        via: ScreenPoint,
        dwell: std::time::Duration,
        to: ScreenPoint,
    ) -> Result<()> {
        self.raise_and_confirm()?;
        // All three, for the reason in [`Self::drag`]. The waypoint matters
        // as much as the endpoints here: this gesture RESTS on `via` until a
        // dwell timer fires, so a covered waypoint is a second of the pointer
        // sitting inside somebody else's window with the button down.
        self.confirm_uncovered(from)?;
        self.confirm_uncovered(via)?;
        self.confirm_uncovered(to)?;
        sys::set_cursor_position(from.x(), from.y())?;
        std::thread::sleep(MOVE_SETTLE);
        sys::mouse_button(true);
        std::thread::sleep(CLICK_HOLD);
        self.walk(from, via)?;
        // The dwell, as a handful of one-pixel jiggles rather than one sleep.
        let ticks = 8;
        let per = dwell / ticks;
        for i in 0..ticks {
            let nudge = i32::from(i % 2 == 0);
            sys::set_cursor_position(via.x() + nudge, via.y())?;
            std::thread::sleep(per);
        }
        self.walk(via, to)?;
        sys::mouse_button(false);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Walk the pointer from `a` to `b` in [`DRAG_STEPS`] increments, with the
    /// button in whatever state the caller left it.
    fn walk(&self, a: ScreenPoint, b: ScreenPoint) -> Result<()> {
        for step in 1..=DRAG_STEPS {
            let lerp = |from: i32, to: i32| -> i32 {
                let n = i64::from(DRAG_STEPS);
                let (wide_a, wide_b) = (i64::from(from), i64::from(to));
                i32::try_from(wide_a + (wide_b - wide_a) * i64::from(step) / n).unwrap_or(to)
            };
            sys::set_cursor_position(lerp(a.x(), b.x()), lerp(a.y(), b.y()))?;
            std::thread::sleep(DRAG_STEP_SETTLE);
        }
        Ok(())
    }

    /// **Carry a floating window's header from `from` to `to` and let go** —
    /// the gesture that drops a torn-out panel back into the dock.
    pub fn carry(&self, from: ScreenPoint, to: ScreenPoint) -> Result<()> {
        self.raise_and_confirm_at(from)?;
        self.confirm_uncovered(from)?;
        self.confirm_uncovered(to)?;
        sys::set_cursor_position(from.x(), from.y())?;
        std::thread::sleep(MOVE_SETTLE);
        sys::mouse_button(true);
        std::thread::sleep(CLICK_HOLD);
        self.walk(from, to)?;
        let ticks = 8;
        let per = CARRY_SETTLE / ticks;
        for i in 0..ticks {
            let nudge = i32::from(i % 2 == 0);
            sys::set_cursor_position(to.x() + nudge, to.y())?;
            std::thread::sleep(per);
        }
        sys::mouse_button(false);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Click twice in the same place, fast enough for the application to read
    /// it as a double click.
    pub fn double_click_at(&self, p: ScreenPoint) -> Result<()> {
        // NOT two `click_at` calls, and the first version of this WAS.
        //
        // `click_at` sleeps `MOVE_SETTLE` before its press and again after its
        // release, so two of them put **390 ms** between the presses — past
        // `egui`'s 300 ms threshold — and the application read four
        // independent single clicks. The check that used it reported "the Node
        // rung was never entered" over a build whose Node rung was fine.
        //
        // The settles exist so a click lands on a settled layout; that argument
        // applies to the FIRST press and to nothing after it, because the
        // second press is at the same point on the same frame's layout. So the
        // pointer is positioned and settled once, and the two press/release
        // pairs follow with only `CLICK_HOLD` between them.
        self.raise_and_confirm()?;
        // Not two `click_at` calls, so it does not inherit their guard — see
        // [`Self::drag`] for the day that distinction cost a false FAIL.
        self.confirm_uncovered(p)?;
        sys::set_cursor_position(p.x(), p.y())?;
        std::thread::sleep(MOVE_SETTLE);
        for _ in 0..2 {
            sys::mouse_button(true);
            std::thread::sleep(CLICK_HOLD);
            sys::mouse_button(false);
            std::thread::sleep(DOUBLE_CLICK_GAP);
        }
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Click with a modifier key held — Shift-click to extend a selection.
    pub fn click_with_modifier(&self, p: ScreenPoint, key: Key) -> Result<()> {
        self.raise_and_confirm()?;
        sys::with_modifiers(&[key.vk()], || self.click_at(p))
    }

    /// **Press `vk` `times` times with `modifiers` HELD DOWN throughout**, the
    /// way a hand does it.
    pub fn press_held(&self, modifiers: &[u16], vk: u16, times: usize) -> Result<()> {
        self.raise_and_confirm()?;
        sys::with_modifiers(modifiers, || {
            // A WHOLE FRAME BEFORE THE FIRST KEY, and this is the fix, not
            // padding. Measured: with `with_modifiers`' own 12 ms gap the
            // application traced `ev=Modifiers::NONE frame=Modifiers { shift:
            // true }` — the modifier HAD arrived and the key that was supposed
            // to carry it did not. egui builds a `Key` event from the modifier
            // state it holds when the key is translated, and a modifier posted
            // less than a frame earlier is applied to `i.modifiers` in the same
            // batch but AFTER the key. A real keyboard cannot produce that
            // ordering: a hand holds Shift for tens of frames first.
            std::thread::sleep(MOVE_SETTLE);
            for _ in 0..times {
                sys::key_stroke(vk);
                std::thread::sleep(MOVE_SETTLE);
            }
        });
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Move the pointer without clicking — for hover assertions, for getting
    /// the pointer off a widget before a screenshot, and for putting it inside
    /// the pane a wheel event is meant for.
    pub fn move_to(&self, p: ScreenPoint) -> Result<()> {
        self.confirm_on_the_desktop(p)?;
        sys::set_cursor_position(p.x(), p.y())?;
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Scroll the pane under a point, then settle.
    pub fn scroll_at(&self, p: ScreenPoint, notches: i32) -> Result<()> {
        // A wheel notch goes to the window under the pointer, exactly as a
        // press does, so this asks the same question a press asks. See
        // [`Self::drag`]. `move_to` alone only checks the point is on a
        // monitor, which a covered point always is.
        self.confirm_uncovered(p)?;
        self.move_to(p)?;
        sys::wheel(notches);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Roll the wheel at `p` with modifiers held — Ctrl+wheel, which in a
    /// document viewer is **zoom about the pointer**.
    pub fn scroll_at_held(
        &self,
        p: ScreenPoint,
        modifiers: &[u16],
        notches: i32,
        times: usize,
    ) -> Result<()> {
        self.raise_and_confirm()?;
        // As [`Self::scroll_at`]. Ctrl+wheel into a foreign window is worse
        // than a plain notch: it zooms whatever owns the pixel.
        self.confirm_uncovered(p)?;
        self.move_to(p)?;
        sys::with_modifiers(modifiers, || {
            std::thread::sleep(MOVE_SETTLE);
            for _ in 0..times {
                sys::wheel(notches);
                std::thread::sleep(MOVE_SETTLE);
            }
        });
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Wheel `times` notches with `modifiers` held and `gap` between notches,
    /// returning as soon as the last notch is sent.
    pub fn scroll_burst(
        &self,
        p: ScreenPoint,
        modifiers: &[u16],
        notches: i32,
        times: usize,
        gap: Duration,
    ) -> Result<()> {
        self.raise_and_confirm()?;
        self.confirm_uncovered(p)?;
        self.move_to(p)?;
        sys::with_modifiers(modifiers, || {
            std::thread::sleep(MOVE_SETTLE);
            for i in 0..times {
                if i > 0 {
                    std::thread::sleep(gap);
                }
                sys::wheel(notches);
            }
        });
        Ok(())
    }

    /// Press and release a virtual key, in the target window.
    pub fn press(&self, vk: u16) -> Result<()> {
        if self.target.is_none() {
            return Err(Error::new(
                "refusing to send a keystroke with no target window: it would go to whatever \
                 window is in front, which may be the operator's own",
            ));
        }
        self.raise();
        std::thread::sleep(MOVE_SETTLE);
        sys::key_stroke(vk);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// **Type an ASCII string, one real keystroke per character.**
    pub fn type_ascii(&self, text: &str) -> Result<()> {
        //
        // The whole incident is written up on [`crate::sys::caps_lock_is_on`].
        // In one line: with the latch on, "press the letter key with no Shift"
        // types a CAPITAL, so this function was silently typing `USERPW` when
        // its caller asked for `userpw` — and the one check that compared what
        // it typed against anything reported the application as rejecting a
        // correct password.
        //
        // Read once, before the loop, rather than per character: the latch
        // cannot change during a synthetic burst, and reading it per key would
        // make the cost linear in the string for no gain.
        //
        // Compensating rather than clearing is deliberate. CapsLock belongs
        // to the operator; a harness that toggles it leaves his keyboard in a
        // state he did not put it in, and this suite runs unattended.
        let caps = crate::sys::caps_lock_is_on();
        for ch in text.chars() {
            match ch {
                'a'..='z' => {
                    // VK codes for letters are the ASCII codes of their
                    // UPPERCASE forms; Shift is what distinguishes the case —
                    // and CapsLock swaps which way round that is.
                    if caps {
                        self.press_chord(&[crate::sys::vk::SHIFT], ch.to_ascii_uppercase() as u16)?;
                    } else {
                        self.press(ch.to_ascii_uppercase() as u16)?;
                    }
                }
                'A'..='Z' => {
                    if caps {
                        self.press(ch as u16)?;
                    } else {
                        self.press_chord(&[crate::sys::vk::SHIFT], ch as u16)?;
                    }
                }
                '0'..='9' => {
                    self.press(ch as u16)?;
                }
                //
                // `a_field_too_small_for_its_text_says_so` needs to type a long
                // realistic value into a form field — a person's name, an
                // address — and refused here on the first space. The tempting
                // fix was to write the fixture value without spaces; that would
                // have been a check quietly testing a string no operator would
                // ever type, because of a limitation in the instrument.
                //
                // `VK_SPACE` is `0x20`, the same as the ASCII code, and it is
                // the one printable key that is **immune to both Shift and
                // CapsLock** — so it needs none of the latch compensation the
                // letter arms above carry, and cannot acquire the defect that
                // made those arms necessary.
                //
                // ⚠ Still deliberately NOT a general "printable ASCII" arm.
                // Punctuation is where the layout dependence lives: `-`, `.`
                // and `/` are `VK_OEM_*` codes whose meaning is keyboard-layout
                // specific, so a check typing them would pass here and type
                // something else on a machine with a different layout. Those
                // stay refused, loudly, which is this function's whole
                // contract.
                ' ' => {
                    self.press(0x20)?;
                }
                // The four arithmetic signs go through the NUMERIC KEYPAD, whose
                // virtual keys type the same character on every layout — unlike
                // the main-row `VK_OEM_*` keys the paragraph above refuses.
                '+' => self.press(0x6B)?,
                '-' => self.press(0x6D)?,
                '*' => self.press(0x6A)?,
                '/' => self.press(0x6F)?,
                other => {
                    return Err(crate::error::Error::new(format!(
                        "`type_ascii` has no key for {other:?}. It refuses rather than skipping, \
                         because typing a shorter string than the caller asked for would make the \
                         application look as though it had rejected correct input."
                    )));
                }
            }
        }
        Ok(())
    }

    /// Press a **chord** — a virtual key with modifiers held — in the target
    /// window.
    pub fn press_chord(&self, modifiers: &[u16], vk: u16) -> Result<()> {
        self.raise_and_confirm()?;
        sys::key_stroke_with(modifiers, vk);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// **Which of the application's windows owns this point**, decided by
    /// geometry rather than by z-order.
    fn window_owning(&self, p: ScreenPoint) -> Option<WindowHandle> {
        let target = self.target?;
        let pid = sys::pid_of_window(target)?;
        let mut best: Option<(WindowHandle, u64)> = None;
        for w in sys::windows_for_pid(pid) {
            let Ok(frame) = sys::window_frame(w) else {
                continue;
            };
            let (x, y) = frame.client_origin;
            let (cw, ch) = frame.client_size;
            let inside = p.x() >= x
                && p.y() >= y
                && p.x() < x.saturating_add(cw as i32)
                && p.y() < y.saturating_add(ch as i32);
            if !inside {
                continue;
            }
            let area = u64::from(cw) * u64::from(ch);
            if best.is_none_or(|(_, a)| area < a) {
                best = Some((w, area));
            }
        }
        best.map(|(w, _)| w)
    }

    /// **Refuse to aim at a coordinate that is not on the screen.**
    fn confirm_on_the_desktop(&self, p: ScreenPoint) -> Result<()> {
        let (dx, dy, dw, dh) = sys::desktop_bounds();
        if dw <= 0 || dh <= 0 {
            return Ok(());
        }
        if p.x() >= dx && p.y() >= dy && p.x() < dx + dw && p.y() < dy + dh {
            return Ok(());
        }
        let where_window = self.target_client_rect().map_or_else(
            || "the window's placement could not be read".to_owned(),
            |((x, y), (w, h))| {
                format!("the application's window is {w}x{h} px at desktop ({x}, {y})")
            },
        );
        Err(Error::new(format!(
            "the point ({}, {}) is OFF THE DESKTOP, which is {dw}x{dh} px at ({dx}, {dy}). \
             `SetCursorPos` would clamp it to the nearest edge and the click would land on a \
             different control, silently — so it is refused instead. {where_window}. The usual \
             cause is a check asking for a viewport bigger than the space \
             `launch::SAFE_ORIGIN_X` leaves for it: origin + size must fit the screen, or part \
             of the window is at coordinates that do not exist.",
            p.x(),
            p.y()
        )))
    }

    /// The target window's client rectangle as `((x, y), (w, h))` in desktop
    /// pixels, or `None` if it cannot be read.
    fn target_client_rect(&self) -> Option<((i32, i32), (u32, u32))> {
        let frame = sys::window_frame(self.target?).ok()?;
        Some((frame.client_origin, frame.client_size))
    }

    /// [`Self::raise_and_confirm`] for a point that may be inside a dialog.
    fn raise_and_confirm_at(&self, p: ScreenPoint) -> Result<()> {
        let Some(w) = self.window_owning(p) else {
            return self.raise_and_confirm();
        };
        // Remember it: the next keystroke belongs to whatever was last
        // clicked, which is what focus means. See [`Self::focus`].
        self.focus.set(Some(w));
        sys::raise_window(w);
        std::thread::sleep(MOVE_SETTLE);
        //
        // A full sweep of 127 checks reported 45 of them SKIPPED on *"could not
        // be brought to the front"*, and every one of them passed when re-run
        // alone seconds later. The application under test is launched and killed
        // once per check, and Windows' foreground lock does not settle between a
        // process dying and the next one asking — so the first ask after a churn
        // is refused and the second is granted.
        //
        // ⇒ Without this, a suite run back-to-back reports a third of itself as
        // *"could not begin"*, which this harness's own rule calls the failure
        // it exists to remove: a check that did not run has told you nothing,
        // and "told you nothing" rendered as a skip is read as "nothing to see".
        //
        // ONE retry, not a loop, and the sentence below is why: a foreground
        // held by a stray system modal is a real condition that no amount of
        // retrying fixes, and turning it into a slow timeout would hide the one
        // message that names the culprit.
        if !sys::is_foreground(w) {
            std::thread::sleep(MOVE_SETTLE * 4);
            sys::raise_window(w);
            std::thread::sleep(MOVE_SETTLE);
        }
        if !sys::is_foreground(w) {
            return Err(Error::new(format!(
                "the window containing ({}, {}) could not be brought to the front. Windows \
                 refuses SetForegroundWindow to a process without foreground rights, and this \
                 harness is a background process. Reported rather than clicked: a click into a \
                 window that is not in front goes wherever IS, and the check would then report \
                 the feature as broken when nothing was ever pressed at it.\n  \
                 THE FOREGROUND IS HELD BY: {}.\n  \
                 If that is not the application under test and not this harness, it is holding \
                 the desktop and no retry will help — dismiss it and run again. A stray system \
                 modal (an \"Open With\" dialog, the on-screen keyboard) does exactly this.",
                p.x(),
                p.y(),
                sys::describe_foreground()
            )));
        }
        Ok(())
    }

    /// Raise the target and confirm it is actually in front.
    fn raise_and_confirm(&self) -> Result<()> {
        // The window the last pointer action focused, if any, and the
        // application's own window otherwise. A keystroke follows the focus.
        let Some(w) = self.focus.get().or(self.target) else {
            return Err(Error::new(
                "refusing to send input with no target window: it would go to whatever window is in front, which may be the operator's own",
            ));
        };
        self.raise();
        std::thread::sleep(MOVE_SETTLE);
        // The same single retry `raise_and_confirm_at` takes, and for the
        // measurement recorded there: the foreground lock does not settle
        // between one launched-and-killed application and the next, so the
        // first ask after a churn is refused and the second is granted.
        if !sys::is_foreground(w) && !self.application_has_the_foreground() {
            std::thread::sleep(MOVE_SETTLE * 4);
            self.raise();
            std::thread::sleep(MOVE_SETTLE);
        }
        if !sys::is_foreground(w) && !self.application_has_the_foreground() {
            return Err(Error::new(format!(
                "the target window could not be brought to the front, so anything typed now \
                 would go to the operator's own window. Windows refuses SetForegroundWindow to \
                 a process without foreground rights, and this harness is a background process. \
                 Reported rather than typed: sending the keystroke anyway would both corrupt \
                 whatever IS in front and make this check report the feature as broken when \
                 nothing was ever typed at it.\n  \
                 THE FOREGROUND IS HELD BY: {}.\n  \
                 If that is not the application under test and not this harness, it is holding \
                 the desktop and no retry will help — dismiss it and run again. A stray system \
                 modal (an \"Open With\" dialog, the on-screen keyboard) does exactly this.",
                sys::describe_foreground()
            )));
        }
        Ok(())
    }

    /// Bring the focused window — or the application's own — to the front.
    ///
    fn raise(&self) {
        if self.application_has_the_foreground() {
            // LEAVE IT ALONE. Raising here would take focus away from a
            // sibling window of the same application — see
            // [`Self::application_has_the_foreground`].
            return;
        }
        if let Some(w) = self.focus.get().or(self.target) {
            sys::raise_window(w);
        }
    }

    /// **Is the foreground window one of the application's?**
    fn application_has_the_foreground(&self) -> bool {
        let Some(target) = self.target else {
            return false;
        };
        let Some(pid) = sys::pid_of_window(target) else {
            return false;
        };
        sys::foreground_window().and_then(sys::pid_of_window) == Some(pid)
    }

    /// **Refuse to click a point another window is sitting on.**
    fn confirm_uncovered(&self, p: ScreenPoint) -> Result<()> {
        let Some(target) = self.target else {
            return Ok(());
        };
        self.confirm_on_the_desktop(p)?;
        let Some(owner) = sys::window_at(p.x(), p.y()) else {
            return Ok(());
        };
        if owner == target {
            return Ok(());
        }
        if self.window_owning(p) == Some(owner) {
            return Ok(());
        }
        // **"OUTSIDE THE WINDOW" AND "COVERED BY ANOTHER WINDOW" ARE
        // DIFFERENT DIAGNOSES**, and this guard reported both as the second
        // until 2026-08-27.
        //
        // If the point is not within the target's own client rectangle at all,
        // then whatever owns it — the desktop (`Progman`), a File Explorer
        // window, anything — owns it *because nothing of the application is
        // there*. Saying "something is drawn OVER the target" then sends the
        // reader looking for an occluder that does not exist. Measured that
        // day: `dimension_groups_panel_makes_a_group` was blamed on `osk.exe`,
        // then on File Explorer, then on `Progman`, across three runs, and the
        // actual fact was that the panel's **Add** button was published at
        // logical y 824 in an 800 px client — 24 points below the bottom edge.
        //
        // That is not a defect. The panel body is a `ScrollArea::vertical`, so
        // the control is reachable by scrolling and an operator sees the bar.
        // It is a **harness** gap: a rect published from inside a scroll region
        // is a position in the scrolled content, not necessarily a position on
        // screen, and a check that clicks one without scrolling to it first is
        // aiming at somewhere the window is not.
        //
        // ⇒ The message now says which of the two it is, because the remedies
        // have nothing in common: close the offending window, versus scroll the
        // region into view.
        if let Some(frame) = self.target_client_rect() {
            let ((x, y), (w, h)) = frame;
            let inside = p.x() >= x
                && p.y() >= y
                && p.x() < x.saturating_add(w as i32)
                && p.y() < y.saturating_add(h as i32);
            if !inside {
                return Err(Error::new(format!(
                    "the point ({}, {}) is OUTSIDE the application's window, which is {w}x{h} px \
                     at desktop ({x}, {y}). Nothing is covering it; there is simply nothing of \
                     the application there, and the desktop is what owns the pixel. The usual \
                     cause is a rect published from inside a `ScrollArea` — that is a position \
                     in the scrolled CONTENT, not on screen — so scroll the region into view \
                     before clicking it, or make the window taller. Reported rather than \
                     clicked: the click would land on {}.",
                    p.x(),
                    p.y(),
                    sys::describe_window(owner)
                )));
            }
        }
        //
        // `sys::describe_foreground`'s own docs already record the rule, from
        // the day a stray `OpenWith.exe` dialog made nine checks skip: *"a
        // check that reports a refusal without naming the refuser has withheld
        // the only fact that distinguishes 'wait' from 'act'."* It had been
        // applied to the foreground guard and not to this one, which refuses
        // for the same kind of reason.
        Err(Error::new(format!(
            "the point ({}, {}) is owned by {}, not by the application under test, so a click \
             there would go to that window instead. Something is drawn OVER the target. The \
             recorded case on this machine is `osk.exe`, the on-screen keyboard, which \
             synthetic keystrokes summon and which cannot be closed from a process of ordinary \
             integrity — but read the name above before assuming it: a stray dialog left on \
             the desktop behaves identically. Reported rather than clicked: sending the click \
             anyway would make this check report a working feature as broken, which it did \
             repeatedly before this guard existed.",
            p.x(),
            p.y(),
            sys::describe_window(owner)
        )))
    }
}

impl Drop for Driver {
    /// Put the operator's pointer back where it was.
    fn drop(&mut self) {
        if let Some((x, y)) = self.original_cursor {
            let _ = sys::set_cursor_position(x, y);
        }
    }
}

/// The same three operations, through PowerShell.
///
/// Kept for the reasons in the module docs. Not the default; one process per
/// event.
pub struct PowerShellDriver;

impl PowerShellDriver {
    /// Move the pointer and click, via `user32` P/Invokes in PowerShell.
    pub fn click_at(p: ScreenPoint) -> Result<()> {
        let script = format!(
            "Add-Type -Namespace UiVerify -Name U -MemberDefinition '\
             [DllImport(\"user32.dll\")] public static extern bool SetCursorPos(int x,int y);\
             [DllImport(\"user32.dll\")] public static extern void mouse_event(uint f,int x,int y,int d,System.UIntPtr e);'; \
             [UiVerify.U]::SetCursorPos({},{}) | Out-Null; Start-Sleep -Milliseconds 80; \
             [UiVerify.U]::mouse_event(0x0002,0,0,0,[System.UIntPtr]::Zero); \
             Start-Sleep -Milliseconds 60; \
             [UiVerify.U]::mouse_event(0x0004,0,0,0,[System.UIntPtr]::Zero)",
            p.x(),
            p.y()
        );
        run_powershell(&script)
    }

    /// Press and release a virtual key, via `user32` P/Invokes in PowerShell.
    pub fn press(vk: u16) -> Result<()> {
        let script = format!(
            "Add-Type -Namespace UiVerify -Name K -MemberDefinition '\
             [DllImport(\"user32.dll\")] public static extern void keybd_event(byte v,byte s,uint f,System.UIntPtr e);'; \
             [UiVerify.K]::keybd_event({vk},0,0,[System.UIntPtr]::Zero); \
             Start-Sleep -Milliseconds 40; \
             [UiVerify.K]::keybd_event({vk},0,2,[System.UIntPtr]::Zero)"
        );
        run_powershell(&script)
    }
}

fn run_powershell(script: &str) -> Result<()> {
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| Error::new(format!("cannot run powershell: {e}")))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(Error::new(format!(
            "powershell input step failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }
}

/// How long between the two presses of a double click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Extend a selection.
    Shift,
    /// Toggle a member of one.
    Ctrl,
}

impl Key {
    /// The Windows virtual-key code.
    #[must_use]
    pub fn vk(self) -> u16 {
        match self {
            Self::Shift => sys::vk::SHIFT,
            Self::Ctrl => sys::vk::CONTROL,
        }
    }
}
