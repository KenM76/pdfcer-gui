//! The print preview showing a page's separation plates, one ink at a time,
//! under Colour ▸ *Separations* (O438).
//!
//! A plate is the engine's labelled plate (`pdfcer_render::plates::render_plates`),
//! the call the spooler makes, so the preview shows the sheet that prints.
//! The plates of one page are rendered together and held under the page's
//! colour [`PreviewKey`]; stepping inks reuses them. A page the engine cannot
//! separate shows in colour and the strip says so, as the receipt does. N-up,
//! booklet and poster sheets print in colour and have no plates to show.
//!
//! The plates are drawn at the preview's base resolution: the zoomed detail
//! render is skipped while a plate shows, since it renders in colour.

use egui::Ui;
use pdfcer_gui_base::icons::{self, Icon};
use pdfcer_gui_base::printpreviewkey::PreviewKey;
use pdfcer_gui_base::text::print as t;
use pdfcer_render::plates::Plates;
use pdfcer_render::tiny_skia::Pixmap;

use super::super::PrintDialog;

/// The strip height the plates controls add under separations: one row of
/// glyph buttons above the small readout they share a line with.
pub(super) const ROW_PTS: f32 = 28.0;

/// Trace region of the plates switch.
pub(super) const REGION_SWITCH: &str = "print.preview.plates";
/// Trace region of the previous-ink arrow.
pub(super) const REGION_PREVIOUS: &str = "print.preview.plate-previous";
/// Trace region of the next-ink arrow.
pub(super) const REGION_NEXT: &str = "print.preview.plate-next";

/// The plates of one page, as the engine rendered them.
enum Set {
    /// One `(ink, plate)` per ink, in plate order.
    Separated(Vec<(String, Pixmap)>),
    /// The engine could not separate the page; it shows and prints in colour.
    Refused,
    /// The render failed; the page shows in colour until the set is forgotten.
    Failed,
}

/// Whether the preview shows plates, which one, and the current page's set.
#[derive(Default)]
pub(in crate::dialogs::print) struct PlateView {
    /// The switch: show plates in place of the page in colour.
    on: bool,
    /// The plate shown, in the page's plate order.
    ink: usize,
    /// The last page's plates, under that page's colour key.
    set: Option<(PreviewKey, Set)>,
}

impl PlateView {
    /// Whether this sheet shows a plate: separations are chosen, the switch is
    /// on, the sheet is a plain page and the engine did not refuse it.
    pub(super) fn wanted(dialog: &PrintDialog, plain: bool, base: &PreviewKey) -> bool {
        dialog.separations
            && dialog.plates.on
            && plain
            && !matches!(&dialog.plates.set, Some((key, Set::Refused | Set::Failed)) if key == base)
    }

    /// The key of the picture this sheet shows: `base` in colour, or with the
    /// shown plate when [`Self::wanted`].
    pub(super) fn key(dialog: &mut PrintDialog, plain: bool, base: &PreviewKey) -> PreviewKey {
        if Self::wanted(dialog, plain, base) {
            let index = dialog.plates.index(base);
            base.clone().with_plate(Some(index))
        } else {
            base.clone()
        }
    }

    /// Render the picture `key` names: the plate, or `base` in colour when
    /// `key` names none or the page would not separate. Returns the key of
    /// what rendered.
    pub(super) fn render(
        &mut self,
        base: &PreviewKey,
        key: PreviewKey,
        view: &pdfcer_render::DocumentView<'_>,
        page: &pdfcer_core::page_tree::Page,
        scale: f32,
        options: &pdfcer_render::RenderOptions,
    ) -> (PreviewKey, Option<Pixmap>) {
        if key.plate().is_some()
            && let Some(pixmap) = self.pixmap(base, view, page, scale, options)
        {
            return (key, Some(pixmap));
        }
        let colour = pdfcer_render::render_page_with_view(view, page, scale, options)
            .ok()
            .map(|r| r.pixmap);
        (base.clone(), colour)
    }

    /// The plate index to key the shown plate by, clamped to `base`'s set
    /// when that is the set held.
    fn index(&mut self, base: &PreviewKey) -> usize {
        if let Some((key, Set::Separated(plates))) = &self.set
            && key == base
        {
            self.ink = self.ink.min(plates.len().saturating_sub(1));
        }
        self.ink
    }

    /// Plate `self.ink` of the page `base` names, rendering the page's plates
    /// if they are not held. `None` when the page will not separate (it then
    /// shows in colour) or the render failed.
    fn pixmap(
        &mut self,
        base: &PreviewKey,
        view: &pdfcer_render::DocumentView<'_>,
        page: &pdfcer_core::page_tree::Page,
        scale: f32,
        options: &pdfcer_render::RenderOptions,
    ) -> Option<Pixmap> {
        if self.set.as_ref().is_none_or(|(key, _)| key != base) {
            let set = match pdfcer_render::plates::render_plates(view, page, scale, options) {
                Ok(Plates::Separated(plates)) => {
                    Set::Separated(plates.into_iter().map(|p| (p.ink, p.pixmap)).collect())
                }
                Ok(_) => Set::Refused,
                Err(_) => Set::Failed,
            };
            self.set = Some((base.clone(), set));
        }
        let Some((_, Set::Separated(plates))) = &self.set else {
            return None;
        };
        self.ink = self.ink.min(plates.len().saturating_sub(1));
        let (ink, pixmap) = plates.get(self.ink)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "print-preview-plate index={} of={} ink={ink}",
                self.ink,
                plates.len()
            )
        });
        Some(pixmap.clone())
    }

    /// Forget the held set, so a page that refused is asked again.
    pub(super) fn forget(&mut self) {
        self.set = None;
    }
}

/// The strip's plates row: the switch, and with it on the ink arrows and the
/// shown ink's name. Drawn only under separations.
pub(super) fn row(ui: &mut Ui, dialog: &mut PrintDialog, plain: bool) {
    if !dialog.separations {
        return;
    }
    let switch = ui
        .add(icons::glyph(ui, Icon::Separations).selected(dialog.plates.on))
        .on_hover_text(t::preview_plates())
        .on_hover_text(t::preview_plates_tooltip());
    crate::diag::ui_rect_visible(REGION_SWITCH, switch.rect, ui.clip_rect());
    if switch.clicked() {
        dialog.plates.on = !dialog.plates.on;
        dialog.plates.forget();
    }
    if !dialog.plates.on {
        return;
    }
    if !plain {
        ui.label(t::preview_plates_imposed());
        return;
    }
    let inks: Vec<String> = match &dialog.plates.set {
        Some((_, Set::Separated(plates))) => plates.iter().map(|(ink, _)| ink.clone()).collect(),
        Some((_, Set::Refused)) => {
            ui.label(t::preview_plates_refused());
            return;
        }
        Some((_, Set::Failed)) | None => return,
    };
    let shown = dialog.plates.ink.min(inks.len().saturating_sub(1));
    let previous = ui
        .add(icons::glyph(ui, Icon::ChevronLeft))
        .on_hover_text(t::preview_plate_previous());
    crate::diag::ui_rect_visible(REGION_PREVIOUS, previous.rect, ui.clip_rect());
    if previous.clicked() {
        dialog.plates.ink = shown.saturating_sub(1);
    }
    if let Some(ink) = inks.get(shown) {
        ui.label(t::preview_plate_position(ink, shown + 1, inks.len()));
    }
    let next = ui
        .add(icons::glyph(ui, Icon::ChevronRight))
        .on_hover_text(t::preview_plate_next());
    crate::diag::ui_rect_visible(REGION_NEXT, next.rect, ui.clip_rect());
    if next.clicked() && shown + 1 < inks.len() {
        dialog.plates.ink = shown + 1;
    }
}
