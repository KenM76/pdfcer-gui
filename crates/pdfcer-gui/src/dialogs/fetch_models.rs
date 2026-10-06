//! # `dialogs::fetch_models` — File ▸ Recognise ▸ Download OCR models…
//!
//! One row per recogniser in `pdfcer_core::ocr::models::FETCHABLE_MODELS`:
//! its files, where they go, whose they are, and a Download button. The
//! download runs on `ocr::fetch`'s thread; after it the window shows the line
//! the licence requires (`FetchableModels::attribution`).
//!
//! Trace: `fetch-models-started engine= dir=`, then
//! `fetch-models-done engine= files=` or `fetch-models-failed engine= why=`.

use egui::Ui;
use pdfcer_core::ocr::models::{FETCHABLE_MODELS, FetchableModels};

use crate::ocr::fetch::{Download, Outcome};
use crate::text::ocrfetch as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "fetch-models.body";
/// Download button `N` is `fetch-models.download.<engine>`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_DOWNLOAD: &str = "fetch-models.download.";
/// The line shown once a download has ended.
// ui-text-exempt: trace region name, never displayed
pub const REGION_RESULT: &str = "fetch-models.result";

/// The window's state.
#[derive(Debug, Default)]
pub struct FetchModelsDialog {
    running: Option<(&'static FetchableModels, Download)>,
    /// The sentence the last download ended with.
    result: Option<String>,
    close_requested: bool,
}

impl FetchModelsDialog {
    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        self.poll(ctx);
        let (frame, ()) = crate::dialogs::host::Host::new(
            "fetch-ocr-models", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(560.0, 300.0),
            egui::vec2(400.0, 200.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn poll(&mut self, ctx: &egui::Context) {
        let Some((set, download)) = &self.running else {
            return;
        };
        let Some(outcome) = download.poll() else {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
            return;
        };
        let engine = set.engine;
        self.result = Some(match outcome {
            Outcome::Done { files } => {
                // ui-text-exempt: diagnostic trace, never displayed
                crate::diag::trace(|| format!("fetch-models-done engine={engine} files={files}"));
                t::done(files, &set.attribution())
            }
            Outcome::Failed(why) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!("fetch-models-failed engine={engine} why={why}")
                });
                t::failed(&why)
            }
        });
        self.running = None;
    }

    fn body(&mut self, ui: &mut Ui) {
        for set in FETCHABLE_MODELS {
            self.engine_row(ui, set);
            ui.add_space(8.0);
        }
        if self.running.is_some() {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().color(egui_shell::Theme::of(ui.ctx()).palette.text));
                ui.label(t::running());
            });
        }
        if let Some(result) = &self.result {
            let shown = ui.label(result);
            crate::diag::ui_rect_visible(REGION_RESULT, shown.rect, ui.clip_rect());
        }
        ui.add_space(12.0);
        ui.separator();
        if ui.button(t::close_button()).clicked() {
            self.close_requested = true;
        }
    }

    fn engine_row(&mut self, ui: &mut Ui, set: &'static FetchableModels) {
        let label = crate::ocr::EngineId::from_key(set.engine)
            .map(crate::text::ocr::engine_label)
            .or_else(|| crate::text::ocr::token_label(set.engine))
            .unwrap_or(set.engine);
        ui.strong(t::engine_line(label, set.files.len()));
        ui.small(t::licence_line(set.creator, set.source, set.licence));
        let Some(dir) = crate::ocr::fetch::target(set) else {
            ui.small(t::no_target());
            return;
        };
        let shown = dir.display().to_string();
        ui.add(egui::Label::new(egui::RichText::new(t::target_line(&shown)).small()).truncate())
            .on_hover_text(&shown);
        let idle = self.running.is_none();
        let button = ui
            .add_enabled(idle, egui::Button::new(t::download_button()))
            .on_hover_text(t::download_hover())
            .on_disabled_hover_text(t::busy_hover());
        let region = format!("{REGION_DOWNLOAD}{}", set.engine);
        crate::diag::ui_rect_visible(&region, button.rect, ui.clip_rect());
        if button.clicked() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("fetch-models-started engine={} dir={shown}", set.engine)
            });
            self.result = None;
            self.running = Some((set, Download::start(set, dir)));
        }
    }
}
