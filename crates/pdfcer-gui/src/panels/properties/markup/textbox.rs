//! # `panels::properties::markup::textbox` — restyling a text box
//!
//! The `/FreeText` rows of `set_text_annot_style`: border colour, width and
//! line style, background, text colour, font and size. Each row raises one
//! restyle naming only its own property; the engine measures the box's
//! wrapping before it re-bakes.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/markup/textbox.md`.

use egui::Ui;
use pdfcer_core::annot_author::{BorderDash, Color, TextAnnotSpec};
use pdfcer_core::edit::{StyleEdit, TextAnnotStyle};
use pdfcer_core::fontdata::{Std14, std14_styled};
use pdfcer_core::vartext::TextColor;
use pdfcer_gui_base::entry;

use super::textannot::untouched;
use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::canvas::markup::linestyle::{DashReading, LineStyle};
use crate::canvas::selection::annot::AnnotTarget;
use crate::text::panels::formfield as ff;
use crate::text::panels::properties as t;
use crate::text::panels::textannotstyle as ts;

/// The region the text box rows publish.
pub(super) const REGION: &str = "properties.markup.textbox"; // ui-text-exempt: trace region name, never displayed
/// The border width field's region.
pub(super) const WIDTH_REGION: &str = "properties.markup.textbox.width"; // ui-text-exempt: trace region name, never displayed
/// The text size field's region.
pub(super) const SIZE_REGION: &str = "properties.markup.textbox.size"; // ui-text-exempt: trace region name, never displayed
/// The border colour swatch's region.
pub(super) const BORDER_REGION: &str = "properties.markup.textbox.border"; // ui-text-exempt: trace region name, never displayed
/// The background swatch's region.
pub(super) const FILL_REGION: &str = "properties.markup.textbox.fill"; // ui-text-exempt: trace region name, never displayed
/// The text colour swatch's region.
pub(super) const INK_REGION: &str = "properties.markup.textbox.ink"; // ui-text-exempt: trace region name, never displayed

/// The widest border offered, in points; 0 removes the border.
const MAX_BORDER_PT: f64 = 12.0;
/// The largest text size offered, in points; 0 is auto-size.
const MAX_TEXT_PT: f64 = 144.0;
/// The Line style chooser's width.
const DASH_WIDTH: f32 = 180.0;

/// What a `/FreeText`'s spec says, in the terms these rows change.
#[derive(Debug, Clone)]
pub(super) struct TextBox {
    /// `/BS /W`, in points.
    pub(super) border_width: f64,
    /// Whether a border is drawn: `/C` present and a width above 0.
    pub(super) bordered: bool,
    /// `/BS /D`, as the chooser shows it.
    pub(super) dash: DashReading,
    /// The background; `rgb: None` when the box has none.
    pub(super) fill: super::Swatch,
    /// The text colour from `/DA`; `None` when it is CMYK.
    pub(super) text_rgb: Option<[u8; 3]>,
    /// The face from `/DA`.
    pub(super) font: Std14,
    /// The size from `/DA`, in points; 0 is auto-size.
    pub(super) font_size: f64,
}

impl TextBox {
    /// Read one off the engine's spec; `None` for any other face.
    pub(super) fn of(spec: &TextAnnotSpec) -> Option<Self> {
        let TextAnnotSpec::FreeText {
            font,
            font_size,
            color,
            border,
            border_width,
            frame,
            ..
        } = spec
        else {
            return None;
        };
        Some(Self {
            border_width: *border_width,
            bordered: border.is_some() && *border_width > 0.0,
            dash: dash_reading(frame.dash.as_ref()),
            fill: super::swatch_of(frame.fill.as_ref()),
            text_rgb: text_rgb(*color),
            font: *font,
            font_size: *font_size,
        })
    }
}

/// A `/BS /D` as the chooser shows it.
fn dash_reading(dash: Option<&BorderDash>) -> DashReading {
    match dash {
        None => DashReading::Solid,
        Some(d) => {
            LineStyle::of_pattern(d.pattern()).map_or(DashReading::Foreign, DashReading::Offered)
        }
    }
}

/// A `/DA` colour as a swatch shows it; `None` for CMYK, which an sRGB swatch
/// would narrow.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn text_rgb(ink: TextColor) -> Option<[u8; 3]> {
    let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    match ink {
        TextColor::Gray(g) => Some([byte(g); 3]),
        TextColor::Rgb(r, g, b) => Some([byte(r), byte(g), byte(b)]),
        TextColor::Cmyk(..) => None,
    }
}

/// A swatch value as its three device RGB components.
fn rgb_of(rgb: [u8; 3]) -> (f64, f64, f64) {
    (
        f64::from(rgb[0]) / 255.0,
        f64::from(rgb[1]) / 255.0,
        f64::from(rgb[2]) / 255.0,
    )
}

/// One colour swatch; `Some` only on the frame its picker closed changed.
fn swatch(ui: &mut Ui, salt: &str, rgb: [u8; 3], region: &str) -> Option<(f64, f64, f64)> {
    // The hint is shown for a mixed value only, which one box never has.
    crate::panels::properties::swatch::show(
        ui,
        salt,
        crate::panels::properties::swatch::Value::Agreed(rgb),
        region,
        "",
    )
    .map(rgb_of)
}

/// Raise one restyle of the box.
fn push(target: &AnnotTarget, actions: &mut Vec<Action>, style: TextAnnotStyle) {
    actions.push(Action::Annot(AnnotAction::SetTextAnnotStyle {
        id: target.id,
        style: Box::new(style),
    }));
}

/// **Draw the text box rows.** `colour` is the box's `/C`, its border colour.
pub(super) fn rows(
    ui: &mut Ui,
    colour: super::Swatch,
    tb: &TextBox,
    target: &AnnotTarget,
    actions: &mut Vec<Action>,
) {
    crate::diag::ui_rect(REGION, ui.max_rect());
    crate::diag::trace_changed("textbox-rows", || {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "textbox-rows id={} width={} bordered={} fill={} text={} font={:?} size={}",
            target.id.num,
            tb.border_width,
            u8::from(tb.bordered),
            u8::from(tb.fill.rgb.is_some()),
            u8::from(tb.text_rgb.is_some()),
            tb.font,
            tb.font_size
        )
    });
    border_rows(ui, colour, tb, target, actions);
    fill_row(ui, tb, target, actions);
    text_colour_row(ui, tb, target, actions);
    font_rows(ui, tb, target, actions);
    ui.label(egui::RichText::new(ts::text_box_note()).small().weak());
}

/// Border colour, width and — when a border is drawn — line style.
fn border_rows(
    ui: &mut Ui,
    colour: super::Swatch,
    tb: &TextBox,
    target: &AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let border = colour.rgb.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(ts::text_box_border_colour_label());
        // ui-text-exempt: an egui id salt, never displayed
        if let Some((r, g, b)) = swatch(ui, "properties-textbox-border", border, BORDER_REGION) {
            let color = Some(Color::Rgb(r, g, b));
            push(
                target,
                actions,
                TextAnnotStyle {
                    color,
                    ..untouched()
                },
            );
        }
    });
    if colour.narrowed {
        ui.label(
            egui::RichText::new(t::markup_colour_narrowed())
                .small()
                .weak(),
        );
    }

    let mut width = tb.border_width;
    ui.horizontal(|ui| {
        ui.label(ts::text_box_border_width_label());
        let (widget, refusal) = entry::drag_value(
            ui,
            &mut width,
            entry::Kind::Length(entry::LengthUnit::Point),
        );
        let range = 0.0..=MAX_BORDER_PT.max(tb.border_width);
        let response = refusal.show(ui.add(widget.range(range).speed(0.1)));
        crate::diag::ui_rect_visible(WIDTH_REGION, response.rect, ui.clip_rect());
        // On release, never per pixel: each commit is a re-bake and an undo
        // entry.
        if (response.drag_stopped() || response.lost_focus()) && width != tb.border_width {
            let border_width = Some(width);
            push(
                target,
                actions,
                TextAnnotStyle {
                    border_width,
                    ..untouched()
                },
            );
        }
    });

    if tb.bordered {
        ui.horizontal(|ui| {
            ui.label(t::markup_line_style_label());
            let picked = crate::canvas::markup::linestyle::chooser(
                ui,
                // ui-text-exempt: internal widget id, never displayed
                "properties-textbox-line-style",
                tb.dash,
                DASH_WIDTH,
            );
            if let Some(edit) = picked.and_then(LineStyle::style_edit) {
                let dash = Some(edit);
                push(
                    target,
                    actions,
                    TextAnnotStyle {
                        dash,
                        ..untouched()
                    },
                );
            }
        });
    }
}

/// The background: a swatch, and Clear when there is one.
fn fill_row(ui: &mut Ui, tb: &TextBox, target: &AnnotTarget, actions: &mut Vec<Action>) {
    // An absent background opens the picker on white without writing
    // anything until a colour is picked.
    let fill = tb.fill.rgb.unwrap_or([255, 255, 255]);
    ui.horizontal(|ui| {
        ui.label(ts::text_box_fill_label());
        // ui-text-exempt: an egui id salt, never displayed
        if let Some((r, g, b)) = swatch(ui, "properties-textbox-fill", fill, FILL_REGION) {
            let fill = Some(StyleEdit::Set(Color::Rgb(r, g, b)));
            push(
                target,
                actions,
                TextAnnotStyle {
                    fill,
                    ..untouched()
                },
            );
        }
        if tb.fill.rgb.is_none() {
            ui.label(egui::RichText::new(ts::text_box_fill_none()).small().weak());
        } else if ui.button(t::markup_clear()).clicked() {
            let fill = Some(StyleEdit::Clear);
            push(
                target,
                actions,
                TextAnnotStyle {
                    fill,
                    ..untouched()
                },
            );
        }
    });
    if tb.fill.narrowed {
        ui.label(
            egui::RichText::new(t::markup_colour_narrowed())
                .small()
                .weak(),
        );
    }
}

/// The text colour, `/DA`'s fill.
fn text_colour_row(ui: &mut Ui, tb: &TextBox, target: &AnnotTarget, actions: &mut Vec<Action>) {
    let ink = tb.text_rgb.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(ts::text_box_text_colour_label());
        // ui-text-exempt: an egui id salt, never displayed
        if let Some((r, g, b)) = swatch(ui, "properties-textbox-ink", ink, INK_REGION) {
            let text_color = Some(TextColor::Rgb(r, g, b));
            push(
                target,
                actions,
                TextAnnotStyle {
                    text_color,
                    ..untouched()
                },
            );
        }
    });
    if tb.text_rgb.is_none() {
        ui.label(
            egui::RichText::new(ts::text_box_text_colour_cmyk())
                .small()
                .weak(),
        );
    }
}

/// The face and the size.
fn font_rows(ui: &mut Ui, tb: &TextBox, target: &AnnotTarget, actions: &mut Vec<Action>) {
    let mut chosen = tb.font;
    ui.horizontal(|ui| {
        ui.label(ts::text_box_font_label());
        // ui-text-exempt: internal widget id, never displayed
        egui::ComboBox::from_id_salt(("properties-textbox-font", target.id.num))
            .selected_text(ff::text_font_name(tb.font))
            .show_ui(ui, |ui| {
                // The faces the re-bake lays text out in: `std14_styled` is
                // `None` exactly for the two symbolic faces `vartext` refuses.
                for f in Std14::ALL
                    .into_iter()
                    .filter(|f| std14_styled(*f, false, false).is_some())
                {
                    ui.selectable_value(&mut chosen, f, ff::text_font_name(f));
                }
            });
    });
    if chosen != tb.font {
        let font = Some(chosen);
        push(
            target,
            actions,
            TextAnnotStyle {
                font,
                ..untouched()
            },
        );
    }

    let mut size = tb.font_size;
    ui.horizontal(|ui| {
        ui.label(ts::text_box_size_label());
        let (widget, refusal) =
            entry::drag_value(ui, &mut size, entry::Kind::Length(entry::LengthUnit::Point));
        let range = 0.0..=MAX_TEXT_PT.max(tb.font_size);
        let response = refusal.show(ui.add(widget.range(range).speed(0.5)));
        crate::diag::ui_rect_visible(SIZE_REGION, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus()) && size != tb.font_size {
            let font_size = Some(size);
            push(
                target,
                actions,
                TextAnnotStyle {
                    font_size,
                    ..untouched()
                },
            );
        }
    });
}
