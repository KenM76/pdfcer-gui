//! **Thumbnail size** — `OPERATOR_REQUESTS.md` O289 item 18: the rail's
//! pictures zoom as Acrobat's do.
//!
//! Contract: [`ThumbZoom`] is a factor on the narrowest tile the grid lays out
//! (`MIN_TILE_WIDTH_PTS`), stepped through [`STEPS`]; a larger factor means
//! fewer, wider columns. Changed by the row's two buttons or by Ctrl+wheel
//! while the pointer is over the grid — the canvas zooms only when it is
//! hovered, so the gesture reaches one surface. A change traces
//! `pages-zoom factor=`. The pictures already held stay on their tiles,
//! stretched, until the finer render for the new size lands
//! ([`super::thumbnails`]), so a zoom never blanks a tile.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/pages/zoom.md`.

use crate::text::pages as t;

/// The sizes on offer, smallest first. `1.0` is the default grid.
pub const STEPS: [f32; 7] = [0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0];

/// The index of `1.0` in [`STEPS`].
const DEFAULT_STEP: usize = 2;

/// Trace region for the smaller-thumbnails button.
const SMALLER_REGION: &str = "panel-pages-zoom-out"; // ui-text-exempt: trace region name, never displayed
/// Trace region for the larger-thumbnails button.
const LARGER_REGION: &str = "panel-pages-zoom-in"; // ui-text-exempt: trace region name, never displayed

/// The rail's thumbnail size, as a step in [`STEPS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThumbZoom {
    step: usize,
}

impl Default for ThumbZoom {
    fn default() -> Self {
        Self { step: DEFAULT_STEP }
    }
}

impl ThumbZoom {
    /// The factor on the narrowest tile.
    #[must_use]
    pub fn factor(self) -> f32 {
        STEPS[self.step]
    }

    /// One step larger (`up`) or smaller; `false` at either end.
    pub fn step(&mut self, up: bool) -> bool {
        let next = if up {
            (self.step + 1).min(STEPS.len() - 1)
        } else {
            self.step.saturating_sub(1)
        };
        let moved = next != self.step;
        self.step = next;
        moved
    }

    /// Draw the two buttons. Greyed at either end of [`STEPS`], with the
    /// reason on hover.
    pub fn buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(t::thumb_size_label());
            let smaller = ui
                .add_enabled(self.step > 0, egui::Button::new("−")) // ui-text-exempt: a glyph, the tooltip carries the words
                .on_hover_text(t::thumb_smaller())
                .on_disabled_hover_text(t::thumb_smallest());
            crate::diag::ui_rect_visible(SMALLER_REGION, smaller.rect, ui.clip_rect());
            let larger = ui
                .add_enabled(self.step + 1 < STEPS.len(), egui::Button::new("+")) // ui-text-exempt: a glyph, the tooltip carries the words
                .on_hover_text(t::thumb_larger())
                .on_disabled_hover_text(t::thumb_largest());
            crate::diag::ui_rect_visible(LARGER_REGION, larger.rect, ui.clip_rect());
            if smaller.clicked() && self.step(false) {
                self.trace();
            }
            if larger.clicked() && self.step(true) {
                self.trace();
            }
        });
    }

    /// Ctrl+wheel over `grid`: one step per notch.
    pub fn wheel(&mut self, ui: &egui::Ui, grid: egui::Rect) {
        let over = ui
            .ctx()
            .pointer_hover_pos()
            .is_some_and(|p| grid.contains(p));
        if !over {
            return;
        }
        let factor = ui.input(|i| i.zoom_delta());
        if (factor - 1.0).abs() <= f32::EPSILON {
            return;
        }
        if self.step(factor > 1.0) {
            self.trace();
        }
    }

    fn trace(self) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("pages-zoom factor={}", self.factor())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_todays_grid() {
        assert!((ThumbZoom::default().factor() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stepping_stops_at_either_end() {
        let mut z = ThumbZoom::default();
        while z.step(true) {}
        assert!((z.factor() - 4.0).abs() < f32::EPSILON);
        assert!(!z.step(true));
        while z.step(false) {}
        assert!((z.factor() - 0.5).abs() < f32::EPSILON);
        assert!(!z.step(false));
    }
}
