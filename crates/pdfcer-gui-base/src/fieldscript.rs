//! # `fieldscript` — a form field's format, validate and calculate scripts,
//! as the Properties panel edits them
//!
//! The engine authors only the helpers it can read back
//! (`pdfcer_core::form_script`): there is no free-text route into `/AA`. So
//! each tab here is a *draft* of one helper — read from what the field holds,
//! edited in the panel, and turned back into a helper (or `None`, which
//! clears) when Apply is pressed. A draft that cannot be a helper says why
//! instead.

use pdfcer_core::form_script::datetime::{DATE_FORMATS, TIME_FORMATS};
use pdfcer_core::form_script::{AdvisoryHelper, CalcHelper, FormatHelper, SimpleOp};

/// One script verb, as `FieldAction::SetScript` carries it. `None` clears.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEdit {
    /// `/AA /F`, and the paired `/K` the engine writes beside it.
    Format(Option<FormatHelper>),
    /// `/AA /V`.
    Validate(Option<AdvisoryHelper>),
    /// `/AA /C`, and the field's `/CO` entry.
    Calculate(Option<CalcHelper>),
}

/// The five operations, in the order the chooser lists them.
pub const OPS: [SimpleOp; 5] = [
    SimpleOp::Sum,
    SimpleOp::Average,
    SimpleOp::Product,
    SimpleOp::Minimum,
    SimpleOp::Maximum,
];

/// Which helper the Format tab is drafting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    /// No format: Apply clears `/F` and `/K`.
    None,
    /// `AFNumber_Format`.
    Number,
    /// `AFPercent_Format`.
    Percent,
    /// `AFDate_Format` / `AFDate_FormatEx`.
    Date,
    /// `AFTime_Format`.
    Time,
    /// `AFSpecial_Format`.
    Special,
}

/// The kinds, in the order the chooser lists them.
pub const FORMAT_KINDS: [FormatKind; 6] = [
    FormatKind::None,
    FormatKind::Number,
    FormatKind::Percent,
    FormatKind::Date,
    FormatKind::Time,
    FormatKind::Special,
];

/// Largest decimal count offered; Acrobat's Format tab stops at 10.
pub const MAX_DECIMALS: i64 = 10;

/// The Format tab's draft. Every kind's parameters are kept while another
/// kind is chosen, so switching back loses nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatDraft {
    /// The chosen kind.
    pub kind: FormatKind,
    /// Number and Percent: digits after the decimal separator.
    pub decimals: i64,
    /// Number and Percent: `sepStyle`, 0–4.
    pub separator_style: i64,
    /// Number: `negStyle`, 0–3.
    pub negative_style: i64,
    /// Number: `currStyle`, carried as read; Acrobat writes 0.
    pub currency_style: i64,
    /// Number: the currency symbol, empty for none.
    pub currency: String,
    /// Number: whether the symbol precedes the number.
    pub prepend_currency: bool,
    /// Date: the format string.
    pub date: String,
    /// Date: the predefined index it was read as, kept so an unchanged date
    /// is written back as the same `AFDate_Format` call.
    pub date_index: Option<i64>,
    /// Time: index into `TIME_FORMATS`.
    pub time_index: i64,
    /// Special: 0 zip, 1 zip+4, 2 phone, 3 social-security number.
    pub special: i64,
}

impl Default for FormatDraft {
    fn default() -> Self {
        Self {
            kind: FormatKind::None,
            decimals: 2,
            separator_style: 0,
            negative_style: 0,
            currency_style: 0,
            currency: String::new(),
            prepend_currency: true,
            date: DATE_FORMATS[2].to_owned(),
            date_index: None,
            time_index: 0,
            special: 0,
        }
    }
}

impl FormatDraft {
    /// The draft for what a field holds; `None` is no format.
    #[must_use]
    pub fn from_helper(helper: Option<&FormatHelper>) -> Self {
        let mut d = Self::default();
        match helper {
            None => {}
            Some(FormatHelper::Number {
                decimals,
                separator_style,
                negative_style,
                currency_style,
                currency,
                prepend_currency,
            }) => {
                d.kind = FormatKind::Number;
                d.decimals = *decimals;
                d.separator_style = *separator_style;
                d.negative_style = *negative_style;
                d.currency_style = *currency_style;
                d.currency = String::from_utf8_lossy(currency).into_owned();
                d.prepend_currency = *prepend_currency;
            }
            Some(FormatHelper::Percent {
                decimals,
                separator_style,
            }) => {
                d.kind = FormatKind::Percent;
                d.decimals = *decimals;
                d.separator_style = *separator_style;
            }
            Some(FormatHelper::Date { index }) => {
                d.kind = FormatKind::Date;
                if let Some(f) = pdfcer_core::form_script::datetime::date_format(*index) {
                    f.clone_into(&mut d.date);
                    d.date_index = Some(*index);
                }
            }
            Some(FormatHelper::DateEx { format }) => {
                d.kind = FormatKind::Date;
                d.date = String::from_utf8_lossy(format).into_owned();
            }
            Some(FormatHelper::Time { index }) => {
                d.kind = FormatKind::Time;
                d.time_index = *index;
            }
            Some(FormatHelper::Special { selector }) => {
                d.kind = FormatKind::Special;
                d.special = *selector;
            }
        }
        d
    }

    /// The helper Apply writes, `None` to clear, or why there is none.
    ///
    /// # Errors
    ///
    /// [`DraftError::EmptyDate`] for a Date with no format string.
    pub fn helper(&self) -> Result<Option<FormatHelper>, DraftError> {
        Ok(Some(match self.kind {
            FormatKind::None => return Ok(None),
            FormatKind::Number => FormatHelper::Number {
                decimals: self.decimals,
                separator_style: self.separator_style,
                negative_style: self.negative_style,
                currency_style: self.currency_style,
                currency: self.currency.trim().as_bytes().to_vec(),
                prepend_currency: self.prepend_currency,
            },
            FormatKind::Percent => FormatHelper::Percent {
                decimals: self.decimals,
                separator_style: self.separator_style,
            },
            FormatKind::Date => self.date_helper()?,
            FormatKind::Time => FormatHelper::Time {
                index: self.time_index,
            },
            FormatKind::Special => FormatHelper::Special {
                selector: self.special,
            },
        }))
    }

    fn date_helper(&self) -> Result<FormatHelper, DraftError> {
        let typed = self.date.trim();
        if typed.is_empty() {
            return Err(DraftError::EmptyDate);
        }
        let read_as = self
            .date_index
            .filter(|i| pdfcer_core::form_script::datetime::date_format(*i) == Some(typed));
        Ok(match read_as {
            Some(index) => FormatHelper::Date { index },
            None => FormatHelper::DateEx {
                format: typed.as_bytes().to_vec(),
            },
        })
    }

    /// The predefined time format the draft names, for the chooser.
    #[must_use]
    pub fn time_format(&self) -> &'static str {
        usize::try_from(self.time_index)
            .ok()
            .and_then(|i| TIME_FORMATS.get(i).copied())
            .unwrap_or("?")
    }
}

/// The Validate tab's draft: an optional lower and upper bound, typed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RangeDraft {
    /// Whether a lower bound is set.
    pub lower_on: bool,
    /// The lower bound, as typed.
    pub lower: String,
    /// Whether an upper bound is set.
    pub upper_on: bool,
    /// The upper bound, as typed.
    pub upper: String,
}

impl RangeDraft {
    /// The draft for what a field holds. Anything but a range reads as none.
    #[must_use]
    pub fn from_helper(helper: Option<&AdvisoryHelper>) -> Self {
        let Some(AdvisoryHelper::RangeValidate { lower, upper }) = helper else {
            return Self::default();
        };
        Self {
            lower_on: lower.is_some(),
            lower: lower.map(|v| v.to_string()).unwrap_or_default(),
            upper_on: upper.is_some(),
            upper: upper.map(|v| v.to_string()).unwrap_or_default(),
        }
    }

    /// The helper Apply writes, `None` to clear, or why there is none.
    ///
    /// # Errors
    ///
    /// [`DraftError::NotANumber`] for a ticked bound that does not parse,
    /// [`DraftError::RangeInverted`] when the lower bound exceeds the upper.
    pub fn helper(&self) -> Result<Option<AdvisoryHelper>, DraftError> {
        let lower = bound(self.lower_on, &self.lower)?;
        let upper = bound(self.upper_on, &self.upper)?;
        if let (Some(lo), Some(hi)) = (lower, upper)
            && lo > hi
        {
            return Err(DraftError::RangeInverted);
        }
        Ok((lower.is_some() || upper.is_some())
            .then_some(AdvisoryHelper::RangeValidate { lower, upper }))
    }
}

fn bound(on: bool, typed: &str) -> Result<Option<f64>, DraftError> {
    if !on {
        return Ok(None);
    }
    typed
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .map(Some)
        .ok_or_else(|| DraftError::NotANumber(typed.trim().to_owned()))
}

/// The Calculate tab's draft: an operation over other fields' values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalcDraft {
    /// Whether the field is calculated at all.
    pub on: bool,
    /// The operation.
    pub op: SimpleOp,
    /// The operand fields' fully-qualified names, in the order chosen.
    pub operands: Vec<String>,
}

impl Default for CalcDraft {
    fn default() -> Self {
        Self {
            on: false,
            op: SimpleOp::Sum,
            operands: Vec::new(),
        }
    }
}

impl CalcDraft {
    /// The draft for what a field holds.
    #[must_use]
    pub fn from_helper(helper: Option<&CalcHelper>) -> Self {
        let Some(CalcHelper::Simple { op, operands }) = helper else {
            return Self::default();
        };
        Self {
            on: true,
            op: *op,
            operands: operands
                .iter()
                .map(|o| String::from_utf8_lossy(o).into_owned())
                .collect(),
        }
    }

    /// Tick or untick one operand. A newly ticked one goes last.
    pub fn toggle(&mut self, name: &str, on: bool) {
        self.operands.retain(|o| o != name);
        if on {
            self.operands.push(name.to_owned());
        }
    }

    /// The helper Apply writes, `None` to clear, or why there is none.
    ///
    /// # Errors
    ///
    /// [`DraftError::NoOperands`] when calculated with nothing to calculate.
    pub fn helper(&self) -> Result<Option<CalcHelper>, DraftError> {
        if !self.on {
            return Ok(None);
        }
        if self.operands.is_empty() {
            return Err(DraftError::NoOperands);
        }
        Ok(Some(CalcHelper::Simple {
            op: self.op,
            operands: self
                .operands
                .iter()
                .map(|o| o.as_bytes().to_vec())
                .collect(),
        }))
    }
}

/// Why a draft is not a helper yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftError {
    /// A date format with no text.
    EmptyDate,
    /// A ticked bound that is not a number; carries what was typed.
    NotANumber(String),
    /// The lower bound is above the upper.
    RangeInverted,
    /// A calculation over no fields.
    NoOperands,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_sample_of_each_format_round_trips_through_its_draft() {
        let helpers = [
            FormatHelper::Number {
                decimals: 3,
                separator_style: 2,
                negative_style: 1,
                currency_style: 0,
                currency: b"$".to_vec(),
                prepend_currency: false,
            },
            FormatHelper::Percent {
                decimals: 1,
                separator_style: 4,
            },
            FormatHelper::Date { index: 5 },
            FormatHelper::DateEx {
                format: b"yyyy-mm-dd".to_vec(),
            },
            FormatHelper::Time { index: 3 },
            FormatHelper::Special { selector: 2 },
        ];
        for h in helpers {
            let draft = FormatDraft::from_helper(Some(&h));
            assert_eq!(draft.helper(), Ok(Some(h)));
        }
        assert_eq!(FormatDraft::from_helper(None).helper(), Ok(None));
    }

    #[test]
    fn an_edited_predefined_date_is_written_as_an_explicit_format() {
        let mut draft = FormatDraft::from_helper(Some(&FormatHelper::Date { index: 1 }));
        draft.date = "dd.mm.yyyy".to_owned();
        assert_eq!(
            draft.helper(),
            Ok(Some(FormatHelper::DateEx {
                format: b"dd.mm.yyyy".to_vec()
            }))
        );
        draft.date = "  ".to_owned();
        assert_eq!(draft.helper(), Err(DraftError::EmptyDate));
    }

    #[test]
    fn a_range_needs_numbers_in_order_and_clears_when_unticked() {
        let h = AdvisoryHelper::RangeValidate {
            lower: Some(1.5),
            upper: None,
        };
        let mut draft = RangeDraft::from_helper(Some(&h));
        assert_eq!(draft.helper(), Ok(Some(h)));
        draft.upper_on = true;
        draft.upper = "x".to_owned();
        assert_eq!(draft.helper(), Err(DraftError::NotANumber("x".to_owned())));
        draft.upper = "1".to_owned();
        assert_eq!(draft.helper(), Err(DraftError::RangeInverted));
        assert_eq!(RangeDraft::default().helper(), Ok(None));
    }

    #[test]
    fn a_calculation_keeps_the_chosen_order_and_needs_an_operand() {
        let mut draft = CalcDraft {
            on: true,
            ..CalcDraft::default()
        };
        assert_eq!(draft.helper(), Err(DraftError::NoOperands));
        draft.toggle("B", true);
        draft.toggle("A", true);
        draft.toggle("B", false);
        draft.toggle("B", true);
        let h = CalcHelper::Simple {
            op: SimpleOp::Sum,
            operands: vec![b"A".to_vec(), b"B".to_vec()],
        };
        assert_eq!(draft.helper(), Ok(Some(h.clone())));
        assert_eq!(CalcDraft::from_helper(Some(&h)), draft);
        draft.on = false;
        assert_eq!(draft.helper(), Ok(None));
    }
}
