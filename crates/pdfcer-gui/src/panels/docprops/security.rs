//! # `panels::docprops::security` — Document properties ▸ Security notes
//!
//! Two engine answers, read-only: whether the file is a §7.6.7 wrapper around
//! an encrypted payload (`wrapper::detect`), and every action it would run in
//! a viewer that runs them (`forms::scan_javascript`, which walks fields,
//! annotations, pages, outlines, the catalog and `/Next` chains, executing
//! nothing).
//!
//! # Contract
//!
//! - Measured once per document and edit, not per frame; traced as
//!   `security-notes` when measured.
//! - A walk that hit its ceiling says *stopped looking*, never *nothing*.
//! - Disclosure only: nothing here blocks, and nothing is drawn on the page.

use egui::{Id, Ui};
use pdfcer_core::forms::FormJavaScript;

use crate::app::state::OpenDoc;
use crate::text::securitynotes as t;

/// The section's region, for the harness.
pub const REGION: &str = "docprops.security-notes"; // ui-text-exempt: trace region name, never displayed

/// What the engine said about one document at one edit.
#[derive(Clone)]
struct Notes {
    /// [`OpenDoc::serial`].
    doc: u64,
    epoch: u64,
    wrapper: Option<String>,
    scan: FormJavaScript,
}

/// The engine's answers for `doc`, measured when the document or edit moved.
fn notes(ui: &Ui, doc: &OpenDoc) -> Notes {
    let id = Id::new("docprops-security-notes");
    let cached: Option<Notes> = ui.ctx().data(|d| d.get_temp(id));
    if let Some(n) = cached
        && n.epoch == doc.edit_epoch
        && n.doc == doc.serial
    {
        return n;
    }
    let view = doc.session.view();
    let notes = Notes {
        doc: doc.serial,
        epoch: doc.edit_epoch,
        wrapper: pdfcer_core::wrapper::detect(&view).message(),
        scan: pdfcer_core::forms::scan_javascript(&view),
    };
    trace(&notes);
    ui.ctx().data_mut(|d| d.insert_temp(id, notes.clone()));
    notes
}

/// The rows' counts, in [`t::row_labels`] order.
const fn counts(s: &FormJavaScript) -> [usize; 9] {
    [
        s.doc_level_scripts,
        s.page_trigger_actions,
        s.outline_actions,
        s.annotation_actions,
        s.custom_scripts,
        s.javascript_actions,
        s.chained_actions,
        s.network_action_count,
        s.launch_action_count,
    ]
}

/// Draw the section.
pub(super) fn section(ui: &mut Ui, doc: &OpenDoc) {
    let notes = notes(ui, doc);
    let block = ui
        .scope(|ui| {
            ui.add_space(6.0);
            ui.label(t::heading());
            if let Some(warning) = &notes.wrapper {
                ui.colored_label(ui.visuals().warn_fg_color, warning);
            }
            ui.label(egui::RichText::new(t::actions_heading()).small());
            body(ui, &notes.scan);
        })
        .response
        .rect;
    crate::diag::ui_rect_visible(REGION, block, ui.clip_rect());
}

/// The census rows, or the sentence that stands for none.
fn body(ui: &mut Ui, scan: &FormJavaScript) {
    if scan.scan_truncated {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t::stopped_looking(scan.actions_scanned),
        );
    }
    if !scan.any() {
        if !scan.scan_truncated {
            ui.label(t::nothing_runs());
        }
        return;
    }
    if scan.open_action_is_javascript {
        ui.label(t::script_on_open());
    }
    // Wrapped labels, not a grid: a grid's label column does not wrap and
    // pushes the count past the dock's right edge.
    for (label, n) in t::row_labels().into_iter().zip(counts(scan)) {
        if n > 0 {
            ui.label(t::row(label, n));
        }
    }
    ui.label(t::reaches_outside(scan.reaches_outside()));
    ui.label(egui::RichText::new(t::none_run_here()).small().weak());
}

/// `security-notes wrapper= scanned= truncated= open_script= doc= page= outline=
/// annot= field= js= chained= network= launch=`.
fn trace(notes: &Notes) {
    let s = &notes.scan;
    let [
        doc,
        page,
        outline,
        annot,
        field,
        js,
        chained,
        network,
        launch,
    ] = counts(s);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "security-notes wrapper={} scanned={} truncated={} open_script={} doc={doc} \
             page={page} outline={outline} annot={annot} field={field} js={js} \
             chained={chained} network={network} launch={launch}",
            u8::from(notes.wrapper.is_some()),
            s.actions_scanned,
            s.scan_truncated,
            s.open_action_is_javascript,
        )
    });
}
