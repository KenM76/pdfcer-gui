//! # `dialogs::diagnostics` — the render report, off the status bar at last
//!
//! The dispatch target for `tools.render_diagnostics`, on **Tools ▸
//! Diagnostics**.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/diagnostics.md`.

use egui_shell::theme::Theme;

use crate::app::state::OpenDoc;
use crate::text::diagnostics as t;

/// Named region: the whole dialog body.
///
/// Matched literally by `tools/ui-verify`, so renaming it silently un-aims
/// whatever check was measuring it.
const REGION_BODY: &str = "dialog:render-diagnostics"; // ui-text-exempt: trace region name, never displayed

/// The Render-diagnostics dialog. Its existence is its "open" state — see
/// [`super::DialogsState`]'s header for why there is no `open: bool`.
#[derive(Debug, Default)]
pub struct DiagnosticsDialog {
    /// Set by the Close button, consumed by [`Self::show`].
    ///
    /// The same two-step every dialog here uses: a widget inside the window's
    /// closure cannot drop the state it is being drawn from, so it records the
    /// request and the caller acts on it after the closure returns.
    close_requested: bool,
}

impl DiagnosticsDialog {
    /// Build the dialog.
    ///
    /// Takes nothing, because it snapshots nothing — see the type's docs.
    #[must_use]
    pub(super) fn open() -> Self {
        Self::default()
    }

    /// Draw one frame of the dialog. Returns `false` when it should close.
    pub(super) fn show(&mut self, ctx: &egui::Context, doc: &OpenDoc) -> bool {
        // ITS OWN OS WINDOW, and this dialog is the one that most needs it:
        // it is read *while* zooming and panning the document it describes, so
        // a window locked inside the application's frame necessarily covers the
        // thing being diagnosed. Off on a second monitor is where it belongs.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "render-diagnostics", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(520.0, 380.0),
            egui::vec2(380.0, 240.0),
        )
        .show(ctx, |ui| self.body(ui, doc));
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui, doc: &OpenDoc) {
        let theme = Theme::of(ui.ctx());
        crate::diag::ui_rect(REGION_BODY, ui.max_rect());

        // The one state that is not a report: a document is open and nothing
        // has been rasterized. Reachable before the first render and after a
        // render failure, which is exactly when an operator is most likely to
        // reach for this command — so it says which of the two nothings this is
        // rather than drawing an empty window that reads as a broken control.
        let Some(texture) = doc.page_texture.as_ref() else {
            ui.label(t::nothing_drawn());
            ui.add_space(12.0);
            self.footer(ui);
            return;
        };

        // The page the TEXTURE is of, not `doc.view.page_index`. They differ
        // for exactly as long as a render is in flight after a page change,
        // and during that window the canvas is still showing the old raster —
        // so the key's page is the one that describes what is on screen. The
        // number is one-based because it is shown to an operator; every index
        // in this crate is not.
        ui.label(
            egui::RichText::new(t::subject(texture.key.page() + 1)).color(theme.palette.text_muted),
        );
        ui.add_space(8.0);

        let size = texture.texture.size();
        ui.label(t::took(texture.elapsed.as_millis()));
        ui.label(t::raster(texture.key.raster_scale(), size[0], size[1]))
            .on_hover_text(t::raster_tooltip());

        // What the page was BLENDED in, and where that was decided.
        //
        // Keyed on the TEXTURE's page for the same reason the subject line
        // above is: the canvas is showing that raster, and describing a
        // different page's colour beside that raster's duration would be a
        // sentence about neither.
        //
        // Drawn only when `learn_ink` has an answer - R9. A page whose ink was
        // learned by OBSERVING a render (`render::settle`, the second writer)
        // has a `composites_in_ink` and no source, because the render counters
        // say the colorant buffer was engaged and do not say who decided it.
        // That page gets nothing here rather than a guessed origin.
        if let Some(&source) = doc.ink_source.get(&texture.key.page()) {
            let in_ink = doc.ink_pages.contains(&texture.key.page());
            ui.label(t::blended_in(in_ink));
            // The ORIGIN is drawn, not hovered, and that is a rule-4 call
            // rather than a layout one: this window is a disclosure surface,
            // and a disclosure a screenshot does not contain is one the
            // operator cannot send anybody. Muted, because it is the reason
            // for the line above rather than a measurement of its own - the
            // same role `absorbed` takes at the foot of the findings list.
            ui.label(
                egui::RichText::new(t::blend_space_from(source)).color(theme.palette.text_muted),
            );
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(6.0);

        ui.label(egui::RichText::new(t::findings_heading()).heading());
        ui.add_space(6.0);

        // Scrolled, and the scroll starts here rather than around the whole
        // body: the measurements above are the two lines that must stay legible
        // without scrolling, and a scroll area over everything would let them be
        // dragged out of sight. The `max` floor guards the same negative-height
        // trap `about::AboutDialog::body` records — a negative `max_height` is
        // not an error, it is a scroll area that silently draws nothing.
        const FOOTER_RESERVE: f32 = 64.0;
        const LIST_FLOOR: f32 = 48.0;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height((ui.available_height() - FOOTER_RESERVE).max(LIST_FLOOR))
            .show(ui, |ui| {
                let findings = crate::app::status::notes::findings(&texture.diagnostics);
                if findings.is_empty() {
                    ui.label(t::clean());
                } else {
                    for finding in &findings {
                        ui.label(finding);
                    }
                }
                ui.add_space(10.0);
                // The two counters the status bar deliberately excludes. Muted
                // rather than plain, because they are context for the list
                // above rather than members of it — the same role About uses
                // for a supporting line, and not `.strong()`, which
                // `DEFECTS.md` D11 records as unusable in this theme.
                ui.label(
                    egui::RichText::new(t::absorbed(
                        texture.diagnostics.tolerated,
                        texture.diagnostics.compat_skipped,
                    ))
                    .color(theme.palette.text_muted),
                );
            });

        self.footer(ui);
    }

    /// The separator and the Close button.
    fn footer(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::close()).clicked() {
                self.close_requested = true;
            }
        });
    }
}
