//! # `find::reveal` — bringing a hit onto the screen
//!
//! The second half of what "go to this hit" means. The first half is a page
//! change, which [`super::apply`] performs directly through
//! [`crate::viewer::ViewState::go_to_page`]; this module is everything after
//! that — the pending intent, the gate that decides when to spend it, the
//! scroll solve, and the projection from PDF geometry into the space the
//! canvas paints in.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/find/reveal.md`.

use egui::{Rect, Vec2};

use crate::app::state::OpenDoc;
use crate::canvas::geometry;
use crate::find::FindState;
use crate::viewer::FitMode;

/// How many frames a pending [`Reveal`] waits for its page to arrive before
/// it is abandoned.
const REVEAL_GRACE_FRAMES: u8 = 4;

/// A hit that has been navigated to and is waiting to be scrolled into view.
///
/// Lives on [`OpenDoc`] beside `zoom_anchor`, and for the identical reason:
/// **it has to span two frames.** The request is made during the apply phase,
/// which is after the canvas has drawn; the page change it carries is applied
/// in the same phase; so the earliest frame on which the target page's real
/// drawn size is known is the next one. Recording the intent and solving it
/// later is the same handshake `ZoomAnchor` documents, minus the zoom.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reveal {
    /// The page the hit is on. The offset is solved only on a frame that is
    /// actually showing this page.
    pub page: usize,
    /// The hit's centre as a fraction of the page's extent.
    ///
    /// A **fraction**, not a canvas point, for exactly the reason
    /// [`crate::app::state::ZoomAnchor`] carries one: it is independent of
    /// the zoom, so it can be recorded before the frame that will spend it
    /// and stays correct if the operator zooms in between.
    pub frac: (f32, f32),
    /// How many frames this has waited for its page. See
    /// [`REVEAL_GRACE_FRAMES`].
    pub waited: u8,
}

/// Navigate to the current hit: the page, then the scroll position.
///
/// Two halves, and both are needed. `go_to_page` is called directly rather
/// than raised as [`crate::app::actions::Action::GoToPage`] because this
/// **is** the apply phase — the funnel's rule is that no widget mutates a
/// document, not that the apply phase may not — and it is the same method
/// that action's arm calls, so the clamp has one owner either way.
pub(super) fn reveal_current(state: &FindState, doc: &mut OpenDoc) {
    let Some(hit) = state.current_hit(doc.edit_epoch) else {
        return;
    };
    let page = hit.page;
    let centre = hit.canvas.map(|r| r.center());
    // The return value is deliberately discarded here. It exists so the tests
    // can see the third guard, which changes nothing a test could otherwise
    // assert on — see the function's own note.
    let _held = hold_the_zoom_if_asked(state, doc, page);
    doc.view.go_to_page(page, doc.pages.len());

    // THE OPTION GOVERNS THE POSITION TOO - `OPERATOR_REQUESTS.md` O179.
    //
    // The *Zoom* control has two halves and this is the second of them.
    // `hold_the_zoom_if_asked` is the ZOOM half: it drops a standing fit so
    // `apply_fit` stops re-scaling. The CENTRING is a wholly separate
    // mechanism - `doc.find_reveal`, spent by `canvas::offset`'s reveal
    // branch - so arming it unconditionally slides the page under the
    // operator on every hit even with the option OFF. His words:
    // *“instead of … leaving the page in its current position on the canvas
    // it still zooms and repositions the page”*.
    //
    // Declining here also closes a zoom hole the zoom half cannot reach,
    // which is why the option OFF can still be seen zooming even though
    // nothing in this module calls `set_zoom`. Under a continuous display
    // mode `view.page_index` is DERIVED from the scroll
    // (`canvas::strip::track_current_page`), so the reveal's own centring
    // scroll can carry `page_index` onto a differently-sized sheet on the
    // next frame - at which point `apply_fit` re-scales to that sheet.
    // `hold_the_zoom_if_asked` cannot see that coming: its third guard
    // returns early when the target IS the current page. Not arming the
    // scroll removes the cause rather than adding a fourth guard.
    //
    // What survives, deliberately: the page change above (that is the whole
    // point of a find), the highlight (`FindState::page_highlights` feeds
    // the overlay independently of `find_reveal`), and the minimum
    // page-change scroll that `canvas::strip::page_scroll_offset` performs
    // under a continuous mode - the next branch down the scroll-priority
    // chain, which brings the page into view without centring anything.
    // Single-page mode scrolls not at all. That is his
    // *“jumping to the page … and leaving the page in its current position”*
    // exactly.
    if !state.zoom_on_jump() {
        doc.find_reveal = None;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("find-reveal page={page} declined=zoom-off")
        });
        return;
    }

    // A hit whose page would not project has no centre to scroll to. The page
    // change still happened, which is the half of the answer that is
    // available, and scrolling to a guess would be worse than leaving the
    // view where the page change put it.
    let Some(centre) = centre else {
        doc.find_reveal = None;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("find-reveal page={page} declined=no-geometry")
        });
        return;
    };

    let extent = doc
        .pages
        .get(page)
        .map_or((0.0, 0.0), crate::viewer::page_extent_pts);
    // `frac_of` — `canvas::zoom`'s own conversion, not a second one. It
    // divides by the page EXTENT rather than by its drawn size, which is
    // exactly what makes the value independent of the zoom and therefore
    // recordable now and spendable on a later frame.
    let frac = crate::canvas::zoom::frac_of(centre, extent);
    doc.find_reveal = Some(Reveal {
        page,
        frac,
        waited: 0,
    });
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-reveal page={page} frac=({:.4},{:.4})",
            frac.0, frac.1
        )
    });
}

/// **Hold the zoom still across a find jump, when the operator has asked
/// for that** — the *Zoom* control, `OPERATOR_REQUESTS.md` **O163**.
fn hold_the_zoom_if_asked(state: &FindState, doc: &mut OpenDoc, target: usize) -> bool {
    if state.zoom_on_jump() || target == doc.view.page_index || doc.view.fit == FitMode::None {
        return false;
    }
    let held = doc.view.zoom;
    // A stable token, not `{:?}`. A `Debug` rendering is a spelling the
    // compiler is free to change when a variant is renamed, and this line is
    // read by a machine: a check that matches on a rendering the compiler owns
    // goes on passing while the string it quotes has stopped meaning what the
    // check was written to assert.
    let dropped = match doc.view.fit {
        // ui-text-exempt: diagnostic trace field values, never displayed
        FitMode::None => "none",
        FitMode::Page => "page",
        FitMode::Width => "width",
        FitMode::Height => "height",
    };
    doc.view.set_fit(FitMode::None);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-zoom-held zoom={held:.4} was-fit={dropped} from={} to={target}",
            doc.view.page_index
        )
    });
    true
}

/// **The scroll offset that puts a pending reveal in the middle of the
/// viewport**, or `None` to leave the scroll area alone.
///
/// Called from [`crate::canvas::offset`]'s scroll-priority chain, once per
/// frame while a reveal is pending: below the fit and the zoom anchor, which
/// are explicit instructions about the view, and above the plain page-change
/// scroll, which would otherwise satisfy the page change without centring the
/// hit.
///
/// # It reuses the anchoring solve rather than writing a second one
///
/// [`crate::canvas::geometry::offset_holding_anchor_at`] is the function
/// `canvas::zoom::place_centred` uses to express *"put this page point at
/// this screen position"*, and it is the single owner of that arithmetic.
/// Asking it for `target = viewport centre` is exactly what a framing zoom
/// asks for; the only difference is that this one does not change the zoom,
/// so it cannot ride `ZoomAnchor`'s handshake — that handshake is gated on
/// the page's **drawn size changing**, which is precisely what a scroll does
/// not do. Hence a separate pending value and a separate gate, and a shared
/// solve.
///
/// # Why the gate is the page index rather than a frame count alone
///
/// The reveal is spent on the first frame that is *showing the hit's page*.
/// Spending it earlier would scroll the outgoing page to a fraction that
/// means nothing on it; spending it on a frame count would be a guess about
/// how long a page change takes. The frame count is the *abandon* rule, not
/// the spend rule — see [`REVEAL_GRACE_FRAMES`].
pub fn take_reveal_offset(
    doc: &mut OpenDoc,
    display: (f32, f32),
    viewport: (f32, f32),
) -> Option<Vec2> {
    let reveal = doc.find_reveal?;
    if reveal.page != doc.view.page_index {
        if reveal.waited >= REVEAL_GRACE_FRAMES {
            doc.find_reveal = None;
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "find-reveal-dropped page={} showing={}",
                    reveal.page, doc.view.page_index
                )
            });
        } else {
            doc.find_reveal = Some(Reveal {
                waited: reveal.waited + 1,
                ..reveal
            });
        }
        return None;
    }

    doc.find_reveal = None;
    let (x, y) = geometry::offset_holding_anchor_at(
        reveal.frac,
        (viewport.0 / 2.0, viewport.1 / 2.0),
        display,
        viewport,
    );
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-reveal-solved page={} offset=({x:.1},{y:.1})",
            reveal.page
        )
    });
    Some(Vec2::new(x, y))
}

/// Project a core [`pdfcer_core::annot_author::Quad`] — **unrotated PDF user
/// space, Y-up** — into a canvas-space rectangle.
///
/// All four corners are mapped and bounded, rather than mapping two: the page
/// transform may rotate (`/Rotate 90` is ordinary on a landscape drawing
/// sheet), and a rotation sends the quad's `ul`/`lr` pair to two corners that
/// are no longer the extremes. Bounding all four is correct under every
/// rotation and costs three extra multiplications.
///
/// `None` when the page's device transform will not invert, which is
/// [`crate::viewer::pdf_space_to_canvas`]'s own decline for a degenerate
/// page. The hit is still counted and still navigable — see [`Hit::canvas`].
///
/// `pub(crate)` rather than `pub(super)` so that canvas text selection can
/// project its line boxes through **this** function rather than mapping two
/// corners of its own. That is a correctness requirement rather than
/// tidiness: a selected word and the same word *found* are two washes over
/// the same glyphs, and on a `/Rotate 90` sheet a two-corner projection puts
/// one of them somewhere else. One projection, two surfaces — the same discipline
/// `canvas::mapping` applies to the screen⟷canvas hop.
pub(crate) fn quad_to_canvas(
    quad: &pdfcer_core::annot_author::Quad,
    page: &pdfcer_core::page_tree::Page,
) -> Option<Rect> {
    let corners = [quad.ul, quad.ur, quad.ll, quad.lr];
    let mut bounds: Option<Rect> = None;
    for (x, y) in corners {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "page coordinates are bounded by the media box; f32 is the canvas's own precision" // ui-text-exempt: clippy lint justification, never displayed
        )]
        let point = egui::pos2(x as f32, y as f32);
        let canvas = crate::viewer::pdf_space_to_canvas(point, page)?;
        bounds = Some(match bounds {
            None => Rect::from_min_max(canvas, canvas),
            Some(r) => r.union(Rect::from_min_max(canvas, canvas)),
        });
    }
    bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, open_fixture};

    // =======================================================================
    // Projecting a quad
    // =======================================================================

    /// **A hit's quad projects into canvas space, once, at search time.**
    #[test]
    fn a_quad_projects_into_canvas_space_with_the_y_axis_flipped() {
        use pdfcer_core::annot_author::Quad;

        let doc = open_fixture(FOUR_PAGES);
        let page = doc.pages.first().expect("the fixture has pages");
        let (_, height) = crate::viewer::page_extent_pts(page);

        // A band near the TOP of the page in PDF terms: high Y.
        let high = Quad {
            ul: (10.0, f64::from(height) - 20.0),
            ur: (60.0, f64::from(height) - 20.0),
            ll: (10.0, f64::from(height) - 30.0),
            lr: (60.0, f64::from(height) - 30.0),
        };
        let rect = quad_to_canvas(&high, page).expect("a real page projects");
        assert!(
            rect.min.y < height / 2.0,
            "a high PDF Y must become a low canvas Y: {rect:?} on a {height} pt page"
        );
        assert!(rect.width() > 0.0 && rect.height() > 0.0);
        // …and the box is where it was horizontally, since this page is
        // unrotated.
        assert!((rect.min.x - 10.0).abs() < 1.0, "{rect:?}");
    }

    // =======================================================================
    // The two-frame handshake
    // =======================================================================

    /// **A reveal waits for its page and is then spent once.**
    ///
    /// The two-frame handshake. Spending it before the page change lands
    /// would scroll the outgoing page to a fraction that means nothing on it.
    #[test]
    fn a_reveal_waits_for_its_page_then_solves_once() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.find_reveal = Some(Reveal {
            page: 2,
            frac: (0.5, 0.5),
            waited: 0,
        });
        let display = (600.0, 800.0);
        let viewport = (400.0, 400.0);

        // The page change has not been applied yet.
        assert_eq!(doc.view.page_index, 0);
        assert!(take_reveal_offset(&mut doc, display, viewport).is_none());
        assert!(doc.find_reveal.is_some(), "it is for a later frame");

        // The page arrives.
        doc.view.go_to_page(2, doc.pages.len());
        let offset = take_reveal_offset(&mut doc, display, viewport).expect("the page is showing");
        assert!(offset.x.is_finite() && offset.y.is_finite());
        assert!(
            doc.find_reveal.is_none(),
            "a reveal is spent once, or it fires again on the next layout change"
        );
        assert!(take_reveal_offset(&mut doc, display, viewport).is_none());
    }

    /// **A reveal whose page never arrives is abandoned.**
    #[test]
    fn a_reveal_whose_page_never_arrives_is_abandoned() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.find_reveal = Some(Reveal {
            page: 3,
            frac: (0.5, 0.5),
            waited: 0,
        });
        for _ in 0..=REVEAL_GRACE_FRAMES {
            assert!(take_reveal_offset(&mut doc, (600.0, 800.0), (400.0, 400.0)).is_none());
        }
        assert!(
            doc.find_reveal.is_none(),
            "a reveal that cannot be spent must be dropped, not held"
        );
    }

    // =======================================================================
    // Holding the zoom across a jump — O163
    // =======================================================================

    /// The viewport every test in this section fits into. Any size would do;
    /// it is fixed so the two arms of a comparison see the same one.
    const VIEWPORT: (f32, f32) = (900.0, 700.0);

    /// Land on `page` **the way the canvas does**: change the page, then apply
    /// whatever fit is still switched on against *that page's* extent.
    fn land_on(doc: &mut OpenDoc, page: usize) {
        doc.view.go_to_page(page, doc.pages.len());
        let extent = doc
            .pages
            .get(page)
            .map_or((0.0, 0.0), crate::viewer::page_extent_pts);
        doc.view
            .apply_fit(extent, VIEWPORT, crate::viewer::MAX_ZOOM);
    }

    /// The local four-page fixture, already settled on page 1 under `fit`,
    /// with a find state whose *Zoom* control is at `zoom_on_jump`.
    fn settled(fit: FitMode, zoom_on_jump: bool) -> (FindState, OpenDoc) {
        let mut doc = crate::app::state::open_local_fixture("four-pages.pdf");
        doc.view.set_fit(fit);
        land_on(&mut doc, 0);
        let mut state = FindState::default();
        state.set_zoom_on_jump(zoom_on_jump);
        (state, doc)
    }

    /// **The control OFF holds the zoom across a jump between differently
    /// sized sheets** — the whole of O163, asserted as the operator sees it.
    #[test]
    fn the_control_off_holds_the_zoom_across_a_jump() {
        let (state, mut doc) = settled(FitMode::Page, false);
        let before = doc.view.zoom;
        assert!(hold_the_zoom_if_asked(&state, &mut doc, 3), "it must act");
        land_on(&mut doc, 3);
        assert_eq!(
            doc.view.zoom, before,
            "the operator asked for the zoom to be left alone"
        );
        assert_eq!(doc.view.page_index, 3, "the page still changed");
        assert_eq!(
            doc.view.fit,
            FitMode::None,
            "holding the zoom means the view stopped following the fit, and the readout must say so"
        );
    }

    /// **The control ON lets the fit re-scale** — the shipped default, and the
    /// arm that proves the test above is measuring something.
    ///
    /// Without this, `the_control_off_holds_the_zoom_across_a_jump` would pass
    /// on a build where the fixture happened not to re-fit at all.
    #[test]
    fn the_control_on_lets_the_fit_re_scale_as_it_always_did() {
        let (state, mut doc) = settled(FitMode::Page, true);
        let before = doc.view.zoom;
        assert!(
            !hold_the_zoom_if_asked(&state, &mut doc, 3),
            "the first guard must decline"
        );
        land_on(&mut doc, 3);
        assert!(
            doc.view.zoom > before * 2.0,
            "page 1 is 2384 pt wide and page 4 is 306 pt, so Fit page must re-scale: {before} -> {}",
            doc.view.zoom
        );
        assert_eq!(
            doc.view.fit,
            FitMode::Page,
            "nothing was held, so nothing may change the fit"
        );
    }

    /// **Stepping between two hits on the same sheet does not drop the
    /// fit** — the second guard.
    #[test]
    fn a_step_within_one_page_leaves_the_fit_alone() {
        let (state, mut doc) = settled(FitMode::Page, false);
        let before = doc.view.zoom;
        assert!(
            !hold_the_zoom_if_asked(&state, &mut doc, 0),
            "the second guard must decline"
        );
        assert_eq!(doc.view.fit, FitMode::Page, "the page is not changing");
        assert_eq!(doc.view.zoom, before);
    }

    /// **With no fit active there is nothing to hold, and nothing is
    /// touched** — the third guard.
    #[test]
    fn no_fit_active_means_no_intervention() {
        let (state, mut doc) = settled(FitMode::None, false);
        let before = doc.view.zoom;
        // The whole of this test. Every other assertion below is true
        // whether the guard is present or not — measured — so without this
        // line the guard could be deleted with the suite still green.
        assert!(
            !hold_the_zoom_if_asked(&state, &mut doc, 3),
            "with no fit active there is nothing to hold, and a build that reported an \
             intervention would put a `find-zoom-held` line on the trace that never happened"
        );
        land_on(&mut doc, 3);
        assert_eq!(doc.view.fit, FitMode::None);
        assert_eq!(
            doc.view.zoom, before,
            "a pinned zoom is already pinned, on any page"
        );
    }

    /// **Fit width is held too, not only Fit page.**
    #[test]
    fn fit_width_is_held_as_well_as_fit_page() {
        let (state, mut doc) = settled(FitMode::Width, false);
        let before = doc.view.zoom;
        assert!(hold_the_zoom_if_asked(&state, &mut doc, 3), "it must act");
        land_on(&mut doc, 3);
        assert_eq!(doc.view.zoom, before);
        assert_eq!(doc.view.fit, FitMode::None);
    }

    /// **The shipped default is ON**, so an operator who never opens the
    /// options menu keeps the fit-following behaviour.
    #[test]
    fn the_shipped_default_leaves_the_zoom_free() {
        assert!(FindState::default().zoom_on_jump());
    }

    /// The solve centres the hit: a hit at the middle of a page bigger than
    /// the viewport lands at the middle of the viewport.
    #[test]
    fn the_solve_puts_the_hit_in_the_middle_of_the_viewport() {
        let mut doc = open_fixture(FOUR_PAGES);
        let frac = (0.5_f32, 0.5_f32);
        doc.find_reveal = Some(Reveal {
            page: 0,
            frac,
            waited: 0,
        });
        let display = (1200.0, 1600.0);
        let viewport = (400.0, 400.0);
        let offset =
            take_reveal_offset(&mut doc, display, viewport).expect("page 0 is already showing");

        let landed = geometry::anchor_screen_pos(frac, (offset.x, offset.y), display, viewport);
        assert!(
            (landed.0 - viewport.0 / 2.0).abs() < 0.01
                && (landed.1 - viewport.1 / 2.0).abs() < 0.01,
            "the hit landed at {landed:?}, not the viewport centre"
        );
    }
}
