//! # `formdraft` — what the dialog collects, and what it remembers
//!
//! A [`Draft`] is the whole of a form field's settings **before** anything
//! reaches the document. It is what the placement dialog edits, what
//! `Action::CommitFormField` carries, and what [`Remembered`] keeps for the
//! next placement.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/formdraft.md`.

use pdfcer_core::fontdata::Std14;
use pdfcer_core::forms::MkColor;
use pdfcer_core::vartext::TextColor;

use crate::formfieldkind::FormFieldKind;

/// How many characters a field name may run to before the dialog stops
/// accepting more.
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
    /// Two widgets with the same name are **one field**. See the header.
    pub name: String,
    /// The `/TU` — what a screen reader announces.
    ///
    /// Empty is meaningful and is not the same as absent: an empty string
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
    /// The border's `/BS` `/S`: solid, dashed, raised, sunken or underline.
    pub border_style: pdfcer_core::edit::BorderStyle,
    /// `/MK` `/BG` — what is painted behind the box, or `None` to state
    /// nothing and let the kind's own default stand.
    ///
    /// Three states, not two: `None` writes no key at all,
    /// `Some(MkColor::None)` writes Table 189's empty array, which
    /// positively means *paint nothing* — including a push button's
    /// plate. `OPERATOR_REQUESTS.md` **O202**, the before-placement half.
    pub background: Option<MkColor>,
    /// `/MK` `/BC` — the ink the outline and a check box's tick or radio
    /// button's dot are drawn in, or `None` for the engine's black.
    ///
    /// No empty-array state is offered here, for the reason
    /// [`crate::panels::properties::mkcolour::Row::no_colour_entry`]
    /// gives: the engine resolves an empty `/BC` and an absent `/BC` to
    /// the same black.
    pub border_color: Option<MkColor>,
    /// **Text / choice** — the face, size and ink the value is drawn in.
    pub text: TextLook,
    /// **Text** — the value the field starts with.
    pub value: String,
    /// **Text** — whether the box wraps onto more than one line.
    pub multiline: bool,
    /// **Text** — whether characters are masked as they are typed.
    pub password: bool,
    /// **Text** — whether characters sit in equal cells.
    ///
    /// Comb needs a maximum length: the cells are `max_len` divisions of the
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
    /// **Check box** — the mark drawn when it is on.
    pub check_style: pdfcer_core::annot_author::CheckStyle,
    /// **Radio button** — the mark drawn when it is chosen. `Circle`, the
    /// default, is the dot and writes no `/MK /CA`.
    pub radio_style: pdfcer_core::annot_author::CheckStyle,
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
    ///
    /// > *"The placement dialog is the only surface for this, and the reason is
    /// > a gap rather than a design: `pdfcer-core` can WRITE a button's action
    /// > and cannot READ one back, so a control over an existing button could
    /// > not say what it currently is."*
    ///
    ///
    /// What is still true, and is the reason this field exists at all: a
    /// button *being placed* has no action yet, so this is the one place the
    /// question is asked before there is anything to read.
    pub action: crate::pushbutton::ButtonDoes,
}

impl Draft {
    /// The options, one per line, with blanks discarded and ends trimmed.
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
    #[must_use]
    pub fn comb_ok(&self) -> bool {
        matches!(self.kind, FormFieldKind::Text) && self.max_len.is_some_and(|n| n > 0)
    }

    /// Whether this draft can be authored at all.
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
            border_style: pdfcer_core::edit::BorderStyle::Solid,
            // No background, so a push button keeps its plate. A text or
            // choice field gets a black outline: with no `/BC` it draws no
            // box, and an empty field is then invisible in a reader that
            // does not highlight fields (O262).
            background: None,
            border_color: matches!(kind, FormFieldKind::Text | FormFieldKind::Choice)
                .then_some(MkColor::Gray(0.0)),
            text: TextLook::ENGINE,
            value: String::new(),
            multiline: false,
            password: false,
            comb: false,
            max_len: None,
            // "Yes" rather than "On": `/AS` may be any name, and the one a
            // check box carries is what a form-filling script reads back. "Yes"
            // is what Acrobat writes and is therefore what most scripts expect.
            export_value: "Yes".to_owned(), // ui-text-exempt: a PDF /AS name written into the file, never displayed as UI copy
            checked: false,
            check_style: pdfcer_core::annot_author::CheckStyle::Check,
            radio_style: pdfcer_core::annot_author::CheckStyle::Circle,
            options: String::new(),
            combo: true,
            editable: false,
            multi_select: false,
            sort: false,
            caption: String::new(),
            // Nothing, which is exactly what `add_push_button` authors on its
            // own. A dialog opening with an action pre-chosen would be pdfcer
            // deciding what somebody's button does.
            action: crate::pushbutton::ButtonDoes::default(),
        }
    }
}

/// **How the next field looks**: the three settings every kind shares and the
/// Forms ▸ Style band edits. Applying one over a draft is the band's whole
/// effect, so the dialog opens on what the band shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldLook {
    /// [`Draft::border_width`].
    pub border_width: f64,
    /// [`Draft::border_style`].
    pub border_style: pdfcer_core::edit::BorderStyle,
    /// [`Draft::background`].
    pub background: Option<MkColor>,
    /// [`Draft::border_color`].
    pub border_color: Option<MkColor>,
    /// [`Draft::text`].
    pub text: TextLook,
}

impl FieldLook {
    /// The look `draft` carries.
    #[must_use]
    pub const fn of(draft: &Draft) -> Self {
        Self {
            border_width: draft.border_width,
            border_style: draft.border_style,
            background: draft.background,
            border_color: draft.border_color,
            text: draft.text,
        }
    }

    /// Write this look into `draft`.
    pub const fn apply(&self, draft: &mut Draft) {
        draft.border_width = self.border_width;
        draft.border_style = self.border_style;
        draft.background = self.background;
        draft.border_color = self.border_color;
        draft.text = self.text;
    }
}

/// **The text a text or choice field's value is drawn in** — its `/DA`
/// (§12.7.3.3). Only the fourteen built-in faces: a new field has no `/DR`
/// font of its own to name.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextLook {
    /// The face.
    pub font: Std14,
    /// Points; `0.0` is Table 224's auto-size.
    pub size: f64,
    /// The ink the glyphs are filled with.
    pub colour: TextColor,
}

impl TextLook {
    /// What the engine writes for a new field on its own: black Helvetica,
    /// auto-sized. A draft carrying this needs no second verb at placement.
    pub const ENGINE: Self = Self {
        font: Std14::Helvetica,
        size: 0.0,
        colour: TextColor::Gray(0.0),
    };

    /// The look a field's `/DA` states, or `None` when it names a face that is
    /// not one of the fourteen or does not parse. A `/DA` that sets no colour
    /// draws in black (§12.7.3.3).
    #[must_use]
    pub fn from_da(da: &[u8]) -> Option<Self> {
        let parsed = pdfcer_core::vartext::parse_default_appearance(da).ok()?;
        Some(Self {
            font: builtin_face(&parsed.font_name)?,
            size: parsed.font_size,
            colour: parsed.color.unwrap_or(TextColor::Gray(0.0)),
        })
    }
}

/// The built-in face a `/DA` resource key names: a `BaseFont` spelling, or one
/// of the short keys Acrobat and pdfcer write into `/DR` (`Helv`, `TiRo`, …).
#[must_use]
pub fn builtin_face(key: &[u8]) -> Option<Std14> {
    let name = std::str::from_utf8(key).ok()?;
    if let Some(font) = pdfcer_core::fontdata::std14_by_base_font(name) {
        return Some(font);
    }
    // ui-text-exempt: /DA resource keys read out of a file, never displayed
    match name {
        "Helv" => Some(Std14::Helvetica),
        "HeBo" => Some(Std14::HelveticaBold),
        "TiRo" => Some(Std14::TimesRoman),
        "Cour" => Some(Std14::Courier),
        "Symb" => Some(Std14::Symbol),
        "ZaDb" => Some(Std14::ZapfDingbats),
        _ => None,
    }
}

/// The last settings the operator accepted, per session.
#[derive(Debug, Clone, Default)]
pub struct Remembered {
    /// The last accepted draft, whatever kind it was.
    last: Option<Draft>,
    /// The look the Style band or the last accepted draft set; `None` until
    /// one of them does, which leaves each kind its own defaults.
    look: Option<FieldLook>,
}

impl Remembered {
    /// Start with `look` remembered, as the preferences file kept it.
    #[must_use]
    pub const fn with_look(look: Option<FieldLook>) -> Self {
        Self { last: None, look }
    }

    /// The look that was set, if one was; [`Self::look`] fills the gap.
    #[must_use]
    pub const fn chosen(&self) -> Option<FieldLook> {
        self.look
    }

    /// The draft to open the dialog with, for a field of `kind` about to be
    /// placed on a document that already has `existing` field names.
    #[must_use]
    pub fn next(&self, kind: FormFieldKind, existing: &[String]) -> Draft {
        self.next_in(kind, existing, |_| Vec::new())
    }

    /// [`Self::next`], with `group_exports` naming the export values a radio
    /// group already uses in the document, so a radio joining the remembered
    /// group takes one no member carries rather than one the last draft did
    /// not.
    #[must_use]
    pub fn next_in(
        &self,
        kind: FormFieldKind,
        existing: &[String],
        group_exports: impl Fn(&str) -> Vec<String>,
    ) -> Draft {
        let mut draft = match &self.last {
            // Same kind: everything carries, name aside.
            Some(prev) if prev.kind == kind => prev.clone(),
            // Different kind: only the settings that mean the same thing in
            // both. Written as an explicit copy of the shared fields rather
            // than a clone-then-reset, so adding a kind-specific field to `Draft`
            // does not silently start leaking across kinds.
            Some(prev) => Draft {
                required: prev.required,
                read_only: prev.read_only,
                border_width: prev.border_width,
                border_style: prev.border_style,
                background: prev.background,
                border_color: prev.border_color,
                text: prev.text,
                ..Draft::fresh(kind)
            },
            None => Draft::fresh(kind),
        };
        draft.kind = kind;
        if let Some(look) = &self.look {
            look.apply(&mut draft);
        }

        if matches!(kind, FormFieldKind::Radio) && !draft.name.trim().is_empty() {
            // The group persists and the EXPORT VALUE advances — the
            // inverse of every other kind. Three radios in one group are three
            // widgets sharing `/T` with distinct export names.
            let mut taken = collected_exports(&self.last);
            taken.extend(group_exports(draft.name.trim()));
            draft.export_value = next_free(&draft.export_value, &taken);
            draft.checked = false;
        } else {
            draft.name = next_free(kind.name_prefix(), existing);
        }
        draft
    }

    /// Record what the operator accepted.
    pub fn remember(&mut self, draft: &Draft) {
        self.last = Some(draft.clone());
        self.look = Some(FieldLook::of(draft));
    }

    /// The look the next field will be placed with: what was set, or a text
    /// field's defaults, which are what the band shows before anything is.
    #[must_use]
    pub fn look(&self) -> FieldLook {
        self.look
            .unwrap_or_else(|| FieldLook::of(&Draft::fresh(FormFieldKind::Text)))
    }

    /// Set the look every following placement opens with, whatever its kind.
    pub const fn set_look(&mut self, look: FieldLook) {
        self.look = Some(look);
    }
}

/// The export values already used by the remembered draft, if any.
fn collected_exports(last: &Option<Draft>) -> Vec<String> {
    last.iter().map(|d| d.export_value.clone()).collect()
}

/// `prefix` with the lowest positive integer suffix that is not in `taken`.
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

    /// **The band's look reaches every kind**, including over a remembered
    /// draft of the same kind, and survives the next acceptance only as what
    /// that acceptance carried.
    #[test]
    fn a_set_look_opens_every_following_kind() {
        let mut mem = Remembered::default();
        let first = mem.next(FormFieldKind::CheckBox, &[]);
        mem.remember(&first);
        let look = FieldLook {
            border_width: 3.0,
            border_style: pdfcer_core::edit::BorderStyle::Dashed,
            background: Some(MkColor::Gray(0.5)),
            border_color: Some(MkColor::Rgb(1.0, 0.0, 0.0)),
            text: TextLook {
                font: Std14::Courier,
                size: 14.0,
                colour: TextColor::Rgb(0.0, 0.0, 1.0),
            },
        };
        mem.set_look(look);
        for kind in [
            FormFieldKind::Text,
            FormFieldKind::CheckBox,
            FormFieldKind::Radio,
            FormFieldKind::Choice,
            FormFieldKind::PushButton,
        ] {
            assert_eq!(FieldLook::of(&mem.next(kind, &[])), look, "{kind:?}");
        }
        let mut accepted = mem.next(FormFieldKind::Text, &[]);
        accepted.border_width = 0.5;
        mem.remember(&accepted);
        assert!((mem.look().border_width - 0.5).abs() < f64::EPSILON);
    }

    /// **Before anything is set, each kind keeps its own defaults** — a check
    /// box is not given a text field's outline by a band nobody touched.
    #[test]
    fn an_untouched_look_changes_nothing() {
        let mem = Remembered::default();
        assert_eq!(
            mem.next(FormFieldKind::CheckBox, &[]).border_color,
            Draft::fresh(FormFieldKind::CheckBox).border_color
        );
        assert_eq!(
            mem.look(),
            FieldLook::of(&Draft::fresh(FormFieldKind::Text))
        );
    }

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

    /// **A second radio DOES inherit the name** — the deliberate inverse,
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

    /// **A third radio takes an export no member uses**, not merely one the
    /// last draft did not: the engine refuses a member repeating an export.
    #[test]
    fn a_third_radio_skips_every_export_the_group_uses() {
        let mut mem = Remembered::default();
        let mut second = mem.next(FormFieldKind::Radio, &[]);
        second.export_value = "Choice2".to_owned();
        mem.remember(&second);
        let group = second.name.clone();
        let third = mem.next_in(FormFieldKind::Radio, std::slice::from_ref(&group), |g| {
            if g == group {
                vec!["Choice1".to_owned(), "Choice2".to_owned()]
            } else {
                Vec::new()
            }
        });
        assert_eq!(third.export_value, "Choice3");
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

    /// **Both `/MK` colours carry across kinds. Only a box you type into or
    /// pick from starts with an outline; no kind starts with a background.**
    #[test]
    fn both_mk_colours_carry_across_kinds_and_only_boxes_start_outlined() {
        for kind in FormFieldKind::ALL {
            let fresh = Draft::fresh(kind);
            assert_eq!(fresh.background, None, "{kind:?} invents a background");
            let outlined = matches!(kind, FormFieldKind::Text | FormFieldKind::Choice);
            assert_eq!(
                fresh.border_color,
                outlined.then_some(MkColor::Gray(0.0)),
                "{kind:?} border colour"
            );
        }

        let mut mem = Remembered::default();
        let mut text = Draft::fresh(FormFieldKind::Text);
        text.background = Some(MkColor::Rgb(1.0, 0.0, 0.0));
        text.border_color = Some(MkColor::None);
        mem.remember(&text);

        let check = mem.next(FormFieldKind::CheckBox, &[]);
        assert_eq!(check.background, text.background);
        assert_eq!(check.border_color, text.border_color);
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
