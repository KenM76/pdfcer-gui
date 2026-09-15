//! # `app::actions::destination` — **arriving where a bookmark points, not
//! merely on its page**
//!
//!
//! > *"in Acrobat clicking on the nested bookmarks in the drawing package takes
//! > you to a zoomed in area of the page for the drawing bookmark that was
//! > clicked on. when we click on ours it just jumps us to the correct page,
//! > but doesn't send us to the spot on the page the bookmark actually points
//! > to."*
//!
//! ## Why the view is carried and not just the page
//!
//! `outline::Destination::Page` carries **both**:
//!
//! ```text
//! Page { page_index, view: DestView }
//! ```
//!
//! A navigator that matches `page_index` and discards `view` reduces an entire
//! outline to a page list. On a drawing package, where every bookmark names a
//! **detail** on a shared sheet with `/XYZ` or `/FitR`, several bookmarks
//! pointing at different details of one sheet then all arrive at the same
//! place — indistinguishable from them being broken. So everything that
//! navigates to a destination comes through [`actions_for`], and nothing
//! pattern-matches a `Destination::Page` for its page number alone.
//!
//! ## The views this translates, and what each one means here
//!
//! | `DestView` | §12.3.2.2 | what this does |
//! |---|---|---|
//! | `Xyz { left, top, zoom }` | a corner and a magnification | put that point at the top-left; honour `zoom` when it is given |
//! | `Fit` | fit the whole page | fit the page |
//! | `FitH { top }` | fit the width, `top` at the top edge | fit width, then scroll so `top` is at the top |
//! | `FitV { left }` | fit the height | fit the page, then scroll so `left` is at the left |
//! | `FitR { rect }` | fit a rectangle | frame that rectangle — the same act the zoom marquee performs |
//!
//! `DestView` is `#[non_exhaustive]` and holds more than these — the `/FitB`
//! family, an unrecognised entry, no destination at all. Each of those gets
//! the page turn and nothing more; see the fallback arm of [`actions_for`].
//!
//! ## A `Point` is scrolled to, and it is the ONLY thing that sets the
//! ## magnification here
//!
//! Every row above that produces a `Point` produces a *position* and nothing
//! else. The magnification, if the destination asked for one, comes from the
//! `Action::Fit(..)` or `Action::ZoomTo(..)` raised beside it, one step earlier
//! in the same list — and because actions are applied in order, the scroll is
//! solved against the size that produced. `canvas::destscroll` does not read
//! the zoom at all.
//!
//! That ordering is the repair of `DEFECTS.md` D47: a `Point` used to be grown
//! into a 150 pt square and handed to the framing solver, which raised its own
//! `ZoomTo` and discarded whatever the fit had just decided. A `/FitH` arrived
//! at whatever magnification 150 pt of paper requires — on a large sheet,
//! several hundred percent — and a Word table-of-contents link, `/XYZ x y
//! null`, arrived magnified despite explicitly declining to name a zoom.
//!
//! **`null` is not zero.** Table 151 lets `left`, `top` and `zoom` each be
//! null, meaning *"leave this one as it is"* — and the standard states the
//! `0`-means-null equivalence **only for `zoom`**, never for the coordinates.
//! So a literal `0` left edge is a real left edge and must be honoured as one,
//! while a `zoom` of `0` means *"keep the current magnification"*. Collapsing
//! those is how a destination at the top-left corner of a page silently becomes
//! "no change".
//!
//! ## What this does NOT do
//!
//! It does not clamp a destination into view. A bookmark pointing off the sheet
//! is a bookmark pointing off the sheet — the engine's own census counts
//! dangling ones — and quietly landing somewhere plausible would hide a
//! document defect the operator may need to fix.

use pdfcer_core::outline::DestView;

use crate::canvas::destination::PendingDestination;
use crate::canvas::destination::PendingDestination::Point;

use super::Action;

/// Turn a resolved destination into the actions that arrive at it.
///
/// The page step is always first and always unconditional: every view is
/// relative to a page, and a view applied before the page turn would frame a
/// region of the wrong sheet. `GoToPage` is idempotent, so a destination on the
/// current page costs nothing.
///
/// Returns actions rather than performing the move, because this is called
/// from a panel body — `panels::bookmarks` states the rule its own header
/// carries: *"it changes no document at all"*, and framing a view is a change
/// to `OpenDoc::view` that belongs in the apply phase with every other.
pub fn actions_for(page_index: usize, view: &DestView, out: &mut Vec<Action>) {
    out.push(Action::GoToPage(page_index));
    match view {
        // Nothing beyond the page. `Fit` is the honest no-op here only because
        // the shell's own fit is what an operator gets by default; a document
        // that asked for `Fit` and a shell that fits are already agreed.
        DestView::Fit => out.push(Action::Fit(crate::viewer::FitMode::Page)),
        DestView::Xyz { left, top, zoom } => {
            // `zoom` first, because the scroll is expressed in the zoom that
            // will be in force when it lands. Reversing them scrolls to a point
            // and then magnifies about a different anchor, which puts the
            // destination off screen by however much the zoom changed.
            if let Some(z) = zoom.filter(|z| *z > 0.0) {
                // `/XYZ`'s zoom is a MAGNIFICATION FACTOR — 1.0 is actual
                // size — and `Action::ZoomTo` takes the same units as an f32.
                // The cast is the only conversion; nothing is scaled by 100
                // here, because a percentage would be a second unit for one
                // quantity and this shell already has one.
                out.push(Action::ZoomTo(z as f32));
            }
            out.push(Action::GoToDestination(Point {
                page: page_index,
                left: *left,
                top: *top,
            }));
        }
        DestView::FitH { top } => {
            out.push(Action::Fit(crate::viewer::FitMode::Width));
            out.push(Action::GoToDestination(Point {
                page: page_index,
                left: None,
                top: *top,
            }));
        }
        DestView::FitV { left } => {
            // `Page`, not a height fit. `/FitV` asks for the page's HEIGHT
            // to fill the window; this arm fits the whole page instead, which
            // is never wrong in the sense of cropping but is wider than the
            // destination asked for. `crate::viewer::FitMode::Height` is the
            // faithful translation and this arm does not yet reach for it.
            out.push(Action::Fit(crate::viewer::FitMode::Page));
            out.push(Action::GoToDestination(Point {
                page: page_index,
                left: *left,
                top: None,
            }));
        }
        // The one that matters most on a drawing package: a rectangle
        // around a detail. Framed by the same code the zoom marquee uses, so a
        // bookmark and a rubber band over the same region arrive identically —
        // which is what makes the result predictable rather than merely close.
        //
        // All four edges are `Option`, and a rectangle missing one has no
        // area to frame. Falling back to the page is the honest answer: it
        // arrives somewhere true rather than framing a region invented from
        // three edges and a guess.
        DestView::FitR {
            left,
            bottom,
            right,
            top,
        } => match (left, bottom, right, top) {
            (Some(l), Some(b), Some(r), Some(t)) => {
                out.push(Action::GoToDestination(PendingDestination::Rect {
                    page: page_index,
                    rect: (*l, *b, *r, *t),
                }));
            }
            _ => out.push(Action::Fit(crate::viewer::FitMode::Page)),
        },
        // `DestView` is `#[non_exhaustive]`. A view this build has never seen
        // gets the page and nothing else — which is exactly the behaviour every
        // bookmark had before this module existed, and is the honest floor: it
        // arrives somewhere true rather than somewhere invented.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(view: &DestView) -> Vec<&'static str> {
        let mut out = Vec::new();
        actions_for(3, view, &mut out);
        out.iter()
            .map(|a| match a {
                Action::GoToPage(_) => "page",
                Action::Fit(crate::viewer::FitMode::Page) => "fit-page",
                Action::Fit(crate::viewer::FitMode::Width) => "fit-width",
                Action::Fit(_) => "fit-other",
                Action::GoToDestination(PendingDestination::Rect { .. }) => "fit-rect",
                Action::ZoomTo(_) => "zoom",
                Action::GoToDestination(PendingDestination::Point { .. }) => "scroll",
                _ => "other",
            })
            .collect()
    }

    /// **Every view turns the page first.**
    ///
    /// A view is relative to a page, so one applied before the turn frames a
    /// region of the wrong sheet — and on a drawing package, where consecutive
    /// bookmarks point at details of *different* sheets, that lands the
    /// operator on a plausible-looking wrong detail.
    #[test]
    fn every_view_turns_the_page_before_it_frames_anything() {
        for view in [
            DestView::Fit,
            DestView::Xyz {
                left: Some(10.0),
                top: Some(20.0),
                zoom: Some(2.0),
            },
            DestView::FitH { top: Some(5.0) },
            DestView::FitV { left: Some(5.0) },
            DestView::FitR {
                left: Some(0.0),
                bottom: Some(0.0),
                right: Some(100.0),
                top: Some(100.0),
            },
        ] {
            assert_eq!(kinds(&view).first().copied(), Some("page"), "{view:?}");
        }
    }

    /// **Zoom is applied before the scroll**, or the scroll lands in the
    /// wrong magnification and the destination is off screen by the difference.
    #[test]
    fn xyz_zooms_before_it_scrolls() {
        let k = kinds(&DestView::Xyz {
            left: Some(10.0),
            top: Some(20.0),
            zoom: Some(2.0),
        });
        let z = k.iter().position(|s| *s == "zoom");
        let s = k.iter().position(|s| *s == "scroll");
        assert!(z < s, "{k:?}");
    }

    /// **A zoom of `0` means "keep the current magnification"** — Table 151
    /// states that equivalence for `zoom` and for nothing else.
    ///
    /// The mirror of this test is the one that cannot be written here and is
    /// stated instead: a `left` of `0.0` is a REAL left edge and must reach the
    /// scroll. Collapsing the two conventions is how a destination at a page's
    /// top-left corner silently becomes "no change".
    #[test]
    fn a_zero_zoom_is_not_a_zoom() {
        let k = kinds(&DestView::Xyz {
            left: Some(0.0),
            top: Some(0.0),
            zoom: Some(0.0),
        });
        assert!(
            !k.contains(&"zoom"),
            "a zero zoom must not be applied: {k:?}"
        );
        assert!(
            k.contains(&"scroll"),
            "a zero LEFT is a real coordinate: {k:?}"
        );
    }

    /// **D47 / O200.** A `/XYZ` whose zoom is null — the shape every Word
    /// table-of-contents link has — asks for a position and for no
    /// magnification whatsoever. The actions must say so: a page turn and a
    /// scroll, nothing else. If a `fit` or a `zoom` ever appears here, the
    /// operator's view is being changed by a destination that declined to
    /// change it.
    #[test]
    fn a_null_zoom_xyz_asks_for_a_position_and_nothing_else() {
        let k = kinds(&DestView::Xyz {
            left: Some(72.0),
            top: Some(720.0),
            zoom: None,
        });
        assert_eq!(k, vec!["page", "scroll"]);
    }

    /// **D47.** A one-axis fit names exactly one magnification, and it is the
    /// fit's. The scroll that follows carries the position only — it used to
    /// be widened into a 150 pt square and reframed, which threw the fit's
    /// answer away one frame after it was applied.
    #[test]
    fn a_one_axis_fit_names_exactly_one_magnification() {
        assert_eq!(
            kinds(&DestView::FitH { top: Some(540.0) }),
            vec!["page", "fit-width", "scroll"]
        );
        assert_eq!(
            kinds(&DestView::FitV { left: Some(72.0) }),
            vec!["page", "fit-page", "scroll"]
        );
    }

    /// A rectangle destination frames the rectangle, and does not also try to
    /// fit or zoom — two framings compete and the second wins for no reason a
    /// reader could predict.
    #[test]
    fn fitr_frames_once() {
        let k = kinds(&DestView::FitR {
            left: Some(10.0),
            bottom: Some(10.0),
            right: Some(200.0),
            top: Some(100.0),
        });
        assert_eq!(k, vec!["page", "fit-rect"]);
    }
}
