//! # `dialogs::ocr_models` — File ▸ Recognise ▸ Manage OCR models…
//!
//! Every supported recogniser, each with its installed models or "not
//! installed": size, version, location, whether it can run, a Default mark or
//! Set as default, and Remove. Below: the total, Remove all, Install from
//! file or folder. A recogniser with pinned files (`FETCHABLE_MODELS`) has a
//! Download button when the build has `model-download`.
//!
//! The default is the `ocr_model` preference Recognise text starts from
//! (`catalog::start`); there is no second one. Removing the default clears
//! it and the window says what Recognise text starts with now.
//!
//! Trace: `ocr-models-listed …` on each discovery, `ocr-models-default
//! name=`, `ocr-models-remove-asked`, `ocr-models-removed` /
//! `ocr-models-remove-failed`, `ocr-models-installed` /
//! `ocr-models-install-failed`, and `fetch-models-*` for a download.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/ocr_models.md`.

mod jobs;
mod listing;

use egui::Ui;

use crate::app::prefs::Prefs;
use crate::ocr::catalog::Choice;
use crate::ocr::manage;
use crate::text::ocrmanage as t;

use jobs::{Finished, Job};
use listing::Listing;

// ui-text-exempt: trace region names, never displayed.
const REGION_BODY: &str = "ocr-models.body";
// ui-text-exempt: trace region names, never displayed.
const REGION_RESULT: &str = "ocr-models.result";
// ui-text-exempt: trace region names, never displayed.
const REGION_UNWRITABLE: &str = "ocr-models.unwritable";
// ui-text-exempt: trace region names, never displayed.
const REGION_CONFIRM: &str = "ocr-models.confirm";
// ui-text-exempt: trace region names, never displayed.
const REGION_KEEP: &str = "ocr-models.keep";
// ui-text-exempt: trace region names, never displayed.
const REGION_REMOVE_ALL: &str = "ocr-models.remove-all";
// ui-text-exempt: trace region names, never displayed.
const REGION_INSTALL_FILE: &str = "ocr-models.install-file";
// ui-text-exempt: trace region names, never displayed.
const REGION_CLOSE: &str = "ocr-models.close";
// ui-text-exempt: trace region names, never displayed.
const REGION_INSTALL_FOLDER: &str = "ocr-models.install-folder";

/// What the confirm strip is asking about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confirm {
    One(usize),
    All,
}

/// The window's state.
#[derive(Debug, Default)]
pub struct OcrModelsDialog {
    listing: Option<Listing>,
    job: Option<Job>,
    confirm: Option<Confirm>,
    /// The sentence the last action ended with.
    result: Option<String>,
    /// The listing no longer matches the preferences; discover again.
    stale: bool,
    close_requested: bool,
}

impl OcrModelsDialog {
    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, prefs: &mut Prefs) -> bool {
        if self
            .listing
            .as_ref()
            .is_none_or(|l| l.generation != manage::generation())
            && self.job.is_none()
        {
            self.listing = Some(Listing::load(prefs));
        }
        self.poll(ctx, prefs);
        let (frame, ()) = crate::dialogs::host::Host::new(
            "ocr-models", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(620.0, 520.0),
            egui::vec2(440.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            egui::ScrollArea::vertical().show(ui, |ui| self.body(ui, prefs));
        });
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn poll(&mut self, ctx: &egui::Context, prefs: &mut Prefs) {
        let Some(job) = &self.job else {
            return;
        };
        let Some(finished) = job.poll() else {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
            return;
        };
        self.job = None;
        let before = self.listing.take();
        let default_before = before
            .as_ref()
            .and_then(|l| l.default_name().map(str::to_owned));
        if let Finished::Removed { names, .. } = &finished
            && let Some(model) = prefs.ocr_models.model.as_deref()
            && names.iter().any(|n| n == model)
        {
            prefs.ocr_models.model = None;
            let _ = prefs.save();
        }
        let after = Listing::load(prefs);
        self.result = Some(jobs::sentence(&finished, default_before.as_deref(), &after));
        self.listing = Some(after);
    }

    fn body(&mut self, ui: &mut Ui, prefs: &mut Prefs) {
        let Some(listing) = self.listing.take() else {
            return;
        };
        if let Some(dir) = &listing.models_dir {
            ui.small(t::folder_line(dir));
            if !listing.writable {
                let line = ui.label(t::folder_unwritable(dir));
                crate::diag::ui_rect_visible(REGION_UNWRITABLE, line.rect, ui.clip_rect());
            }
        }
        ui.add_space(6.0);
        for token in manage::SUPPORTED {
            self.engine_group(ui, prefs, &listing, token);
        }
        let others: Vec<usize> = listing.others().map(|(i, _)| i).collect();
        for i in others {
            self.model_row(ui, prefs, &listing, i);
        }
        ui.separator();
        self.footer(ui, &listing);
        if !std::mem::take(&mut self.stale) {
            self.listing = Some(listing);
        }
    }

    fn engine_group(
        &mut self,
        ui: &mut Ui,
        prefs: &mut Prefs,
        listing: &Listing,
        token: &'static str,
    ) {
        ui.strong(engine_label(token));
        let installed: Vec<usize> = listing.of_engine(token).map(|(i, _)| i).collect();
        if installed.is_empty() {
            self.missing_row(ui, listing, token);
        }
        for i in installed {
            self.model_row(ui, prefs, listing, i);
        }
        ui.add_space(8.0);
    }

    fn missing_row(&mut self, ui: &mut Ui, listing: &Listing, token: &'static str) {
        let row = ui.horizontal_wrapped(|ui| {
            ui.label(t::not_installed());
            if !manage::in_build(token) {
                ui.small(t::not_in_build());
            } else if !self.download_button(ui, listing, token) {
                ui.small(t::install_how());
            }
        });
        crate::diag::ui_rect_visible(
            &format!("ocr-models.missing.{token}"),
            row.response.rect,
            ui.clip_rect(),
        );
    }

    /// The Download button for `token`, if it has pinned files; whether one
    /// was drawn.
    #[cfg(feature = "model-download")]
    fn download_button(&mut self, ui: &mut Ui, listing: &Listing, token: &str) -> bool {
        let Some(set) = pdfcer_core::ocr::models::fetchable_models(token) else {
            return false;
        };
        let Some(dir) = listing.models_dir.as_ref().map(|d| d.join(set.folder)) else {
            return false;
        };
        let button = self.gated(
            ui,
            listing,
            egui::Button::new(crate::text::ocrfetch::download_button()),
        );
        let button = button.on_hover_text(crate::text::ocrfetch::licence_line(
            set.creator,
            set.source,
            set.licence,
        ));
        crate::diag::ui_rect_visible(
            &format!("ocr-models.download.{token}"),
            button.rect,
            ui.clip_rect(),
        );
        if button.clicked() {
            let shown = dir.display().to_string();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("fetch-models-started engine={} dir={shown}", set.engine)
            });
            self.result = None;
            self.job = Some(Job::download(set, dir));
        }
        true
    }

    #[cfg(not(feature = "model-download"))]
    fn download_button(&mut self, _: &mut Ui, _: &Listing, _: &str) -> bool {
        false
    }

    fn model_row(&mut self, ui: &mut Ui, prefs: &mut Prefs, listing: &Listing, i: usize) {
        let choice = &listing.catalog.choices[i];
        let bytes = listing.sizes[i];
        let version = choice
            .model
            .manifest
            .as_ref()
            .and_then(|m| m.version.as_deref());
        let row = ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(t::installed_line(
                    &crate::text::ocrmodels::label(choice),
                    version,
                    bytes,
                ));
                self.default_control(ui, prefs, listing, i);
                self.remove_control(ui, listing, i);
            });
            let status = choice.unrunnable.as_ref().map_or_else(
                || t::ready().to_owned(),
                |w| t::cannot_run(&crate::text::ocrmodels::why(w)),
            );
            ui.small(status);
            let folder = t::location_line(&choice.folder);
            ui.add(egui::Label::new(egui::RichText::new(&folder).small().weak()).truncate())
                .on_hover_text(&folder);
        });
        crate::diag::ui_rect_visible(
            &format!("ocr-models.row.{}", choice.name),
            row.response.rect,
            ui.clip_rect(),
        );
        if self.confirm == Some(Confirm::One(i)) {
            self.confirm_strip(
                ui,
                listing,
                &t::confirm_remove(
                    &crate::text::ocrmodels::label(choice),
                    &choice.folder,
                    bytes,
                ),
            );
        }
    }

    fn default_control(&mut self, ui: &mut Ui, prefs: &mut Prefs, listing: &Listing, i: usize) {
        let choice = &listing.catalog.choices[i];
        if listing.default == Some(i) {
            let mark = ui
                .strong(t::is_default())
                .on_hover_text(t::is_default_hover());
            crate::diag::ui_rect_visible(
                &format!("ocr-models.is-default.{}", choice.name),
                mark.rect,
                ui.clip_rect(),
            );
            return;
        }
        if !choice.runnable() {
            return;
        }
        let button = ui
            .button(t::set_default())
            .on_hover_text(t::set_default_hover());
        crate::diag::ui_rect_visible(
            &format!("ocr-models.default.{}", choice.name),
            button.rect,
            ui.clip_rect(),
        );
        if button.clicked() {
            set_default(prefs, choice);
            self.result = Some(t::default_set(&crate::text::ocrmodels::label(choice)));
            self.stale = true;
        }
    }

    fn remove_control(&mut self, ui: &mut Ui, listing: &Listing, i: usize) {
        let choice = &listing.catalog.choices[i];
        let region = format!("ocr-models.remove.{}", choice.name);
        if choice.model.folder == choice.model.root {
            let _ = ui
                .add_enabled(false, egui::Button::new(t::remove_button()))
                .on_disabled_hover_text(t::remove_root_hover());
            return;
        }
        let button = self.gated(ui, listing, egui::Button::new(t::remove_button()));
        let button = button.on_hover_text(t::remove_hover(listing.sizes[i]));
        crate::diag::ui_rect_visible(&region, button.rect, ui.clip_rect());
        if button.clicked() {
            self.ask(Confirm::One(i), &choice.name, listing.sizes[i]);
        }
    }

    fn ask(&mut self, what: Confirm, name: &str, bytes: u64) {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("ocr-models-remove-asked name={name} bytes={bytes}"));
        self.confirm = Some(what);
        self.result = None;
    }

    fn confirm_strip(&mut self, ui: &mut Ui, listing: &Listing, question: &str) {
        ui.group(|ui| {
            ui.label(question);
            ui.horizontal(|ui| {
                let yes = ui.button(t::confirm_button());
                crate::diag::ui_rect_visible(REGION_CONFIRM, yes.rect, ui.clip_rect());
                let no = ui.button(t::keep_button());
                crate::diag::ui_rect_visible(REGION_KEEP, no.rect, ui.clip_rect());
                if yes.clicked() {
                    self.start_removal(listing);
                } else if no.clicked() {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    crate::diag::trace(|| "ocr-models-remove-kept".to_owned());
                    self.confirm = None;
                }
            });
        });
    }

    fn start_removal(&mut self, listing: &Listing) {
        let models = match self.confirm.take() {
            Some(Confirm::One(i)) => vec![listing.catalog.choices[i].clone()],
            Some(Confirm::All) => listing.removable().map(|(_, c)| c.clone()).collect(),
            None => return,
        };
        self.job = Some(Job::remove(models, listing.writable));
    }

    fn footer(&mut self, ui: &mut Ui, listing: &Listing) {
        let count = listing.catalog.choices.len();
        if count == 0 {
            ui.label(t::none_installed());
        } else {
            ui.label(t::total_line(count, listing.total()));
        }
        ui.horizontal_wrapped(|ui| {
            let removable = listing.removable().count();
            if removable > 0 {
                let all = self.gated(ui, listing, egui::Button::new(t::remove_all_button()));
                let all = all.on_hover_text(t::remove_all_hover());
                crate::diag::ui_rect_visible(REGION_REMOVE_ALL, all.rect, ui.clip_rect());
                if all.clicked() {
                    let bytes = listing.removable().map(|(i, _)| listing.sizes[i]).sum();
                    self.ask(Confirm::All, "all", bytes);
                }
            }
            let file = self.gated(ui, listing, egui::Button::new(t::install_file_button()));
            let file = file.on_hover_text(t::install_file_hover());
            crate::diag::ui_rect_visible(REGION_INSTALL_FILE, file.rect, ui.clip_rect());
            if file.clicked()
                && let crate::app::files::Picked::Path(zip) =
                    crate::app::files::pick_ocr_addon_zip()
            {
                self.install(listing, manage::Source::Zip(zip));
            }
            let folder = self.gated(ui, listing, egui::Button::new(t::install_folder_button()));
            let folder = folder.on_hover_text(t::install_folder_hover());
            crate::diag::ui_rect_visible(REGION_INSTALL_FOLDER, folder.rect, ui.clip_rect());
            if folder.clicked()
                && let crate::app::files::Picked::Path(dir) =
                    crate::app::files::pick_ocr_addon_folder()
            {
                self.install(listing, manage::Source::Folder(dir));
            }
        });
        if self.confirm == Some(Confirm::All) {
            let bytes = listing.removable().map(|(i, _)| listing.sizes[i]).sum();
            self.confirm_strip(
                ui,
                listing,
                &t::confirm_remove_all(listing.removable().count(), bytes),
            );
        }
        if self.job.is_some() {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().color(egui_shell::Theme::of(ui.ctx()).palette.text));
                ui.label(t::working());
            });
        }
        if let Some(result) = &self.result {
            let shown = ui.label(result);
            crate::diag::ui_rect_visible(REGION_RESULT, shown.rect, ui.clip_rect());
        }
        ui.add_space(8.0);
        let close = ui.button(t::close_button());
        crate::diag::ui_rect_visible(REGION_CLOSE, close.rect, ui.clip_rect());
        if close.clicked() {
            self.close_requested = true;
        }
    }

    fn install(&mut self, listing: &Listing, source: manage::Source) {
        let Some(models) = listing.models_dir.clone() else {
            return;
        };
        self.result = None;
        self.job = Some(Job::install(source, models));
    }

    /// A button that changes the models folder: greyed, with the reason,
    /// while a job runs or when the folder cannot be written.
    fn gated(&self, ui: &mut Ui, listing: &Listing, button: egui::Button<'_>) -> egui::Response {
        let idle = self.job.is_none();
        let response = ui.add_enabled(idle && listing.writable, button);
        if !idle {
            response.on_disabled_hover_text(t::busy_hover())
        } else {
            response.on_disabled_hover_text(t::unwritable_hover())
        }
    }
}

/// Make `choice` the model Recognise text starts with: the same two
/// preferences Recognise text itself remembers.
fn set_default(prefs: &mut Prefs, choice: &Choice) {
    prefs.ocr_models.model = Some(choice.name.clone());
    if let Some(engine) = choice.engine {
        prefs.ocr_engine = Some(engine);
    }
    let saved = prefs.save().is_ok();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("ocr-models-default name={} saved={saved}", choice.name)
    });
}

/// The heading for a recogniser.
fn engine_label(token: &str) -> &str {
    crate::ocr::EngineId::from_key(token)
        .map(crate::text::ocr::engine_label)
        .or_else(|| crate::text::ocr::token_label(token))
        .unwrap_or(token)
}
