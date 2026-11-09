//! The spool itself: this crate's sheets handed to `pdfcer_print::spool_sheets`,
//! with the traces that record what was sent and how each sheet travelled.

use super::{
    DeviceSettings, DriverConfig, PageBitmap, SettingsSource, SpoolReport, Unavailable,
    to_engine_bitmap, to_engine_settings,
};

/// Hand the rendered sheets to the spooler. A sheet carrying a metafile is
/// played as vector graphics; the rest are blitted as images.
///
/// The dialog is the confirmation, so this is never a dry run unless the
/// diagnostic switch in [`dry_run`] is set. The job is traced before the call,
/// because a spool that hangs or crashes leaves that line as its only record.
pub fn spool(
    printer: &str,
    bitmaps: &[PageBitmap],
    settings: DeviceSettings,
    config: Option<&DriverConfig>,
    first_page_pt: (f64, f64),
) -> Result<SpoolReport, Unavailable> {
    trace_spool(printer, bitmaps, settings, first_page_pt);
    let pages: Vec<pdfcer_print::PageBitmap> = bitmaps.iter().map(to_engine_bitmap).collect();
    let engine_settings = to_engine_settings(settings);
    // The one setup `spool_with_config` would give every sheet: orientation
    // resolved from the first planned page, the job's paper.
    let setup = pdfcer_print::SheetSetup {
        orientation: pdfcer_print::resolve_orientation(engine_settings.orientation, first_page_pt),
        paper: engine_settings.paper,
    };
    let sheets: Vec<pdfcer_print::Sheet<'_>> = pages
        .iter()
        .zip(bitmaps)
        .map(|(bitmap, ours)| pdfcer_print::Sheet {
            bitmap,
            setup,
            metafile: ours.metafile.as_deref(),
        })
        .collect();
    let outcome = pdfcer_print::spool_sheets(
        printer,
        &sheets,
        dry_run(),
        None,
        engine_settings,
        config.map(DriverConfig::engine),
    );
    match outcome {
        Ok(report) => Ok(from_engine_report(printer, &report, bitmaps)),
        Err(error) => {
            let detail = error.to_string();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("print-spool-failed printer={printer} reason={detail}")
            });
            Err(Unavailable::Spooler(detail))
        }
    }
}

/// The `print-spool` line: what is about to be sent.
fn trace_spool(
    printer: &str,
    bitmaps: &[PageBitmap],
    settings: DeviceSettings,
    first_page_pt: (f64, f64),
) {
    crate::diag::trace(|| {
        let bytes: usize = bitmaps.iter().map(|b| b.rgba.len()).sum();
        let vector = bitmaps.iter().filter(|b| b.metafile.is_some()).count();
        let first = bitmaps.first();
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-spool printer={printer} sheets={} vector={vector} px={:?} bytes={bytes} first_page_pt={first_page_pt:?} placement={:?} page_pt={:?} orientation={:?} duplex={:?} tray={}",
            bitmaps.len(),
            first.map(|b| (b.width, b.height)),
            first.map(|b| b.placement),
            first.map(|b| b.page_pt),
            settings.orientation,
            settings.duplex,
            settings.pick_tray_by_page_size,
        )
    });
}

/// The engine's report in this crate's terms, with each sheet's route.
fn from_engine_report(
    printer: &str,
    report: &pdfcer_print::SpoolReport,
    bitmaps: &[PageBitmap],
) -> SpoolReport {
    let vector_sheets = bitmaps.iter().filter(|b| b.metafile.is_some()).count();
    let mut image_reasons: Vec<String> = Vec::new();
    for why in bitmaps.iter().filter_map(|b| b.image_reason.as_ref()) {
        if !image_reasons.contains(why) {
            image_reasons.push(why.clone());
        }
    }
    let fell_back = bitmaps.iter().filter(|b| b.image_reason.is_some()).count();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-spool-done printer={printer} pages={} printed={} dpi={:?} clipped={} job_id={:?} vector={vector_sheets} fell_back={fell_back} why={}",
            report.pages,
            report.printed,
            report.dpi,
            report.clipped_pages,
            report.job_id,
            image_reasons.join("|"),
        )
    });
    SpoolReport {
        pages: report.pages,
        printed: report.printed,
        dpi: report.dpi,
        clipped_pages: report.clipped_pages,
        job_id: report.job_id,
        settings_source: match report.settings_source {
            pdfcer_print::SettingsSource::DeviceDefault => SettingsSource::DeviceDefault,
            pdfcer_print::SettingsSource::DriverSupplied => SettingsSource::DriverSupplied,
            pdfcer_print::SettingsSource::CallerSupplied => SettingsSource::CallerSupplied,
            pdfcer_print::SettingsSource::Synthesised => SettingsSource::Synthesised,
        },
        vector_sheets,
        fell_back,
        image_reasons,
    }
}

/// The diagnostic switch that makes every spool a dry run: the engine does
/// everything but start the job, so a driven check can press Print without
/// paper. Never set in normal use.
pub const DIAG_DRY_RUN: &str = "PDFCER_DIAG_PRINT_DRY_RUN"; // ui-text-exempt: an environment variable name, never displayed

/// `DryRun::Yes` when [`DIAG_DRY_RUN`] is set to `1`, else `DryRun::No`.
fn dry_run() -> pdfcer_print::DryRun {
    if std::env::var(DIAG_DRY_RUN).is_ok_and(|v| v == "1") {
        crate::diag::trace(|| "print-dry-run".to_owned()); // ui-text-exempt: diagnostic trace
        pdfcer_print::DryRun::Yes
    } else {
        pdfcer_print::DryRun::No
    }
}
