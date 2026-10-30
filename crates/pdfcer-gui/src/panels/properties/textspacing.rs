//! # `panels::properties::textspacing` — tracking, word spacing, width and baseline
//!
//! A fold under the Text section with Acrobat's `Tc`/`Tw`/`Tz`/`Ts` controls,
//! sent through `StyleChange` to `FormatRequest::char_spacing`, `word_spacing`,
//! `h_scale`, `script` and `rise`. Like *Drawn as*, the rows apply rather than
//! reflect: the extraction reads no per-run spacing back, so each row holds the
//! operator's last value (egui memory, per row) and *Set* sends it. Spacing and
//! shift are in thousandths of an em (`MetricSpec::Relative`), the unit the
//! engine re-derives at every run's own size.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::text::panels::properties as t;
use pdfcer_gui_base::editactions::ScriptShift;

/// The fold's header.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.text.spacing";
/// Prefix of each row's value and *Set* regions: `<prefix>.<row>.value|set`.
// ui-text-exempt: trace region name, never displayed
pub const ROW_REGION: &str = "properties.text.spacing";
/// Prefix of the three position buttons: `<prefix>.normal|super|sub`.
// ui-text-exempt: trace region name, never displayed
pub const SCRIPT_REGION: &str = "properties.text.script";

/// One numeric row.
struct Row {
    /// Region and memory key.
    key: &'static str,
    label: &'static str,
    hint: &'static str,
    suffix: &'static str,
    range: std::ops::RangeInclusive<f64>,
    start: f64,
    change: fn(f64) -> StyleChange,
}

fn rows() -> [Row; 4] {
    [
        Row {
            key: "tracking",
            label: t::text_tracking_label(),
            hint: t::text_tracking_hint(),
            suffix: t::per_mille_em_suffix(),
            range: -200.0..=1000.0,
            start: 0.0,
            change: StyleChange::CharSpacing,
        },
        Row {
            key: "word",
            label: t::text_word_spacing_label(),
            hint: t::text_word_spacing_hint(),
            suffix: t::per_mille_em_suffix(),
            range: -500.0..=1000.0,
            start: 0.0,
            change: StyleChange::WordSpacing,
        },
        Row {
            key: "width",
            label: t::text_width_label(),
            hint: t::text_width_hint(),
            suffix: t::percent_suffix(),
            range: 1.0..=400.0,
            start: 100.0,
            change: StyleChange::HScale,
        },
        Row {
            key: "rise",
            label: t::text_rise_label(),
            hint: t::text_rise_hint(),
            suffix: t::per_mille_em_suffix(),
            range: -1000.0..=1000.0,
            start: 0.0,
            change: StyleChange::Rise,
        },
    ]
}

/// Draw the fold for the runs `runs` on `page`.
pub fn fold(ui: &mut Ui, page: usize, runs: &[usize], actions: &mut Vec<Action>) {
    let header = egui::CollapsingHeader::new(t::text_spacing_heading())
        // ui-text-exempt: an egui id salt, never displayed
        .id_salt("properties-text-spacing")
        .default_open(false)
        .show(ui, |ui| {
            for row in rows() {
                if let Some(change) = numeric(ui, &row) {
                    push(actions, page, runs, change);
                }
            }
            if let Some(shift) = script_row(ui) {
                push(actions, page, runs, StyleChange::Script(shift));
            }
        });
    crate::diag::ui_rect_visible(REGION, header.header_response.rect, ui.clip_rect());
}

fn push(actions: &mut Vec<Action>, page: usize, runs: &[usize], change: StyleChange) {
    actions.push(Action::TextStyle {
        page,
        runs: runs.to_vec(),
        change,
    });
}

/// One value and its *Set* button; the change when *Set* is pressed.
fn numeric(ui: &mut Ui, row: &Row) -> Option<StyleChange> {
    let id = egui::Id::new(("properties-text-spacing", row.key));
    let mut value = ui.data(|d| d.get_temp::<f64>(id)).unwrap_or(row.start);
    let mut pressed = false;
    ui.horizontal(|ui| {
        ui.label(row.label).on_hover_text(row.hint);
        let field = ui.add(
            egui::DragValue::new(&mut value)
                .range(row.range.clone())
                .speed(1.0)
                .suffix(row.suffix),
        );
        crate::diag::ui_rect_visible(
            &format!("{ROW_REGION}.{}.value", row.key),
            field.rect,
            ui.clip_rect(),
        );
        let set = ui.button(t::text_spacing_set()).on_hover_text(row.hint);
        crate::diag::ui_rect_visible(
            &format!("{ROW_REGION}.{}.set", row.key),
            set.rect,
            ui.clip_rect(),
        );
        pressed = set.clicked();
    });
    ui.data_mut(|d| d.insert_temp(id, value));
    pressed.then(|| (row.change)(value))
}

/// Normal, Superscript, Subscript; the one pressed.
fn script_row(ui: &mut Ui) -> Option<ScriptShift> {
    // ui-text-exempt: region suffixes, never displayed
    let keys = ["normal", "super", "sub"];
    let shifts = [
        ScriptShift::Normal,
        ScriptShift::Superscript,
        ScriptShift::Subscript,
    ];
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.label(t::text_script_label());
        for ((key, name), shift) in keys.into_iter().zip(t::text_script_names()).zip(shifts) {
            let button = ui.button(name).on_hover_text(t::text_script_hint());
            crate::diag::ui_rect_visible(
                &format!("{SCRIPT_REGION}.{key}"),
                button.rect,
                ui.clip_rect(),
            );
            if button.clicked() {
                chosen = Some(shift);
            }
        }
    });
    chosen
}
