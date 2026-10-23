//! # `dialogs::deskew` — File ▸ Straighten scans and File ▸ Enlarge scans
//!
//! Contract: one window runs either [`ScanKind`] over the pictures the
//! operator picks with Recognise text's [`PageScope`], or — when pictures are
//! selected on the current page — only those pictures. Straighten also asks
//! whether pages that already have text are skipped (on by default: text
//! does not turn with its picture) and how sure a tilt must be; Enlarge asks
//! the factor and kernel, with Recognise text's controls. The run queues one
//! [`DeskewStep::Page`] per page or picture and raises the next only once the
//! previous one's outcome is logged, so each frame does at most one page and
//! the window keeps painting; Stop ends the queue after the page in hand.
//! The run ends with [`DeskewStep::Finish`], which folds it into one undo
//! entry. The window then lists what was done to each page.

use std::sync::atomic::{AtomicU64, Ordering};

use pdfcer_core::ocr::upscale::UpscaleFilter;
use pdfcer_core::vector::{ImageSource, VectorObject};

use crate::app::actions::Action;
use crate::app::actions::deskew::{self as step, Folded, Logged, Outcome};
use crate::app::state::{OpenDoc, Status};
use crate::text::deskew as t;
use crate::text::enlarge as e;
use pdfcer_gui_base::deskewstep::{DeskewStep, ScanJob};
use pdfcer_gui_base::prefs::Prefs;

use super::page_scope::PageScope;

/// The window body.
pub const REGION_BODY: &str = "deskew.body"; // ui-text-exempt: trace region name, never displayed
/// The page-scope group.
pub const REGION_SCOPE: &str = "deskew.scope"; // ui-text-exempt: trace region name, never displayed
/// The skip-pages-with-text checkbox.
pub const REGION_SKIP_TEXT: &str = "deskew.skip-text"; // ui-text-exempt: trace region name, never displayed
/// The least-confidence slider.
pub const REGION_MIN_CONFIDENCE: &str = "deskew.min-confidence"; // ui-text-exempt: trace region name, never displayed
/// Enlarge's Lanczos choice.
pub const REGION_LANCZOS: &str = "deskew.lanczos"; // ui-text-exempt: trace region name, never displayed
/// The only-the-selected-pictures checkbox.
pub const REGION_SELECTED: &str = "deskew.selected"; // ui-text-exempt: trace region name, never displayed
/// The button that starts the run.
pub const REGION_COMMIT: &str = "deskew.commit"; // ui-text-exempt: trace region name, never displayed
/// The Stop button.
pub const REGION_STOP: &str = "deskew.stop"; // ui-text-exempt: trace region name, never displayed

/// Which command opened the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanKind {
    /// File ▸ Straighten scans.
    Straighten,
    /// File ▸ Enlarge scans.
    Enlarge,
}

/// Run ids, unique for the process so two windows' logs never mix.
static NEXT_RUN: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
enum Phase {
    Choosing,
    Running {
        run: u64,
        queue: Vec<DeskewStep>,
        raised: usize,
        stopping: bool,
    },
    Finished {
        run: u64,
        of: usize,
    },
}

/// The window.
#[derive(Debug)]
pub struct DeskewDialog {
    kind: ScanKind,
    /// `OpenDoc::serial` of the document it was opened on.
    doc: u64,
    scope: PageScope,
    page_count: usize,
    /// The page the selection is on, and its selected images' indices.
    selected: (usize, Vec<usize>),
    only_selected: bool,
    skip_text: bool,
    /// Least confidence to turn, percent; seeded from and saved to
    /// `Prefs::deskew_min_confidence`.
    min_confidence: u8,
    /// Enlarge's factor and kernel.
    factor: f32,
    filter: UpscaleFilter,
    phase: Phase,
    close_requested: bool,
}

impl DeskewDialog {
    fn open(doc: &OpenDoc, picked: Vec<usize>, min_confidence: u8, kind: ScanKind) -> Self {
        let page = doc.view.page_index;
        let selected = selected_images(doc, page);
        Self {
            kind,
            doc: doc.serial,
            scope: PageScope::new(page, picked),
            page_count: doc.pages.len(),
            only_selected: !selected.is_empty(),
            selected: (page, selected),
            skip_text: true,
            min_confidence,
            factor: super::ocr_upscale::DEFAULT_FACTOR,
            filter: UpscaleFilter::default(),
            phase: Phase::Choosing,
            close_requested: false,
        }
    }

    /// What each turn does, as the window is set.
    fn job(&self) -> ScanJob {
        match self.kind {
            ScanKind::Straighten => ScanJob::Straighten {
                skip_text: self.skip_text,
                min_confidence_percent: self.min_confidence,
            },
            ScanKind::Enlarge => ScanJob::Enlarge {
                factor_halves: halves(self.factor),
                filter: self.filter,
            },
        }
    }

    /// The turns the current answer names.
    fn queue(&self, run: u64) -> Vec<DeskewStep> {
        let job = self.job();
        if self.only_selected && !self.selected.1.is_empty() {
            let page = self.selected.0;
            return self
                .selected
                .1
                .iter()
                .map(|&i| DeskewStep::Page {
                    run,
                    doc: self.doc,
                    page,
                    object: Some(i),
                    job,
                })
                .collect();
        }
        self.scope
            .pages(self.page_count)
            .unwrap_or_default()
            .into_iter()
            .map(|page| DeskewStep::Page {
                run,
                doc: self.doc,
                page,
                object: None,
                job,
            })
            .collect()
    }

    /// Draw it. `active` is the `OpenDoc::serial` on screen; the run waits
    /// while it is not this window's document. Returns whether it stays open.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        active: u64,
        actions: &mut Vec<Action>,
        prefs: &mut Prefs,
    ) -> bool {
        let here = active == self.doc;
        if here {
            self.advance(actions);
        }
        let (frame, ()) = crate::dialogs::host::Host::new(
            "deskew", // ui-text-exempt: a viewport key, never displayed.
            match self.kind {
                ScanKind::Straighten => t::title(),
                ScanKind::Enlarge => e::title(),
            },
            egui::vec2(480.0, 440.0),
            egui::vec2(360.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            match self.phase {
                Phase::Choosing => self.choosing(ui, prefs),
                Phase::Running { .. } => self.running(ui, here),
                Phase::Finished { run, of } => self.finished(ui, run, of),
            }
        });
        if matches!(self.phase, Phase::Running { .. }) {
            ctx.request_repaint();
        }
        let open = !frame.closed && !std::mem::take(&mut self.close_requested);
        if !open {
            self.close(actions);
        }
        open
    }

    /// Raise the next turn once the previous one has landed.
    fn advance(&mut self, actions: &mut Vec<Action>) {
        let Phase::Running {
            run,
            ref queue,
            ref mut raised,
            stopping,
        } = self.phase
        else {
            return;
        };
        let landed = step::outcomes(run).len();
        if landed < *raised {
            return;
        }
        if stopping || *raised == queue.len() {
            let of = queue.len();
            actions.push(Action::Deskew(DeskewStep::Finish { run, doc: self.doc }));
            self.phase = Phase::Finished { run, of };
            return;
        }
        actions.push(Action::Deskew(queue[*raised].clone()));
        *raised += 1;
    }

    /// Closing mid-run ends the run where it is: what was done is kept and
    /// folded, as Stop does.
    fn close(&mut self, actions: &mut Vec<Action>) {
        match self.phase {
            Phase::Running { run, .. } => {
                actions.push(Action::Deskew(DeskewStep::Finish { run, doc: self.doc }));
            }
            Phase::Finished { run, .. } => step::forget(run),
            Phase::Choosing => {}
        }
    }

    fn choosing(&mut self, ui: &mut egui::Ui, prefs: &mut Prefs) {
        ui.label(match self.kind {
            ScanKind::Straighten => t::intro(),
            ScanKind::Enlarge => e::intro(),
        });
        ui.add_space(8.0);
        if !self.selected.1.is_empty() {
            let r = ui.checkbox(
                &mut self.only_selected,
                t::selected_only(self.selected.1.len()),
            );
            crate::diag::ui_rect(REGION_SELECTED, r.rect);
        }
        if !(self.only_selected && !self.selected.1.is_empty()) {
            self.scope
                .show(ui, self.page_count, REGION_SCOPE, "deskew-scope");
        }
        match self.kind {
            ScanKind::Straighten => self.straighten_choices(ui),
            ScanKind::Enlarge => super::ocr_upscale::factor_and_filter(
                ui,
                &mut self.factor,
                &mut self.filter,
                REGION_LANCZOS,
            ),
        }
        ui.add_space(10.0);
        ui.separator();
        let named = self.queue(0).len();
        let run = match self.kind {
            ScanKind::Straighten => t::run_button(),
            ScanKind::Enlarge => e::run_button(),
        };
        ui.horizontal(|ui| {
            let go = ui
                .add_enabled(named > 0, egui::Button::new(run))
                .on_disabled_hover_text(t::names_nothing());
            crate::diag::ui_rect_visible(REGION_COMMIT, go.rect, ui.clip_rect());
            if go.clicked() {
                self.remember(prefs);
                self.start();
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// Straighten's own choices: skip pages with text, least confidence.
    fn straighten_choices(&mut self, ui: &mut egui::Ui) {
        let r = ui
            .checkbox(&mut self.skip_text, t::skip_text())
            .on_hover_text(t::skip_text_tooltip());
        crate::diag::ui_rect(REGION_SKIP_TEXT, r.rect);
        ui.horizontal(|ui| {
            ui.label(t::min_confidence())
                .on_hover_text(t::min_confidence_tooltip());
            // The rail alone carries the region, so its ends are 0 and 100;
            // the number box beside it takes a typed value.
            let r = ui
                .add(egui::Slider::new(&mut self.min_confidence, 0..=100).show_value(false))
                .on_hover_text(t::min_confidence_tooltip());
            crate::diag::ui_rect(REGION_MIN_CONFIDENCE, r.rect);
            ui.add(
                egui::DragValue::new(&mut self.min_confidence)
                    .range(0..=100)
                    .suffix("%"),
            );
        });
    }

    /// Keep the least confidence for the next run, when it changed.
    fn remember(&self, prefs: &mut Prefs) {
        if self.kind != ScanKind::Straighten || prefs.deskew_min_confidence == self.min_confidence {
            return;
        }
        prefs.deskew_min_confidence = self.min_confidence;
        let saved = prefs.save().is_ok();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "deskew-min-confidence-saved value={} saved={saved}",
                self.min_confidence
            )
        });
    }

    fn start(&mut self) {
        let run = NEXT_RUN.fetch_add(1, Ordering::Relaxed);
        let queue = self.queue(run);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "deskew-started run={run} job={} turns={} skip-text={} min-confidence={} selected={}",
                job_token(self.job()),
                queue.len(),
                self.skip_text,
                self.min_confidence,
                self.only_selected && !self.selected.1.is_empty()
            )
        });
        self.phase = Phase::Running {
            run,
            queue,
            raised: 0,
            stopping: false,
        };
    }

    fn running(&mut self, ui: &mut egui::Ui, here: bool) {
        let Phase::Running {
            run,
            ref queue,
            ref mut stopping,
            ..
        } = self.phase
        else {
            return;
        };
        let landed = step::outcomes(run).len();
        let at = (landed + 1).min(queue.len());
        ui.label(match self.kind {
            ScanKind::Straighten => t::progress(at, queue.len()),
            ScanKind::Enlarge => e::progress(at, queue.len()),
        });
        ui.add(egui::ProgressBar::new(fraction(landed, queue.len())));
        if !here {
            ui.label(t::paused());
        }
        ui.add_space(8.0);
        let stop = ui.add_enabled(!*stopping, egui::Button::new(t::stop_button()));
        crate::diag::ui_rect_visible(REGION_STOP, stop.rect, ui.clip_rect());
        if stop.clicked() {
            *stopping = true;
        }
    }

    fn finished(&mut self, ui: &mut egui::Ui, run: u64, of: usize) {
        let log = step::outcomes(run);
        let changed = log.iter().filter(|l| bytes(&l.outcome).is_some()).count();
        let (stopped, done): (Counted, Counted) = match self.kind {
            ScanKind::Straighten => (t::stopped, t::done),
            ScanKind::Enlarge => (e::stopped, e::done),
        };
        if log.len() < of {
            ui.label(stopped(log.len(), of));
        }
        ui.label(done(changed, log.len()));
        for line in summary(self.kind, &log, step::folded(run)) {
            ui.label(line);
        }
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                for entry in &log {
                    for line in describe(self.kind, entry, self.min_confidence) {
                        ui.label(line);
                    }
                }
            });
        ui.add_space(8.0);
        if ui.button(t::close_button()).clicked() {
            self.close_requested = true;
        }
    }
}

/// Progress as a bar fraction.
fn fraction(done: usize, of: usize) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let f = done as f32 / of.max(1) as f32;
    f.clamp(0.0, 1.0)
}

/// The factor slider's value in the halves `ScanJob::Enlarge` carries.
fn halves(factor: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let h = (factor * 2.0).round().clamp(2.0, 255.0) as u8;
    h
}

/// The trace's word for a job.
/// A sentence about `n` of `of` pages.
type Counted = fn(usize, usize) -> String;

fn job_token(job: ScanJob) -> String {
    // ui-text-exempt: diagnostic trace vocabulary, never displayed
    match job {
        ScanJob::Straighten { .. } => "straighten".to_owned(),
        ScanJob::Enlarge {
            factor_halves,
            filter,
        } => format!(
            "enlarge factor-halves={factor_halves} filter={}", // ui-text-exempt: diagnostic trace, never displayed
            filter.as_str()
        ),
    }
}

/// A changed picture's stream before and after; `None` when nothing changed.
fn bytes(outcome: &Outcome) -> Option<(usize, usize)> {
    match *outcome {
        Outcome::Straightened { bytes, .. } | Outcome::Enlarged { bytes, .. } => Some(bytes),
        _ => None,
    }
}

/// The run-wide disclosures: growth, and whether it is one undo step.
fn summary(kind: ScanKind, log: &[Logged], folded: Option<Folded>) -> Vec<String> {
    let (old, new) = log
        .iter()
        .filter_map(|l| bytes(&l.outcome))
        .fold((0, 0), |acc, b| (acc.0 + b.0, acc.1 + b.1));
    let mut lines = Vec::new();
    if new > old {
        lines.push(match kind {
            ScanKind::Straighten => t::growth(old, new),
            ScanKind::Enlarge => e::growth(old, new),
        });
    }
    if folded == Some(Folded::Unfolded) {
        lines.push(
            match kind {
                ScanKind::Straighten => t::unfolded(),
                ScanKind::Enlarge => e::unfolded(),
            }
            .to_owned(),
        );
    }
    lines
}

/// One page's lines. Pages are shown 1-based.
fn describe(kind: ScanKind, entry: &Logged, least: u8) -> Vec<String> {
    let page = entry.page + 1;
    let enlarge = kind == ScanKind::Enlarge;
    match &entry.outcome {
        Outcome::Straightened {
            degrees,
            confidence,
            had_text,
            ..
        } => {
            let mut lines = vec![t::straightened(page, *degrees, *confidence)];
            if *had_text {
                lines.push(t::text_not_turned(page));
            }
            lines
        }
        Outcome::Level { degrees } => vec![t::already_level(page, *degrees)],
        Outcome::Unsure {
            degrees,
            confidence,
        } => vec![t::unsure(page, *degrees, *confidence, least)],
        Outcome::Unmeasurable => vec![t::unmeasurable(page)],
        Outcome::NoPicture => vec![t::no_picture(page)],
        Outcome::HasText => vec![t::has_text(page)],
        Outcome::Busy if enlarge => vec![e::busy(page)],
        Outcome::Busy => vec![t::busy(page)],
        Outcome::Elsewhere if enlarge => vec![e::elsewhere(page)],
        Outcome::Elsewhere => vec![t::elsewhere(page)],
        Outcome::Refused(reason) if enlarge => vec![e::refused(page, reason)],
        Outcome::Refused(reason) => vec![t::refused(page, reason)],
        Outcome::Enlarged { from, to, bytes } => vec![e::enlarged(page, *from, *to, *bytes)],
    }
}

/// The selected objects on `page` that are images the engine can turn.
fn selected_images(doc: &OpenDoc, page: usize) -> Vec<usize> {
    let indices = doc.selection.object_indices_on(page);
    if indices.is_empty() {
        return Vec::new();
    }
    let Some(provider) = doc.page_objects() else {
        return Vec::new();
    };
    let objects = &provider.page_objects().objects;
    indices
        .into_iter()
        .filter(|&i| {
            matches!(objects.get(i), Some(VectorObject::Image(img)) if img.source != ImageSource::Form)
        })
        .collect()
}

/// Build it for the current document. `None` when there is none.
pub fn open_for(
    status: &Status,
    picked: Vec<usize>,
    min_confidence: u8,
    kind: ScanKind,
) -> Option<DeskewDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(DeskewDialog::open(doc, picked, min_confidence, kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_straightened_page_with_text_says_its_text_did_not_turn() {
        let entry = Logged {
            page: 2,
            outcome: Outcome::Straightened {
                degrees: -1.5,
                confidence: 0.8,
                bytes: (10, 20),
                had_text: true,
            },
        };
        let lines = describe(ScanKind::Straighten, &entry, 20);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("Page 3:"), "{lines:?}");
    }

    #[test]
    fn growth_is_said_only_when_the_pictures_grew() {
        let grew = Logged {
            page: 0,
            outcome: Outcome::Straightened {
                degrees: 1.0,
                confidence: 0.9,
                bytes: (100, 300),
                had_text: false,
            },
        };
        assert_eq!(
            summary(ScanKind::Straighten, &[grew], Some(Folded::Nothing)).len(),
            1
        );
        let level = Logged {
            page: 0,
            outcome: Outcome::Level { degrees: 0.01 },
        };
        assert!(summary(ScanKind::Straighten, &[level], Some(Folded::Nothing)).is_empty());
        assert!((fraction(1, 0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn an_enlarged_page_says_its_sizes_and_what_it_cost() {
        let entry = Logged {
            page: 0,
            outcome: Outcome::Enlarged {
                from: (100, 200),
                to: (200, 400),
                bytes: (1000, 3000),
            },
        };
        let lines = describe(ScanKind::Enlarge, &entry, 20);
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains("100 × 200") && lines[0].contains("200 × 400"),
            "{lines:?}"
        );
        assert_eq!(summary(ScanKind::Enlarge, &[entry], None).len(), 1);
    }

    #[test]
    fn every_offered_factor_reaches_the_engine_unchanged() {
        let mut f = super::super::ocr_upscale::MIN_FACTOR;
        while f <= pdfcer_core::ocr::upscale::MAX_FACTOR {
            assert!((f32::from(halves(f)) / 2.0 - f).abs() < f32::EPSILON, "{f}");
            f += 0.5;
        }
    }
}
