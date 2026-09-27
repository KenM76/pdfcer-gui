//! # `canvas::guides` — draggable alignment lines, whose home is a page and whose life is a file
//!
//! The third of `RIBBON_IA.md` §5.2's *"Rulers · Grid · Guides"*, and the one
//! with a condition on it: a guide must be **draggable**, and it must
//! **survive a reopen**, which takes a per-document store. This header answers
//! both, plus the two questions they imply: *what does a guide belong to*, and
//! *where does it live on disk*.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/guides.md`.

use egui::{Context, Id, Pos2, Rect, Sense, Stroke, Ui};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::rulers::{CanvasGeometry, Gutters};
use crate::canvas::strip::PageView;

pub use pdfcer_gui_base::guidestore::{
    CAP, GUIDES_FILE, default_path, opening, recall, recall_at, remember, remember_at,
};

/// The alpha, out of 255, of a placed guide.
const GUIDE_ALPHA: u8 = 170;

/// The alpha of a guide preview that would be **discarded** on release.
const DISCARD_ALPHA: u8 = 60;

/// `egui::Memory` key for the in-flight guide drag.
const DRAG_KEY: &str = "pdfcer-canvas-guide-drag"; // ui-text-exempt: internal memory id, never displayed

/// `egui::Id` base for the guides' catch bands.
const BAND_KEY: &str = "pdfcer-canvas-guide-band"; // ui-text-exempt: internal widget id, never displayed

// ---------------------------------------------------------------------------
// The model
// ---------------------------------------------------------------------------

pub use pdfcer_gui_base::guidemodel::GuideAxis;
pub use pdfcer_gui_base::guidemodel::MAX_PER_DOCUMENT;
#[cfg(test)]
use {egui::pos2, pdfcer_gui_base::guidemodel::CATCH_PTS};

pub use pdfcer_gui_base::guidemodel::Guide;

pub use pdfcer_gui_base::guidemodel::Guides;

// ---------------------------------------------------------------------------
// The drag
// ---------------------------------------------------------------------------

/// A guide drag in flight.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Drag {
    /// Which way the line being dragged runs.
    axis: GuideAxis,
    /// The guide being moved, as `(page, index into the collection)`, or
    /// `None` when one is being created from a ruler.
    ///
    /// The index is the identity — see [`Guides::on_page`] — and the page
    /// rides with it so a move that lands on a *different* page can remove the
    /// old entry and add the new one without searching for it.
    moving: Option<(usize, usize)>,
}

/// Plant a drag in flight, for tests in sibling modules.
#[cfg(test)]
pub(super) fn plant_drag_for_test(ctx: &Context) {
    store(
        ctx,
        Some(Drag {
            axis: GuideAxis::Horizontal,
            moving: None,
        }),
    );
}

/// Read the in-flight guide drag.
fn load(ctx: &Context) -> Option<Drag> {
    ctx.data(|d| d.get_temp::<Drag>(Id::new(DRAG_KEY)))
}

/// Write the in-flight guide drag, or clear it.
fn store(ctx: &Context, drag: Option<Drag>) {
    let id = Id::new(DRAG_KEY);
    ctx.data_mut(|d| match drag {
        Some(drag) => {
            d.insert_temp(id, drag);
        }
        None => d.remove::<Drag>(id),
    });
}

/// **Abandon a guide drag in flight.** Returns whether there was one.
pub(super) fn cancel_drag(ctx: &Context) -> bool {
    if load(ctx).is_none() {
        return false;
    }
    store(ctx, None);
    true
}

/// Where the pointer is, and which page it is over — the two facts every
/// resolution needs.
fn pointer_page(ctx: &Context, geometry: &CanvasGeometry) -> Option<(usize, PageMapping, Pos2)> {
    let p = ctx.pointer_latest_pos()?;
    if !geometry.viewport.contains(p) {
        return None;
    }
    let (page, map) = geometry.page_at(p)?;
    Some((page, map, p))
}

/// Finish an in-flight drag, if the pointer has come up.
fn release(ctx: &Context, doc: &OpenDoc, geometry: &CanvasGeometry, actions: &mut Vec<Action>) {
    let Some(drag) = load(ctx) else {
        return;
    };
    if !ctx.input(|i| i.pointer.any_released()) {
        return;
    }
    store(ctx, None);

    let landed = pointer_page(ctx, geometry);
    let mut next = doc.guides.clone();
    match (drag.moving, landed) {
        // A new guide, dropped on a page.
        (None, Some((page, map, p))) => {
            next.add(Guide {
                page,
                axis: drag.axis,
                at: drag.axis.of(map.to_page(p)),
            });
        }
        // An existing guide, dropped on a page — possibly a different one.
        (Some((_, index)), Some((page, map, p))) => {
            next.replace(
                index,
                Guide {
                    page,
                    axis: drag.axis,
                    at: drag.axis.of(map.to_page(p)),
                },
            );
        }
        // An existing guide, dropped anywhere that is not a page.
        (Some((_, index)), None) => next.remove(index),
        // A new guide that never reached a page. Nothing to do, and
        // deliberately no action: raising one would rewrite `guides.txt` for a
        // gesture that changed nothing.
        (None, None) => return,
    }
    if next != doc.guides {
        actions.push(Action::SetGuides(next));
    }
}

/// The ruler gutters' half of the gesture: **drag out of a ruler to create a
/// guide.**
pub(super) fn ruler_drag(ui: &mut Ui, doc: &OpenDoc, gutters: Gutters) {
    if !doc.view.guides {
        return;
    }
    let (Some(top), Some(left)) = (gutters.top, gutters.left) else {
        return;
    };
    for (rect, axis, salt) in [
        (top, GuideAxis::Horizontal, 0u8),
        (left, GuideAxis::Vertical, 1u8),
    ] {
        let response = ui.interact(rect, Id::new((BAND_KEY, salt)), Sense::click_and_drag());
        if response.hovered() {
            ui.ctx().set_cursor_icon(cursor(axis));
        }
        if response.drag_started() {
            store(ui.ctx(), Some(Drag { axis, moving: None }));
        }
    }
}

/// Draw the in-flight guide, and commit it when the pointer comes up.
pub(super) fn settle(
    ui: &Ui,
    doc: &OpenDoc,
    geometry: Option<&CanvasGeometry>,
    actions: &mut Vec<Action>,
) {
    let Some(geometry) = geometry else {
        return;
    };
    preview(ui, geometry);
    release(ui.ctx(), doc, geometry, actions);
}

/// The cursor over a guide, or over the ruler that yields one.
fn cursor(axis: GuideAxis) -> egui::CursorIcon {
    match axis {
        GuideAxis::Horizontal => egui::CursorIcon::ResizeVertical,
        GuideAxis::Vertical => egui::CursorIcon::ResizeHorizontal,
    }
}

/// The canvas's half of the gesture: **grab a guide to move it, or
/// double-click it to remove it.**
pub(super) fn canvas_drag(
    ui: &mut Ui,
    doc: &OpenDoc,
    pages: &[PageView],
    actions: &mut Vec<Action>,
) {
    if !doc.view.guides || doc.guides.is_empty() {
        return;
    }
    let mut removed: Option<usize> = None;
    for view in pages {
        for (index, guide) in doc.guides.on_page(view.page) {
            let response = ui.interact(
                guide.band(view.map),
                Id::new((BAND_KEY, view.page, index)),
                Sense::click_and_drag(),
            );
            if response.hovered() {
                ui.ctx().set_cursor_icon(cursor(guide.axis));
            }
            if response.double_clicked() {
                removed = Some(index);
            } else if response.drag_started() {
                store(
                    ui.ctx(),
                    Some(Drag {
                        axis: guide.axis,
                        moving: Some((view.page, index)),
                    }),
                );
            }
        }
    }
    // Applied after the loop: removing inside it would renumber the indices
    // the remaining iterations are keyed on, which is the same renumbering
    // hazard `canvas::moving`'s header tabulates for the delete verbs.
    if let Some(index) = removed {
        let mut next = doc.guides.clone();
        next.remove(index);
        // An in-flight drag on the guide that has just gone would resolve
        // against an index that now names a different guide. Cancelled rather
        // than remapped: a double-click is a complete gesture and there is
        // nothing left to drag.
        store(ui.ctx(), None);
        actions.push(Action::SetGuides(next));
    }
}

/// Draw the guide being dragged, if one is.
fn preview(ui: &Ui, geometry: &CanvasGeometry) {
    let Some(drag) = load(ui.ctx()) else {
        return;
    };
    let Some(p) = ui.ctx().pointer_latest_pos() else {
        return;
    };
    // The content-area selection ink, by its role name. Not
    // `visuals().selection.stroke` — that is `egui`'s selected-WIDGET channel,
    // and a canvas that reads it is borrowing a colour that belongs to another
    // surface. Same colour, named address.
    let base = egui_shell::theme::Theme::canvas_selection_ink(ui.ctx());
    let painter = ui.painter().with_clip_rect(geometry.viewport);
    match pointer_page(ui.ctx(), geometry) {
        Some((page, map, p)) => {
            let guide = Guide {
                page,
                axis: drag.axis,
                at: drag.axis.of(map.to_page(p)),
            };
            painter.line_segment(
                guide.segment(map),
                Stroke::new(1.0, super::overlay::at_alpha(base, GUIDE_ALPHA)),
            );
        }
        None => {
            let stroke = Stroke::new(1.0, super::overlay::at_alpha(base, DISCARD_ALPHA));
            let vp = geometry.viewport;
            match drag.axis {
                GuideAxis::Horizontal => painter.hline(vp.x_range(), p.y, stroke),
                GuideAxis::Vertical => painter.vline(p.x, vp.y_range(), stroke),
            };
        }
    }
}

/// Draw every guide on every page the frame is showing.
pub(super) fn draw(ui: &Ui, doc: &OpenDoc, pages: &[PageView], clip: Rect) {
    if !doc.view.guides || doc.guides.is_empty() {
        return;
    }
    let stroke = Stroke::new(
        1.0,
        super::overlay::at_alpha(
            egui_shell::theme::Theme::canvas_selection_ink(ui.ctx()),
            GUIDE_ALPHA,
        ),
    );
    let painter = ui.painter().with_clip_rect(clip);
    for view in pages {
        for (_, guide) in doc.guides.on_page(view.page) {
            painter.line_segment(guide.segment(view.map), stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Guides {
        let mut g = Guides::default();
        assert!(g.add(Guide {
            page: 0,
            axis: GuideAxis::Horizontal,
            at: 120.5,
        }));
        assert!(g.add(Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: 64.0,
        }));
        assert!(g.add(Guide {
            page: 3,
            axis: GuideAxis::Horizontal,
            at: -12.25,
        }));
        g
    }

    /// **Every guide survives a round trip through the on-disk spelling**,
    /// including a negative coordinate.
    #[test]
    fn every_guide_round_trips_through_its_on_disk_spelling() {
        let guides = sample();
        assert_eq!(Guides::decode(&guides.encode()), guides);
    }

    /// Every axis has a spelling and every spelling names an axis.
    ///
    /// Both directions, so a variant added with a colliding or missing id
    /// fails here rather than becoming a guide that cannot be saved.
    #[test]
    fn every_axis_round_trips_through_its_on_disk_spelling() {
        for axis in [GuideAxis::Horizontal, GuideAxis::Vertical] {
            assert_eq!(GuideAxis::from_id(axis.id()), Some(axis));
        }
        assert_eq!(GuideAxis::from_id("x"), None);
        assert_eq!(GuideAxis::from_id(""), None);
        assert_ne!(GuideAxis::Horizontal.id(), GuideAxis::Vertical.id());
    }

    /// **A corrupt payload degrades into fewer guides, never into an
    /// error.**
    #[test]
    fn a_corrupt_payload_drops_only_the_guides_it_breaks() {
        let good = "0:h:10 1:v:20";
        let mixed = "0:h:10 nonsense 2:q:5 3:h: :: 4:v:zz 1:v:20 5:h:1:2";
        assert_eq!(Guides::decode(mixed), Guides::decode(good));
        assert!(Guides::decode("").is_empty());
        assert!(Guides::decode("   ").is_empty());
    }

    /// A non-finite coordinate is refused rather than stored.
    #[test]
    fn a_non_finite_guide_is_refused() {
        let mut g = Guides::default();
        for at in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(!g.add(Guide {
                page: 0,
                axis: GuideAxis::Horizontal,
                at,
            }));
        }
        assert!(g.is_empty());
        assert!(Guides::decode("0:h:NaN 0:v:inf").is_empty());
    }

    /// The per-document ceiling is enforced, and enforcing it does not corrupt
    /// what is already there.
    #[test]
    fn a_document_stops_accepting_guides_at_the_ceiling() {
        let mut g = Guides::default();
        for i in 0..MAX_PER_DOCUMENT {
            assert!(g.add(Guide {
                page: 0,
                axis: GuideAxis::Vertical,
                at: i as f32,
            }));
        }
        assert!(!g.add(Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: -1.0,
        }));
        assert_eq!(g.len(), MAX_PER_DOCUMENT);
    }

    /// `on_page` selects one page's guides and reports the index the whole
    /// collection knows them by — which is what a drag names.
    #[test]
    fn on_page_reports_the_index_a_drag_names() {
        let guides = sample();
        let on_zero: Vec<_> = guides.on_page(0).collect();
        assert_eq!(on_zero.len(), 2);
        assert_eq!(on_zero[0].0, 0);
        assert_eq!(on_zero[1].0, 1);
        let on_three: Vec<_> = guides.on_page(3).collect();
        assert_eq!(on_three.len(), 1);
        assert_eq!(on_three[0].0, 2, "the index is into the whole collection");
        assert_eq!(guides.on_page(9).count(), 0);
    }

    /// **A guide is stored against a page in canvas space, so it does not
    /// move when the view does.**
    #[test]
    fn a_guide_holds_still_on_the_page_at_every_zoom() {
        let guide = Guide {
            page: 0,
            axis: GuideAxis::Horizontal,
            at: 100.0,
        };
        let extent = (612.0_f32, 792.0_f32);
        for zoom in [0.25_f32, 1.0, 4.0] {
            let rect = Rect::from_min_size(
                pos2(37.0, 11.0),
                egui::vec2(extent.0 * zoom, extent.1 * zoom),
            );
            let map = PageMapping::new(rect, extent, zoom);
            let [a, b] = guide.segment(map);
            // The line spans the page and sits 100 canvas units down it.
            assert!((a.x - rect.min.x).abs() < 0.01 && (b.x - rect.max.x).abs() < 0.01);
            let expected = rect.min.y + 100.0 * zoom;
            assert!(
                (a.y - expected).abs() < 0.01,
                "at {zoom}× the guide drew at {} rather than {expected}",
                a.y
            );
            // …and reading the screen position back gives the stored value.
            assert!((map.to_page(a).y - guide.at).abs() < 0.01);
        }
    }

    /// **The catch band is the same number of screen points wide at every
    /// zoom.**
    #[test]
    fn the_catch_band_is_the_same_width_at_every_zoom() {
        let guide = Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: 300.0,
        };
        let extent = (612.0_f32, 792.0_f32);
        for zoom in [0.1_f32, 0.5, 1.0, 3.0, 8.0] {
            let rect =
                Rect::from_min_size(Pos2::ZERO, egui::vec2(extent.0 * zoom, extent.1 * zoom));
            let band = guide.band(PageMapping::new(rect, extent, zoom));
            assert!(
                (band.width() - CATCH_PTS * 2.0).abs() < 0.01,
                "at {zoom}× the band is {} pt wide",
                band.width()
            );
            assert!(
                (band.height() - rect.height()).abs() < 0.01,
                "the band must not run past its own page"
            );
        }
    }
}
