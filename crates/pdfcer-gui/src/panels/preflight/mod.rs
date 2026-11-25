//! # `panels::preflight` — the document checked against a preflight profile
//!
//! Pick a built-in profile, run it on a worker over a fork of the session,
//! read the findings grouped by rule, jump to one, copy the report, and fix
//! a rule or everything as one undo step each.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/preflight.md`.

mod job;
mod results;
mod standard;

use std::time::Instant;

use pdfcer_core::preflight::{FindingKind, PreflightReport, Profile, report_text};
use pdfcer_gui_base::preflightfix::PreflightFix;

use self::job::{Job, Poll};
use self::results::Group;
use super::PanelsState;
use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::panels::preflight as t;

/// The profile picked when the panel first opens.
const DEFAULT_PROFILE: &str = "digital-printing"; // ui-text-exempt: an engine profile name

const REGION_BODY: &str = "preflight.body"; // ui-text-exempt: diagnostic region name
const REGION_RUN: &str = "preflight.run"; // ui-text-exempt: diagnostic region name
const REGION_CANCEL: &str = "preflight.cancel"; // ui-text-exempt: diagnostic region name
const REGION_FIX_ALL: &str = "preflight.fix_all"; // ui-text-exempt: diagnostic region name
const REGION_COPY: &str = "preflight.copy"; // ui-text-exempt: diagnostic region name
/// The profile picker; each entry is `preflight.profile:<engine name>`.
const REGION_PROFILE: &str = "preflight.profile"; // ui-text-exempt: diagnostic region name

/// The panel's state between frames. Reset with the document.
pub struct PreflightUi {
    /// The built-in profile name in the picker.
    profile: &'static str,
    pending: Option<Pending>,
    results: Option<Results>,
    /// The last run's worker went away without an answer.
    lost: bool,
}

impl Default for PreflightUi {
    fn default() -> Self {
        Self {
            profile: DEFAULT_PROFILE,
            pending: None,
            results: None,
            lost: false,
        }
    }
}

/// A run in flight, and the document state it was started on.
struct Pending {
    job: Job,
    serial: u64,
    epoch: u64,
    profile: Profile,
    /// The `Profile::BUILTIN_NAMES` key, which the trace names: a profile's
    /// display name holds spaces, so it cannot be a trace field.
    key: &'static str,
    began: Instant,
    /// UI frames drawn while the run was still going — the evidence, read
    /// by the driven check, that the window kept answering.
    frames: u32,
}

/// A finished run of one open document at one edit epoch.
struct Results {
    serial: u64,
    epoch: u64,
    profile: Profile,
    report: PreflightReport,
    groups: Vec<Group>,
    standard: Option<job::Standard>,
}

impl PreflightUi {
    /// Forget a run of a different document.
    fn sync(&mut self, doc: &OpenDoc) {
        if self
            .pending
            .as_ref()
            .is_some_and(|p| p.serial != doc.serial)
        {
            self.pending = None;
        }
        if self
            .results
            .as_ref()
            .is_some_and(|r| r.serial != doc.serial)
        {
            self.results = None;
        }
    }

    /// Start a run of the picked profile over the document as it stands.
    fn start(&mut self, doc: &OpenDoc, ctx: &egui::Context) {
        let Some(profile) = Profile::builtin(self.profile) else {
            return;
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "preflight-started profile={} pages={} epoch={}",
                self.profile,
                doc.pages.len(),
                doc.edit_epoch
            )
        });
        let fork = doc.session.fork_for_print();
        self.lost = false;
        self.pending = Some(Pending {
            job: Job::spawn(fork, profile.clone(), self.profile, ctx.clone()),
            serial: doc.serial,
            epoch: doc.edit_epoch,
            profile,
            key: self.profile,
            began: Instant::now(),
            frames: 0,
        });
    }

    /// Collect a finished run; count a frame drawn while it is still going.
    fn poll(&mut self) {
        let Some(pending) = &mut self.pending else {
            return;
        };
        let report = match pending.job.poll() {
            Poll::Running => {
                pending.frames += 1;
                return;
            }
            Poll::Lost => None,
            Poll::Done(answer) => Some(answer),
        };
        let pending = self.pending.take().expect("checked above"); // ui-text-exempt: panic message, never displayed
        let Some(job::Answer { report, standard }) = report else {
            self.lost = true;
            return;
        };
        trace_finished(&pending, &report);
        let groups = results::groups(&pending.profile, &report);
        results::trace_rows(&report, &groups);
        if let Some(s) = &standard {
            standard::trace(s);
        }
        self.results = Some(Results {
            serial: pending.serial,
            epoch: pending.epoch,
            profile: pending.profile,
            report,
            groups,
            standard,
        });
    }
}

fn trace_finished(pending: &Pending, report: &PreflightReport) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "preflight-finished profile={} findings={} dropped={} pages={}/{} ms={} cancelled={} frames={}",
            pending.key,
            report.findings.len(),
            report.dropped,
            report.pages_checked,
            report.page_count,
            pending.began.elapsed().as_millis(),
            u8::from(report.cancelled),
            pending.frames
        )
    });
}

/// Draw the Preflight panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    crate::diag::ui_rect(REGION_BODY, ui.max_rect());
    let pf = state.preflight_mut();
    pf.sync(doc);
    pf.poll();
    run_row(ui, doc, pf);
    if pf.lost {
        ui.label(egui::RichText::new(t::worker_lost()).weak());
    }
    let fixing = crate::canvas::tool::capabilities(ui.ctx()).edit_content;
    if let Some(results) = &pf.results {
        ui.separator();
        results_view(ui, doc, results, fixing, actions);
    }
    fix_note(ui, doc, pf.results.as_ref());
}

/// The profile picker and Run, or the progress line and Cancel.
fn run_row(ui: &mut egui::Ui, doc: &OpenDoc, pf: &mut PreflightUi) {
    let running = pf.pending.is_some();
    ui.horizontal(|ui| {
        ui.label(t::profile_label());
        ui.add_enabled_ui(!running, |ui| {
            let picker = egui::ComboBox::from_id_salt(REGION_PROFILE)
                .selected_text(t::profile_name(pf.profile))
                // Every built-in profile at once: the default height hides the PDF/X ones.
                .height(f32::INFINITY)
                .show_ui(ui, |ui| {
                    for name in Profile::BUILTIN_NAMES {
                        let hover = Profile::builtin(name)
                            .map(|p| p.description)
                            .unwrap_or_default();
                        let entry = ui
                            .selectable_value(&mut pf.profile, name, t::profile_name(name))
                            .on_hover_text(hover);
                        crate::diag::ui_rect(&format!("{REGION_PROFILE}:{name}"), entry.rect);
                    }
                })
                .response;
            crate::diag::ui_rect(REGION_PROFILE, picker.rect);
        });
    });
    if let Some(pending) = &pf.pending {
        let (done, of) = pending.job.progress();
        crate::diag::trace_changed("preflight-progress", || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("preflight-progress done={done} of={of}")
        });
        ui.horizontal(|ui| {
            ui.add(egui::Spinner::new().color(ui.visuals().text_color()));
            ui.label(t::progress(done, of.max(doc.pages.len())));
            let cancel = ui.button(t::cancel());
            crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
            if cancel.clicked() {
                pending.job.cancel();
            }
        });
        return;
    }
    let stale = pf
        .results
        .as_ref()
        .is_some_and(|r| r.epoch != doc.edit_epoch);
    let label = if stale { t::run_again() } else { t::run() };
    let run = ui.button(label).on_hover_text(t::run_hover());
    crate::diag::ui_rect(REGION_RUN, run.rect);
    if run.clicked() {
        pf.start(doc, ui.ctx());
    }
}

/// The headline, the disclosures, Copy report, Fix all and the groups.
fn results_view(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    results: &Results,
    fixing: bool,
    actions: &mut Vec<Action>,
) {
    let report = &results.report;
    let stale = results.epoch != doc.edit_epoch;
    if stale {
        ui.label(egui::RichText::new(t::stale()));
    }
    let fix_reason = if !fixing {
        Some(t::fix_needs_edit())
    } else if stale {
        Some(t::fix_needs_fresh())
    } else {
        None
    };
    if let Some(s) = &results.standard {
        let convert_off = (!fixing).then(t::fix_needs_edit);
        standard::view(ui, s, convert_off, actions);
    }
    headline(ui, report);
    ui.horizontal(|ui| {
        let copy = ui
            .button(t::copy_report())
            .on_hover_text(t::copy_report_hover());
        crate::diag::ui_rect(REGION_COPY, copy.rect);
        if copy.clicked() {
            let text = report_text(report, &results.profile);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("preflight-copied bytes={}", text.len())
            });
            ui.ctx().copy_text(text);
        }
        if results.groups.iter().any(|g| g.fixable) {
            let all = fix_button(ui, t::fix_all(), t::fix_all_hover(), fix_reason);
            crate::diag::ui_rect(REGION_FIX_ALL, all.rect);
            if all.clicked() {
                actions.push(fix_action(results, None));
            }
        }
    });
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let mut row = 0;
            for group in &results.groups {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let n = group.findings.len();
                    ui.label(egui::RichText::new(t::rule_heading(
                        &group.id,
                        group.severity,
                        n,
                    )));
                    if group.fixable {
                        let fix = fix_button(ui, t::fix(), t::fix_hover(), fix_reason);
                        // ui-text-exempt: diagnostic region name, never displayed
                        crate::diag::ui_rect(&format!("preflight.fix.{}", group.id), fix.rect);
                        if fix.clicked() {
                            actions.push(fix_action(results, Some(group.id.clone())));
                        }
                    }
                });
                results::draw_rows(ui, doc, report, group, &mut row, actions);
            }
        });
}

/// Clean or not clean, the kinds counted, and what the run left out.
fn headline(ui: &mut egui::Ui, report: &PreflightReport) {
    if report.cancelled {
        ui.label(egui::RichText::new(t::cancelled(
            report.pages_checked,
            report.page_count,
        )));
    }
    if report.is_clean() {
        ui.label(egui::RichText::new(t::clean(report.pages_checked)));
    } else if !report.findings.is_empty() {
        let count = |k: FindingKind| report.findings.iter().filter(|f| f.kind == k).count();
        let v = count(FindingKind::Violation);
        let p = count(FindingKind::NeedsPerson);
        let i = count(FindingKind::Incomplete);
        let other = report.findings.len() - v - p - i;
        ui.label(egui::RichText::new(t::summary(v, p, i, other)));
    }
    if report.dropped > 0 {
        ui.label(t::dropped(report.dropped));
    }
}

/// A fix button, greyed with its reason when fixing is off.
fn fix_button(ui: &mut egui::Ui, label: &str, hover: &str, off: Option<&str>) -> egui::Response {
    match off {
        None => ui.button(label).on_hover_text(hover),
        Some(why) => ui
            .add_enabled(false, egui::Button::new(label))
            .on_disabled_hover_text(why),
    }
}

fn fix_action(results: &Results, rule: Option<String>) -> Action {
    Action::PreflightFix(Box::new(PreflightFix {
        profile: results.profile.clone(),
        report: results.report.clone(),
        rule,
    }))
}

/// The last fix's report, until a run starts after it.
fn fix_note(ui: &mut egui::Ui, doc: &OpenDoc, results: Option<&Results>) {
    let Some(note) = &doc.preflight_fixed else {
        return;
    };
    if results.is_some_and(|r| r.epoch >= note.epoch) {
        return;
    }
    ui.separator();
    ui.label(egui::RichText::new(t::fixed_heading()));
    ui.label(egui::RichText::new(&note.text).monospace());
}
