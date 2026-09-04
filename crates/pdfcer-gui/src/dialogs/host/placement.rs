//! `dialogs::host::placement` — WHERE a dialog's window opens.
//!
//! # Why this is its own file
//!
//! The same seam `print/layout.rs` was split on, and stated in the same terms:
//! **geometry is a different subject from the transaction.** `host.rs` owns
//! what a dialog window *is* — the viewport, the ownership, the focus window,
//! the fit budget, the button pair. This file owns one question and no others:
//! given the application window's rectangles and a dialog's size, what desktop
//! point should the window be created at?
//!
//! It is a `mod placement;` declared inside `host.rs`, so it lives at
//! `dialogs/host/placement.rs` and needs no entry in `dialogs/mod.rs` — which
//! is at 1,496 lines and has no room for one. R2 is satisfied by splitting on a
//! real seam rather than by shaving comments off the file that grew.
//!
//! # ★★★ The defect this file was written for: A16c, 2026-09-03
//!
//! An outside review found the sticky-note dialog *"opens at the window
//! origin"*. It did, and the cause was two lines in two files that each looked
//! finished:
//!
//! - `dialogs/textannot.rs` computed a considered opening position — centred
//!   across the application window, a third of the way down — and then wrote
//!   `let _ = pos;`, with a comment saying the computation *"is retired with the
//!   `egui::Window` it fed"*;
//! - `dialogs/host.rs` placed every dialog with no remembered position at a flat
//!   [`OPEN_INSET_PT`] from the application window's top-left corner, because
//!   that was the only placement any caller had ever asked for.
//!
//! Neither half is wrong on its own. Together they are a dialog that discards a
//! position it computed and opens in the corner instead — **every single
//! time**, because a note dialog is opened and dismissed dozens of times in a
//! markup session and therefore almost never has a remembered position to
//! restore. The half-second of hunting for it is small; it is paid on every
//! note.
//!
//! ★ The review's wording was *"a click-relative position"*, and that is not
//! what the discarded computation was. It is centred horizontally and a third
//! of the way down the application window — the same placement the Set-scale
//! dialog uses — and it is deliberately **not** over the annotation: the
//! dialog's own header records that *"an operator writing a callout is usually
//! looking at the thing they are calling out, and a window pinned over it would
//! make them close the window to read what they were annotating."* A genuinely
//! click-relative dialog would be a regression against that stance, and would
//! additionally need the canvas's page-to-screen transform, which this half of
//! the program does not own. What was restored is the position that was
//! computed, not a different one.
//!
//! # ★★ The rule this file holds
//!
//! > **A chosen opening position is clamped onto the application window, and
//! > nothing about the dialog's own content participates in the arithmetic.**
//!
//! Both halves matter.
//!
//! The clamp is what makes a caller-supplied position safe to honour at all. A
//! caller computes in the application window's own coordinates, which say
//! nothing about how big the desktop is or where on it the application sits; a
//! position that is sensible in that space can still put half a dialog past the
//! right-hand edge of the monitor. Clamping onto the **parent window** is the
//! monitor-agnostic way to say "on screen": the application window is on a
//! monitor, so anything inside it is too, and neither `ViewportInfo::monitor_size`
//! (a size with no origin, useless on a second monitor) nor a work-area query
//! (which `eframe 0.35` does not expose) is needed.
//!
//! The second half is R128's rule, the one this project has been bitten by
//! three times — see `print/layout.rs`'s header and `Host::fit`'s. Every number
//! below comes from the **parent window's rectangle**, the dialog's **declared**
//! size and **constants**. Nothing is measured from a laid-out body, so no
//! quantity here can be its own cause.

use egui::{Pos2, Rect, Vec2};

/// How far in from the application window a dialog opens when its caller
/// expressed no preference.
///
/// Not centred on the parent, and not at the OS's own default. Centring puts a
/// dialog exactly over the thing it is asking about, which is the one place it
/// must not be for a *print* dialog whose preview the operator is comparing
/// against the page behind it. A small inset reads as "this belongs to that
/// window" without covering its middle.
pub(crate) const OPEN_INSET_PT: f32 = 48.0;

/// How far below the application window's top edge a **chosen** opening
/// position may be honoured, in points.
///
/// # ★★ What it protects, and why it is generous
///
/// The application's own chrome lives in that band: the native title bar, the
/// quick-access strip, the ribbon's tab row and the two-row band beneath it.
/// A dialog that opens over it hides the control that opened it — which is not
/// a hypothetical, it is written down as a defect the Settings window already
/// met: *"egui's own default position put it top-left, over the quick-access
/// toolbar and the ribbon tabs — so opening Settings hid the control that
/// opened it."*
///
/// 180 pt is a deliberate over-estimate of that band, summed from the parts:
/// a native title bar (~32), the quick-access strip (~30), the tab row (~28)
/// and a two-row band, which `mockups/ribbon.html` specifies at
/// `min-height:86px` and which this shell renders from the theme's own metrics
/// so it varies with the preset.
///
/// ★★ It is a **constant and not a measurement**, and that is the decision
/// rather than a shortcut. The real band height is `ribbon::band::band_height`,
/// which needs a live `Ui` and changes with the theme preset — so reading it
/// would make where a dialog opens depend on how tall the ribbon laid itself
/// out this frame, which is precisely the class of feedback `Host::fit`'s
/// doc comment records an unbounded growth loop for. Over-estimating costs a
/// dialog that opens a few points lower than it asked for; under-estimating
/// costs the operator the control they just pressed.
///
/// ★ In the common case this floor is never reached. The note dialog's chosen
/// position on an 800 pt window is roughly 290 pt down, and only a window
/// squeezed to a few hundred points brings the two into contact.
pub(crate) const CHROME_RESERVE_PTS: f32 = 180.0;

/// Everything the placement arithmetic knows about the application window,
/// read once from the live viewport by `Host::show`.
///
/// ★ A struct rather than three loose arguments, because the three are only
/// ever meaningful together and two of them are rectangles in **different
/// coordinate spaces** — a pair that is very easy to swap at a call site and
/// impossible to notice afterwards, since both are plausible-looking numbers
/// that differ by the width of a window border.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AppWindow {
    /// The application window's **outer** rectangle — chrome included — in
    /// desktop points.
    ///
    /// This is the space `ViewportBuilder::with_position` speaks, so it is what
    /// the clamp is expressed against and what the inset default is measured
    /// from.
    pub outer: Rect,
    /// The application window's **client** rectangle, in desktop points.
    ///
    /// The origin of this rectangle is where the application's own egui
    /// coordinate space begins on the desktop, which is what turns a
    /// caller-supplied position into a desktop one.
    pub inner: Rect,
    /// The origin of the application's own egui screen space.
    ///
    /// Zero in practice for a root viewport, and carried anyway rather than
    /// assumed: `content_rect` — which is what callers compute against —
    /// subtracts safe-area insets, so the two origins are not the same quantity
    /// even when they hold the same numbers today.
    pub screen_min: Pos2,
}

/// **Read the application window's geometry** out of the live viewport, or
/// `None` when the platform has not reported it.
///
/// ★ All three rectangles are read in **one** `input` closure. Reading them
/// separately would take three locks and — much worse — could straddle a frame
/// boundary during a resize, producing an outer rect from before the drag and a
/// client rect from after it. The clamp would then be computed against a window
/// that never existed.
///
/// `outer_rect` is the one that decides: without it there is no desktop
/// coordinate to place against at all, and the caller lets the platform choose.
/// `inner_rect` is documented as `None` on exactly the same platforms (Android,
/// Wayland), so it falls back to the outer rect rather than refusing — a
/// placement off by the width of a window border is still enormously better
/// than the corner.
pub(crate) fn app_window(ctx: &egui::Context) -> Option<AppWindow> {
    let (outer, inner, screen_min) = ctx.input(|i| {
        (
            i.viewport().outer_rect,
            i.viewport().inner_rect,
            i.viewport_rect().min,
        )
    });
    let outer = outer?;
    Some(AppWindow {
        outer,
        inner: inner.unwrap_or(outer),
        screen_min,
    })
}

/// **Where a dialog with no remembered position should open**, in the desktop
/// coordinates `ViewportBuilder::with_position` takes.
///
/// # The two cases, and why only one of them clamps
///
/// * `preferred == None` — the thirteen dialogs that never asked. They get
///   [`OPEN_INSET_PT`] from the application window's corner, **exactly as
///   before and deliberately unclamped**. Clamping them would change where
///   every one of them opens for the sake of a degenerate case none of them can
///   reach: a fixed 48 pt inset is inside any window big enough to have raised
///   the dialog, and driven checks aim at those windows.
/// * `preferred == Some(at)` — a caller that computed a position in the
///   application's own screen coordinates. It is converted to the desktop and
///   then clamped by [`onto_window`], because a caller's arithmetic knows the
///   application window's size and nothing about the desktop's.
///
/// ★ The conversion is `inner.min + (at - screen_min)` and not
/// `outer.min + at`. The application's egui coordinates start at its **client**
/// area, so measuring from the outer corner would slide every chosen position
/// down and right by the window's decoration — about thirty points on Windows,
/// which is exactly the size of the discrepancy nobody notices and everybody
/// blames on something else.
///
/// ★★ The result positions the dialog's **outer** corner while the caller was
/// thinking about its content, so the dialog lands one title bar higher than
/// the caller's arithmetic imagined. That is accepted rather than corrected: a
/// child window's decoration height is not knowable before the window exists,
/// and this is a "roughly here" placement whose whole job is to not be the
/// corner.
pub(crate) fn opening(app: AppWindow, size: Vec2, preferred: Option<Pos2>) -> Pos2 {
    match preferred {
        None => app.outer.min + Vec2::splat(OPEN_INSET_PT),
        Some(at) => onto_window(app.inner.min + (at - app.screen_min), size, app.outer),
    }
}

/// Pull `desired` back until a dialog of `size` sits wholly on `parent`, and
/// below its chrome.
///
/// # The order the bounds are applied, which is the whole of the decision
///
/// Horizontally there is nothing to choose: the window is pushed left until its
/// right edge is on the parent, and never past the parent's left edge.
///
/// Vertically the two bounds can **conflict**, and which one wins is a
/// judgement rather than arithmetic. On a parent window shorter than
/// [`CHROME_RESERVE_PTS`] plus the dialog, there is no position that both
/// clears the chrome and keeps the bottom edge on the window. This function
/// clears the chrome and lets the bottom overhang, for one reason: the chrome
/// holds the control the operator just pressed, and a dialog covering it is a
/// dialog they cannot get back behind. An overhanging bottom edge costs a
/// scroll or a drag; a hidden ribbon costs the way out.
///
/// ★ Both `max` calls exist to keep the clamp total. `Pos2::clamp` panics on an
/// inverted range, and an inverted range here is not a programming error — it
/// is the ordinary consequence of a dialog larger than the window that raised
/// it, which every one of these dialogs can be on a window dragged small.
fn onto_window(desired: Pos2, size: Vec2, parent: Rect) -> Pos2 {
    let left = parent.left();
    let right = (parent.right() - size.x).max(left);
    let top = parent.top() + CHROME_RESERVE_PTS;
    let bottom = (parent.bottom() - size.y).max(top);
    Pos2::new(desired.x.clamp(left, right), desired.y.clamp(top, bottom))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1,200 x 900 application window whose client area is inset by a border
    /// and a title bar, sitting on a second monitor so that a test which
    /// silently assumed a desktop origin of zero would fail.
    fn app() -> AppWindow {
        AppWindow {
            outer: Rect::from_min_size(Pos2::new(1920.0, 100.0), Vec2::new(1200.0, 900.0)),
            inner: Rect::from_min_size(Pos2::new(1928.0, 140.0), Vec2::new(1184.0, 852.0)),
            screen_min: Pos2::ZERO,
        }
    }

    /// **A dialog that expressed no preference is placed exactly as it was
    /// before**, which is what makes this change safe for the other thirteen.
    ///
    /// The inset path is the one every other dialog in the program takes and
    /// the one the driven checks were written against. If it moved, this file
    /// would have fixed one dialog's opening position by changing thirteen.
    #[test]
    fn a_dialog_with_no_preference_opens_inset_from_the_application_window() {
        let at = opening(app(), Vec2::new(420.0, 240.0), None);
        assert_eq!(at, Pos2::new(1920.0 + OPEN_INSET_PT, 100.0 + OPEN_INSET_PT));
    }

    /// ★★ **A chosen position reaches the desktop, measured from the CLIENT
    /// area.**
    ///
    /// The regression test for A16c itself. A caller computed 390, 290 in the
    /// application's own coordinates; the dialog must open there and not in the
    /// corner. The client origin is used rather than the outer one, so a
    /// version of `opening` that measured from `outer` — off by the border and
    /// the title bar, which is the mistake that looks right — fails here.
    #[test]
    fn a_chosen_position_is_carried_into_desktop_coordinates() {
        let at = opening(
            app(),
            Vec2::new(420.0, 240.0),
            Some(Pos2::new(390.0, 290.0)),
        );
        assert_eq!(
            at,
            Pos2::new(1928.0 + 390.0, 140.0 + 290.0),
            "a chosen position must be measured from the application's CLIENT origin"
        );
        assert_ne!(
            at,
            opening(app(), Vec2::new(420.0, 240.0), None),
            "a dialog that chose a position must not land where one that did not would"
        );
    }

    /// **A chosen position that would hang off the window is pulled back onto
    /// it**, on both axes.
    ///
    /// The application window is on a monitor, so a dialog wholly inside the
    /// application window is wholly on that monitor — which is the whole reason
    /// the clamp is expressed against the parent rather than against a monitor
    /// size that carries no origin.
    #[test]
    fn a_chosen_position_is_pulled_back_onto_the_application_window() {
        let size = Vec2::new(420.0, 240.0);
        let app = app();
        let at = opening(app, size, Some(Pos2::new(5_000.0, 5_000.0)));
        assert!(
            at.x + size.x <= app.outer.right() + f32::EPSILON,
            "the right edge ({}) ran past the window's ({})",
            at.x + size.x,
            app.outer.right()
        );
        assert!(
            at.y + size.y <= app.outer.bottom() + f32::EPSILON,
            "the bottom edge ({}) ran past the window's ({})",
            at.y + size.y,
            app.outer.bottom()
        );
        assert!(at.x >= app.outer.left() && at.y >= app.outer.top());
    }

    /// **A chosen position never covers the ribbon**, however far up it asks to
    /// go.
    ///
    /// A dialog over the ribbon hides the control that raised it — the defect
    /// the Settings window already met and recorded. Nothing in a caller's
    /// arithmetic knows where the chrome ends, so the floor is applied here.
    #[test]
    fn a_chosen_position_never_covers_the_ribbon() {
        let app = app();
        let at = opening(app, Vec2::new(420.0, 240.0), Some(Pos2::new(100.0, 0.0)));
        assert!(
            at.y >= app.outer.top() + CHROME_RESERVE_PTS,
            "opened {} pt from the window's top, inside the {CHROME_RESERVE_PTS} pt \
             the ribbon and the tab strip occupy",
            at.y - app.outer.top()
        );
    }

    /// ★ **On a window too short to hold the dialog below its chrome, the
    /// chrome still wins**, and the clamp does not panic.
    ///
    /// The conflicting case named in [`onto_window`]'s doc comment. It is not a
    /// programming error — a window dragged down to a few hundred points
    /// reaches it with any of these dialogs — so the arithmetic has to have an
    /// answer rather than an assertion.
    #[test]
    fn a_window_too_short_for_the_dialog_still_keeps_the_ribbon_clear() {
        let short = AppWindow {
            outer: Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(600.0, 300.0)),
            inner: Rect::from_min_size(Pos2::new(8.0, 40.0), Vec2::new(584.0, 252.0)),
            screen_min: Pos2::ZERO,
        };
        let at = opening(short, Vec2::new(420.0, 240.0), Some(Pos2::new(10.0, 10.0)));
        assert!(
            (at.y - CHROME_RESERVE_PTS).abs() < f32::EPSILON,
            "the chrome floor must win the conflict; got {}",
            at.y
        );
    }
}
