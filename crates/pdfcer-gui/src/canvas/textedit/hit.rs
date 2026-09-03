//! # `canvas::textedit::hit` — where the pointer is inside the editor box
//!
//! ## What this is
//!
//! One fact, published once a frame by [`super::paint`] and read by everything
//! that needs to know whether a pointer event belongs to the draft: **the
//! editor box's rectangle, and the galley that was drawn inside it.**
//!
//! ## ★★★ Why the galley has to be shared rather than re-derived
//!
//! Because *"which character is under the pointer"* and *"where is the caret
//! drawn"* must be the **same** derivation, and this module exists to make that
//! true by construction rather than by two functions agreeing.
//!
//! The alternative is to lay the draft out a second time in the click handler.
//! That looks identical and drifts the moment anything differs — a wrap width
//! computed slightly differently, a font resolved on a different frame, a
//! `TextStyle` read from a `Ui` with different spacing. `super::paint`'s header
//! already records what that cost once: a caret derived from the *page's* glyph
//! advances while the text was drawn in the shell's font, drifting further the
//! more the operator typed, and the fix was to delete the second derivation.
//!
//! So the galley that was **drawn** is the galley that is **hit-tested**, and
//! `Galley::cursor_from_pos` is the inverse of the `Galley::pos_from_cursor`
//! the caret is painted with. One layout, two questions.
//!
//! ## ★★ Why a frame late is not a bug here
//!
//! `paint` runs after `interact` in the frame, so a pointer handler reads the
//! rectangle and galley **the previous frame** produced. That is correct rather
//! than tolerated:
//!
//! - The operator can only press on something they can **see**, and what they
//!   can see is the previous frame.
//! - A press on the frame the editor box first appears has nothing to hit, and
//!   answering `None` there is right: the box was not on screen when the button
//!   went down.
//!
//! It is the same argument `canvas::markup::ink` makes for reading the gesture
//! machine's answer before advancing it, and the opposite of a stale-coordinate
//! bug — the coordinate is *deliberately* the one the operator was looking at.
//!
//! ## What this deliberately does not do
//!
//! It does not decide what a press **means**. That is [`super::keys`] for a
//! draft and `canvas::clicking` for the page, and both ask this module the same
//! question and act on it differently.

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
    /// ★ Both, rather than one and a conversion at the call site: the two
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
    ///
    /// Clamped by the galley: a position above or left of the text answers 0, a
    /// position past the end answers the length. That is what every text field
    /// does and it is what makes a drag that runs off the end select to the
    /// end rather than stopping.
    ///
    /// ★ `CCursor::index` is a **character** index, which is the unit
    /// `super::Draft::caret` is documented in. `Galley` also speaks in rows and
    /// byte offsets; taking either would compile and would put the caret inside
    /// a multi-byte character on the first document with an accent in it.
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
///
/// # ★ Why nothing clears this
///
/// Because every caller asks *"is the pointer inside the box"* and a stale
/// rectangle from a draft that has ended answers that question wrongly only if
/// the pointer is inside a box that is no longer drawn — and the callers all
/// gate on a **live draft** first, which is the fact that actually ended. A
/// clear step would be a second place to forget.
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
