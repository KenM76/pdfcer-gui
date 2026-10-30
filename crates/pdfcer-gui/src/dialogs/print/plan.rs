//! Planning one document's job from the window's settings: its page sizes,
//! the chosen printer, and the sheets. `show` plans the open document every
//! frame; a batch plans each of its files the same way before spooling it.

use super::{PrintDialog, autopaper, impose, poster, spooler};
use crate::app::state::OpenDoc;
use crate::dialogs::print::spooler::{Job, PaperChoice};

/// One document's planned job.
pub(super) struct Planned {
    /// Each page's rotated extent, points.
    pub(super) page_sizes: Vec<(f64, f64)>,
    /// The selected printer, when there is one.
    pub(super) printer_name: Option<String>,
    /// The sheets, when the printer's capabilities are known.
    pub(super) job: Option<Job>,
}

impl PrintDialog {
    /// Plan `doc` with the window's settings. Sets `device_pending` while the
    /// printer has not yet reported its capabilities for the paper chosen.
    pub(super) fn plan(&mut self, doc: &OpenDoc) -> Planned {
        let page_sizes = page_sizes(doc);

        let spec = self.job_spec(&page_sizes, doc.view.page_index);
        self.impose.first_landscape = spec
            .pages
            .first()
            .and_then(|&i| page_sizes.get(i))
            .is_some_and(|&(w, h)| w > h);

        // Resolved after the sizes and before the first reader, so every surface
        // sees one decision. It is chosen for the whole document, not the range:
        // narrowing the range must not re-pick the sheet. See [`Self::auto_paper`].
        self.auto_paper = match self.device.paper {
            PaperChoice::AutoFromPages => autopaper::choose(&self.forms, &page_sizes),
            PaperChoice::DeviceDefault | PaperChoice::Form(_) => autopaper::AutoPaper::NotChosen,
        };
        let device = self.effective_device();

        let printer_name = self.printers.get(self.selected).map(|p| p.name.clone());
        let job = printer_name
            .as_deref()
            .filter(|_| !self.device_pending)
            .and_then(
                |name| match self.caps.get(name, self.config.as_ref(), device.paper) {
                    spooler::probe::CapsState::Ready(caps) => {
                        Some(spooler::plan_from_caps(&caps, device, &page_sizes, &spec))
                    }
                    spooler::probe::CapsState::Pending => {
                        self.device_pending = true;
                        None
                    }
                    spooler::probe::CapsState::Failed(_) => None,
                },
            )
            // O208: the operator's chosen positions are applied HERE, on the
            // value the plan returned, before any reader. Not inside
            // [`spooler`], whose header rules that nothing in it computes a
            // placement — the displacement is a shell decision. Applying it at
            // one point rather than at each reader is what makes the preview,
            // the per-edge readout, the clip count, the ink-verdict cache key
            // and the spooled job agree by construction.
            .map(|job| {
                if self.poster.on || self.impose.on() {
                    job
                } else {
                    self.page_positions.displace(job, &page_sizes)
                }
            });
        let job = poster::apply(self, job, &page_sizes);
        let job = impose::apply(self, job, &page_sizes, &spec);
        Planned {
            page_sizes,
            printer_name,
            job,
        }
    }
}

/// Each page's ROTATED extent, points: the extent `pdfcer-render` rasterises
/// at, so a `/Rotate 90` page plans landscape. `viewer::page_extent_pts` is
/// the canvas's own measure, so the preview, the canvas and the paper agree.
fn page_sizes(doc: &OpenDoc) -> Vec<(f64, f64)> {
    doc.pages
        .iter()
        .map(|page| {
            let (w, h) = crate::viewer::page_extent_pts(page);
            (f64::from(w), f64::from(h))
        })
        .collect()
}
