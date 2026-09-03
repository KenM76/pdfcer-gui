//! Drive the pointer and the keyboard **through the operating system**.
//!
//! # Why OS-level injection, and not the two easier alternatives
//!
//! This is the module's central design decision, and it is required by the
//! defect the harness exists to catch. Three ways to get input into an egui
//! application were available; two of them cannot see D1.
//!
//! ## Rejected: `PostMessage(WM_MOUSEMOVE / WM_LBUTTONDOWN)`
//!
//! Tried first in this project's predecessor, and it **does not work for an
//! off-screen window** — with a silent failure, which is the worst kind. winit
//! calls `TrackMouseEvent` on the move; Windows answers `WM_MOUSELEAVE`
//! because the physical cursor is elsewhere; `egui-winit` then drops the
//! button entirely, because it emits `PointerButton` only when it knows the
//! pointer position. The observed event list was `[PointerMoved, PointerGone]`
//! in **every** message ordering tried, including move and button posted back
//! to back. That finding is recorded in `D:\dev\rag\egui\`, and it is recorded
//! here too so nobody rebuilds it.
//!
//! ## Rejected as the *primary* driver: in-process injection
//!
//! The application already has one — `PDFCER_DIAG_SCRIPT` feeds steps through
//! eframe's `raw_input_hook`. It is excellent, it needs no screen, and it is
//! the right tool for a behavioural question on a machine the operator is
//! using. It is **the wrong oracle for D1**, and precisely because of what it
//! skips.
//!
//! D1's causal chain is:
//!
//! 1. the canvas calls `request_focus()` when `Response::clicked()` fires;
//! 2. `ctx.egui_wants_keyboard_input()` — which means *any widget has focus*,
//!    not *a text field has focus* — therefore returns `true` forever after;
//! 3. so the unmodified-key bindings, `Delete` among them, are never
//!    installed.
//!
//! Every link in that chain is about **what egui's focus machinery does with a
//! real click**. A harness that hands egui a synthetic `PointerButton` event
//! is asserting on the same layer that is broken. It might well reproduce the
//! bug — but a green result from it would not be evidence, because the thing
//! it skipped is the thing in question. The only way to be sure the click that
//! selects the object is the same click that focuses the canvas is to make the
//! window manager deliver it.
//!
//! Put plainly: **the harness that must catch D1 has to go through the OS,
//! because D1 is a defect in how the application responds to the OS.**
//!
//! ## Chosen: `SetCursorPos` + `mouse_event` + `keybd_event`
//!
//! System-level injection. The cursor really moves, the click lands on
//! whatever window is in front, and the keystroke goes to the foreground
//! window. Everything downstream — hit testing, focus, hover, capture — runs
//! exactly as it does for a person.
//!
//! `mouse_event`/`keybd_event` rather than `SendInput`: for a button at the
//! current position they are equivalent, and their signatures have no
//! variable-length array to get wrong. The one thing `SendInput` would buy is
//! atomic multi-event batches, which is not wanted — a real click is not
//! atomic either.
//!
//! ## What that costs, and how it is paid
//!
//! It commandeers the real desktop. Three mitigations, all mechanical:
//!
//! * [`Driver::new`] records the pointer position and [`Driver`]'s [`Drop`]
//!   puts it back, on every path including a panic.
//! * [`Driver::click_at`] and [`Driver::press`] raise the target window first,
//!   and `press` refuses if there is no window — a keystroke sent to the wrong
//!   window is not a failed keystroke, it is a keystroke into the operator's
//!   editor.
//! * Checks are short, and each one holds the desktop for a couple of seconds
//!   rather than a couple of minutes.
//!
//! The harness is honest about this rather than clever: it is a foreground
//! activity, it says so when it starts, and `--no-input` turns it off (whereupon
//! the checks that need it report SKIPPED, never PASS).
//!
//! # The PowerShell fallback
//!
//! [`PowerShellDriver`] does the same three operations by shelling out to
//! `Add-Type`'d `user32` P/Invokes. It exists for two reasons: it is what the
//! predecessor scripts used, so a finding reproduced there can be reproduced
//! here; and it keeps the harness usable if the `windows-sys` binding ever has
//! to be dropped. It is **not** the default — it costs a process per event,
//! which turns a three-event click into three process spawns and makes the
//! timing unlike a real click.

use std::time::Duration;

use crate::coords::ScreenPoint;
use crate::error::{Error, Result};
use crate::sys::{self, WindowHandle};

/// How long the primary button stays down during a synthetic click.
///
/// Long enough that the application sees a press and a release on different
/// frames, which is what a real click looks like. Zero-length clicks have been
/// observed to be coalesced by frameworks that sample input once per frame,
/// and a coalesced click is a click the application never saw.
const CLICK_HOLD: Duration = Duration::from_millis(60);

/// How long to wait after moving the pointer before pressing.
///
/// The application needs at least one frame to process the move and update its
/// hover state; several widgets only respond to a click when they were hovered
/// on the preceding frame.
const MOVE_SETTLE: Duration = Duration::from_millis(80);

/// How long between the RELEASE of one click and the press of the next, in a
/// double click.
///
/// ★ `egui`'s own threshold is **300 ms between PRESSES**, and it is a
/// compiled-in constant rather than the operator's Windows double-click speed —
/// which is the thing a reader assumes and which would make this harness behave
/// differently on a machine where that setting had been changed.
///
/// With `CLICK_HOLD` at 60 ms, 40 ms here puts the presses 100 ms apart:
/// comfortably inside the threshold with room for two slow frames, and slow
/// enough that they land in **different frames**, which they must — `egui`
/// counts clicks, and two presses inside one frame are one press.
const DOUBLE_CLICK_GAP: Duration = Duration::from_millis(40);

/// How many intermediate positions [`Driver::drag`] walks through.
///
/// Enough that the application sees the pointer *travel* rather than teleport —
/// see that method's docs on why a two-point drag can be delivered as a click.
/// Not more, because each step costs [`DRAG_STEP_SETTLE`] and a check that holds
/// the operator's desktop is one that should finish.
const DRAG_STEPS: u32 = 8;

/// How long to pause between the intermediate positions of a drag.
///
/// Shorter than [`MOVE_SETTLE`]: the application does not need to settle at each
/// waypoint, it only needs to *observe* each one, which is one frame at 60 Hz.
/// 25 ms is comfortably more than one frame on any machine that can render a
/// PDF page at all.
const DRAG_STEP_SETTLE: Duration = Duration::from_millis(25);

/// The OS-level input driver.
///
/// Owns the operator's pointer position for its lifetime and returns it on
/// drop.
pub struct Driver {
    original_cursor: Option<(i32, i32)>,
    target: Option<WindowHandle>,
    /// ★★ **The window the last pointer action put the focus in.**
    ///
    /// A keystroke goes to whatever has keyboard focus, and what has keyboard
    /// focus is whatever was last clicked — that is the model on the real
    /// desktop, and until 2026-08-21 this harness could ignore it because the
    /// application had exactly one window.
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
    /// `target` is the window every action is aimed at. It is required for
    /// keystrokes and used to raise before pointer actions.
    #[must_use]
    pub fn new(target: Option<WindowHandle>) -> Self {
        Self {
            original_cursor: sys::cursor_position().ok(),
            target,
            focus: std::cell::Cell::new(None),
        }
    }

    /// Move the pointer and click the primary button.
    ///
    /// The window is raised first: a click on a window that is not in front is
    /// consumed by the click-to-focus of whatever *is*, and the application
    /// under test sees nothing. That failure looks identical to a hit test
    /// returning nothing.
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
    ///
    /// ## ★★★ The first driver for a gesture class this project has shipped
    /// ## since Phase 1
    ///
    /// pdfcer has had canvas context menus for months and **not one driven check
    /// has ever opened one**. Everything asserted about them is a unit test over
    /// `MenuHost::would_open`, which asks whether the *manifest* would offer
    /// something — a real question, and not the same question as *"does a right
    /// -click on this pixel open a menu"*.
    ///
    /// ⇒ R1's own words: *"the tests pass" is not a report of working
    /// software*. A gesture with no driver is a gesture R1 cannot reach, and
    /// the gap left no failing test behind to advertise itself.
    ///
    /// ## ★★ What this deliberately does NOT do
    ///
    /// It does not click a menu **item**. An `egui` popup is positioned by the
    /// pointer and sized by its content, so a harness that aimed at "the second
    /// row" would be encoding a layout, and would silently start clicking the
    /// wrong verb the day a menu grows an entry. The oracle is the application's
    /// own `canvas-menu context=…` trace line, which says which menu it
    /// resolved and how many items it offered — the fact under test, without a
    /// coordinate to go stale.
    ///
    /// ★ Escape is **not** pressed afterwards, deliberately: leaving the popup
    /// open is what lets a following screenshot show it. A check that wants it
    /// closed presses Escape itself, and says why.
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
    ///
    /// # Why the harness had none until now
    ///
    /// It did not need one. Every check that drives the ribbon uses
    /// [`Self::click_at`], and even `markup_rectangle` — whose *subject* is a
    /// drag gesture — asserts on the ribbon arming rather than on the band,
    /// because a markup band is checkable from the command trace. Canvas **text
    /// selection** is the first feature whose entire behaviour is a drag: there
    /// is no button to press that produces one, and a click alone can only ever
    /// clear a selection.
    ///
    /// # The intermediate moves are the whole reason this is not three calls
    ///
    /// egui decides that a press has become a *drag* rather than a *click* by
    /// distance travelled, and it samples the pointer once per frame. A press
    /// followed immediately by a release at a distant point is delivered as a
    /// single jump: egui sees one position, then another, and may report a
    /// **click** at the far end rather than a drag at all — which for this
    /// feature is the difference between selecting a paragraph and clearing the
    /// selection.
    ///
    /// So the pointer is walked in [`DRAG_STEPS`] increments with a settle
    /// between each, which is what a hand does. The application gets several
    /// frames of `dragged_by(Primary)` with a moving position, which is
    /// precisely the sequence `canvas::gesture::GestureState` is written for.
    ///
    /// The button is held across the walk rather than being pressed at each
    /// step: a released-and-pressed pointer is *n* gestures, not one.
    pub fn drag(&self, from: ScreenPoint, to: ScreenPoint) -> Result<()> {
        self.raise_and_confirm()?;
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
    ///
    /// The gesture a **spring-loaded** target needs: press here, walk to
    /// there, *stay* long enough for the application's dwell timer to fire,
    /// then walk on and release. Windows Explorer's folders, every browser's
    /// tabs and pdfcer's document tab strip all work this way, and none of them
    /// can be driven by [`Self::drag`] — which walks straight through and never
    /// rests anywhere.
    ///
    /// `dwell` is how long the pointer sits on `via`. It must exceed the
    /// application's own threshold with room to spare: the check that uses this
    /// passes twice `crate::pdfcer::SPRING_DWELL`, because a dwell measured
    /// against a *frame clock* on a machine that is also rasterizing a CAD
    /// sheet is not a dwell measured against a stopwatch.
    ///
    /// ★ The pointer is **moved slightly** during the dwell rather than being
    /// held perfectly still, in the same place, for a second. A stationary
    /// pointer generates no input, and an application that only repaints on
    /// input would never run the frame its own timer fires on. pdfcer asks for a
    /// repaint while a spring is armed precisely so this is not required — but
    /// a harness that depended on that would be testing the repaint request
    /// rather than the spring, and would report a false failure the day the
    /// request moved.
    ///
    /// # Errors
    ///
    /// As [`Self::drag`].
    /// `modifier` is held down for the **whole** gesture, press to release.
    ///
    /// ★ Which is more than the application strictly needs — pdfcer samples the
    /// drag modifier at the *release*, as Windows does — and it is deliberately
    /// more. Holding it throughout is what an operator's hand actually does,
    /// and it also exercises the frames in between, where the caption has to
    /// follow the key. A harness that pressed the key only at the last instant
    /// would pass against a build whose caption never updated.
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
    ///
    /// Extracted from [`Self::drag`] when [`Self::drag_via`] needed the same
    /// walk twice. The arithmetic is unchanged and the reason for it is
    /// unchanged: integers, because the endpoints are whole pixels and an
    /// intermediate point should be one a real mouse could produce.
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

    /// Click twice in the same place, fast enough for the application to read
    /// it as a double click.
    ///
    /// # ★ Why the gap is a named constant and not a guess
    ///
    /// `egui` decides a double click from the interval between two presses, and
    /// its threshold is a fixed 300 ms — it does **not** read the operator's
    /// Windows double-click speed, which is the thing a reader assumes. A
    /// harness that clicked twice as fast as it could would be relying on
    /// scheduler luck; one that used the OS setting would break on a machine
    /// where the operator has slowed it down. So the gap is chosen against the
    /// framework's own number, with room for a slow frame.
    ///
    /// # Errors
    ///
    /// As [`Self::click_at`].
    pub fn double_click_at(&self, p: ScreenPoint) -> Result<()> {
        // ★★ NOT two `click_at` calls, and the first version of this WAS.
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
    ///
    /// # ★ Why this is not `press_chord` plus `click_at`
    ///
    /// Because the modifier has to be held **across** the mouse press, and
    /// `press_chord` releases it as part of sending a keystroke. The
    /// application reads `modifiers.shift` on the frame it processes the
    /// pointer event, so a Shift that went down and up before the click is a
    /// plain click — which is the failure mode that would make this check
    /// report "the second anchor was not picked" over a perfectly working
    /// build.
    ///
    /// # Errors
    ///
    /// As [`Self::click_at`], and additionally refuses with no target window:
    /// a modifier held over the operator's own desktop is a stuck key.
    pub fn click_with_modifier(&self, p: ScreenPoint, key: Key) -> Result<()> {
        self.raise_and_confirm()?;
        sys::with_modifiers(&[key.vk()], || self.click_at(p))
    }

    /// **Press `vk` `times` times with `modifiers` HELD DOWN throughout**, the
    /// way a hand does it.
    ///
    /// # ★★ Why this exists beside [`Self::press_chord`], which looks identical
    ///
    /// Because they are not identical and the difference is a whole class of
    /// silent failure. `press_chord` posts the modifier down, sleeps, posts the
    /// key, and releases the modifier — **per press**. This holds the modifier
    /// across all of them, so the application sees one modifier transition and
    /// N key presses inside it.
    ///
    /// The reason it was written: `shift_arrows_select_text` sent
    /// `press_chord(&[SHIFT], ARROW_RIGHT)` three times and the application
    /// traced `Modifiers::NONE` on all three. Every arrow arrived; not one of
    /// them carried Shift. The pointer path had never had that problem, and it
    /// uses `with_modifiers` — the modifier held across the whole gesture —
    /// which is what this is.
    ///
    /// ★ The finding is about the toolkit, not about this harness: modifier
    /// state reaches `egui` through winit's `ModifiersChanged`, and a modifier
    /// that goes down and up again inside one frame's event batch can be
    /// applied and undone before the key that was supposed to carry it is
    /// dispatched. Holding it removes the race instead of tuning it — the same
    /// answer this project reached about the fit-zoom loop and about
    /// `ViewportCommand` lag, and for the same reason: **an intermittent is a
    /// defect with a timing dependency, and a sleep is not a fix.**
    ///
    /// # Errors
    ///
    /// If the target window cannot be brought to the front, for the reason
    /// [`Self::press_chord`] refuses.
    pub fn press_held(&self, modifiers: &[u16], vk: u16, times: usize) -> Result<()> {
        self.raise_and_confirm()?;
        sys::with_modifiers(modifiers, || {
            // ★★★ A WHOLE FRAME BEFORE THE FIRST KEY, and this is the fix, not
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

    /// Move the pointer without clicking — for hover assertions, and for
    /// getting the pointer off a widget before a screenshot.
    pub fn move_to(&self, p: ScreenPoint) -> Result<()> {
        sys::set_cursor_position(p.x(), p.y())?;
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Scroll the pane under a point, then settle.
    ///
    /// Moves the pointer there first, because a wheel event goes to whatever is
    /// under the cursor — scrolling "the panel" means putting the pointer in it.
    ///
    /// # ★ Why a check needs this, and what its absence looked like
    ///
    /// A dock panel is a few hundred points tall and a real document's content
    /// is not, so a check that can only reach what is on screen at launch can
    /// only verify the top of any list. Worse, it reports everything below the
    /// fold as *"the control is drawn and inert"* — which is a **confident,
    /// specific, wrong defect report about a control that works**, and this
    /// harness produced three of those in one day before this existed.
    ///
    /// # Errors
    ///
    /// If the pointer cannot be moved.
    pub fn scroll_at(&self, p: ScreenPoint, notches: i32) -> Result<()> {
        self.move_to(p)?;
        sys::wheel(notches);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Roll the wheel at `p` with modifiers held — Ctrl+wheel, which in a
    /// document viewer is **zoom about the pointer**.
    ///
    /// # ★★ Why this is not `scroll_at` with a flag
    ///
    /// It shares `with_modifiers`' whole-frame lead-in, and that lead-in is
    /// load-bearing for the same reason [`Self::press_held`]'s is: a modifier
    /// posted less than a frame before the event it is meant to carry arrives
    /// in the same batch but *after* it, so egui builds the event with
    /// `Modifiers::NONE`. A real hand holds Ctrl for tens of frames first. A
    /// Ctrl+wheel that loses its Ctrl is an ordinary scroll — the view pans
    /// instead of zooming, and the check reports the zoom as broken.
    ///
    /// # ★ Why a check wants this rather than the status bar's `+`
    ///
    /// Zoom-to-cursor keeps the point under the pointer fixed, so a check can
    /// put the pointer on the content it cares about **once** and keep
    /// rolling — the content stays under it all the way down. The `+` button
    /// zooms about the viewport centre, which on a page whose interesting
    /// detail is off-centre magnifies blank paper. The operator's own words,
    /// 2026-08-22: *"Right now you are just zooming into a blank area on the
    /// canvas."*
    pub fn scroll_at_held(
        &self,
        p: ScreenPoint,
        modifiers: &[u16],
        notches: i32,
        times: usize,
    ) -> Result<()> {
        self.raise_and_confirm()?;
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

    /// Press and release a virtual key, in the target window.
    ///
    /// # Errors
    ///
    /// If there is no target window. Refusing is the whole point: keystrokes
    /// go to the foreground window, and if the harness does not know which
    /// window that should be, the keystroke lands in whatever the operator was
    /// typing in. There is no safe default here, so there is no default.
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

    /// Press a **chord** — a virtual key with modifiers held — in the target
    /// window.
    ///
    /// The reason this exists: `Ctrl+F` and every other letter chord in the
    /// manifest keymap were unreachable from this harness, so the checks that
    /// would have driven them could not be written. `press` sends a bare
    /// virtual key with no modifiers, and a shell that binds a command to
    /// `Ctrl+F` cannot be reached by sending `F`.
    ///
    /// # Errors
    ///
    /// If there is no target window, for exactly the reason [`Self::press`]
    /// refuses — and more sharply. A bare keystroke into the operator's editor
    /// types a character. **A chord into the operator's editor runs a
    /// command**, and `Ctrl+W`, `Ctrl+Q` and `Ctrl+S` are all one letter away
    /// from a chord a UI test might plausibly send.
    ///
    /// Modifiers are released by [`sys::key_stroke_with`] on every path; see
    /// its docs for why that is not merely tidy.
    /// **Type an ASCII string, one real keystroke per character.**
    ///
    /// # ★★ Why this refuses rather than skipping what it cannot type
    ///
    /// It handles lowercase letters, uppercase letters (with Shift) and digits,
    /// and returns an error for anything else. The alternative — silently
    /// dropping a character it has no virtual-key code for — would type
    /// `userp` where the caller asked for `userpw`, and the check would then
    /// report the *application* as rejecting a correct password. A harness that
    /// mistypes and blames the program is the worst failure available to it,
    /// and this project has recorded several.
    ///
    /// ★ It is **real keystrokes through the OS**, not a seeded buffer. That is
    /// the whole point: a password field is a focused `TextEdit` behind a real
    /// viewport, and a check that wrote the string into memory would be
    /// asserting about a program nobody can operate.
    ///
    /// # Errors
    ///
    /// If a character has no mapping, or a keystroke cannot be delivered.
    pub fn type_ascii(&self, text: &str) -> Result<()> {
        for ch in text.chars() {
            match ch {
                'a'..='z' => {
                    // VK codes for letters are the ASCII codes of their
                    // UPPERCASE forms; Shift is what distinguishes the case.
                    self.press(ch.to_ascii_uppercase() as u16)?;
                }
                'A'..='Z' => {
                    self.press_chord(&[crate::sys::vk::SHIFT], ch as u16)?;
                }
                '0'..='9' => {
                    self.press(ch as u16)?;
                }
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

    pub fn press_chord(&self, modifiers: &[u16], vk: u16) -> Result<()> {
        self.raise_and_confirm()?;
        sys::key_stroke_with(modifiers, vk);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Raise the target and confirm it is actually in front.
    ///
    /// ★ **`raise()` is a request, not a result.** `SetForegroundWindow` is
    /// refused outright for a process without foreground rights — silently,
    /// via a boolean return nobody is obliged to read — so a window that was
    /// created behind an already-active one can stay behind it through any
    /// number of raise calls. Windows' foreground lock exists precisely to
    /// stop background processes stealing focus, and this harness IS a
    /// background process.
    ///
    /// Without this check the failure is not "the keystroke did not arrive".
    /// It is:
    ///
    /// * the keystroke arriving **in the operator's own window** — and for a
    ///   chord that means running one of their commands, not typing a
    ///   character; and
    /// * the check reporting that the FEATURE is broken, when the truth is
    ///   that nothing was ever typed at it. A false failure naming the wrong
    ///   subsystem is worse than no check, because somebody then goes and
    ///   looks at working code.
    ///
    /// Both were observed: a `find_opens_and_finds` run reported "Ctrl+F did
    /// not dispatch `edit.find`" against a build in which Ctrl+F works.
    /// **Bring the target to the front and PROVE it got there**, or refuse.
    ///
    /// ★★ Every input method uses this, and until 2026-08-20 only the keyboard
    /// ones did. The pointer methods called the fire-and-forget `raise` below,
    /// and the consequence is the reason this doc comment exists:
    ///
    /// > **A click sent to a window that is not in front goes to whatever
    /// > window IS, and the check then reports the feature as broken.**
    ///
    /// Windows refuses `SetForegroundWindow` to a process without foreground
    /// rights, and this harness is a background process — so the raise is a
    /// *request*, not a fact, and whether it is honoured depends on which
    /// process last had focus and on how recently the operator typed. That
    /// makes it **intermittent**, which is the worst available property: it
    /// works while a suite is running and fails on a single check run alone,
    /// or the reverse, and every failure it produces is a confident, specific
    /// accusation against code that is fine.
    ///
    /// Measured on 2026-08-20: `markup_rectangle_arms_from_the_ribbon` and
    /// `insert_image_places_a_picture` both reported the ribbon as unresponsive
    /// — *"the click on `ribbon.tab.markup` produced no `ribbon-tab-activated`"*
    /// — over a build in which the ribbon works, and both had passed in a full
    /// suite an hour earlier. The old build reproduced it too, which is what
    /// ruled the application out.
    ///
    /// The message this returns is deliberately long. Whoever meets it is one
    /// step from diagnosing a feature that was never clicked.
    /// **Which of the application's windows owns this point**, decided by
    /// geometry rather than by z-order.
    ///
    /// # ★★★ Why z-order cannot be the answer
    ///
    /// Because the raise is about to change it. Before 2026-08-21 the
    /// application had one window and this question did not exist; it now has
    /// one per open dialog, and the sequence that broke six checks in a single
    /// suite run is:
    ///
    /// 1. the check aims at a control inside a dialog, correctly;
    /// 2. `click_at` raises **the main window**, because that is the target;
    /// 3. the dialog is not *owned* by the main window — `eframe 0.35` has no
    ///    owner option at all — so it goes **behind** it;
    /// 4. the point is now on the application's own canvas, `window_at`
    ///    agrees it belongs to the target, the cover guard is satisfied, and
    ///    the click lands on a page.
    ///
    /// Every step is individually correct and the result is a check reporting
    /// a working feature as broken. So the window is chosen from the process's
    /// windows by **whose client rectangle contains the point**, which no
    /// raise can change.
    ///
    /// ★ Ties go to the SMALLEST window. A dialog is inside the application's
    /// bounds on screen, so both contain the point; the dialog is the one in
    /// front of the other in every arrangement an operator would produce, and
    /// it is always the smaller. Stated rather than implied because the
    /// alternative — first match — depends on enumeration order, which is
    /// z-order, which is the thing this function exists to not use.
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

    /// The target window's client rectangle as `((x, y), (w, h))` in desktop
    /// pixels, or `None` if it cannot be read.
    ///
    /// Used by [`Self::confirm_uncovered`] to tell *"covered"* from *"off the
    /// window"*, which are different diagnoses with different remedies — see
    /// the argument there.
    fn target_client_rect(&self) -> Option<((i32, i32), (u32, u32))> {
        let frame = sys::window_frame(self.target?).ok()?;
        Some((frame.client_origin, frame.client_size))
    }

    /// [`Self::raise_and_confirm`] for a point that may be inside a dialog.
    ///
    /// Raises whichever of the application's windows actually contains the
    /// point — see [`Self::window_owning`] — and falls back to the target when
    /// the point is on no window of this process, which is the case a check
    /// aiming off-window is entitled to and which the cover guard reports on
    /// its own terms.
    fn raise_and_confirm_at(&self, p: ScreenPoint) -> Result<()> {
        let Some(w) = self.window_owning(p) else {
            return self.raise_and_confirm();
        };
        // Remember it: the next keystroke belongs to whatever was last
        // clicked, which is what focus means. See [`Self::focus`].
        self.focus.set(Some(w));
        sys::raise_window(w);
        std::thread::sleep(MOVE_SETTLE);
        // ★★★ **ONE RETRY, and the whole-suite measurement is why** —
        // 2026-09-01.
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
        // ★ ONE retry, not a loop, and the sentence below is why: a foreground
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

    fn raise_and_confirm(&self) -> Result<()> {
        // ★ The window the last pointer action focused, if any, and the
        // application's own window otherwise. A keystroke follows the focus.
        let Some(w) = self.focus.get().or(self.target) else {
            return Err(Error::new(
                "refusing to send input with no target window: it would go to whatever window is in front, which may be the operator's own",
            ));
        };
        self.raise();
        std::thread::sleep(MOVE_SETTLE);
        // ★ The same single retry `raise_and_confirm_at` takes, and for the
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
    /// ★ `focus` first, and that ordering is the whole of the 2026-08-21 fix:
    /// a keystroke belongs to whatever the last pointer action focused, which
    /// since dialogs became real OS windows is frequently not the application's
    /// main window. Raising the main window here takes focus AWAY from the
    /// dialog a check just clicked into, and the characters land on the page.
    fn raise(&self) {
        if self.application_has_the_foreground() {
            // ★★★ LEAVE IT ALONE. Raising here would take focus away from a
            // sibling window of the same application — see
            // [`Self::application_has_the_foreground`].
            return;
        }
        if let Some(w) = self.focus.get().or(self.target) {
            sys::raise_window(w);
        }
    }

    /// **Is the foreground window one of the application's?**
    ///
    /// # ★★★ Why a harness must not raise when the answer is yes
    ///
    /// Because the application now opens windows *of its own accord*, and a
    /// window it opened has the foreground without anybody having clicked it.
    /// A text-annotation dialog appears in answer to a drag on the canvas and
    /// takes the keyboard immediately — which is the behaviour under test, and
    /// which the check verifies by typing WITHOUT clicking the field, *"the way
    /// an operator does"*.
    ///
    /// A `press` that raises the main window first destroys exactly that: the
    /// dialog loses focus, the characters land on the page, and the check
    /// reports that the dialog ignored the keyboard. The feature is fine; the
    /// harness broke it and then measured it.
    ///
    /// So the rule is **do not steal focus from the application** — only
    /// reclaim it from something else. It is the same idea as the cover guard's
    /// correction on the same day: the question is whose PROCESS owns what is
    /// in front, not which handle.
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
    ///
    /// ★★★ The lesson of 2026-08-20, and it cost an afternoon.
    ///
    /// `SetForegroundWindow` succeeding means the target has **focus**. It says
    /// nothing about what is **drawn over it** — an always-on-top window sits
    /// above a focused one and swallows every click aimed at the region it
    /// covers. The click is delivered, to something else, and the check reports
    /// the feature as broken.
    ///
    /// The culprit here was `osk.exe`, the Windows on-screen keyboard, lying
    /// across the ribbon's tab row. It is summoned by synthetic keystrokes, so
    /// **this harness brings it on itself**, and it cannot be closed from a
    /// process of ordinary integrity — `taskkill`, `CloseMainWindow` and
    /// `ShowWindow(SW_HIDE)` were all refused by UIPI.
    ///
    /// The symptoms were the worst available: intermittent, and confidently
    /// wrong. `markup_rectangle_arms_from_the_ribbon` and
    /// `insert_image_places_a_picture` reported the ribbon as unresponsive over
    /// a build in which it works; both had passed in a full suite an hour
    /// before; the pre-multi-document build reproduced it, which is what ruled
    /// the application out. The oracle that settled it was a **screenshot** —
    /// `D:/dev/rag/egui/` is unambiguous that a reachability defect has exactly
    /// one — and the on-screen keyboard was plainly visible in it.
    ///
    /// So: ask who owns the point, and if it is not the target, say so and
    /// stop. An error becomes a SKIP, which is *"this did not run"*, rather
    /// than a FAIL, which is an accusation.
    fn confirm_uncovered(&self, p: ScreenPoint) -> Result<()> {
        let Some(target) = self.target else {
            return Ok(());
        };
        let Some(owner) = sys::window_at(p.x(), p.y()) else {
            return Ok(());
        };
        if owner == target {
            return Ok(());
        }
        // ★ A DIALOG OF THE SAME APPLICATION IS NOT A COVER. As of 2026-08-21
        // the application has one window per open dialog, and a click aimed
        // into one legitimately lands on a window that is not the target. The
        // guard is about a FOREIGN window — `osk.exe` is the recorded case —
        // so the question it should have been asking all along is *whose
        // process owns what is on top*, not *which handle*.
        if self.window_owning(p) == Some(owner) {
            return Ok(());
        }
        // ★★★ **"OUTSIDE THE WINDOW" AND "COVERED BY ANOTHER WINDOW" ARE
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
        // ★★ **Name the window**, added 2026-08-27. The message used to say
        // only that the point belonged to "another window" and then guess that
        // it was `osk.exe` — and on the day this was written the on-screen
        // keyboard was not running, which left the SKIP unactionable.
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
///
/// A modifier key this harness can hold across a mouse gesture.
///
/// An enum rather than a bare `u16` virtual-key code, because the whole point
/// of this type is that a caller cannot accidentally hold something that is not
/// a modifier — a mouse gesture with `A` held is not a gesture any application
/// defines, and it would arrive as a stray keystroke.
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
