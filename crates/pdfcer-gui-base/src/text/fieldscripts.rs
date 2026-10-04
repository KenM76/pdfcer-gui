//! # `text::fieldscripts` — the words for a form field's format, validate
//! and calculate scripts
//!
//! The scripts run in other readers, not in pdfcer, so every sentence that
//! describes one says what it does there; a format changes only the display,
//! never the stored value, and a range is written for other readers to
//! enforce.

use pdfcer_core::edit::FieldScriptChange;
use pdfcer_core::form_script::{AdvisoryHelper, CalcHelper, FormatHelper, ScriptClass, SimpleOp};

use crate::fieldscript::{DraftError, FormatKind};

/// The section heading.
#[must_use]
pub fn heading() -> String {
    "Format, validation and calculation".to_owned()
}

/// Under the heading.
#[must_use]
pub fn intro() -> String {
    "Written as Acrobat's own helper scripts, which other readers run when the form is filled."
        .to_owned()
}

/// The three tabs.
#[must_use]
pub fn tab(trigger: &str) -> String {
    match trigger {
        "format" => "Format",
        "validate" => "Validate",
        _ => "Calculate",
    }
    .to_owned()
}

/// When the trigger holds a script pdfcer cannot describe.
#[must_use]
pub fn custom_held() -> String {
    "This field already carries a script here that pdfcer cannot describe. Applying replaces it."
        .to_owned()
}

/// A format kind, in the chooser.
#[must_use]
pub fn kind(kind: FormatKind) -> String {
    match kind {
        FormatKind::None => "None",
        FormatKind::Number => "Number",
        FormatKind::Percent => "Percentage",
        FormatKind::Date => "Date",
        FormatKind::Time => "Time",
        FormatKind::Special => "Special",
    }
    .to_owned()
}

/// The Number and Percent parameters.
#[must_use]
pub fn decimals() -> String {
    "Decimal places".to_owned()
}

/// The separator chooser's label.
#[must_use]
pub fn separator() -> String {
    "Separators".to_owned()
}

/// `sepStyle` 0–4, shown as the number it produces.
#[must_use]
pub fn separator_style(code: i64) -> String {
    match code {
        0 => "1,234.56",
        1 => "1234.56",
        2 => "1.234,56",
        3 => "1234,56",
        4 => "1'234.56",
        _ => "?",
    }
    .to_owned()
}

/// The negative-number chooser's label.
#[must_use]
pub fn negative() -> String {
    "Negative numbers".to_owned()
}

/// `negStyle` 0–3.
#[must_use]
pub fn negative_style(code: i64) -> String {
    match code {
        0 => "-1,234.56",
        1 => "1,234.56 in red",
        2 => "(1,234.56)",
        3 => "(1,234.56) in red",
        _ => "?",
    }
    .to_owned()
}

/// The currency symbol field's label.
#[must_use]
pub fn currency() -> String {
    "Currency symbol".to_owned()
}

/// The checkbox for the symbol's side.
#[must_use]
pub fn prepend() -> String {
    "Before the number".to_owned()
}

/// The date format field's label.
#[must_use]
pub fn date_format() -> String {
    "Date format".to_owned()
}

/// The presets menu beside the date format.
#[must_use]
pub fn presets() -> String {
    "Presets".to_owned()
}

/// The hover on the date format field.
#[must_use]
pub fn date_hover() -> String {
    "d, m, y for day, month and year; mmm for a month's short name, mmmm for its full name; HH, MM, ss and tt for a time."
        .to_owned()
}

/// The time format chooser's label.
#[must_use]
pub fn time_format() -> String {
    "Time format".to_owned()
}

/// The special chooser's label.
#[must_use]
pub fn special() -> String {
    "Mask".to_owned()
}

/// `AFSpecial_Format`'s selector 0–3.
#[must_use]
pub fn special_kind(selector: i64) -> String {
    match selector {
        0 => "Zip code",
        1 => "Zip+4",
        2 => "Phone number",
        3 => "Social security number",
        _ => "?",
    }
    .to_owned()
}

/// The live sample under a number or percentage format.
#[must_use]
pub fn preview(sample: &str, shown: &str, red: bool) -> String {
    let red = if red { ", in red" } else { "" };
    format!("{sample} shows as {shown}{red}")
}

/// Under every format.
#[must_use]
pub fn stored_note() -> String {
    "Only the display changes: the value stored in the field stays as typed.".to_owned()
}

/// The two range checkboxes.
#[must_use]
pub fn lowest() -> String {
    "Lowest allowed".to_owned()
}

/// See [`lowest`].
#[must_use]
pub fn highest() -> String {
    "Highest allowed".to_owned()
}

/// Under the range.
#[must_use]
pub fn range_note() -> String {
    "Other readers refuse a value outside the range. pdfcer records it for them and does not enforce it itself."
        .to_owned()
}

/// The calculate checkbox.
#[must_use]
pub fn calculated() -> String {
    "Calculated from other fields".to_owned()
}

/// Before the operation chooser.
#[must_use]
pub fn value_is_the() -> String {
    "Value is the".to_owned()
}

/// An operation, in the chooser.
#[must_use]
pub fn op(op: SimpleOp) -> String {
    match op {
        SimpleOp::Sum => "sum",
        SimpleOp::Average => "average",
        SimpleOp::Product => "product",
        SimpleOp::Minimum => "minimum",
        SimpleOp::Maximum => "maximum",
    }
    .to_owned()
}

/// Before the operand list.
#[must_use]
pub fn of_these() -> String {
    "of these fields:".to_owned()
}

/// When the form has nothing else to calculate from.
#[must_use]
pub fn no_operands_available() -> String {
    "There are no other text or drop-down fields to calculate from.".to_owned()
}

/// Under the calculation.
#[must_use]
pub fn order_note() -> String {
    "A newly calculated field joins the end of the form's calculation order, which other readers run from first to last."
        .to_owned()
}

/// The Apply button.
#[must_use]
pub fn apply() -> String {
    "Apply".to_owned()
}

/// The Apply button when it would displace an indescribable script.
#[must_use]
pub fn replace_script() -> String {
    "Replace script".to_owned()
}

/// Hover on a greyed Apply.
#[must_use]
pub fn no_change() -> String {
    "Nothing to apply: this is what the field already holds.".to_owned()
}

/// Why a draft cannot be applied.
#[must_use]
pub fn draft_error(error: &DraftError) -> String {
    match error {
        DraftError::EmptyDate => "Type a date format, or choose a preset.".to_owned(),
        DraftError::NotANumber(typed) => format!("\"{typed}\" is not a number."),
        DraftError::RangeInverted => "The lowest allowed value is above the highest.".to_owned(),
        DraftError::NoOperands => "Tick at least one field to calculate from.".to_owned(),
    }
}

/// The status lines after a script verb.
#[must_use]
pub fn changed(change: &FieldScriptChange) -> Vec<String> {
    let what = trigger_noun(change.trigger);
    let mut lines = vec![match &change.applied {
        Some(class) => format!("{}: {what} set to {}.", change.name, describe(class)),
        None => format!("{}: {what} removed.", change.name),
    }];
    if let Some(old) = &change.replaced
        && (matches!(old, ScriptClass::Custom) || change.applied.is_none())
    {
        lines.push(format!("It replaced {}.", describe(old)));
    }
    if change.keystroke_paired {
        lines.push(if change.applied.is_some() {
            "A matching input filter was written with it, as Acrobat does.".to_owned()
        } else {
            "Its input filter was removed with it.".to_owned()
        });
    }
    if matches!(change.applied, Some(ScriptClass::Advisory(_))) {
        lines.push(range_note());
    }
    if let Some(order) = &change.calculation_order {
        if let Some(p) = order.position {
            let at_end = if order.appended_at_end {
                ", added at the end"
            } else {
                ""
            };
            lines.push(format!(
                "It is number {} of {} in the form's calculation order{at_end}.",
                p + 1,
                order.entries
            ));
        }
        if order.array_created {
            lines.push("The form had no calculation order and now has one.".to_owned());
        }
        if order.array_removed {
            lines.push(
                "No calculations are left, so the form's calculation order was removed.".to_owned(),
            );
        }
    }
    lines
}

fn trigger_noun(trigger: &str) -> &'static str {
    match trigger {
        "format" => "format",
        "validate" => "validation",
        "calculate" => "calculation",
        _ => "script",
    }
}

/// A script, in the operator's words.
#[must_use]
pub fn describe(class: &ScriptClass) -> String {
    match class {
        ScriptClass::Calculate(CalcHelper::Simple { op: o, operands }) => {
            let names: Vec<_> = operands
                .iter()
                .map(|n| String::from_utf8_lossy(n).into_owned())
                .collect();
            format!("the {} of {}", op(*o), names.join(", "))
        }
        ScriptClass::Format(f) => describe_format(f),
        ScriptClass::Advisory(AdvisoryHelper::RangeValidate { lower, upper }) => {
            match (lower, upper) {
                (Some(lo), Some(hi)) => format!("a range from {lo} to {hi}"),
                (Some(lo), None) => format!("a range of {lo} or more"),
                (None, Some(hi)) => format!("a range of {hi} or less"),
                (None, None) => "a range with no bounds".to_owned(),
            }
        }
        ScriptClass::Advisory(AdvisoryHelper::Keystroke { name }) => {
            format!("an input filter ({name})")
        }
        ScriptClass::Custom => "a script pdfcer cannot describe".to_owned(),
    }
}

fn describe_format(f: &FormatHelper) -> String {
    match f {
        FormatHelper::Number {
            decimals, currency, ..
        } => {
            let symbol = String::from_utf8_lossy(currency);
            let symbol = symbol.trim();
            let with = if symbol.is_empty() {
                String::new()
            } else {
                format!(" with {symbol}")
            };
            format!("a number, {decimals} decimal places{with}")
        }
        FormatHelper::Percent { decimals, .. } => {
            format!("a percentage, {decimals} decimal places")
        }
        FormatHelper::Date { index } => {
            let shown = pdfcer_core::form_script::datetime::date_format(*index).unwrap_or("?");
            format!("a date ({shown})")
        }
        FormatHelper::DateEx { format } => {
            format!("a date ({})", String::from_utf8_lossy(format))
        }
        FormatHelper::Time { index } => {
            let shown = pdfcer_core::form_script::datetime::time_format(*index).unwrap_or("?");
            format!("a time ({shown})")
        }
        FormatHelper::Special { selector } => special_kind(*selector).to_lowercase(),
    }
}
