//! A ce dimension's line-style chooser: the engine's named drafting line
//! types (`pdfcer_core::dimension::LineType::ALL`), which re-derive their
//! runs from the drafting standard, line width and line-type scale. A value
//! holding custom runs reads as the chooser's custom entry until a named type
//! is picked.
//!
//! With `region`, publishes `<region>.combo` and `<region>.option.<token>`,
//! `<token>` being `LineType::token`.

use egui::Ui;
use pdfcer_core::dimension::{LineType, LineTypeRef};

use crate::text::panels::dimension as t;

/// What the closed chooser shows for `value`.
fn shown(value: LineTypeRef) -> &'static str {
    value
        .line_type()
        .and_then(t::line_type_name)
        .unwrap_or(t::line_type_custom())
}

/// Offer every named line type; set `value` to the one picked.
pub fn line_type_editor(ui: &mut Ui, id_salt: &str, region: Option<&str>, value: &mut LineTypeRef) {
    let current = value.line_type();
    let combo = egui::ComboBox::from_id_salt(id_salt)
        .width(120.0)
        .selected_text(shown(*value))
        .show_ui(ui, |ui| {
            for line_type in LineType::ALL {
                let Some(name) = t::line_type_name(line_type) else {
                    continue;
                };
                let is_current = current == Some(line_type);
                let entry = ui.selectable_label(is_current, name);
                if let Some(region) = region {
                    // ui-text-exempt: trace region name, never displayed
                    crate::diag::ui_rect(
                        &format!("{region}.option.{}", line_type.token()),
                        entry.rect,
                    );
                }
                if entry.clicked() && !is_current {
                    *value = LineTypeRef::Named(line_type);
                }
            }
        });
    if let Some(region) = region {
        // ui-text-exempt: trace region name, never displayed
        crate::diag::ui_rect(&format!("{region}.combo"), combo.response.rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::dimension::DimDash;

    #[test]
    fn every_engine_line_type_is_offered_by_name() {
        for line_type in LineType::ALL {
            assert!(t::line_type_name(line_type).is_some(), "{line_type:?}");
        }
    }

    #[test]
    fn custom_runs_read_as_custom_and_solid_runs_as_solid() {
        let dashed = DimDash::new(&[5.0, 2.0]).expect("a valid pattern");
        assert_eq!(shown(LineTypeRef::Custom(dashed)), t::line_type_custom());
        let solid = DimDash::new(&[]).expect("solid");
        assert_eq!(shown(LineTypeRef::Custom(solid)), "Solid");
        assert_eq!(shown(LineTypeRef::Named(LineType::Phantom)), "Phantom");
    }
}
