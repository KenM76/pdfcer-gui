//! # `canvas::formfield::draft` — what the dialog collects, and what it remembers
//!
//! A [`Draft`] is the whole of a form field's settings **before** anything
//! reaches the document. It is what the placement dialog edits, what
//! `Action::CommitFormField` carries, and what [`Remembered`] keeps for the
//! next placement.
//!
//! ## ★★★ One struct for five kinds, and why that is not laziness
//!
//! `pdfcer-core` has five distinct spec types — `NewTextField`, `NewCheckBox`,
//! `NewRadioButton`, `NewChoiceField`, `NewPushButton` — and this is one type
//! with a `kind`. That inverts the usual advice, so it needs its reason.
//!
//! The five specs share nine of their fields (page, name, rect, tooltip,
//! read-only, required, border, visibility) and differ in one to five. Modelled
//! as five GUI structs, the **dialog** would be five dialogs, the **remembered
//! settings** five stores, and the shared half — which is the half an operator
//! actually adjusts — would be written five times. Worse, this is the type the
//! operator's *"remember last settings"* attaches to, and remembering across
//! kinds is the useful behaviour: someone who turns the border off for a text
//! field wants it off for the check box they place next.
//!
//! The conversion to the five engine specs happens in exactly one place
//! ([`crate::app::actions::forms`]), where the unused fields are simply not
//! read. That is the correct location for the narrowing: at the boundary, once.
//!
//! ## ★★ What is remembered and what is not, and the hazard in between
//!
//! The operator, 2026-08-26: *"remember last settings"*. Everything here is
//! remembered **except the name**, and the exception is a correctness one
//! rather than a taste one.
//!
//! In PDF, **two widgets that share a fully-qualified name are one field**.
//! `FieldAuthorOutcome::merged` is the engine reporting exactly that. So
//! remembering the name would mean the second text field an operator places
//! silently becomes a second *view* of the first — type in one and the other
//! changes — and nothing on the page would say so.
//!
//! ★ **Radio buttons are the deliberate inverse**, and are the reason this is a
//! per-kind rule rather than a blanket one. Radios that share a name are one
//! control, which is what makes them exclusive; a group of three is three
//! widgets, one name, three export values. So for [`FormFieldKind::Radio`] the
//! name **is** remembered and the *export value* is what advances. Getting this
//! backwards in either direction produces a form that looks right and behaves
//! wrongly, which is why [`Remembered::next`] states it in code and the tests
//! assert both halves.

use super::FormFieldKind;

/// How many characters a field name may run to before the dialog stops
/// accepting more.
///
/// Not a PDF limit — the format allows a long name — but a name is a `/T`
/// string that appears in the Forms panel, in tab order and in a screen
/// reader's announcement, and one that runs past this stops being readable in
/// all three. The number is generous enough that no reasonable name reaches it.
pub const NAME_MAX: usize = 120;

/// The settings a form field is authored with.
///
/// Every field is present for every kind. See the header for why one struct
/// serves five engine specs and where the narrowing happens.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// Which kind of control this is.
    pub kind: FormFieldKind,
    /// The field's `/T` — its name, and its identity.
    ///
    /// ★ Two widgets with the same name are **one field**. See the header.
    pub name: String,
    /// The `/TU` — what a screen reader announces.
    ///
    /// ★★ Empty is meaningful and is not the same as absent: an empty string
    /// here becomes `TooltipChoice::Declined`, which is the operator saying
    /// *"this control needs no name"* rather than the engine's default
    /// `Undecided`, which it refuses to author. That refusal is the entire
    /// "blocker" this feature was parked behind for nine days.
    pub tooltip: String,
    /// Whether the form may not be submitted without a value here.
    pub required: bool,
    /// Whether the operator may change the value.
    pub read_only: bool,
    /// The border width in points; `0.0` for no border.
    pub border_width: f64,
    /// **Text** — the value the field starts with.
    pub value: String,
    /// **Text** — whether the box wraps onto more than one line.
    pub multiline: bool,
    /// **Text** — whether characters are masked as they are typed.
    pub password: bool,
    /// **Text** — whether characters sit in equal cells.
    ///
    /// ★ Comb needs a maximum length: the cells are `max_len` divisions of the
    /// width, so without one there is nothing to divide by. [`Draft::comb_ok`]
    /// is the predicate, and the dialog uses it rather than restating the rule.
    pub comb: bool,
    /// **Text** — the maximum number of characters, if any.
    pub max_len: Option<i64>,
    /// **Check box / radio** — the `/AS` name for the on state, or the radio's
    /// export value.
    pub export_value: String,
    /// **Check box / radio** — whether it starts on.
    pub checked: bool,
    /// **Choice** — the options, one per line.
    ///
    /// Held as one string rather than a `Vec<String>` because that is what the
    /// operator edits: a multi-line box. Split at the boundary by
    /// [`Draft::options`], so a blank line cannot become an empty option.
    pub options: String,
    /// **Choice** — a drop-down (`true`) rather than a list box (`false`).
    pub combo: bool,
    /// **Choice** — whether the operator may type an option that is not listed.
    pub editable: bool,
    /// **Choice** — whether more than one option may be chosen.
    pub multi_select: bool,
    /// **Choice** — whether the viewer sorts the options.
    pub sort: bool,
    /// **Push button** — the words on the button.
    pub caption: String,
    /// **Push button** — what pressing it does.
    ///
    /// The placement dialog is the only surface for this, and the reason is a
    /// gap rather than a design: `pdfcer-core` can WRITE a button's action and
    /// cannot READ one back, so a control over an *existing* button could not
    /// say what it currently is. A button being placed has no action yet, which
    /// makes this the one place the question has a known answer. See
    /// `canvas::formfield::action`'s header and the tripwire test in it.
    pub action: super::action::ButtonDoes,
}

impl Draft {
    /// The options, one per line, with blanks discarded and ends trimmed.
    ///
    /// ★ A trailing newline is what a text box has after the last line the
    /// operator typed, so discarding empties is not tidying — without it every
    /// choice field would carry a final option that is the empty string, which
    /// renders as a selectable blank row.
    #[must_use]
    pub fn options(&self) -> Vec<String> {
        self.options
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    /// Whether **comb** may be switched on: a text field with a maximum length.
    ///
    /// The rule is the format's, not a preference — comb divides the width into
    /// `max_len` cells. Offered as a predicate so the dialog and the commit
    /// cannot disagree about it.
    #[must_use]
    pub fn comb_ok(&self) -> bool {
        matches!(self.kind, FormFieldKind::Text) && self.max_len.is_some_and(|n| n > 0)
    }

    /// Whether this draft can be authored at all.
    ///
    /// A name is the only genuinely required thing: it is the field's identity,
    /// and the engine has nothing to key a field on without one. Everything
    /// else has a defensible default, including the tooltip — empty means
    /// *declined*, which is a decision the engine accepts.
    #[must_use]
    pub fn is_authorable(&self) -> bool {
        !self.name.trim().is_empty()
    }

    /// A fresh draft of `kind` with this project's defaults.
    ///
    /// Used only when nothing has been remembered yet — the first placement in
    /// a session. Afterwards [`Remembered::next`] supplies the draft.
    #[must_use]
    pub fn fresh(kind: FormFieldKind) -> Self {
        Self {
            kind,
            name: String::new(),
            tooltip: String::new(),
            required: false,
            read_only: false,
            border_width: 1.0,
            value: String::new(),
            multiline: false,
            password: false,
            comb: false,
            max_len: None,
            // ★ "Yes" rather than "On": `/AS` may be any name, and the one a
            // check box carries is what a form-filling script reads back. "Yes"
            // is what Acrobat writes and is therefore what most scripts expect.
            export_value: "Yes".to_owned(), // ui-text-exempt: a PDF /AS name written into the file, never displayed as UI copy
            checked: false,
            options: String::new(),
            combo: true,
            editable: false,
            multi_select: false,
            sort: false,
            caption: String::new(),
            // Nothing, which is exactly what `add_push_button` authors on its
            // own. A dialog opening with an action pre-chosen would be pdfcer
            // deciding what somebody's button does.
            action: super::action::ButtonDoes::default(),
        }
    }
}

/// The last settings the operator accepted, per session.
///
/// ★ Per **session**, deliberately not persisted to `userdata`. A remembered
/// setting is a convenience within one sitting — "I am placing a row of
/// identical check boxes" — and one that survived a restart would silently
/// govern a different document days later, which is the shape of a setting
/// nobody can find the source of.
#[derive(Debug, Clone, Default)]
pub struct Remembered {
    /// The last accepted draft, whatever kind it was.
    last: Option<Draft>,
}

impl Remembered {
    /// The draft to open the dialog with, for a field of `kind` about to be
    /// placed on a document that already has `existing` field names.
    ///
    /// ## The three rules, in the order they apply
    ///
    /// 1. **The shared settings carry over** from whatever was placed last,
    ///    even across kinds — border, required, read-only. That is the
    ///    operator's *"remember last settings"*.
    /// 2. **The kind-specific settings carry over only within a kind.** A
    ///    check box does not inherit a text field's multiline flag, because it
    ///    has none; but the *next* check box inherits the previous one's export
    ///    value, which is exactly what placing a column of them wants.
    /// 3. **★★ The name never carries over — except for a radio.** See the
    ///    header: sharing a name merges two widgets into one field, and for
    ///    radio buttons that merging *is* the group.
    #[must_use]
    pub fn next(&self, kind: FormFieldKind, existing: &[String]) -> Draft {
        let mut draft = match &self.last {
            // Same kind: everything carries, name aside.
            Some(prev) if prev.kind == kind => prev.clone(),
            // Different kind: only the settings that mean the same thing in
            // both. Written as an explicit copy of three fields rather than a
            // clone-then-reset, so adding a kind-specific field to `Draft`
            // does not silently start leaking across kinds.
            Some(prev) => Draft {
                required: prev.required,
                read_only: prev.read_only,
                border_width: prev.border_width,
                ..Draft::fresh(kind)
            },
            None => Draft::fresh(kind),
        };
        draft.kind = kind;

        if matches!(kind, FormFieldKind::Radio) && !draft.name.trim().is_empty() {
            // ★★ The group persists and the EXPORT VALUE advances — the
            // inverse of every other kind. Three radios in one group are three
            // widgets sharing `/T` with distinct export names.
            draft.export_value = next_free(&draft.export_value, &collected_exports(&self.last));
            draft.checked = false;
        } else {
            draft.name = next_free(kind.name_prefix(), existing);
        }
        draft
    }

    /// Record what the operator accepted.
    pub fn remember(&mut self, draft: &Draft) {
        self.last = Some(draft.clone());
    }
}

/// The export values already used by the remembered draft, if any.
///
/// A deliberately thin view: the shell does not track a radio group's full
/// membership, so the best it can do is advance past the one it last wrote.
/// That is enough for the sequential placing this exists to serve, and the
/// dialog shows the value so an operator placing out of order can correct it.
fn collected_exports(last: &Option<Draft>) -> Vec<String> {
    last.iter().map(|d| d.export_value.clone()).collect()
}

/// `prefix` with the lowest positive integer suffix that is not in `taken`.
///
/// ★ It starts at 1 and scans upward rather than counting `taken`, because
/// counting gives a collision the moment anything has been deleted: a document
/// with `Text1` and `Text3` has two fields, and `Text2` is free while `Text2`
/// derived from the count would collide with nothing and `Text3` would.
///
/// The scan is bounded by `taken.len() + 1` iterations by construction — with
/// *n* names taken, one of the first *n + 1* candidates must be free.
fn next_free(prefix: &str, taken: &[String]) -> String {
    let stem = prefix.trim_end_matches(|c: char| c.is_ascii_digit());
    let stem = if stem.is_empty() { prefix } else { stem };
    for n in 1..=taken.len().saturating_add(1) {
        let candidate = format!("{stem}{n}");
        if !taken.iter().any(|t| t == &candidate) {
            return candidate;
        }
    }
    // Unreachable by the pigeonhole argument above; an honest fallback rather
    // than a panic, because a name that is merely ugly still authors a field
    // and a panic loses the operator's placement.
    format!("{stem}{}", taken.len().saturating_add(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A second field of the same kind gets a different name**, which is the
    /// guard against the silent merge described in the header.
    #[test]
    fn a_second_field_does_not_inherit_the_first_name() {
        let mut mem = Remembered::default();
        let first = mem.next(FormFieldKind::Text, &[]);
        mem.remember(&first);
        let second = mem.next(FormFieldKind::Text, std::slice::from_ref(&first.name));
        assert_ne!(
            first.name, second.name,
            "two text fields sharing a name would be ONE field with two widgets"
        );
    }

    /// ★★ **A second radio DOES inherit the name** — the deliberate inverse,
    /// because a shared name is what makes radios exclusive.
    #[test]
    fn a_second_radio_joins_the_group() {
        let mut mem = Remembered::default();
        let mut first = mem.next(FormFieldKind::Radio, &[]);
        first.export_value = "Option1".to_owned();
        mem.remember(&first);
        let second = mem.next(FormFieldKind::Radio, std::slice::from_ref(&first.name));
        assert_eq!(
            second.name, first.name,
            "radios in a group share their name"
        );
        assert_ne!(
            second.export_value, first.export_value,
            "…and are told apart by their export value"
        );
    }

    /// **The shared settings carry across kinds**, which is the operator's ask.
    #[test]
    fn border_and_flags_carry_across_kinds() {
        let mut mem = Remembered::default();
        let mut text = Draft::fresh(FormFieldKind::Text);
        text.border_width = 0.0;
        text.required = true;
        mem.remember(&text);
        let check = mem.next(FormFieldKind::CheckBox, &[]);
        assert!((check.border_width - 0.0).abs() < f64::EPSILON);
        assert!(check.required);
    }

    /// **…and the kind-specific ones do not.** A check box has no multiline.
    #[test]
    fn kind_specific_settings_do_not_leak() {
        let mut mem = Remembered::default();
        let mut text = Draft::fresh(FormFieldKind::Text);
        text.multiline = true;
        text.password = true;
        mem.remember(&text);
        let check = mem.next(FormFieldKind::CheckBox, &[]);
        assert!(!check.multiline && !check.password);
    }

    /// **A gap in the existing names is filled**, not skipped past.
    #[test]
    fn naming_fills_a_gap_rather_than_counting() {
        let taken = vec!["Text1".to_owned(), "Text3".to_owned()];
        assert_eq!(next_free("Text", &taken), "Text2");
    }

    /// **Blank option lines are discarded**, so a trailing newline does not
    /// become a selectable empty row.
    #[test]
    fn blank_option_lines_are_discarded() {
        let mut d = Draft::fresh(FormFieldKind::Choice);
        d.options = "  Red \n\n Green\n".to_owned();
        assert_eq!(d.options(), vec!["Red".to_owned(), "Green".to_owned()]);
    }

    /// **Comb needs a maximum length**, because the cells are divisions of it.
    #[test]
    fn comb_needs_a_maximum_length() {
        let mut d = Draft::fresh(FormFieldKind::Text);
        assert!(!d.comb_ok(), "no max length, so nothing to divide by");
        d.max_len = Some(8);
        assert!(d.comb_ok());
        d.kind = FormFieldKind::CheckBox;
        assert!(!d.comb_ok(), "comb is a text-field property alone");
    }

    /// **A nameless draft cannot be authored**, and everything else can.
    #[test]
    fn a_name_is_the_only_hard_requirement() {
        for kind in FormFieldKind::ALL {
            let mut d = Draft::fresh(kind);
            assert!(!d.is_authorable(), "{kind:?} with no name");
            d.name = "  ".to_owned();
            assert!(!d.is_authorable(), "{kind:?} with a blank name");
            d.name = "Field".to_owned();
            assert!(d.is_authorable(), "{kind:?} with a name and nothing else");
        }
    }
}
