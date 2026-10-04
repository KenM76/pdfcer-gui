//! # `panels::properties::fieldscripts` — a text or drop-down field's format,
//! validate and calculate scripts
//!
//! Three tabs, each a draft of one engine helper
//! (`pdfcer_gui_base::fieldscript`), read from what the field holds and
//! applied by `FieldAction::SetScript`. The drafts live in egui memory keyed
//! by field and are re-read whenever the document's edit epoch moves, so an
//! Apply, an Undo or another edit shows what the document now holds.
//!
//! Drawn only for the kinds the engine writes scripts for: a text field and
//! a combo box (`EditError::FieldScriptWrongFieldType` names the rest).

use egui::Ui;
use pdfcer_core::form_script::datetime::{DATE_FORMATS, TIME_FORMATS};
use pdfcer_core::form_script::format::{FormatOutcome, render};
use pdfcer_core::form_script::{ScriptClass, Trigger};
use pdfcer_core::forms::{AcroForm, Field, FieldFlags, FieldType};
use pdfcer_gui_base::fieldscript::{
    CalcDraft, DraftError, FORMAT_KINDS, FormatDraft, FormatKind, MAX_DECIMALS, OPS, RangeDraft,
    ScriptEdit,
};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::app::state::OpenDoc;
use crate::text::fieldscripts as t;

/// The section's rect, for `ui-verify`.
pub const REGION: &str = "properties.field_scripts"; // ui-text-exempt: trace region name, never displayed
/// Each tab publishes `<this>.<trigger token>`, e.g. `….tab.calculate`.
pub const REGION_TAB: &str = "properties.field_scripts.tab"; // ui-text-exempt: trace region name, never displayed
/// The Calculate tab's on/off checkbox.
pub const REGION_CALCULATED: &str = "properties.field_scripts.calculated"; // ui-text-exempt: trace region name, never displayed
/// Each operand checkbox publishes `<this>.<field name>`.
pub const REGION_OPERAND: &str = "properties.field_scripts.operand"; // ui-text-exempt: trace region name, never displayed
/// The Apply button of the tab showing.
pub const REGION_APPLY: &str = "properties.field_scripts.apply"; // ui-text-exempt: trace region name, never displayed
/// The two range checkboxes and their fields.
pub const REGION_LOWEST: &str = "properties.field_scripts.lowest"; // ui-text-exempt: trace region name, never displayed
/// See [`REGION_LOWEST`].
pub const REGION_HIGHEST: &str = "properties.field_scripts.highest"; // ui-text-exempt: trace region name, never displayed

/// Samples for the live preview under a number and a percentage format.
const NUMBER_SAMPLE: &str = "-1234.5";
const PERCENT_SAMPLE: &str = "0.125";

/// What one trigger holds now.
#[derive(Clone, Debug, PartialEq)]
enum Held<T> {
    Absent,
    Helper(T),
    /// A script the drafts cannot represent; Apply replaces it.
    Foreign,
}

impl<T> Held<T> {
    const fn helper(&self) -> Option<&T> {
        match self {
            Self::Helper(x) => Some(x),
            _ => None,
        }
    }
}

/// The section's state for one field.
#[derive(Clone)]
struct Drafts {
    epoch: u64,
    tab: Trigger,
    format_held: Held<pdfcer_core::form_script::FormatHelper>,
    range_held: Held<pdfcer_core::form_script::AdvisoryHelper>,
    calc_held: Held<pdfcer_core::form_script::CalcHelper>,
    format: FormatDraft,
    range: RangeDraft,
    calc: CalcDraft,
}

impl Drafts {
    /// Read the field's three triggers from the document.
    fn read(doc: &OpenDoc, fqn: &str, tab: Trigger) -> Self {
        let inventory = pdfcer_core::form_script::inventory::inventory(&doc.session.view());
        let class = |trigger: Trigger| {
            inventory
                .scripts
                .iter()
                .find(|s| s.field == fqn && s.trigger == trigger)
                .map(|s| s.class.clone())
        };
        let format_held = match class(Trigger::Format) {
            None => Held::Absent,
            Some(ScriptClass::Format(f)) => Held::Helper(f),
            Some(_) => Held::Foreign,
        };
        let range_held = match class(Trigger::Validate) {
            None => Held::Absent,
            Some(ScriptClass::Advisory(
                a @ pdfcer_core::form_script::AdvisoryHelper::RangeValidate { .. },
            )) => Held::Helper(a),
            Some(_) => Held::Foreign,
        };
        let calc_held = match class(Trigger::Calculate) {
            None => Held::Absent,
            Some(ScriptClass::Calculate(c)) => Held::Helper(c),
            Some(_) => Held::Foreign,
        };
        let drafts = Self {
            epoch: doc.edit_epoch,
            tab,
            format: FormatDraft::from_helper(format_held.helper()),
            range: RangeDraft::from_helper(range_held.helper()),
            calc: CalcDraft::from_helper(calc_held.helper()),
            format_held,
            range_held,
            calc_held,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "field-scripts-read field={fqn} format={} validate={} calculate={}",
                held_token(&drafts.format_held),
                held_token(&drafts.range_held),
                held_token(&drafts.calc_held),
            )
        });
        drafts
    }
}

const fn held_token<T>(held: &Held<T>) -> &'static str {
    match held {
        Held::Absent => "none",
        Held::Helper(_) => "helper",
        Held::Foreign => "foreign",
    }
}

/// Whether the engine writes scripts for this field's kind.
#[must_use]
pub fn takes_scripts(field: &Field) -> bool {
    match field.field_type {
        Some(FieldType::Text) => true,
        Some(FieldType::Choice) => field.flags.has(FieldFlags::COMBO),
        _ => false,
    }
}

/// Draw the section for the selected field, if its kind takes scripts.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    form: &AcroForm,
    field: &Field,
    actions: &mut Vec<Action>,
) {
    if !takes_scripts(field) {
        return;
    }
    let fqn = field.fully_qualified_name.as_str();
    let id = egui::Id::new(("field-scripts", fqn));
    let stored = ui.data(|d| d.get_temp::<Drafts>(id));
    let mut drafts = match stored {
        Some(d) if d.epoch == doc.edit_epoch => d,
        Some(d) => Drafts::read(doc, fqn, d.tab),
        None => Drafts::read(doc, fqn, Trigger::Format),
    };
    let top = ui.cursor().min;
    ui.label(t::heading());
    ui.small(t::intro());
    ui.add_space(4.0);
    tabs(ui, &mut drafts.tab);
    ui.add_space(4.0);
    match drafts.tab {
        Trigger::Validate => validate_tab(ui, &mut drafts, fqn, actions),
        Trigger::Calculate => calculate_tab(ui, &mut drafts, form, fqn, actions),
        _ => format_tab(ui, &mut drafts, fqn, actions),
    }
    let rect = egui::Rect::from_min_max(top, ui.min_rect().max);
    crate::diag::ui_rect_visible(REGION, rect, ui.clip_rect());
    ui.data_mut(|d| d.insert_temp(id, drafts));
}

fn tabs(ui: &mut Ui, tab: &mut Trigger) {
    ui.horizontal(|ui| {
        for trigger in [Trigger::Format, Trigger::Validate, Trigger::Calculate] {
            let r = ui.selectable_value(tab, trigger, t::tab(trigger.token()));
            let region = format!("{REGION_TAB}.{}", trigger.token());
            crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
        }
    });
}

fn format_tab(ui: &mut Ui, drafts: &mut Drafts, fqn: &str, actions: &mut Vec<Action>) {
    if drafts.format_held == Held::Foreign {
        ui.small(t::custom_held());
    }
    let d = &mut drafts.format;
    egui::ComboBox::from_id_salt("field-scripts.kind")
        .selected_text(t::kind(d.kind))
        .show_ui(ui, |ui| {
            for kind in FORMAT_KINDS {
                ui.selectable_value(&mut d.kind, kind, t::kind(kind));
            }
        });
    match d.kind {
        FormatKind::None => {}
        FormatKind::Number => {
            decimals_row(ui, d);
            numeric_rows(ui, d);
        }
        FormatKind::Percent => decimals_row(ui, d),
        FormatKind::Date => date_rows(ui, d),
        FormatKind::Time => index_combo(
            ui,
            "field-scripts.time",
            &t::time_format(),
            &mut d.time_index,
            &TIME_FORMATS.map(str::to_owned),
        ),
        FormatKind::Special => {
            let names = [0, 1, 2, 3].map(t::special_kind);
            index_combo(
                ui,
                "field-scripts.special",
                &t::special(),
                &mut d.special,
                &names,
            );
        }
    }
    if d.kind != FormatKind::None {
        preview(ui, d);
        ui.small(t::stored_note());
    }
    let draft = d.helper();
    apply_row(
        ui,
        draft,
        &drafts.format_held,
        fqn,
        ScriptEdit::Format,
        actions,
    );
}

fn decimals_row(ui: &mut Ui, d: &mut FormatDraft) {
    ui.horizontal(|ui| {
        ui.label(t::decimals());
        ui.add(egui::DragValue::new(&mut d.decimals).range(0..=MAX_DECIMALS));
    });
    let names = [0, 1, 2, 3, 4].map(t::separator_style);
    index_combo(
        ui,
        "field-scripts.sep",
        &t::separator(),
        &mut d.separator_style,
        &names,
    );
}

fn numeric_rows(ui: &mut Ui, d: &mut FormatDraft) {
    let names = [0, 1, 2, 3].map(t::negative_style);
    index_combo(
        ui,
        "field-scripts.neg",
        &t::negative(),
        &mut d.negative_style,
        &names,
    );
    ui.horizontal(|ui| {
        ui.label(t::currency());
        // escape-disposition: keeps-draft — the draft lives in egui memory until
        // Apply; Escape leaves the box and the symbol stays in it.
        ui.add(egui::TextEdit::singleline(&mut d.currency).desired_width(40.0));
        ui.checkbox(&mut d.prepend_currency, t::prepend());
    });
}

fn date_rows(ui: &mut Ui, d: &mut FormatDraft) {
    ui.horizontal(|ui| {
        ui.label(t::date_format());
        // escape-disposition: keeps-draft — as the currency box.
        ui.text_edit_singleline(&mut d.date)
            .on_hover_text(t::date_hover());
        ui.menu_button(t::presets(), |ui| {
            for preset in DATE_FORMATS {
                if ui.button(preset).clicked() {
                    preset.clone_into(&mut d.date);
                    ui.close();
                }
            }
        });
    });
}

/// A chooser over `names`, writing the chosen index into `value`.
fn index_combo(ui: &mut Ui, salt: &str, label: &str, value: &mut i64, names: &[String]) {
    ui.horizontal(|ui| {
        ui.label(label);
        let shown = usize::try_from(*value)
            .ok()
            .and_then(|i| names.get(i))
            .cloned()
            .unwrap_or_else(|| value.to_string());
        egui::ComboBox::from_id_salt(salt)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for (i, name) in (0_i64..).zip(names) {
                    ui.selectable_value(value, i, name);
                }
            });
    });
}

/// The engine's own formatter run over a sample, for Number and Percent.
fn preview(ui: &mut Ui, d: &FormatDraft) {
    let sample = match d.kind {
        FormatKind::Number => NUMBER_SAMPLE,
        FormatKind::Percent => PERCENT_SAMPLE,
        _ => return,
    };
    let Ok(Some(helper)) = d.helper() else {
        return;
    };
    if let FormatOutcome::Rendered(shown) = render(&helper, sample, Default::default()) {
        ui.small(t::preview(sample, &shown.text, shown.red));
    }
}

fn validate_tab(ui: &mut Ui, drafts: &mut Drafts, fqn: &str, actions: &mut Vec<Action>) {
    if drafts.range_held == Held::Foreign {
        ui.small(t::custom_held());
    }
    let d = &mut drafts.range;
    bound_row(
        ui,
        &t::lowest(),
        REGION_LOWEST,
        &mut d.lower_on,
        &mut d.lower,
    );
    bound_row(
        ui,
        &t::highest(),
        REGION_HIGHEST,
        &mut d.upper_on,
        &mut d.upper,
    );
    ui.small(t::range_note());
    let draft = d.helper();
    apply_row(
        ui,
        draft,
        &drafts.range_held,
        fqn,
        ScriptEdit::Validate,
        actions,
    );
}

fn bound_row(ui: &mut Ui, label: &str, region: &str, on: &mut bool, typed: &mut String) {
    ui.horizontal(|ui| {
        let r = ui.checkbox(on, label);
        crate::diag::ui_rect_visible(region, r.rect, ui.clip_rect());
        // escape-disposition: keeps-draft — as the currency box.
        let r = ui.add_enabled(*on, egui::TextEdit::singleline(typed).desired_width(80.0));
        crate::diag::ui_rect_visible(&format!("{region}.value"), r.rect, ui.clip_rect());
    });
}

fn calculate_tab(
    ui: &mut Ui,
    drafts: &mut Drafts,
    form: &AcroForm,
    fqn: &str,
    actions: &mut Vec<Action>,
) {
    if drafts.calc_held == Held::Foreign {
        ui.small(t::custom_held());
    }
    let d = &mut drafts.calc;
    let r = ui.checkbox(&mut d.on, t::calculated());
    crate::diag::ui_rect_visible(REGION_CALCULATED, r.rect, ui.clip_rect());
    if d.on {
        ui.horizontal(|ui| {
            ui.label(t::value_is_the());
            egui::ComboBox::from_id_salt("field-scripts.op")
                .selected_text(t::op(d.op))
                .show_ui(ui, |ui| {
                    for op in OPS {
                        ui.selectable_value(&mut d.op, op, t::op(op));
                    }
                });
            ui.label(t::of_these());
        });
        operands(ui, d, form, fqn);
        ui.small(t::order_note());
    }
    let draft = d.helper();
    apply_row(
        ui,
        draft,
        &drafts.calc_held,
        fqn,
        ScriptEdit::Calculate,
        actions,
    );
}

/// One checkbox per other text or drop-down field, in form order.
fn operands(ui: &mut Ui, d: &mut CalcDraft, form: &AcroForm, fqn: &str) {
    let candidates: Vec<&str> = form
        .fields
        .iter()
        .filter(|f| {
            f.fully_qualified_name != fqn
                && matches!(f.field_type, Some(FieldType::Text | FieldType::Choice))
        })
        .map(|f| f.fully_qualified_name.as_str())
        .collect();
    if candidates.is_empty() {
        ui.small(t::no_operands_available());
        return;
    }
    for name in candidates {
        let mut on = d.operands.iter().any(|o| o == name);
        let r = ui.checkbox(&mut on, name);
        crate::diag::ui_rect_visible(&format!("{REGION_OPERAND}.{name}"), r.rect, ui.clip_rect());
        if r.changed() {
            d.toggle(name, on);
        }
    }
}

/// Apply, greyed with its reason when the draft is unusable or unchanged.
fn apply_row<T: PartialEq>(
    ui: &mut Ui,
    draft: Result<Option<T>, DraftError>,
    held: &Held<T>,
    fqn: &str,
    edit: impl FnOnce(Option<T>) -> ScriptEdit,
    actions: &mut Vec<Action>,
) {
    ui.add_space(4.0);
    let helper = match draft {
        Ok(helper) => helper,
        Err(why) => {
            ui.small(t::draft_error(&why));
            let r = ui.add_enabled(false, egui::Button::new(t::apply()));
            crate::diag::ui_rect_visible(REGION_APPLY, r.rect, ui.clip_rect());
            return;
        }
    };
    let unchanged = match (held, &helper) {
        (Held::Absent, None) => true,
        (Held::Helper(h), Some(d)) => h == d,
        _ => false,
    };
    let label = if *held == Held::Foreign {
        t::replace_script()
    } else {
        t::apply()
    };
    let r = ui
        .add_enabled(!unchanged, egui::Button::new(label))
        .on_disabled_hover_text(t::no_change());
    crate::diag::ui_rect_visible(REGION_APPLY, r.rect, ui.clip_rect());
    if r.clicked() {
        actions.push(
            FieldAction::SetScript {
                field: fqn.to_owned(),
                edit: Box::new(edit(helper)),
            }
            .into(),
        );
    }
}
