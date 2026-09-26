//! `dialogs::host::placement` — WHERE a dialog's window opens.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/dialoghost/placement.md`.

use egui::{Pos2, Rect, Vec2};

/// How far in from the application window a dialog opens when its caller
/// expressed no preference.
pub(crate) const OPEN_INSET_PT: f32 = 48.0;

/// How far below the application window's top edge a **chosen** opening
/// position may be honoured, in points.
pub(crate) const CHROME_RESERVE_PTS: f32 = 180.0;

/// Everything the placement arithmetic knows about the application window,
/// read once from the live viewport by `Host::show`.
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
pub(crate) fn opening(app: AppWindow, size: Vec2, preferred: Option<Pos2>) -> Pos2 {
    match preferred {
        None => app.outer.min + Vec2::splat(OPEN_INSET_PT),
        Some(at) => onto_window(app.inner.min + (at - app.screen_min), size, app.outer),
    }
}

/// Pull `desired` back until a dialog of `size` sits wholly on `parent`, and
/// below its chrome.
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
    #[test]
    fn a_dialog_with_no_preference_opens_inset_from_the_application_window() {
        let at = opening(app(), Vec2::new(420.0, 240.0), None);
        assert_eq!(at, Pos2::new(1920.0 + OPEN_INSET_PT, 100.0 + OPEN_INSET_PT));
    }

    /// **A chosen position reaches the desktop, measured from the CLIENT
    /// area.**
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

    /// **On a window too short to hold the dialog below its chrome, the
    /// chrome still wins**, and the clamp does not panic.
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
