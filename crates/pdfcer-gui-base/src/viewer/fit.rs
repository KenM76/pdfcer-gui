//! # `viewer::fit` — how the zoom is decided from the viewport, and where the view goes
//!
//!
//! ## The seam
//!
//! [`super`]'s subject is *the view state* — what is shown, at what scale, in
//! what arrangement, and every rule about the zoom LADDER and its ceilings.
//! This file's is the narrower question **"what does the window tell the zoom
//! to be, and where does the page then sit?"**, and the two change for
//! different reasons: a new ceiling or a new rung touches the parent, a new
//! fitting mode touches this.
//!
//! ## Why one file holds both the scale and the placement
//!
//! Because O28 proved they are one decision. [`fit_scale`] answers *how big*
//! and [`FitMode::pinned_axes`] answers *where*, and the second is derived
//! entirely from the first: an axis is pinned exactly when the fit has just
//! decided its extent. Separating them would put two `match`es over one enum
//! in two files, free to disagree — and
//! [`tests::a_pinned_axis_is_one_the_scale_makes_fill_the_viewport`] exists
//! precisely because they *could*, and asserts that they do not.
//!
//! That relationship is also why the placement lives here rather than in
//! `canvas::geometry`, which owns the arithmetic that *uses* it: the canvas
//! asks "which axes did the fit decide?" and does not need to know what a
//! fitting mode is to act on the answer.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/viewer/fit.md`.

/// How `ViewState::zoom` is being decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FitMode {
    /// The operator pinned an explicit zoom; the viewport no longer
    /// influences it.
    None,
    /// Recompute each frame so the whole page is visible.
    #[default]
    Page,
    /// Recompute each frame so the page's full width is visible.
    Width,
    /// Recompute each frame so the page's full height is visible.
    ///
    /// `OPERATOR_REQUESTS.md` O29 — *"Adobe has fit height, so add that
    /// too."* On a landscape drawing sheet in a portrait window it is the
    /// useful one: fit-page leaves the sheet a band across the middle with
    /// empty space above and below, and fit-width makes it taller than the
    /// screen. Fit-height puts both long edges on screen at the largest scale
    /// that keeps them there.
    Height,
}

impl FitMode {
    /// **Which axes this mode pins, and therefore which the view must be
    /// placed on** — `(horizontal, vertical)`, or `None` for a mode that
    /// places nothing.
    #[must_use]
    pub fn pinned_axes(self) -> Option<(bool, bool)> {
        match self {
            Self::None => None,
            Self::Page => Some((true, true)),
            Self::Width => Some((true, false)),
            Self::Height => Some((false, true)),
        }
    }
}

/// The scale at which `page_pts` fits `viewport` under `fit`.
#[must_use]
pub fn fit_scale(page_pts: (f32, f32), viewport: (f32, f32), fit: FitMode) -> f32 {
    let (pw, ph) = page_pts;
    let (vw, vh) = viewport;
    if pw <= 0.0 || ph <= 0.0 || vw <= 0.0 || vh <= 0.0 {
        return 1.0;
    }
    match fit {
        FitMode::None => 1.0,
        FitMode::Width => vw / pw,
        // The exact mirror of `Width`, and deliberately NOT clamped against
        // the other axis: a fit that quietly refused to overflow the width
        // would be fit-page under a second name, and fit-page already exists.
        // Overflowing horizontally is what the operator is asking for — the
        // page scrolls sideways and both long edges of a landscape sheet are
        // on screen at once.
        FitMode::Height => vh / ph,
        // Fit-page is the *smaller* of the two ratios: satisfying the
        // tighter constraint necessarily satisfies the looser one, and
        // taking the larger would overflow the other axis.
        FitMode::Page => (vw / pw).min(vh / ph),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewer::{MAX_ZOOM, ViewState};

    // ---- fit scale -----------------------------------------------

    #[test]
    fn fit_width_uses_the_width_ratio_only() {
        // A tall page in a wide, short viewport: fit-width overflows
        // vertically on purpose (that is what scrolling is for).
        assert_eq!(
            fit_scale((100.0, 400.0), (300.0, 200.0), FitMode::Width),
            3.0
        );
    }

    /// O29's mirror of the test above, and it asserts the OVERFLOW as well as
    /// the ratio.
    #[test]
    fn fit_height_uses_the_height_ratio_only_and_lets_the_width_overflow() {
        // A wide page in a narrow, tall viewport: height ratio 3.0, width 0.5.
        let scale = fit_scale((400.0, 100.0), (200.0, 300.0), FitMode::Height);
        assert_eq!(
            scale, 3.0,
            "fit-height is the height ratio and nothing else"
        );
        assert!(
            400.0 * scale > 200.0,
            "the page must overflow horizontally; a fit-height that clamped to the width would be fit-page wearing another name"
        );
    }

    /// The three fitting modes pin the axes they fit, and actual size pins
    /// none — the table `canvas::show` places the view from. O28.
    #[test]
    fn each_fit_pins_exactly_the_axes_it_decides() {
        assert_eq!(
            FitMode::None.pinned_axes(),
            None,
            "actual size places nothing"
        );
        assert_eq!(FitMode::Page.pinned_axes(), Some((true, true)));
        assert_eq!(FitMode::Width.pinned_axes(), Some((true, false)));
        assert_eq!(FitMode::Height.pinned_axes(), Some((false, true)));
    }

    /// A pinned axis really is one whose extent the fit decided.
    #[test]
    fn a_pinned_axis_is_one_the_scale_makes_fill_the_viewport() {
        let page = (400.0_f32, 100.0_f32);
        let viewport = (200.0_f32, 300.0_f32);
        for mode in [FitMode::Page, FitMode::Width, FitMode::Height] {
            let scale = fit_scale(page, viewport, mode);
            let (pin_x, pin_y) = mode.pinned_axes().expect("a fitting mode pins something");
            if pin_x {
                assert!(
                    page.0 * scale <= viewport.0 + 1e-3,
                    "{mode:?} pins the horizontal, so the page cannot overflow it"
                );
            }
            if pin_y {
                assert!(
                    page.1 * scale <= viewport.1 + 1e-3,
                    "{mode:?} pins the vertical, so the page cannot overflow it"
                );
            }
        }
    }

    #[test]
    fn fit_page_takes_the_tighter_of_the_two_constraints() {
        // width ratio 3.0, height ratio 0.5 -> 0.5, so the whole page
        // fits.
        assert_eq!(
            fit_scale((100.0, 400.0), (300.0, 200.0), FitMode::Page),
            0.5
        );
        // And symmetrically when height is the loose axis.
        assert_eq!(
            fit_scale((400.0, 100.0), (200.0, 300.0), FitMode::Page),
            0.5
        );
    }

    #[test]
    fn fit_page_result_never_overflows_either_axis() {
        // The property, checked over a spread of shapes rather than one
        // hand-picked case.
        for &(pw, ph) in &[(612.0, 792.0), (792.0, 612.0), (1.0, 5000.0), (5000.0, 1.0)] {
            for &(vw, vh) in &[(800.0, 600.0), (300.0, 1200.0), (50.0, 50.0)] {
                let s = fit_scale((pw, ph), (vw, vh), FitMode::Page);
                assert!(pw * s <= vw * 1.001);
                assert!(ph * s <= vh * 1.001);
            }
        }
    }

    #[test]
    fn degenerate_geometry_falls_back_to_actual_size() {
        assert_eq!(fit_scale((0.0, 100.0), (300.0, 300.0), FitMode::Page), 1.0);
        assert_eq!(fit_scale((100.0, 100.0), (0.0, 300.0), FitMode::Width), 1.0);
        assert_eq!(fit_scale((100.0, 100.0), (300.0, -1.0), FitMode::Page), 1.0);
    }

    #[test]
    fn fit_mode_survives_a_viewport_change_but_an_explicit_zoom_does_not() {
        // "Fit page" is a mode, not a one-shot: resizing the window
        // re-fits. Pinning a zoom ends that.
        let mut v = ViewState::default();
        v.set_fit(FitMode::Page);
        v.apply_fit((100.0, 100.0), (200.0, 200.0), MAX_ZOOM);
        assert_eq!(v.zoom, 2.0);
        v.apply_fit((100.0, 100.0), (400.0, 400.0), MAX_ZOOM);
        assert_eq!(v.zoom, 4.0);

        v.set_zoom(1.0, MAX_ZOOM);
        assert_eq!(v.fit, FitMode::None);
        v.apply_fit((100.0, 100.0), (800.0, 800.0), MAX_ZOOM);
        assert_eq!(v.zoom, 1.0);
    }

    #[test]
    fn zooming_by_a_factor_leaves_fit_mode() {
        let mut v = ViewState::default();
        v.set_fit(FitMode::Width);
        v.zoom_by(1.1, MAX_ZOOM);
        assert_eq!(v.fit, FitMode::None);
    }
}
