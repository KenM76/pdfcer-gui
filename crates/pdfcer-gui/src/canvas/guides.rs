//! # `canvas::guides` — draggable alignment lines, whose home is a page and whose life is a file
//!
//! The third of `RIBBON_IA.md` §5.2's *"Rulers · Grid · Guides"*, and the one
//! with a condition on it: a guide must be **draggable**, and it must
//! **survive a reopen**, which takes a per-document store. This header answers
//! both, plus the two questions they imply: *what does a guide belong to*, and
//! *where does it live on disk*.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/guides.md`.

use std::path::{Path, PathBuf};

use egui::{Context, Id, Pos2, Rect, Sense, Stroke, Ui, pos2};
use pdfcer_core::settings;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::rulers::{CanvasGeometry, Gutters};
use crate::canvas::strip::PageView;

/// The file's name, inside the settings directory.
pub const GUIDES_FILE: &str = "guides.txt"; // ui-text-exempt: a file name, never displayed as copy

/// The separator between the guide payload and the path.
///
/// A tab, because it is the one ASCII character a Windows path cannot contain
/// and therefore the one that needs no escaping. See the module header.
const SEPARATOR: char = '\t';

/// The separator between a guide's three fields.
const FIELD: char = ':';

/// How many documents are remembered.
pub const CAP: usize = 200;

/// How many guides one document may carry.
pub const MAX_PER_DOCUMENT: usize = 256;

/// The half-width, in logical points, of the band that catches a guide drag.
const CATCH_PTS: f32 = 4.0;

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

/// Which way a guide runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideAxis {
    /// A horizontal line, pinning a canvas **y**. Dragged from the top ruler.
    Horizontal,
    /// A vertical line, pinning a canvas **x**. Dragged from the left ruler.
    Vertical,
}

impl GuideAxis {
    /// The on-disk spelling.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            // ui-text-exempt: on-disk spelling, never displayed as copy
            GuideAxis::Horizontal => "h",
            // ui-text-exempt: on-disk spelling, never displayed as copy
            GuideAxis::Vertical => "v",
        }
    }

    /// The axis an on-disk spelling names, or `None`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "h" => Some(GuideAxis::Horizontal),
            "v" => Some(GuideAxis::Vertical),
            _ => None,
        }
    }

    /// The component of a canvas point this axis pins.
    fn of(self, p: Pos2) -> f32 {
        match self {
            GuideAxis::Horizontal => p.y,
            GuideAxis::Vertical => p.x,
        }
    }
}

/// One guide: a line at a fixed place on one page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Guide {
    /// The 0-based page it belongs to.
    pub page: usize,
    /// Which way the line runs.
    pub axis: GuideAxis,
    /// The canvas-space coordinate it pins.
    pub at: f32,
}

impl Guide {
    /// Its line on screen, spanning the page it belongs to.
    fn segment(self, map: PageMapping) -> [Pos2; 2] {
        let page = map.image_rect();
        match self.axis {
            GuideAxis::Horizontal => {
                let y = map.to_screen(pos2(0.0, self.at)).y;
                [pos2(page.min.x, y), pos2(page.max.x, y)]
            }
            GuideAxis::Vertical => {
                let x = map.to_screen(pos2(self.at, 0.0)).x;
                [pos2(x, page.min.y), pos2(x, page.max.y)]
            }
        }
    }

    /// The screen-space band a press must land in to grab this guide.
    fn band(self, map: PageMapping) -> Rect {
        let [a, b] = self.segment(map);
        Rect::from_two_pos(a, b).expand2(match self.axis {
            GuideAxis::Horizontal => egui::vec2(0.0, CATCH_PTS),
            GuideAxis::Vertical => egui::vec2(CATCH_PTS, 0.0),
        })
    }
}

/// Every guide one document carries.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Guides(Vec<Guide>);

impl Guides {
    /// Whether there are none at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many there are, across every page.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Every guide, in the order they will be written.
    pub fn iter(&self) -> impl Iterator<Item = Guide> + '_ {
        self.0.iter().copied()
    }

    /// The guides on `page`, with their index in the whole collection.
    pub fn on_page(&self, page: usize) -> impl Iterator<Item = (usize, Guide)> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter(move |(_, g)| g.page == page)
            .map(|(i, g)| (i, *g))
    }

    /// Add `guide`, unless the document is already at [`MAX_PER_DOCUMENT`].
    pub fn add(&mut self, guide: Guide) -> bool {
        if self.0.len() >= MAX_PER_DOCUMENT || !guide.at.is_finite() {
            return false;
        }
        self.0.push(guide);
        true
    }

    /// Replace the guide at `index`, if there is one.
    pub fn replace(&mut self, index: usize, guide: Guide) {
        if guide.at.is_finite()
            && let Some(slot) = self.0.get_mut(index)
        {
            *slot = guide;
        }
    }

    /// Remove the guide at `index`, if there is one.
    pub fn remove(&mut self, index: usize) {
        if index < self.0.len() {
            self.0.remove(index);
        }
    }

    /// The on-disk payload — the part of a line before the tab.
    fn encode(&self) -> String {
        self.0
            .iter()
            .map(|g| format!("{}{FIELD}{}{FIELD}{}", g.page, g.axis.id(), g.at))
            .collect::<Vec<_>>()
            .join(" ") // ui-text-exempt: the on-disk field separator, never displayed
    }

    /// Parse an on-disk payload, dropping anything that does not parse.
    fn decode(payload: &str) -> Self {
        let mut out = Self::default();
        for token in payload.split_whitespace() {
            let mut parts = token.split(FIELD);
            let (Some(page), Some(axis), Some(at), None) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let (Ok(page), Some(axis), Ok(at)) = (
                page.parse::<usize>(),
                GuideAxis::from_id(axis),
                at.parse::<f32>(),
            ) else {
                continue;
            };
            out.add(Guide { page, axis, at });
        }
        out
    }
}

// ---------------------------------------------------------------------------
// The store
// ---------------------------------------------------------------------------

/// The path this store reads and writes, or `None` when `pdfcer-core` found no
/// writable location.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    settings::resolve_store()
        .directory()
        .map(|dir| dir.join(GUIDES_FILE))
}

/// **The guides remembered for `document`**, or none.
///
/// Never fails. A missing file, an unreadable one and a corrupt one all answer
/// an empty set, because every one of them means the same thing to the caller.
#[must_use]
pub fn recall(document: &Path) -> Guides {
    recall_at(default_path().as_deref(), document)
}

/// **Remember `guides` against `document`.**
pub fn remember(document: &Path, guides: &Guides) {
    remember_at(default_path().as_deref(), document, guides);
}

/// **What a freshly opened document starts with**: its remembered guides,
/// and the view state that shows them.
///
/// The two halves are returned together because the rule joining them is the
/// point, and it belongs here rather than in
/// [`crate::app::state::OpenDoc::new`]: **a document that has remembered
/// guides opens with `view.guides` already on.**
///
/// The presence of the work *is* the preference — see this module's header §2
/// on why the three View ▸ Display toggles are not persisted while the guides
/// are. The alternative, restoring guides and leaving them invisible until the
/// operator finds a switch, is a feature that appears not to have worked; and
/// storing a fourth flag to say "show the things I just restored" would be
/// storing something derivable.
///
/// Every other field of the returned [`crate::viewer::ViewState`] is
/// `Default`, which is the conservative one — the same division of labour
/// `ViewState::default`'s own docs describe for Read mode's continuous
/// default: the path that knows the document is the path that may know better.
#[must_use]
pub fn opening(document: &Path) -> (Guides, crate::viewer::ViewState) {
    let guides = recall(document);
    let view = crate::viewer::ViewState {
        guides: !guides.is_empty(),
        ..crate::viewer::ViewState::default()
    };
    (guides, view)
}

/// [`recall`], against an explicit file — the seam tests use.
#[must_use]
pub fn recall_at(file: Option<&Path>, document: &Path) -> Guides {
    let wanted = absolute(document);
    let Some(file) = file else {
        return Guides::default();
    };
    let Ok(text) = std::fs::read_to_string(file) else {
        return Guides::default();
    };
    for (payload, path) in parse(&text) {
        if path == wanted {
            return Guides::decode(&payload);
        }
    }
    Guides::default()
}

/// [`remember`], against an explicit file.
pub fn remember_at(file: Option<&Path>, document: &Path, guides: &Guides) {
    let Some(file) = file else {
        return;
    };
    let wanted = absolute(document);
    let previous = std::fs::read_to_string(file).unwrap_or_default();
    let mut lines: Vec<String> = Vec::with_capacity(CAP);
    if !guides.is_empty() {
        lines.push(format!(
            "{}{SEPARATOR}{}",
            guides.encode(),
            wanted.display()
        ));
    }
    for (payload, path) in parse(&previous) {
        if path == wanted || lines.len() >= CAP {
            continue;
        }
        lines.push(format!("{payload}{SEPARATOR}{}", path.display()));
    }

    // The parent directory may not exist on a first run — the same situation
    // `recent.rs` and `remembered.rs` handle, and the same answer: create it,
    // and let the write's own failure be the one that is reported.
    if let Some(dir) = file.parent()
        && !dir.as_os_str().is_empty()
    {
        let _ = std::fs::create_dir_all(dir);
    }
    let mut body = lines.join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    if let Err(err) = std::fs::write(file, body) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("guides-write-failed path={} err={err}", file.display())
        });
    }
}

/// Split a file's text into `(payload, path)` pairs, dropping unparseable
/// lines.
fn parse(text: &str) -> Vec<(String, PathBuf)> {
    text.lines()
        .filter_map(|line| {
            let (payload, path) = line.split_once(SEPARATOR)?;
            let path = path.trim_end_matches(['\r']);
            if path.is_empty() {
                return None;
            }
            Some((payload.to_owned(), PathBuf::from(path)))
        })
        .collect()
}

/// A document's path as this store keys it.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

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

    /// **A document's guides come back after a reopen, and a second
    /// document's do not leak into the first.**
    #[test]
    fn guides_survive_a_reopen_and_stay_with_their_own_document() {
        let dir = std::env::temp_dir().join(format!("pdfcer-guides-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("guides-roundtrip.txt");
        let _ = std::fs::remove_file(&file);

        let drawing = PathBuf::from("D:\\Drawings\\job 4471\\sheet set.pdf");
        let report = PathBuf::from("C:\\reports\\quarterly.pdf");

        remember_at(Some(&file), &drawing, &sample());
        assert_eq!(recall_at(Some(&file), &drawing), sample());
        // A document nobody has ruled up has no guides, and asking does not
        // hand it the other document's.
        assert!(recall_at(Some(&file), &report).is_empty());

        // A second document writes its own line without disturbing the first
        // — the read-modify-write property, and the one a naive "write my
        // line" implementation loses.
        let mut theirs = Guides::default();
        theirs.add(Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: 306.0,
        });
        remember_at(Some(&file), &report, &theirs);
        assert_eq!(recall_at(Some(&file), &drawing), sample());
        assert_eq!(recall_at(Some(&file), &report), theirs);

        // Clearing the last guide forgets the document rather than leaving an
        // empty marker behind.
        remember_at(Some(&file), &report, &Guides::default());
        assert!(recall_at(Some(&file), &report).is_empty());
        let text = std::fs::read_to_string(&file).expect("the file is still there");
        assert!(
            !text.contains("quarterly.pdf"),
            "an emptied document must not keep a line: {text:?}"
        );
        assert!(
            text.contains("sheet set.pdf"),
            "the other document survived"
        );

        let _ = std::fs::remove_file(&file);
    }

    /// **A path containing spaces round-trips**, which is why the payload is
    /// written first and the path is the whole remainder of the line.
    #[test]
    fn a_path_with_spaces_survives_the_format() {
        let pairs = parse("0:h:10 1:v:20\tC:\\Program Files\\a b\\c d.pdf\n");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "0:h:10 1:v:20");
        assert_eq!(pairs[0].1, PathBuf::from("C:\\Program Files\\a b\\c d.pdf"));
    }

    /// A line with no tab, or with an empty path, is dropped rather than
    /// producing a guide set attached to nothing.
    #[test]
    fn a_malformed_line_is_dropped() {
        assert!(parse("no tab here\n").is_empty());
        assert!(parse("0:h:10\t\n").is_empty());
        assert!(parse("").is_empty());
        // …and a well-formed line among broken ones still reads.
        let pairs = parse("junk\n0:h:10\tC:\\a.pdf\n\t\n");
        assert_eq!(pairs.len(), 1);
    }

    /// Reading a store that does not exist answers "no guides" rather than
    /// failing — the same posture `remembered::recall` takes, and the reason
    /// a first run needs no special case.
    #[test]
    fn a_missing_store_answers_no_guides() {
        // temp-path-exempt: nothing is created here; the test needs a path
        // that is absent, and absence is not contended.
        let missing = std::env::temp_dir().join("pdfcer-guides-does-not-exist-4471.txt");
        let _ = std::fs::remove_file(&missing);
        assert!(recall_at(Some(&missing), Path::new("C:\\a.pdf")).is_empty());
        assert!(recall_at(None, Path::new("C:\\a.pdf")).is_empty());
    }
}
