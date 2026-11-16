//! # `dialogs::print::direct::compose` — the job's sheets, laid out by the engine
//!
//! A job the printer cannot lay out from IPP attributes alone — n-up, a
//! booklet, a poster, a chosen paper, a percentage scale, reverse order — is
//! composed into a new PDF by `pdfcer_print::compose::compose` and that PDF is
//! sent at actual size. Whether a job needs it is the engine's
//! `Shape::needs_composing`, plus reverse order, which the engine leaves to
//! the caller's page sequence.
//!
//! # Contract
//!
//! - The shape is built from the dialog's own n-up, booklet and poster specs,
//!   not `Shape::for_job`: those carry the page order, auto-rotate and the
//!   booklet's sheet range the operator chose and the preview shows.
//! - The paper is the planned sheet (`Job::device`), made portrait by
//!   `Paper::portrait`; the orientation is the one the driver route plans with.
//! - Composing works on a scratch session opened from the print-ready copy's
//!   bytes. The operator's editing session is never handed to the engine.

use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use pdfcer_core::document::Document;
use pdfcer_print::compose::{Composed, Paper, Request, Shape};
use pdfcer_print::imposition::PosterSpec;

use super::super::PrintDialog;
use crate::app::prefs::print_impose::ImposeMode;
use crate::dialogs::print::spooler::Job;
use pdfcer_gui_base::printspooler::{Orientation, PaperChoice, ScaleMode};

/// How this job reaches a printer that reads PDF.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::dialogs::print) struct Layout {
    pub(super) shape: Shape,
    pub(super) mode: pdfcer_print::ScaleMode,
    pub(super) orientation: pdfcer_print::Orientation,
    /// The planned sheet, portrait.
    pub(super) paper: Paper,
    /// 0-based pages in send order, each once.
    pub(super) sequence: Vec<usize>,
    /// Whether the printer is handed composed sheets rather than the
    /// document's own pages.
    pub(super) composed: bool,
}

impl PrintDialog {
    /// How `job` goes to a printer that reads PDF.
    pub(in crate::dialogs::print) fn direct_layout(
        &self,
        job: &Job,
        page_sizes: &[(f64, f64)],
        current_page: usize,
    ) -> Layout {
        let spec = self.job_spec(page_sizes, current_page);
        let device = self.effective_device();
        let shape = self.direct_shape();
        let mode = engine_scale(spec.mode);
        let paper_chosen = device.paper != PaperChoice::DeviceDefault;
        Layout {
            shape,
            mode,
            orientation: engine_orientation(device.orientation),
            paper: Paper::portrait(
                job.device.physical_pt,
                job.device.printable_pt,
                job.device.offset_pt,
            ),
            sequence: pdfcer_gui_base::imposition::sequence_once(&spec),
            composed: shape.needs_composing(paper_chosen, mode) || self.reverse,
        }
    }

    /// The physical sheet a composed job is laid out for, points, portrait;
    /// `None` when the document's own pages would be sent.
    pub(in crate::dialogs::print) fn direct_composed_for(
        &self,
        job: Option<&Job>,
        page_sizes: &[(f64, f64)],
        current_page: usize,
    ) -> Option<(f64, f64)> {
        let layout = self.direct_layout(job?, page_sizes, current_page);
        layout.composed.then_some(layout.paper.physical_pt)
    }

    fn direct_shape(&self) -> Shape {
        if self.poster.on {
            let o = super::super::poster::options(self.poster);
            return Shape::Poster(PosterSpec {
                tile_scale: o.tile_scale,
                overlap_pt: o.overlap_pt,
                cut_marks: o.cut_marks,
                labels: o.labels,
                tile_only_large_pages: o.large_only,
                ..PosterSpec::default()
            });
        }
        match self.impose.prefs.mode {
            ImposeMode::Off => Shape::Pages,
            ImposeMode::NUp => Shape::NUp(self.impose.nup_spec()),
            ImposeMode::Booklet => Shape::Booklet(self.impose.booklet_spec()),
        }
    }
}

/// Compose `layout`'s sheets from `bytes`, the print-ready copy, on a scratch
/// session. The error is the engine's sentence.
pub(super) fn sheets(
    doc: &OpenDoc,
    bytes: &[u8],
    layout: &Layout,
    label: &str,
) -> Result<Composed, String> {
    let document = Document::from_bytes(bytes.to_vec()).map_err(|e| e.to_string())?;
    let mut scratch = doc.settings.open_session(document);
    let request = Request {
        paper: layout.paper,
        shape: layout.shape,
        mode: layout.mode,
        orientation: layout.orientation,
        label,
    };
    let composed = pdfcer_print::compose::compose(&mut scratch, &layout.sequence, &request)
        .map_err(|e| e.to_string())?;
    trace(layout, &composed);
    Ok(composed)
}

/// The shape's name in the `print-direct-compose` line.
const fn shape_name(shape: Shape) -> &'static str {
    match shape {
        Shape::Pages => "pages",        // ui-text-exempt: diagnostic trace token
        Shape::NUp(_) => "nup",         // ui-text-exempt: diagnostic trace token
        Shape::Booklet(_) => "booklet", // ui-text-exempt: diagnostic trace token
        Shape::Poster(_) => "poster",   // ui-text-exempt: diagnostic trace token
    }
}

fn trace(layout: &Layout, composed: &Composed) {
    let (w, h) = layout.paper.physical_pt;
    crate::diag::trace(|| {
        format!(
            "print-direct-compose shape={} sheets={} paper={w:.0}x{h:.0} notes={}", // ui-text-exempt: diagnostic trace
            shape_name(layout.shape),
            composed.pages,
            composed.notes.len()
        )
    });
    for (i, note) in composed.notes.iter().enumerate() {
        crate::diag::trace(|| format!("print-direct-note i={i} text={note:?}")); // ui-text-exempt: diagnostic trace
    }
}

/// Points to whole millimetres, for the disclosure.
pub(super) fn mm(physical_pt: (f64, f64)) -> (u32, u32) {
    let one = |pt: f64| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mm = crate::units::mm_from_points(pt).round().max(0.0) as u32;
        mm
    };
    (one(physical_pt.0), one(physical_pt.1))
}

const fn engine_scale(mode: ScaleMode) -> pdfcer_print::ScaleMode {
    match mode {
        ScaleMode::Fit => pdfcer_print::ScaleMode::Fit,
        ScaleMode::ActualSize => pdfcer_print::ScaleMode::ActualSize,
        ScaleMode::ShrinkOversized => pdfcer_print::ScaleMode::ShrinkOversized,
        ScaleMode::Custom(factor) => pdfcer_print::ScaleMode::Custom(factor),
    }
}

const fn engine_orientation(orientation: Orientation) -> pdfcer_print::Orientation {
    match orientation {
        Orientation::Auto => pdfcer_print::Orientation::Auto,
        Orientation::Portrait => pdfcer_print::Orientation::Portrait,
        Orientation::Landscape => pdfcer_print::Orientation::Landscape,
    }
}

#[cfg(test)]
mod tests {
    use super::{mm, shape_name};
    use pdfcer_print::compose::Shape;

    #[test]
    fn a4_reads_in_whole_millimetres() {
        assert_eq!(mm((595.276, 841.89)), (210, 297));
    }

    #[test]
    fn whole_pages_are_named_pages() {
        assert_eq!(shape_name(Shape::Pages), "pages");
    }
}
