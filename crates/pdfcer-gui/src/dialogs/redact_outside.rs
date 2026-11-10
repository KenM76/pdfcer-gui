//! # `dialogs::redact_outside` — the Redact outside the region window
//!
//! Asks which pages before marking everything outside the snapshot box. The
//! region's rect is used only on pages the region page's size; a page of
//! another size is left alone and counted, never given a rect that would sit
//! somewhere else on it. While the window is open the canvas hatches the area
//! that would go ([`preview_active`]).
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/redact_outside.md`.

use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::regionaction::RegionAction;
use pdfcer_gui_base::snapshotbox::SnapshotBox;

use super::page_scope::{PageScope, Scope};
use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::region as t;

/// The window's body.
pub const REGION_BODY: &str = "redact_outside.body"; // ui-text-exempt: trace region name, never displayed
/// The page choice.
pub const REGION_SCOPE: &str = "redact_outside.scope"; // ui-text-exempt: trace region name, never displayed
/// The Mark outside button.
pub const REGION_MARK: &str = "redact_outside.mark"; // ui-text-exempt: trace region name, never displayed

/// How far two sheets' sides may differ, in points, and still be one size.
const SAME_SIZE_PT: f64 = 0.5;

/// The `egui::Memory` key the canvas reads to hatch the outside.
const PREVIEW_KEY: &str = "pdfcer-redact-outside-preview"; // ui-text-exempt: internal memory id, never displayed

/// The open window.
#[derive(Debug)]
pub struct RedactOutsideDialog {
    region: SnapshotBox,
    scope: PageScope,
    mark_requested: bool,
    close_requested: bool,
}

/// Whether the window was drawn this frame or the one before, for the
/// canvas's hatching. A frame stamp rather than a flag, so a window dropped
/// without being drawn again (its document closed) stops the hatching too.
#[must_use]
pub fn preview_active(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<u64>(egui::Id::new(PREVIEW_KEY)))
        .is_some_and(|drawn| drawn + 1 >= ctx.cumulative_frame_nr())
}

fn set_preview(ctx: &egui::Context, on: bool) {
    let id = egui::Id::new(PREVIEW_KEY);
    let frame = ctx.cumulative_frame_nr();
    ctx.data_mut(|d| {
        if on {
            d.insert_temp(id, frame);
        } else {
            d.remove::<u64>(id);
        }
    });
}

/// Whether two rects have the same width and height within [`SAME_SIZE_PT`].
fn same_size(a: Rect, b: Rect) -> bool {
    ((a.urx - a.llx) - (b.urx - b.llx)).abs() <= SAME_SIZE_PT
        && ((a.ury - a.lly) - (b.ury - b.lly)).abs() <= SAME_SIZE_PT
}

/// The `(page, keep)` pairs for `pages`, and how many were left out for size.
/// The keep rect is moved by each page's MediaBox origin, so a sheet whose
/// box starts elsewhere keeps the same place on the paper.
fn keeps_for(doc: &OpenDoc, region: SnapshotBox, pages: &[usize]) -> (Vec<(usize, Rect)>, usize) {
    let Some(home) = doc.pages.get(region.page).map(|p| p.media_box) else {
        return (Vec::new(), pages.len());
    };
    let r = region.rect;
    let mut keeps = Vec::new();
    let mut skipped = 0;
    for &index in pages {
        match doc.pages.get(index).map(|p| p.media_box) {
            Some(m) if same_size(m, home) => {
                let (dx, dy) = (m.llx - home.llx, m.lly - home.lly);
                keeps.push((
                    index,
                    Rect::from_corners(r.llx + dx, r.lly + dy, r.urx + dx, r.ury + dy),
                ));
            }
            _ => skipped += 1,
        }
    }
    (keeps, skipped)
}

impl RedactOutsideDialog {
    /// Open over `region`, scoped to its own page.
    #[must_use]
    pub fn open(region: SnapshotBox) -> Self {
        let mut scope = PageScope::new(region.page, Vec::new());
        scope.scope = Scope::CurrentPage;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("redact-outside-opened {}", region.trace_fields())
        });
        Self {
            region,
            scope,
            mark_requested: false,
            close_requested: false,
        }
    }

    /// Draw the window; `false` once it is done with. Raises the marks on Mark.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        actions: &mut Vec<Action>,
        appearance: pdfcer_core::annot_author::RedactAppearance,
    ) -> bool {
        // The box can be moved while the window is open; follow it.
        if let Some(now) = doc.snapshot.filter(|s| s.page == self.region.page) {
            self.region = now;
        }
        let pages = self.scope.pages(doc.pages.len()).unwrap_or_default();
        let (keeps, skipped) = keeps_for(doc, self.region, &pages);
        let (frame, ()) = crate::dialogs::host::Host::new(
            "redact-outside", // ui-text-exempt: a viewport key, never displayed.
            t::outside_title(),
            egui::vec2(460.0, 360.0),
            egui::vec2(360.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui, doc.pages.len(), keeps.len(), skipped);
        });
        let open = !frame.closed && doc.snapshot.is_some();
        if std::mem::take(&mut self.mark_requested) && !keeps.is_empty() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "redact-outside-mark pages={} skipped={skipped}",
                    keeps.len()
                )
            });
            actions.push(Action::Region(RegionAction::RedactOutside {
                keeps,
                skipped,
                appearance,
            }));
            set_preview(ctx, false);
            return false;
        }
        let open = open && !std::mem::take(&mut self.close_requested);
        set_preview(ctx, open);
        open
    }

    fn body(&mut self, ui: &mut egui::Ui, count: usize, marking: usize, skipped: usize) {
        let text = egui_shell::theme::Theme::of(ui.ctx()).palette.text;
        ui.label(
            egui::RichText::new(t::outside_warning(marking))
                .strong()
                .color(text),
        );
        ui.add_space(4.0);
        ui.label(t::outside_when());
        ui.add_space(8.0);
        // ui-text-exempt: trace tag, never displayed.
        self.scope
            .show(ui, count, REGION_SCOPE, "redact-outside-scope");
        if skipped > 0 {
            ui.add_space(4.0);
            ui.label(t::outside_skipped(skipped, self.region.page));
        }
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            let mark = ui.add_enabled(marking > 0, egui::Button::new(t::outside_mark()));
            crate::diag::ui_rect_visible(REGION_MARK, mark.rect, ui.clip_rect());
            if mark.on_disabled_hover_text(t::outside_no_pages()).clicked() {
                self.mark_requested = true;
            }
            if ui.button(t::outside_cancel()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: f64, h: f64) -> Rect {
        Rect::from_corners(0.0, 0.0, w, h)
    }

    #[test]
    fn sheets_within_half_a_point_are_one_size() {
        assert!(same_size(rect(612.0, 792.0), rect(612.3, 791.8)));
        assert!(!same_size(rect(612.0, 792.0), rect(792.0, 612.0)));
        assert!(!same_size(rect(612.0, 792.0), rect(595.0, 842.0)));
    }
}
