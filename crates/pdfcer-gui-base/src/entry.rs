//! # `entry` — one reader for everything the operator types into a value box
//!
//! Every value box routes its text through [`preprocess`], which is told what
//! the box can hold ([`Kind`]) and what it holds now. Contract:
//!
//! - **Text** passes through untouched. No arithmetic, no units.
//! - **Numbers** accept arithmetic (`+ - * / ×  ÷`, parentheses) and refuse
//!   units, except the box's own decoration (`%`, `°`, `dpi`), which is
//!   accepted and ignored.
//! - **Lengths** accept units in any written form (`12mm`, `1.2 m`, `55 5/8"`,
//!   `4'-7 1/2"`, `10px`, `3 pt`) mixed with arithmetic (`12mm + 1/4"`). A bare
//!   number is in the box's own unit.
//! - **Relative entry.** A leading `+`, `*`, `/`, `×` or `÷`, or a `-` followed
//!   by a space, applies to the current value: `+10px` moves ten pixels. A `-`
//!   against a digit (`-10`) is a negative number, as everywhere else.
//! - **Refusal beats guessing.** Anything not read in full is an error that
//!   names the problem. A box never receives a partial reading.
//!
//! `px` is the CSS pixel, 1/96 in, which is what a document program means by it.
//! It is not a screen pixel, because a screen pixel's size in the document
//! changes with zoom.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/entry.md`.

use std::cell::RefCell;
use std::rc::Rc;

use pdfcer_core::dimension::Unit;

use crate::text::entry as t;
use crate::units;

/// What a value box holds, and so what its text may contain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// Free text: returned exactly as typed.
    Text,
    /// A whole number (a count, a page number). Arithmetic, no units.
    Count,
    /// A plain number (a ratio, a percentage, a DPI). Arithmetic, no units;
    /// the listed suffixes are the box's own unit and are accepted and ignored.
    Number(&'static [&'static str]),
    /// A length, held and returned in the given unit.
    Length(LengthUnit),
}

/// The unit a length box holds its value in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthUnit {
    /// PDF points, 1/72 in.
    Point,
    /// One of the engine's measurement units.
    Of(Unit),
}

impl LengthUnit {
    fn to_points(self, v: f64) -> f64 {
        match self {
            LengthUnit::Point => v,
            LengthUnit::Of(u) => units::to_points(v, u),
        }
    }

    fn of_points(self, pt: f64) -> f64 {
        match self {
            LengthUnit::Point => pt,
            LengthUnit::Of(u) => units::from_points(pt, u),
        }
    }

    /// The label a box shows beside its value.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            LengthUnit::Point => "pt",
            LengthUnit::Of(u) => u.abbrev(),
        }
    }
}

/// What [`preprocess`] read.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    /// A text box's text, verbatim.
    Text(String),
    /// A value in the box's own unit.
    Value(f64),
}

/// Why typed text could not be read. Worded by `text::entry::describe`.
#[derive(Clone, Debug, PartialEq)]
pub enum EntryError {
    /// Nothing was typed.
    Empty,
    /// A character the reader does not understand, at this char index.
    Unexpected(usize),
    /// The text ended where a value was still expected.
    Incomplete,
    /// A unit word the reader does not know.
    UnknownUnit(String),
    /// A unit this box cannot hold (a length typed into a count).
    UnitNotAllowed(String),
    /// Arithmetic that mixes a length with a plain number where only one
    /// makes sense (`2mm * 3mm`, `2 / 3mm`).
    Mismatch,
    /// Division by zero, or a result too large to hold.
    NotFinite,
    /// A count box received a fraction.
    NotWhole,
}

/// Read `input` as a value for a box of `kind` currently holding `current`
/// (in the box's own unit; ignored by [`Kind::Text`]).
///
/// # Errors
///
/// See [`EntryError`]. A text box never errors.
pub fn preprocess(input: &str, current: f64, kind: Kind) -> Result<Entry, EntryError> {
    if kind == Kind::Text {
        return Ok(Entry::Text(input.to_owned()));
    }
    let tokens = lex(input, kind)?;
    if tokens.is_empty() {
        return Err(EntryError::Empty);
    }
    let tokens = relative(tokens, current, kind);
    let mut p = Parser {
        t: &tokens,
        i: 0,
        kind,
    };
    let v = p.expr()?;
    if p.i < tokens.len() {
        return Err(EntryError::Unexpected(tokens[p.i].at));
    }
    let out = match (kind, v) {
        (Kind::Length(u), Val::Len(pt)) => u.of_points(pt),
        (_, Val::Bare(n)) => n,
        (_, Val::Len(_)) => return Err(EntryError::Mismatch),
    };
    if !out.is_finite() {
        return Err(EntryError::NotFinite);
    }
    if kind == Kind::Count {
        if (out - out.round()).abs() > 1e-9 {
            return Err(EntryError::NotWhole);
        }
        return Ok(Entry::Value(out.round()));
    }
    Ok(Entry::Value(out))
}

/// [`preprocess`] for a numeric box, shaped for `egui::DragValue::custom_parser`.
#[must_use]
pub fn value(input: &str, current: f64, kind: Kind) -> Option<f64> {
    match preprocess(input, current, kind) {
        Ok(Entry::Value(v)) => Some(v),
        _ => None,
    }
}

/// A `DragValue` whose typed text goes through [`preprocess`], showing the
/// unit of a length box beside its value, and the [`Refusal`] that reports a
/// reading it declined. Configure the `DragValue` (speed, range), add it as
/// the very next widget in `ui`, then hand the response to [`Refusal::show`].
///
/// Relative entry is anchored to the value the box held when editing began.
/// `DragValue` parses on every keystroke and once more as focus leaves, and
/// writes each reading back; reading "the current value" afresh each time would
/// make `+10` add 1, then 10 more, then 10 again on Enter. The anchor lives in
/// `ui`'s temporary data under the id the widget is about to take, which is why
/// it must be added next.
pub fn drag_value<'a, N: egui::emath::Numeric>(
    ui: &egui::Ui,
    value: &'a mut N,
    kind: Kind,
) -> (egui::DragValue<'a>, Refusal) {
    let (dv, refusal) = drag_value_unlabelled(ui, value, kind);
    let dv = match kind {
        Kind::Length(u) => dv.suffix(t::unit_suffix(u.label())),
        _ => dv,
    };
    (dv, refusal)
}

/// [`drag_value`] without the unit drawn beside the value, for a box whose
/// unit is shown by a control next to it, or whose value reads as a word.
/// egui's `DragValue::suffix` appends, so a unit once added cannot be removed.
pub fn drag_value_unlabelled<'a, N: egui::emath::Numeric>(
    ui: &egui::Ui,
    value: &'a mut N,
    kind: Kind,
) -> (egui::DragValue<'a>, Refusal) {
    let id = ui.next_auto_id();
    let key = id.with("entry-anchor");
    // The frame after focus leaves still counts: `DragValue` re-parses its text
    // then.
    let focused = ui.memory(|m| m.has_focus(id));
    let flag = key.with("focused");
    let was_focused = ui.data_mut(|d| {
        let was = d.get_temp::<bool>(flag).unwrap_or(false);
        d.insert_temp(flag, focused);
        was
    });
    let editing = focused || was_focused;
    let current = if editing {
        ui.data(|d| d.get_temp::<f64>(key))
            .unwrap_or(value.to_f64())
    } else {
        let v = value.to_f64();
        ui.data_mut(|d| d.insert_temp(key, v));
        v
    };
    let refusal = Refusal {
        kind,
        id,
        error: Rc::new(RefCell::new(None)),
    };
    let slot = Rc::clone(&refusal.error);
    let dv =
        egui::DragValue::new(value).custom_parser(move |s| match preprocess(s, current, kind) {
            Ok(Entry::Value(v)) => {
                *slot.borrow_mut() = None;
                Some(v)
            }
            Ok(Entry::Text(_)) => None,
            Err(e) => {
                *slot.borrow_mut() = Some(e);
                None
            }
        });
    (dv, refusal)
}

/// What a [`drag_value`] box could not read, reported beside it while the
/// operator is still typing. A declined reading leaves the value unchanged, so
/// the box reverts when it loses focus.
pub struct Refusal {
    kind: Kind,
    id: egui::Id,
    error: Rc<RefCell<Option<EntryError>>>,
}

impl Refusal {
    /// Attach the box's hover help and, while its text is unreadable, the
    /// sentence saying why.
    pub fn show(self, response: egui::Response) -> egui::Response {
        debug_assert_eq!(
            response.id, self.id,
            "a widget was added between `drag_value` and its box, so the relative-entry anchor is keyed to the wrong id"
        );
        if response.has_focus()
            && let Some(e) = self.error.borrow().as_ref()
        {
            response.show_tooltip_text(t::describe(e));
        }
        match self.kind {
            Kind::Length(_) => response.on_hover_text(t::length_help()),
            Kind::Count | Kind::Number(_) => response.on_hover_text(t::number_help()),
            Kind::Text => response,
        }
    }
}

// ---------------------------------------------------------------------------
// Lexing
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tok {
    Num(f64),
    /// A length unit, as points per one of it.
    Unit(f64),
    Op(char),
    Open,
    Close,
    /// Juxtaposition: `4'-7"` reads as 4 ft plus 7 in.
    Join,
}

#[derive(Clone, Copy, Debug)]
struct Token {
    tok: Tok,
    at: usize,
    /// Whitespace separates this token from the next.
    spaced: bool,
}

fn lex(input: &str, kind: Kind) -> Result<Vec<Token>, EntryError> {
    let chars: Vec<char> = input
        .chars()
        .map(|c| match c {
            '\u{2032}' | '\u{2019}' | '\u{2018}' => '\'',
            '\u{2033}' | '\u{201C}' | '\u{201D}' => '"',
            '\u{2212}' => '-',
            c => c,
        })
        .collect();
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let at = i;
        if c.is_whitespace() {
            if let Some(last) = out.last_mut() {
                last.spaced = true;
            }
            i += 1;
            continue;
        }
        let tok = if c.is_ascii_digit() || c == '.' {
            let (n, next) = number(&chars, i)?;
            i = next;
            Tok::Num(n)
        } else if matches!(c, '+' | '-' | '*' | '/' | '×' | '÷') {
            i += 1;
            // `4'-7"`: a hyphen straight after a foot mark joins feet to inches.
            let after_feet = out.last().is_some_and(|t| !t.spaced && is_feet(t.tok));
            if c == '-' && after_feet && chars.get(i).is_some_and(char::is_ascii_digit) {
                Tok::Join
            } else {
                Tok::Op(match c {
                    '×' => '*',
                    '÷' => '/',
                    c => c,
                })
            }
        } else if c == '(' {
            i += 1;
            Tok::Open
        } else if c == ')' {
            i += 1;
            Tok::Close
        } else {
            let start = i;
            if matches!(c, '"' | '\'' | '%' | '°') {
                i += 1;
            } else if c.is_alphabetic() {
                while i < chars.len() && (chars[i].is_alphabetic()) {
                    i += 1;
                }
            } else {
                return Err(EntryError::Unexpected(at));
            }
            let word: String = chars[start..i].iter().collect::<String>().to_lowercase();
            match unit(&word, kind)? {
                Some(per) => Tok::Unit(per),
                None => continue,
            }
        };
        out.push(Token {
            tok,
            at,
            spaced: false,
        });
    }
    Ok(out)
}

fn is_feet(t: Tok) -> bool {
    matches!(t, Tok::Unit(per) if (per - units::to_points(1.0, Unit::DecimalFeet)).abs() < 1e-9)
}

/// A decimal, or a tight `a/b` fraction (`5/8`): the slash of a fraction has
/// no space either side, which is what separates it from division. The two
/// read the same value, except where a unit follows (`1/4"` is a quarter inch,
/// not one divided by four inches).
fn number(chars: &[char], mut i: usize) -> Result<(f64, usize), EntryError> {
    let start = i;
    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
        i += 1;
    }
    let s: String = chars[start..i].iter().collect();
    let n: f64 = s.parse().map_err(|_| EntryError::Unexpected(start))?;
    let whole = !s.contains('.');
    if whole && chars.get(i) == Some(&'/') && chars.get(i + 1).is_some_and(char::is_ascii_digit) {
        let d0 = i + 1;
        let mut j = d0;
        while j < chars.len() && chars[j].is_ascii_digit() {
            j += 1;
        }
        if chars.get(j) != Some(&'.') {
            let d: f64 = chars[d0..j]
                .iter()
                .collect::<String>()
                .parse()
                .map_err(|_| EntryError::Unexpected(d0))?;
            if d == 0.0 {
                return Err(EntryError::NotFinite);
            }
            return Ok((n / d, j));
        }
    }
    Ok((n, i))
}

/// Points per one of the unit named by `word`, `None` for a suffix the box
/// accepts and ignores.
fn unit(word: &str, kind: Kind) -> Result<Option<f64>, EntryError> {
    if let Kind::Number(suffixes) = kind
        && suffixes.contains(&word)
    {
        return Ok(None);
    }
    let engine = match word {
        "pt" | "pts" | "point" | "points" => return length(kind, word, 1.0),
        "px" | "pixel" | "pixels" => {
            return length(kind, word, units::points_from_inches(1.0) / 96.0);
        }
        "\"" | "in" | "inch" | "inches" => Unit::Inch,
        "'" | "ft" | "foot" | "feet" => Unit::DecimalFeet,
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => Unit::Millimeter,
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => Unit::Centimeter,
        "m" | "meter" | "meters" | "metre" | "metres" => Unit::Meter,
        "km" | "kilometers" | "kilometres" => Unit::Kilometer,
        w => match Unit::parse(w) {
            Some(u) => u,
            None => return Err(EntryError::UnknownUnit(word.to_owned())),
        },
    };
    length(kind, word, units::to_points(1.0, engine))
}

fn length(kind: Kind, word: &str, per: f64) -> Result<Option<f64>, EntryError> {
    match kind {
        Kind::Length(_) => Ok(Some(per)),
        _ => Err(EntryError::UnitNotAllowed(word.to_owned())),
    }
}

/// Rewrite a leading operator into an operation on the current value.
fn relative(mut tokens: Vec<Token>, current: f64, kind: Kind) -> Vec<Token> {
    let first = tokens[0];
    let rel = match first.tok {
        Tok::Op('-') => first.spaced,
        Tok::Op(_) => true,
        _ => false,
    };
    if !rel {
        return tokens;
    }
    let mut head = vec![Token {
        tok: Tok::Num(current),
        at: 0,
        spaced: false,
    }];
    if let Kind::Length(u) = kind {
        head.push(Token {
            tok: Tok::Unit(u.to_points(1.0)),
            at: 0,
            spaced: true,
        });
    }
    head.append(&mut tokens);
    head
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Val {
    Bare(f64),
    /// A length in points.
    Len(f64),
}

struct Parser<'t> {
    t: &'t [Token],
    i: usize,
    kind: Kind,
}

impl Parser<'_> {
    fn peek(&self) -> Option<Tok> {
        self.t.get(self.i).map(|t| t.tok)
    }

    /// A bare number in a sum with a length is in the box's own unit.
    fn as_len(&self, v: Val) -> Val {
        match (v, self.kind) {
            (Val::Bare(n), Kind::Length(u)) => Val::Len(u.to_points(n)),
            (v, _) => v,
        }
    }

    fn expr(&mut self) -> Result<Val, EntryError> {
        let mut a = self.term()?;
        while let Some(Tok::Op(op @ ('+' | '-'))) = self.peek() {
            self.i += 1;
            let b = self.term()?;
            let s = if op == '+' { 1.0 } else { -1.0 };
            a = match (a, b) {
                (Val::Bare(x), Val::Bare(y)) => Val::Bare(x + s * y),
                (x, y) => match (self.as_len(x), self.as_len(y)) {
                    (Val::Len(x), Val::Len(y)) => Val::Len(x + s * y),
                    _ => return Err(EntryError::Mismatch),
                },
            };
        }
        Ok(a)
    }

    fn term(&mut self) -> Result<Val, EntryError> {
        let mut a = self.factor()?;
        while let Some(Tok::Op(op @ ('*' | '/'))) = self.peek() {
            self.i += 1;
            let b = self.factor()?;
            a = match (op, a, b) {
                ('*', Val::Bare(x), Val::Bare(y)) => Val::Bare(x * y),
                ('*', Val::Len(x), Val::Bare(y)) | ('*', Val::Bare(y), Val::Len(x)) => {
                    Val::Len(x * y)
                }
                ('/', Val::Bare(x), Val::Bare(y)) => Val::Bare(x / y),
                ('/', Val::Len(x), Val::Bare(y)) => Val::Len(x / y),
                ('/', Val::Len(x), Val::Len(y)) => Val::Bare(x / y),
                _ => return Err(EntryError::Mismatch),
            };
        }
        Ok(a)
    }

    fn factor(&mut self) -> Result<Val, EntryError> {
        match self.peek() {
            Some(Tok::Op('-')) => {
                self.i += 1;
                Ok(match self.factor()? {
                    Val::Bare(x) => Val::Bare(-x),
                    Val::Len(x) => Val::Len(-x),
                })
            }
            Some(Tok::Op('+')) => {
                self.i += 1;
                self.factor()
            }
            Some(Tok::Open) => {
                self.i += 1;
                let v = self.expr()?;
                if self.peek() != Some(Tok::Close) {
                    return Err(self.here());
                }
                self.i += 1;
                Ok(v)
            }
            Some(Tok::Num(_)) => self.quantity(),
            Some(_) => Err(self.here()),
            None => Err(EntryError::Incomplete),
        }
    }

    fn here(&self) -> EntryError {
        self.t
            .get(self.i)
            .map_or(EntryError::Incomplete, |t| EntryError::Unexpected(t.at))
    }

    /// Juxtaposed numbers sum: `55 5/8"`, `4' 7 1/2"`, `1 m 20 cm`. A bare
    /// number takes the unit of the next one that has one; a trailing bare
    /// number after feet is inches, as `4' 7` is written.
    fn quantity(&mut self) -> Result<Val, EntryError> {
        let mut parts: Vec<(f64, Option<f64>)> = Vec::new();
        let second = self.t.get(self.i + 1).map_or(0, |t| t.at);
        while let Some(Tok::Num(n)) = self.peek() {
            self.i += 1;
            let per = if let Some(Tok::Unit(per)) = self.peek() {
                self.i += 1;
                Some(per)
            } else {
                None
            };
            parts.push((n, per));
            match self.peek() {
                Some(Tok::Join) => self.i += 1,
                Some(Tok::Num(_)) => {}
                _ => break,
            }
        }
        if parts.iter().all(|p| p.1.is_none()) {
            // `2 1/2` is a mixed number; `1 000` is not a sum of 1 and 0.
            return match parts.as_slice() {
                [(n, None)] => Ok(Val::Bare(*n)),
                [(w, None), (f, None)] if *f > 0.0 && *f < 1.0 => Ok(Val::Bare(w + f)),
                _ => Err(EntryError::Unexpected(second)),
            };
        }
        let inch = units::points_from_inches(1.0);
        let mut total = 0.0;
        let mut pending = 0.0;
        let mut last_per = None;
        for (n, per) in parts {
            match per {
                Some(per) => {
                    total += (pending + n) * per;
                    pending = 0.0;
                    last_per = Some(per);
                }
                None => pending += n,
            }
        }
        if pending != 0.0 {
            let foot = units::to_points(1.0, Unit::DecimalFeet);
            let per = match last_per {
                Some(p) if (p - foot).abs() < 1e-9 => inch,
                _ => match self.kind {
                    Kind::Length(u) => u.to_points(1.0),
                    _ => return Err(EntryError::Mismatch),
                },
            };
            total += pending * per;
        }
        Ok(Val::Len(total))
    }
}

#[cfg(test)]
mod tests;
