//! # `panels::compare` — Compare documents
//!
//! Acrobat's Compare Files: the open document is the newer version, the
//! operator picks the older one, chooses text and/or appearance and the
//! pages, and gets a list of changes. A row click shows the change on the
//! page; `Mark changes as comments` writes them to this document through the
//! engine's marking verbs. Nothing is drawn on the canvas but what the engine
//! writes; what the comparison inferred is listed here.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/compare.md`.

pub mod job;
pub mod rows;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

use pdfcer_gui_base::newcomment::{ComparisonMarks, NewComment};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::compare as t;
use job::{Found, Newer, Request};
use rows::{Row, RowKind};

/// The ribbon command that opens this panel and picks the older version.
pub const COMMAND_ID: &str = "tools.compare"; // ui-text-exempt: command id, never displayed
/// The panel body.
pub const REGION_BODY: &str = "panel:compare"; // ui-text-exempt: trace region name, never displayed
/// The Compare button.
pub const REGION_RUN: &str = "compare.run"; // ui-text-exempt: trace region name, never displayed
/// The mark button.
pub const REGION_MARK: &str = "compare.mark"; // ui-text-exempt: trace region name, never displayed
/// One row, suffixed with its 0-based index in the list.
pub const REGION_ROW_PREFIX: &str = "compare.row:"; // ui-text-exempt: trace region name, never displayed
/// Why a row click reveals; trace only.
const WHY_ROW: &str = "compare-row"; // ui-text-exempt: diagnostic token, never displayed

/// What to compare. They answer to the operator, so a new document keeps
/// them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub text: bool,
    pub formatting: bool,
    pub appearance: bool,
    /// Page ranges as typed; empty is every page.
    pub old_pages: String,
    pub new_pages: String,
    /// Insert the engine's summary page when marking.
    pub summary_page: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            text: true,
            formatting: false,
            appearance: true,
            old_pages: String::new(),
            new_pages: String::new(),
            summary_page: false,
        }
    }
}

/// A comparison under way, with what it was asked of.
struct Pending {
    answer: Receiver<Result<Found, String>>,
    serial: u64,
    epoch: u64,
    older_name: String,
}

/// A finished comparison of one open document at one edit epoch.
struct Results {
    serial: u64,
    epoch: u64,
    older_name: String,
    newer_name: String,
    found: Found,
    rows: Vec<Row>,
}

/// The panel's state between frames.
#[derive(Default)]
pub struct CompareUi {
    /// The older version, once picked.
    pub older: Option<PathBuf>,
    pub options: Options,
    pending: Option<Pending>,
    results: Option<Results>,
    /// Why the last comparison produced nothing.
    error: Option<String>,
}

impl CompareUi {
    /// Start comparing `self.older` with `doc`, replacing any earlier result.
    /// A document with edits not on disk is compared as it stands: its
    /// incremental save is taken here, on the UI thread, because the worker
    /// must never hold the session.
    pub fn start(&mut self, doc: &OpenDoc, ctx: &egui::Context) {
        let Some(older) = self.older.clone() else {
            return;
        };
        self.error = None;
        let newer =
            if crate::app::save::has_unsaved_edits(doc) || !crate::app::save::has_a_file(doc) {
                match doc.session.to_incremental_bytes(
                    &pdfcer_gui_base::settings::SettingsExt::save_options(&doc.settings),
                ) {
                    Ok((bytes, _report)) => Newer::Bytes(bytes),
                    Err(e) => {
                        self.error = Some(t::newer_unreadable(&e.to_string()));
                        return;
                    }
                }
            } else {
                Newer::File(doc.path.clone())
            };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "compare-started bytes={} text={} appearance={}",
                u8::from(matches!(newer, Newer::Bytes(_))),
                u8::from(self.options.text),
                u8::from(self.options.appearance)
            )
        });
        let request = Request {
            older: older.clone(),
            newer,
            options: self.options.clone(),
        };
        self.pending = Some(Pending {
            answer: job::spawn(request, ctx.clone()),
            serial: doc.serial,
            epoch: doc.edit_epoch,
            older_name: name_of(&older),
        });
    }

    /// Collect a finished comparison.
    fn poll(&mut self, doc: &OpenDoc) {
        let Some(pending) = &self.pending else {
            return;
        };
        let answer = match pending.answer.try_recv() {
            Ok(answer) => answer,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err(t::text_failed(t::worker_lost())),
        };
        let pending = self.pending.take().expect("checked above"); // ui-text-exempt: panic message, never displayed
        match answer {
            Ok(found) => {
                let rows = rows::rows(found.text.as_ref(), found.visual.as_ref());
                trace_rows(&rows);
                self.results = Some(Results {
                    serial: pending.serial,
                    epoch: pending.epoch,
                    older_name: pending.older_name,
                    newer_name: name_of(&doc.path),
                    found,
                    rows,
                });
            }
            Err(why) => {
                self.results = None;
                self.error = Some(why);
            }
        }
    }
}

fn name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// One line naming every row's kind and page, so a check reads the list.
fn trace_rows(rows: &[Row]) {
    crate::diag::trace(|| {
        let list: Vec<String> = rows
            .iter()
            .map(|r| match r.at {
                Some((page, _)) => format!("{}@{}", r.kind.token(), page + 1),
                None => format!("{}@-", r.kind.token()),
            })
            .collect();
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "compare-listed rows={} kinds={}",
            rows.len(),
            list.join(",")
        )
    });
}

/// Draw the Compare panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    crate::diag::ui_rect(REGION_BODY, ui.max_rect());
    let ui_state = state.compare_mut();
    ui_state.poll(doc);
    older_row(ui, ui_state, actions);
    ui.separator();
    options(ui, &mut ui_state.options);
    run_row(ui, doc, ui_state);
    if let Some(why) = &ui_state.error {
        ui.label(egui::RichText::new(why).weak());
    }
    if let Some(results) = &ui_state.results {
        ui.separator();
        results_view(ui, doc, results, &mut ui_state.options, actions);
    }
}

fn older_row(ui: &mut egui::Ui, state: &CompareUi, actions: &mut Vec<Action>) {
    ui.label(t::older_heading());
    ui.horizontal(|ui| {
        let name = state.older.as_deref().map(name_of);
        let line = name.as_deref().unwrap_or_else(|| t::older_none());
        let label = ui.add(egui::Label::new(line).truncate());
        if let Some(path) = &state.older {
            label.on_hover_text(path.display().to_string());
        }
        if ui
            .button(t::choose())
            .on_hover_text(t::choose_hover())
            .clicked()
        {
            actions.push(Action::Command(COMMAND_ID.to_owned()));
        }
    });
}

fn options(ui: &mut egui::Ui, o: &mut Options) {
    ui.checkbox(&mut o.text, t::opt_text())
        .on_hover_text(t::opt_text_hover());
    ui.add_enabled_ui(o.text, |ui| {
        ui.indent("compare-formatting", |ui| {
            ui.checkbox(&mut o.formatting, t::opt_formatting())
                .on_hover_text(t::opt_formatting_hover());
        });
    });
    ui.checkbox(&mut o.appearance, t::opt_appearance())
        .on_hover_text(t::opt_appearance_hover());
    egui::Grid::new("compare-pages")
        .num_columns(2)
        .show(ui, |ui| {
            for (label, spec) in [
                (t::old_pages(), &mut o.old_pages),
                (t::new_pages(), &mut o.new_pages),
            ] {
                ui.label(label);
                // escape-disposition: keeps-draft — the range lives in the
                // panel's options and is read only when Compare runs.
                ui.add(egui::TextEdit::singleline(spec).desired_width(90.0))
                    .on_hover_text(t::pages_hover());
                ui.end_row();
            }
        });
}

fn run_row(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut CompareUi) {
    if state.pending.is_some() {
        ui.horizontal(|ui| {
            ui.add(egui::Spinner::new().color(egui_shell::Theme::of(ui.ctx()).palette.text));
            ui.label(t::running());
        });
        return;
    }
    let why_not = if state.older.is_none() {
        Some(t::run_needs_older())
    } else if !state.options.text && !state.options.appearance {
        Some(t::run_needs_option())
    } else {
        None
    };
    let run = ui.add_enabled(why_not.is_none(), egui::Button::new(t::run()));
    crate::diag::ui_rect(REGION_RUN, run.rect);
    let run = match why_not {
        Some(why) => run.on_disabled_hover_text(why),
        None => run.on_hover_text(t::run_hover()),
    };
    if run.clicked() {
        state.start(doc, ui.ctx());
    }
}

fn results_view(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    results: &Results,
    options: &mut Options,
    actions: &mut Vec<Action>,
) {
    if results.serial != doc.serial {
        ui.label(t::other_document(&results.newer_name));
        return;
    }
    let text = results.found.text.as_ref().map_or(0, |r| r.changes.len());
    let pages = results
        .rows
        .iter()
        .filter(|r| matches!(r.kind, RowKind::PageAdded | RowKind::PageRemoved))
        .count();
    let areas = results
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Area)
        .count();
    if results.rows.is_empty() {
        ui.label(t::identical());
    } else {
        ui.label(t::counts(text, areas, pages));
    }
    for note in &results.found.notes {
        ui.label(egui::RichText::new(note).small().weak());
    }
    mark_row(ui, doc, results, options, actions);
    egui::ScrollArea::vertical()
        .id_salt("compare-rows")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for (index, row) in results.rows.iter().enumerate() {
                row_view(ui, doc, index, row, actions);
            }
        });
}

fn mark_row(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    results: &Results,
    options: &mut Options,
    actions: &mut Vec<Action>,
) {
    let marks =
        results.found.text.as_ref().map_or(0, |r| r.changes.len()) + results.found.regions.len();
    let stale = results.epoch != doc.edit_epoch;
    ui.checkbox(&mut options.summary_page, t::opt_summary())
        .on_hover_text(t::opt_summary_hover());
    let mark = ui.add_enabled(marks > 0 && !stale, egui::Button::new(t::mark()));
    crate::diag::ui_rect(REGION_MARK, mark.rect);
    let mark = if stale {
        mark.on_disabled_hover_text(t::stale())
    } else if marks == 0 {
        mark.on_disabled_hover_text(t::mark_nothing())
    } else {
        mark.on_hover_text(t::mark_hover())
    };
    if mark.clicked() {
        let marks = ComparisonMarks {
            report: results.found.text.clone(),
            regions: results.found.regions.clone(),
            labels: (
                t::label_older(&results.older_name),
                t::label_newer(&results.newer_name),
            ),
            summary_page: options.summary_page,
            summary_note: results
                .found
                .visual
                .as_ref()
                .map(|v| t::summary_note(v.region_count(), &rows::visual_notes(v))),
        };
        actions.push(Action::NewComment(NewComment::ComparisonMarks(Box::new(
            marks,
        ))));
    }
}

fn row_view(ui: &mut egui::Ui, doc: &OpenDoc, index: usize, row: &Row, actions: &mut Vec<Action>) {
    let response = ui
        .horizontal(|ui| {
            let (swatch, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            if let Some(colour) = row.kind.swatch() {
                ui.painter().rect_filled(swatch, 2.0, colour);
            }
            let page = row.at.map(|(p, _)| t::page_cell(p + 1)).unwrap_or_default();
            ui.add(egui::Label::new(row.kind.label()).selectable(false));
            ui.add(egui::Label::new(page).selectable(false));
            ui.add(egui::Label::new(&row.line).truncate().selectable(false));
        })
        .response
        .interact(egui::Sense::click());
    crate::diag::ui_rect_visible(
        &format!("{REGION_ROW_PREFIX}{index}"),
        response.rect,
        ui.clip_rect(),
    );
    let hover = match (row.at, row.kind) {
        (None, _) => t::row_older_only().to_owned(),
        (Some(_), RowKind::Deleted) => t::deleted_row_hover(),
        _ => t::row_hover().to_owned(),
    };
    let response = response.on_hover_text(hover);
    if response.clicked() {
        jump(doc, index, row, actions);
    }
}

/// Show `row`'s place: turn to its page and bring its rectangle into view.
fn jump(doc: &OpenDoc, index: usize, row: &Row, actions: &mut Vec<Action>) {
    let Some((page, rect)) = row.at else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "compare-jump row={index} kind={} page={}",
            row.kind.token(),
            page + 1
        )
    });
    if doc.view.page_index != page {
        actions.push(Action::GoToPage(page));
    }
    let Some(info) = doc.pages.get(page) else {
        return;
    };
    // A page with no counterpart carries no area: turning to it is the jump.
    if rect.urx <= rect.llx && rect.ury <= rect.lly {
        return;
    }
    if let Some(canvas) =
        crate::canvas::geometry::pdf_rect_to_canvas((rect.llx, rect.lly, rect.urx, rect.ury), info)
    {
        let (min, max) = crate::canvas::minreveal::fracs_for_canvas_rect(canvas, info);
        actions.push(Action::RevealRect {
            page,
            min,
            max,
            why: WHY_ROW,
        });
    }
}
