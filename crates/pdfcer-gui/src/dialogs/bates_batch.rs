//! # `dialogs::bates_batch` — **Bates-number several files in one run**
//!
//! `tools.bates_files`, Tools ▸ Batch ▸ Bates numbering…. A list of PDFs in
//! the operator's order, the numbering form Pages ▸ Stamp ▸ Bates numbering…
//! uses, and where the numbered copies go. Each file is numbered from the
//! number after the previous file's last page and written as a copy; the
//! originals are never written. Application-scoped: it works with nothing
//! open, and closing a document does not close it.
//!
//! Every file is read on a worker as soon as it is listed ([`run::probe`]), so
//! a file the engine would refuse — password, signature, certification — is
//! named in its row before anything runs, and Number files is absent until
//! the list is clean (R9). The run itself is another worker.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/bates_batch.md`.

pub mod naming;
pub mod run;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use egui::Ui;
use pdfcer_core::settings::Settings;

use super::bates::BatesForm;
use crate::text::bates_batch as t;
use naming::{Labels, NameMode, Output};
use run::{Msg, Outcome};

/// The window body's published region, for `ui-verify`.
pub const REGION_BODY: &str = "bates_batch.body";
/// The run button.
pub const REGION_RUN: &str = "bates_batch.run";
/// Row controls: `bates_batch.row.<i>.up` / `.down` / `.remove`.
pub const REGION_ROW_PREFIX: &str = "bates_batch.row.";
/// The prefix the shared numbering controls publish under.
pub const REGION_FORM: &str = "bates_batch";

/// One listed file.
struct Row {
    path: PathBuf,
    /// `None` while the worker is still reading it.
    probe: Option<Result<usize, String>>,
}

/// Where the window is.
enum Stage {
    Editing,
    Running {
        done: usize,
        of: usize,
    },
    /// The run ended; the sentence and whether it numbered everything.
    Ended(String, bool),
}

/// The batch window's state.
pub struct BatesBatchDialog {
    rows: Vec<Row>,
    form: BatesForm,
    output: Output,
    settings: Settings,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    /// Listed files not yet handed to a probe worker; spawned on the next
    /// frame, which is the first place an `egui::Context` is to hand.
    unprobed: Vec<PathBuf>,
    stage: Stage,
    /// The number after the last label a completed run wrote.
    next: Option<u64>,
    run_requested: bool,
    add_requested: bool,
    folder_requested: bool,
    close_requested: bool,
}

impl BatesBatchDialog {
    /// Open on `sources`, seeded with the numbering the last Bates window left.
    #[must_use]
    pub fn open(sources: Vec<PathBuf>, form: Option<&BatesForm>, settings: Settings) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "bates-batch-opened n={} start={}",
                sources.len(),
                form.map_or(1, BatesForm::start),
            )
        });
        let mut dialog = Self {
            rows: Vec::new(),
            form: form.cloned().unwrap_or_default(),
            output: Output::default(),
            settings,
            tx,
            rx,
            unprobed: Vec::new(),
            stage: Stage::Editing,
            next: None,
            run_requested: false,
            add_requested: false,
            folder_requested: false,
            close_requested: false,
        };
        dialog.add(sources);
        dialog
    }

    /// The numbering to seed the next Bates window with: after a completed
    /// run, the number after its last label.
    #[must_use]
    pub fn remembered(&self) -> BatesForm {
        let mut form = self.form.clone();
        if let Some(next) = self.next {
            form.set_start(next);
        }
        form
    }

    /// Append `paths` not already listed.
    fn add(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if self.rows.iter().any(|r| r.path == path) {
                continue;
            }
            self.unprobed.push(path.clone());
            self.rows.push(Row { path, probe: None });
        }
    }

    /// Every row's page count, or `None` while one is unread or refused.
    fn counts(&self) -> Option<Vec<usize>> {
        self.rows
            .iter()
            .map(|r| r.probe.as_ref().and_then(|p| p.as_ref().ok().copied()))
            .collect()
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        if !self.unprobed.is_empty() {
            let paths = std::mem::take(&mut self.unprobed);
            run::spawn_probe(ctx, self.settings.clone(), paths, self.tx.clone());
        }
        self.drain();
        let (frame, ()) = crate::dialogs::host::Host::new(
            "bates_batch", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(620.0, 640.0),
            egui::vec2(460.0, 420.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.add_requested) {
            self.add(crate::app::files::pick_bates_sources());
        }
        if std::mem::take(&mut self.folder_requested)
            && let crate::app::files::Picked::Path(folder) = crate::app::files::pick_bates_folder()
        {
            self.output.folder = Some(folder);
            self.output.one_folder = true;
        }
        if std::mem::take(&mut self.run_requested) {
            self.start(ctx);
        }
        let running = matches!(self.stage, Stage::Running { .. });
        // A run in progress keeps the window: closing it would drop the
        // receiver and leave the operator no sentence about what was written.
        running || (!frame.closed && !std::mem::take(&mut self.close_requested))
    }

    /// Take what the workers sent.
    fn drain(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(Msg::Probed(path, verdict)) => {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.path == path) {
                        row.probe = Some(verdict);
                    }
                }
                Ok(Msg::Wrote(index)) => {
                    if let Stage::Running { done, .. } = &mut self.stage {
                        *done = index + 1;
                    }
                }
                Ok(Msg::Finished(outcome)) => self.finish(outcome),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return,
            }
        }
    }

    fn finish(&mut self, outcome: Outcome) {
        self.stage = match outcome {
            Outcome::Done {
                files,
                pages,
                first,
                last,
                next,
            } => {
                self.next = Some(next);
                Stage::Ended(t::done(files, pages, &first, &last, next), true)
            }
            Outcome::Exists(names) => Stage::Ended(t::exists(&names), false),
            Outcome::Failed { name, detail } => Stage::Ended(t::failed(&name, &detail), false),
            Outcome::Partly {
                written,
                of,
                detail,
            } => Stage::Ended(t::partly_placed(written, of, &detail), false),
        };
    }

    /// The labels and targets of the batch as listed, or why it cannot run.
    fn plan(&self) -> Result<(pdfcer_core::bates::BatesStamp, Vec<run::Job>, String), String> {
        let counts = self.counts().ok_or_else(String::new)?;
        if counts.is_empty() {
            return Err(t::empty().to_owned());
        }
        let (stamp, spans) = self.form.batch(&counts)?;
        let rows: Vec<(PathBuf, Labels<'_>)> = self
            .rows
            .iter()
            .zip(&spans)
            .map(|(r, s)| {
                (
                    r.path.clone(),
                    Labels {
                        first: &s.first_label,
                        last: &s.last_label,
                    },
                )
            })
            .collect();
        let targets = self.output.plan(&rows)?;
        let next = spans.last().map_or(0, |s| s.next);
        let summary = t::summary(
            spans.len(),
            spans.first().map_or("", |s| s.first_label.as_str()),
            spans.last().map_or("", |s| s.last_label.as_str()),
            next,
        );
        let jobs = self
            .rows
            .iter()
            .zip(spans)
            .zip(targets)
            .map(|((r, s), target)| run::Job {
                source: r.path.clone(),
                target,
                first: s.first,
            })
            .collect();
        Ok((stamp, jobs, summary))
    }

    fn start(&mut self, ctx: &egui::Context) {
        let Ok((stamp, jobs, _)) = self.plan() else {
            return;
        };
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "bates-batch-run n={} first={} replace={}",
                jobs.len(),
                jobs.first().map_or(0, |j| j.first),
                u8::from(self.output.replace),
            )
        });
        self.stage = Stage::Running {
            done: 0,
            of: jobs.len(),
        };
        run::spawn_run(
            ctx,
            self.settings.clone(),
            jobs,
            stamp,
            self.output.replace,
            self.tx.clone(),
        );
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(6.0);
        let editing = matches!(self.stage, Stage::Editing | Stage::Ended(..));
        ui.add_enabled_ui(editing, |ui| {
            self.file_list(ui);
            ui.add_space(6.0);
            self.form.controls(ui, REGION_FORM);
            ui.add_space(6.0);
            self.output_controls(ui);
        });
        ui.add_space(6.0);
        let plan = self.plan();
        match (&self.stage, &plan) {
            (Stage::Running { done, of }, _) => {
                ui.label(t::running(*done, *of));
            }
            (Stage::Ended(sentence, ok), _) => {
                let text = egui::RichText::new(sentence);
                ui.label(if *ok {
                    text
                } else {
                    text.color(ui.visuals().error_fg_color)
                });
            }
            (Stage::Editing, Ok((_, _, summary))) => {
                ui.label(summary);
            }
            (Stage::Editing, Err(why)) if !why.is_empty() => {
                ui.label(egui::RichText::new(why).color(ui.visuals().error_fg_color));
            }
            (Stage::Editing, Err(_)) => {}
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::close()).clicked() {
                self.close_requested = true;
            }
            // Absent, not greyed, while the batch cannot run: the line above
            // says why (R9). After a run it comes back for the next batch.
            if editing && plan.is_ok() {
                let r = ui.button(t::run());
                crate::diag::ui_rect(REGION_RUN, r.rect);
                if r.clicked() {
                    self.stage = Stage::Editing;
                    self.run_requested = true;
                }
            }
        });
    }

    /// The listed files, each with its labels and the copy it becomes.
    fn file_list(&mut self, ui: &mut Ui) {
        ui.label(egui::RichText::new(t::files_heading(self.rows.len())));
        if self.rows.is_empty() {
            ui.label(egui::RichText::new(t::empty()).weak());
        }
        let spans = self
            .counts()
            .and_then(|c| self.form.batch(&c).ok())
            .map(|(_, s)| s);
        let mut moved: Option<(usize, isize)> = None;
        let mut removed: Option<usize> = None;
        let last = self.rows.len().saturating_sub(1);
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (i, row) in self.rows.iter().enumerate() {
                    let name = row
                        .path
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    ui.horizontal(|ui| {
                        let id = format!("{REGION_ROW_PREFIX}{i}");
                        let up = ui.add_enabled(i > 0, egui::Button::new(t::up()).small());
                        crate::diag::ui_rect(&format!("{id}.up"), up.rect);
                        let down = ui.add_enabled(i < last, egui::Button::new(t::down()).small());
                        crate::diag::ui_rect(&format!("{id}.down"), down.rect);
                        let remove = ui.add(egui::Button::new(t::remove()).small());
                        crate::diag::ui_rect(&format!("{id}.remove"), remove.rect);
                        if up.clicked() {
                            moved = Some((i, -1));
                        }
                        if down.clicked() {
                            moved = Some((i, 1));
                        }
                        if remove.clicked() {
                            removed = Some(i);
                        }
                        ui.label(&name);
                    });
                    let detail = match (&row.probe, &spans) {
                        (None, _) => t::reading().to_owned(),
                        (Some(Err(why)), _) => t::refused(&name, why),
                        (Some(Ok(n)), Some(s)) => {
                            let span = &s[i];
                            let labels = Labels {
                                first: &span.first_label,
                                last: &span.last_label,
                            };
                            format!(
                                "{}  {}", // ui-text-exempt: joins two catalog strings
                                t::row_range(*n, &span.first_label, &span.last_label),
                                t::row_target(&self.output.name(&row.path, &labels))
                            )
                        }
                        (Some(Ok(n)), None) => t::row_pages(*n),
                    };
                    let refused = matches!(row.probe, Some(Err(_)));
                    let text = egui::RichText::new(detail).small();
                    ui.label(if refused {
                        text.color(ui.visuals().error_fg_color)
                    } else {
                        text.weak()
                    });
                }
            });
        if let Some(i) = removed {
            self.rows.remove(i);
        } else if let Some((i, by)) = moved {
            let j = i.saturating_add_signed(by);
            if j < self.rows.len() {
                self.rows.swap(i, j);
            }
        }
        if ui.button(t::add()).clicked() {
            self.add_requested = true;
        }
    }

    /// The folder and naming choices.
    fn output_controls(&mut self, ui: &mut Ui) {
        ui.label(egui::RichText::new(t::output_heading()));
        ui.radio_value(&mut self.output.one_folder, false, t::beside());
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.output.one_folder,
                true,
                t::in_folder(self.output.folder.as_deref()),
            );
            if ui.button(t::choose_folder()).clicked() {
                self.folder_requested = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label(t::naming());
            egui::ComboBox::from_id_salt("bates_batch.naming") // ui-text-exempt: widget id
                .selected_text(mode_name(self.output.mode))
                .show_ui(ui, |ui| {
                    for mode in NameMode::ALL {
                        ui.selectable_value(&mut self.output.mode, mode, mode_name(mode));
                    }
                });
        });
        if self.output.mode == NameMode::Affix {
            egui::Grid::new("bates_batch.affix") // ui-text-exempt: widget id
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(t::before());
                    ui.text_edit_singleline(&mut self.output.before);
                    ui.end_row();
                    ui.label(t::after());
                    ui.text_edit_singleline(&mut self.output.after);
                    ui.end_row();
                });
        }
        ui.checkbox(&mut self.output.replace, t::replace_existing())
            .on_hover_text(t::replace_tooltip());
    }
}

const fn mode_name(mode: NameMode) -> &'static str {
    match mode {
        NameMode::Keep => t::name_keep(),
        NameMode::Affix => t::name_affix(),
        NameMode::Range => t::name_range(),
        NameMode::RangeAndName => t::name_range_and_name(),
    }
}
