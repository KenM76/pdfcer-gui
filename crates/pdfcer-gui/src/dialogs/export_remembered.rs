//! **What the three export windows remember between jobs** —
//! `OPERATOR_REQUESTS.md` **O196**, the operator's words of 2026-09-13:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/export_remembered.md`.

use crate::app::prefs::exporting;
use crate::app::prefs::{ExportDxfPrefs, ExportImagePrefs, ExportTextPrefs, Prefs};

/// Persist the Export-image window's habits.
pub(super) fn remember_image(habits: ExportImagePrefs, prefs: &mut Prefs) {
    if prefs.export.image == habits {
        return;
    }
    prefs.export.image = habits;
    let saved = prefs.save();
    let image = &prefs.export.image;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "export-image-remembered saved={} format={} scope={} dpi={} transparent={} quality={} keep_text={}",
            saved.is_ok(),
            exporting::image_format_key(image.format),
            // The same reduction the preferences file performs, and it must be
            // the same one: a check that reads this line and then reads the
            // file would otherwise see `Typed` disclosed here and `current` on
            // disk, and conclude the write had gone wrong.
            exporting::page_scope_key_or(image.scope, ExportImagePrefs::default().scope),
            image.dpi,
            u8::from(image.transparent),
            image.quality,
            u8::from(image.keep_text),
        )
    });
}

/// Persist the Export-text window's habits.
pub(super) fn remember_text(habits: ExportTextPrefs, prefs: &mut Prefs) {
    if prefs.export.text == habits {
        return;
    }
    prefs.export.text = habits;
    let saved = prefs.save();
    let text = &prefs.export.text;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "export-text-remembered saved={} scope={} separator={} order={} endings={} bom={}",
            saved.is_ok(),
            exporting::page_scope_key_or(text.scope, ExportTextPrefs::default().scope),
            exporting::separator_key(text.separator),
            exporting::text_order_key(text.order),
            exporting::line_endings_key(text.line_endings),
            u8::from(text.byte_order_mark),
        )
    });
}

/// Persist the Export-DXF window's habits.
pub(super) fn remember_dxf(habits: ExportDxfPrefs, prefs: &mut Prefs) {
    if prefs.export.dxf == habits {
        return;
    }
    prefs.export.dxf = habits;
    let saved = prefs.save();
    let dxf = &prefs.export.dxf;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "export-dxf-remembered saved={} units={} arcs={} text={}",
            saved.is_ok(),
            exporting::dxf_units_key(dxf.units),
            u8::from(dxf.fit_arcs),
            exporting::dxf_text_key(dxf.text),
        )
    });
}
