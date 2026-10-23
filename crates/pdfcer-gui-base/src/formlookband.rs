//! # `formlookband` — the Forms ▸ Style band
//!
//! Fill, outline colour, outline width, border style and the text's font, size
//! and colour for the **next** form field placed,
//! edited in the same [`Remembered`] the placement dialog opens from, so the
//! dialog shows what the band shows. Like the Markup pen it edits a tool
//! setting: no command, no undo entry, nothing written to `userdata`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/formlookband.md`.

use egui::Ui;
use pdfcer_core::edit::BorderStyle;

use crate::entry;
use crate::formdraft::{FieldLook, Remembered, TextLook};
use crate::mkcolour::{self, Pick};
use crate::text::formfield as dialog_text;
use crate::text::formlook as t;

/// The fill swatch's trace region.
pub const REGION_FILL: &str = "forms.style.fill"; // ui-text-exempt: trace region name, never displayed
/// The outline swatch's trace region.
pub const REGION_OUTLINE: &str = "forms.style.outline"; // ui-text-exempt: trace region name, never displayed
/// The width field's trace region.
pub const REGION_WIDTH: &str = "forms.style.width"; // ui-text-exempt: trace region name, never displayed
/// The border-style picker's trace region; its rows are `<region>.<index>`
/// in [`STYLES`] order.
pub const REGION_STYLE: &str = "forms.style.border_style"; // ui-text-exempt: trace region name, never displayed
/// The text-font picker's trace region; its rows are `<region>.<index>` in
/// `Std14::ALL` order.
pub const REGION_FONT: &str = "forms.style.text_font"; // ui-text-exempt: trace region name, never displayed
/// The text-size field's trace region.
pub const REGION_SIZE: &str = "forms.style.text_size"; // ui-text-exempt: trace region name, never displayed
/// The text-colour swatch's trace region.
pub const REGION_TEXT_COLOUR: &str = "forms.style.text_colour"; // ui-text-exempt: trace region name, never displayed

/// The five border styles the engine authors, in the order offered.
pub const STYLES: [BorderStyle; 5] = [
    BorderStyle::Solid,
    BorderStyle::Dashed,
    BorderStyle::Beveled,
    BorderStyle::Inset,
    BorderStyle::Underline,
];

/// Where one set of text controls publishes its three regions.
pub struct TextRegions {
    /// The font picker; its rows are `<font>.<i>` in `Std14::ALL` order.
    pub font: &'static str,
    /// The size field.
    pub size: &'static str,
    /// The colour swatch.
    pub colour: &'static str,
}

/// The band's own text controls.
const BAND_TEXT: TextRegions = TextRegions {
    font: REGION_FONT,
    size: REGION_SIZE,
    colour: REGION_TEXT_COLOUR,
};

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
        style_picker(ui, &mut look.border_style, REGION_STYLE);
        ui.separator();
        text_controls(ui, &mut look.text, &BAND_TEXT);
    });
    if look != before {
        mem.set_look(look);
        trace(look);
    }
}

/// A drop-down of [`STYLES`] editing `style`, shared by the band and the
/// placement dialog. `region` names the button; each row is `<region>.<i>`.
pub fn style_picker(ui: &mut Ui, style: &mut BorderStyle, region: &str) {
    use crate::text::panels::formfield::border_style_label;
    let combo = egui::ComboBox::from_id_salt(region)
        .selected_text(border_style_label(*style))
        .show_ui(ui, |ui| {
            for (i, each) in STYLES.into_iter().enumerate() {
                let row = ui.selectable_value(style, each, border_style_label(each));
                crate::diag::ui_rect_visible(
                    &format!("{region}.{i}"), // ui-text-exempt: trace region name
                    row.rect,
                    ui.clip_rect(),
                );
            }
        });
    crate::diag::ui_rect(region, combo.response.on_hover_text(t::style_hover()).rect);
}

/// Font, size and ink of a text or choice field's value, shared by the band
/// and the placement dialog.
pub fn text_controls(ui: &mut Ui, text: &mut TextLook, regions: &TextRegions) {
    use crate::colourpick::{self, Pick as Ink, Shown, Spaces};
    use crate::text::panels::formfield::{text_font_name, text_size_auto};
    use pdfcer_core::fontdata::Std14;
    let combo = egui::ComboBox::from_id_salt(regions.font)
        .selected_text(text_font_name(text.font))
        .show_ui(ui, |ui| {
            for (i, face) in Std14::ALL.into_iter().enumerate() {
                let row = ui.selectable_value(&mut text.font, face, text_font_name(face));
                crate::diag::ui_rect_visible(
                    &format!("{}.{i}", regions.font), // ui-text-exempt: trace region name
                    row.rect,
                    ui.clip_rect(),
                );
            }
        });
    crate::diag::ui_rect(
        regions.font,
        combo.response.on_hover_text(t::font_hover()).rect,
    );

    let kind = entry::Kind::Length(entry::LengthUnit::Point);
    let (widget, refusal) = if text.size <= 0.0 {
        entry::drag_value_unlabelled(ui, &mut text.size, kind)
    } else {
        entry::drag_value(ui, &mut text.size, kind)
    };
    let response = refusal
        .show(
            ui.add(
                widget
                    .speed(0.5)
                    .range(0.0..=144.0)
                    .custom_formatter(|n, _| {
                        if n <= 0.0 {
                            text_size_auto().to_owned()
                        } else {
                            format!("{n:.0}")
                        }
                    }),
            ),
        )
        .on_hover_text(t::size_hover());
    crate::diag::ui_rect(regions.size, response.rect);

    ui.label(t::text_colour_label())
        .on_hover_text(t::text_colour_hover());
    if let Some(pick) = colourpick::show(
        ui,
        regions.colour,
        Shown::Agreed(Ink {
            ink: text.colour.into(),
            opacity: None,
        }),
        Spaces::ALL,
        regions.colour,
        "",
    ) {
        text.colour = pick.ink.into();
    }
}

/// A swatch's pick as the draft's value: `Remove` states nothing.
const fn resolve(pick: Pick) -> Option<pdfcer_core::forms::MkColor> {
    match pick {
        Pick::Set(colour) => Some(colour),
        Pick::Remove => None,
    }
}

/// The band's one trace line, read by `forms_style_sets_the_next_field`.
fn trace(look: FieldLook) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "form-look width={} style={} font={:?} size={} bg={} bc={}",
            look.border_width,
            look.border_style.token(),
            look.text.font,
            look.text.size,
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
