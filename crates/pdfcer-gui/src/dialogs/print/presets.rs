//! # `dialogs::print::presets` — the document's own print settings
//!
//! A PDF can say how it wants to be printed: `/ViewerPreferences`' scaling,
//! duplex, tray-by-size, page ranges and copies, read by
//! `pdfcer_core::print_presets::print_presets`. As Acrobat does, the Print
//! dialog opens with these in place of the remembered settings, and lists
//! which ones it used under the scale choices. `/Enforce [/PrintScaling]`
//! fixes actual size: the scale choices are then greyed, with the reason on
//! hover. `/PrintArea` and `/PrintClip` are disclosed, not honoured: the
//! dialog prints the crop box.
//!
//! A document's setting is never remembered as the operator's habit: a value
//! still equal to what the document put there is saved as the remembered
//! value it displaced ([`keep_own_habits`]).

use pdfcer_core::print_presets::{PresetDuplex, PresetScaling, PrintPresets};
use pdfcer_gui_base::printspooler::{Duplex, ScaleMode};

use super::PrintDialog;
use super::tabs::PrintRange;
use crate::text::print::presets as t;

/// The most copies the dialog accepts.
const MAX_COPIES: u16 = 999;

/// What the document asked for and what the dialog did with it.
#[derive(Debug, Clone, Default)]
pub(super) struct Applied {
    /// One sentence per setting used or set aside, in the order read.
    pub notes: Vec<String>,
    /// `/Enforce [/PrintScaling]`: the scale choices are fixed at actual size.
    pub scale_locked: bool,
    /// The values the document put into the dialog, `None` where it set none.
    scale: Option<ScaleMode>,
    copies: Option<u16>,
    duplex: Option<Duplex>,
    tray: Option<bool>,
}

/// Put `presets` into `dialog` for a document of `page_count` pages.
pub(super) fn apply(dialog: &mut PrintDialog, presets: &PrintPresets, page_count: usize) {
    let mut notes = Vec::new();
    let mut applied = Applied::default();
    match presets.scaling {
        Some(PresetScaling::None) => {
            dialog.scale = ScaleMode::ActualSize;
            applied.scale = Some(ScaleMode::ActualSize);
            notes.push(if presets.scaling_enforced {
                t::scale_enforced().to_owned()
            } else {
                t::scale_actual().to_owned()
            });
        }
        Some(PresetScaling::AppDefault) | None => {}
    }
    if let Some(spec) = ranges(&presets.page_ranges, page_count) {
        notes.push(t::pages(&spec));
        dialog.range = PrintRange::Custom;
        dialog.range_text = spec;
    } else if !presets.page_ranges.is_empty() {
        notes.push(t::pages_outside(page_count).to_owned());
    }
    if let Some(n) = presets.copies {
        let copies = u16::try_from(n).unwrap_or(u16::MAX).min(MAX_COPIES);
        notes.push(t::copies(n, copies));
        dialog.copies = copies;
        applied.copies = Some(copies);
    }
    if let Some(d) = presets.duplex {
        dialog.device.duplex = match d {
            PresetDuplex::Simplex => Duplex::Simplex,
            PresetDuplex::FlipLongEdge => Duplex::LongEdge,
            PresetDuplex::FlipShortEdge => Duplex::ShortEdge,
        };
        applied.duplex = Some(dialog.device.duplex);
        notes.push(t::duplex(dialog.device.duplex).to_owned());
    }
    if let Some(by_size) = presets.pick_tray_by_pdf_size {
        dialog.device.pick_tray_by_page_size = by_size;
        applied.tray = Some(by_size);
        notes.push(t::tray(by_size).to_owned());
    }
    for boundary in [presets.print_area, presets.print_clip]
        .into_iter()
        .flatten()
    {
        if boundary != pdfcer_core::print_presets::PresetBoundary::CropBox {
            notes.push(t::boundary(boundary.name()));
        }
    }
    notes.extend(presets.ignored.iter().map(|i| t::ignored(i)));
    applied.notes = notes;
    applied.scale_locked = presets.scaling_enforced;
    dialog.presets = applied;
}

/// Put back, in `habits`, every remembered value a document setting displaced
/// and the operator left alone.
pub(super) fn keep_own_habits(
    applied: &Applied,
    habits: &mut crate::app::prefs::PrintPrefs,
    before: &crate::app::prefs::PrintPrefs,
) {
    if applied.scale == Some(habits.scale) {
        habits.scale = before.scale;
    }
    if applied.copies == Some(habits.copies) {
        habits.copies = before.copies;
    }
    if applied.duplex == Some(habits.duplex) {
        habits.duplex = before.duplex;
    }
    if applied.tray == Some(habits.pick_tray_by_page_size) {
        habits.pick_tray_by_page_size = before.pick_tray_by_page_size;
    }
}

/// `/PrintPageRange` as the dialog's range text, clamped to the document.
/// `None` when no range names a page that exists.
fn ranges(pairs: &[(u32, u32)], page_count: usize) -> Option<String> {
    let last = u32::try_from(page_count).unwrap_or(u32::MAX);
    let parts: Vec<String> = pairs
        .iter()
        .filter(|(first, _)| *first <= last)
        .map(|&(first, end)| {
            let end = end.min(last);
            if first == end {
                first.to_string()
            } else {
                format!("{first}-{end}")
            }
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join(","))
}

/// The list of what came from the document, drawn under the scale choices.
pub(super) fn notes(ui: &mut egui::Ui, applied: &Applied) {
    if applied.notes.is_empty() {
        return;
    }
    let muted = egui_shell::theme::Theme::of(ui.ctx()).palette.text_muted;
    ui.label(egui::RichText::new(t::heading()).color(muted));
    for line in &applied.notes {
        ui.label(egui::RichText::new(t::bullet(line)).small().color(muted));
    }
}

#[cfg(test)]
mod tests {
    use super::{Applied, keep_own_habits, ranges};
    use crate::app::prefs::PrintPrefs;
    use pdfcer_gui_base::printspooler::{Duplex, ScaleMode};

    #[test]
    fn a_document_preset_left_alone_is_not_remembered_as_a_habit() {
        let before = PrintPrefs::default();
        let applied = Applied {
            copies: Some(2),
            duplex: Some(Duplex::LongEdge),
            scale: Some(ScaleMode::ActualSize),
            ..Applied::default()
        };
        let mut kept = PrintPrefs {
            copies: 2,
            duplex: Duplex::ShortEdge,
            scale: ScaleMode::ActualSize,
            ..before.clone()
        };
        keep_own_habits(&applied, &mut kept, &before);
        assert_eq!(
            kept.copies, before.copies,
            "left at the preset: the old habit"
        );
        assert_eq!(
            kept.scale, before.scale,
            "left at the preset: the old habit"
        );
        assert_eq!(
            kept.duplex,
            Duplex::ShortEdge,
            "changed by the operator: kept"
        );
    }

    #[test]
    fn ranges_join_and_clamp_to_the_document() {
        assert_eq!(ranges(&[(1, 2), (5, 5)], 10).as_deref(), Some("1-2,5"));
        assert_eq!(ranges(&[(3, 40)], 10).as_deref(), Some("3-10"));
        assert_eq!(ranges(&[(11, 12)], 10), None);
        assert_eq!(ranges(&[], 10), None);
    }
}
