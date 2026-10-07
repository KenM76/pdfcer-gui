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
//! - Password values kept in saved versions (`scan_stored_password_values`)
//!   are measured on the file as loaded, once per document and base: they
//!   change only on save, and pdfcer never stores a password value itself.

use egui::{Id, Ui};
use pdfcer_core::document::Document;
use pdfcer_core::forms::FormJavaScript;
use pdfcer_core::password_history::{PasswordValueScan, scan_stored_password_values};

use crate::app::state::OpenDoc;
use crate::text::securitynotes as t;

/// The section's region, for the harness.
pub const REGION: &str = "docprops.security-notes"; // ui-text-exempt: trace region name, never displayed

/// The stored-passwords lines, for the harness.
pub const PASSWORDS_REGION: &str = "docprops.security-notes.passwords"; // ui-text-exempt: trace region name, never displayed

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

/// Password values kept in the file's saved versions, for one base.
#[derive(Clone)]
struct Passwords {
    /// [`OpenDoc::serial`].
    doc: u64,
    /// The base's byte length; a save replaces the base.
    base_len: usize,
    scan: PasswordValueScan,
    /// Whether the file as loaded is a form. An unopenable version is only
    /// worth saying on one: a linearized drawing has one by construction.
    form: bool,
}

/// The scan for `doc`'s base, measured when the document or its base moved.
fn passwords(ui: &Ui, doc: &OpenDoc) -> Passwords {
    let id = Id::new("docprops-security-passwords");
    let base = doc.session.document();
    let base_len = base.bytes().len();
    let cached: Option<Passwords> = ui.ctx().data(|d| d.get_temp(id));
    if let Some(p) = cached
        && p.doc == doc.serial
        && p.base_len == base_len
    {
        return p;
    }
    let started = std::time::Instant::now();
    let p = Passwords {
        doc: doc.serial,
        base_len,
        scan: scan_stored_password_values(base.bytes(), Document::from_bytes),
        form: pdfcer_core::forms::parse_acroform(base).is_some(),
    };
    let ms = started.elapsed().as_millis();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "security-passwords revisions={} unreadable={} earlier={} current={} form={} ms={ms}",
            p.scan.revisions,
            p.scan.unreadable_revisions,
            p.scan.in_superseded().count(),
            p.scan.in_latest().count(),
            u8::from(p.form),
        )
    });
    ui.ctx().data_mut(|d| d.insert_temp(id, p.clone()));
    p
}

/// The stored-password lines, or nothing when there is nothing to say.
fn stored_passwords(ui: &mut Ui, p: &Passwords) {
    let scan = &p.scan;
    let unchecked = p.form && scan.unreadable_revisions > 0;
    if scan.stored.is_empty() && !unchecked {
        return;
    }
    let block = ui
        .scope(|ui| {
            if !scan.stored.is_empty() {
                let found: Vec<String> = scan
                    .stored
                    .iter()
                    .map(|s| t::stored_password_at(&s.field, s.revision + 1, scan.revisions))
                    .collect();
                let remedy = crate::text::purge_passwords::file_purge_password_values().label;
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    t::stored_passwords(
                        &found,
                        scan.in_superseded().count(),
                        scan.in_latest().count(),
                        remedy,
                    ),
                );
            }
            if unchecked {
                ui.label(
                    egui::RichText::new(t::stored_passwords_unchecked(scan.unreadable_revisions))
                        .small()
                        .weak(),
                );
            }
        })
        .response
        .rect;
    crate::diag::ui_rect_visible(PASSWORDS_REGION, block, ui.clip_rect());
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
            stored_passwords(ui, &passwords(ui, doc));
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
