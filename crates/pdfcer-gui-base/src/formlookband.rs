//! # `formlookband` — the Forms ▸ Style band
//!
//! Fill, outline colour and outline width for the **next** form field placed,
//! edited in the same [`Remembered`] the placement dialog opens from, so the
//! dialog shows what the band shows. Like the Markup pen it edits a tool
//! setting: no command, no undo entry, nothing written to `userdata`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/formlookband.md`.

use egui::Ui;

use crate::entry;
use crate::formdraft::{FieldLook, Remembered};
use crate::mkcolour::{self, Pick};
use crate::text::formfield as dialog_text;
use crate::text::formlook as t;

/// The fill swatch's trace region.
pub const REGION_FILL: &str = "forms.style.fill"; // ui-text-exempt: trace region name, never displayed
/// The outline swatch's trace region.
pub const REGION_OUTLINE: &str = "forms.style.outline"; // ui-text-exempt: trace region name, never displayed
/// The width field's trace region.
pub const REGION_WIDTH: &str = "forms.style.width"; // ui-text-exempt: trace region name, never displayed

/// The widest outline offered, matching the placement dialog's field.
const MAX_WIDTH_PTS: f64 = 12.0;

/// Draw the band, editing `mem`'s look in place.
pub fn show(ui: &mut Ui, mem: &mut Remembered) {
    let before = mem.look();
    let mut look = before;
    ui.horizontal(|ui| {
        if let Some(pick) = mkcolour::row(
            ui,
            &mkcolour::Row {
                label: t::fill_label(),
                hover: t::fill_hover(),
                id_salt: REGION_FILL,
                region: REGION_FILL,
                colour: look.background,
                unstated_note: dialog_text::background_unstated_note(),
                no_colour_note: dialog_text::background_no_colour_note(),
                no_colour_entry: Some(dialog_text::background_no_colour_entry()),
                no_colour_unavailable: dialog_text::background_no_colour_unavailable(),
                remove_entry: Some(dialog_text::background_remove_entry()),
                remove_unavailable: dialog_text::background_remove_unavailable(),
                disc: false,
            },
        ) {
            look.background = resolve(pick);
        }
        if let Some(pick) = mkcolour::row(
            ui,
            &mkcolour::Row {
                label: t::outline_label(),
                hover: t::outline_hover(),
                id_salt: REGION_OUTLINE,
                region: REGION_OUTLINE,
                colour: look.border_color,
                unstated_note: dialog_text::border_colour_note(),
                no_colour_note: dialog_text::border_colour_note(),
                // The engine draws an empty `/BC` and an absent one in the same
                // black; see `mkcolour::Row::no_colour_entry`.
                no_colour_entry: None,
                no_colour_unavailable: "",
                remove_entry: Some(dialog_text::border_colour_remove_entry()),
                remove_unavailable: dialog_text::border_colour_remove_unavailable(),
                disc: false,
            },
        ) {
            look.border_color = resolve(pick);
        }
        let (widget, refusal) = entry::drag_value(
            ui,
            &mut look.border_width,
            entry::Kind::Length(entry::LengthUnit::Point),
        );
        let response = refusal
            .show(ui.add(widget.range(0.0..=MAX_WIDTH_PTS).speed(0.1)))
            .on_hover_text(t::width_hover());
        crate::diag::ui_rect(REGION_WIDTH, response.rect);
    });
    if look != before {
        mem.set_look(look);
        trace(look);
    }
}

/// A swatch's pick as the draft's value: `Remove` states nothing.
const fn resolve(pick: Pick) -> Option<pdfcer_core::forms::MkColor> {
    match pick {
        Pick::Set(colour) => Some(colour),
        Pick::Remove => None,
    }
}

/// The band's one trace line, read by `forms_style_band_sets_the_next_field`.
fn trace(look: FieldLook) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "form-look width={} bg={} bc={}",
            look.border_width,
            key(look.background),
            key(look.border_color)
        )
    });
}

/// A colour as a trace token: `unset`, `none`, or its components.
fn key(colour: Option<pdfcer_core::forms::MkColor>) -> String {
    use pdfcer_core::forms::MkColor;
    match colour {
        None => "unset".to_owned(),
        Some(MkColor::None) => "none".to_owned(),
        Some(MkColor::Gray(g)) => format!("g:{g}"),
        Some(MkColor::Rgb(r, g, b)) => format!("rgb:{r},{g},{b}"),
        Some(MkColor::Cmyk(c, m, y, k)) => format!("cmyk:{c},{m},{y},{k}"),
    }
}
