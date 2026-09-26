//! # `canvas::textedit::hit` — where the pointer is inside the editor box
//!
//! ## What this is
//!
//! One fact, published once a frame by [`super::paint`] and read by everything
//! that needs to know whether a pointer event belongs to the draft: **the
//! editor box's rectangle, and the galley that was drawn inside it.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/hit.md`.

use std::sync::Arc;

use egui::{Galley, Pos2, Rect};

/// Where the layout is parked between the frame that draws it and the frame
/// that hit-tests it.
const KEY: &str = "textedit-hit-layout"; // ui-text-exempt: a memory key, never displayed.

/// **The editor box as it was last drawn.**
///
/// Cloned in and out of `egui::Memory`, which is why the galley is an `Arc`:
/// egui already hands them out that way and the clone is a refcount bump.
#[derive(Clone)]
pub struct Layout {
    /// The box, in **screen** coordinates — what a raw pointer position is in.
    pub body: Rect,
    /// The same box in **canvas** coordinates, for the click ladder, which
    /// works in page space and never sees a screen point.
    ///
    /// Both, rather than one and a conversion at the call site: the two
    /// callers live in different coordinate spaces and neither has the other's
    /// map to hand at the moment it asks. Publishing both puts the one
    /// conversion in the one place that owns the map.
    pub body_canvas: Rect,
    /// Where the galley's origin sits on screen, so a screen position can be
    /// made galley-relative.
    pub origin: Pos2,
    /// The galley that was drawn — see the module header.
    pub galley: Arc<Galley>,
}

impl Layout {
    /// **Which character index is under `screen`**, as a character offset into
    /// the draft's text.
    #[must_use]
    pub fn index_at(&self, screen: Pos2) -> usize {
        self.galley
            .cursor_from_pos(screen - self.origin)
            .index
            .into()
    }
}

/// Publish this frame's editor box. Called by [`super::paint`] only.
pub fn publish(ctx: &egui::Context, layout: Layout) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), layout));
}

/// The editor box as of the last frame that drew one, if any.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<Layout> {
    ctx.data(|d| d.get_temp::<Layout>(egui::Id::new(KEY)))
}

/// **Is `screen` inside the editor box a live draft is being composed in?**
///
/// `false` when there is no draft, which is what makes this safe to ask from
/// the click ladder on every press.
#[must_use]
pub fn owns_screen(ctx: &egui::Context, screen: Pos2) -> bool {
    super::read(ctx).is_some() && read(ctx).is_some_and(|l| l.body.contains(screen))
}

/// [`owns_screen`] for a caller that only has a canvas-space point.
#[must_use]
pub fn owns_canvas(ctx: &egui::Context, canvas: Pos2) -> bool {
    super::read(ctx).is_some() && read(ctx).is_some_and(|l| l.body_canvas.contains(canvas))
}
