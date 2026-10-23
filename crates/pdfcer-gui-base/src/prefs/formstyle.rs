//! # `prefs::formstyle` — Forms ▸ Style, kept across sessions
//!
//! Keys `form_style_*`: the look the next placed form field opens with. Every
//! key absent means *not set*, and the placement dialog then opens each kind on
//! its own defaults; any key present sets the look, the absent ones taking a
//! text field's defaults.
//!
//! Colour values: `none` (a stated no-colour), `gray <v>`, `rgb <r> <g> <b>`,
//! `cmyk <c> <m> <y> <k>`, components 0–1. A fill or outline key that is absent
//! leaves that colour unstated. Fonts are spelled as `Std14`'s variants
//! (`TimesRoman`); a text size of 0 is the engine's auto size.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/formstyle.md`.

use std::fmt::Write as _;

use pdfcer_core::fontdata::Std14;
use pdfcer_core::forms::MkColor;
use pdfcer_core::vartext::TextColor;

use super::printing::KeyOutcome;
use crate::formdraft::{Draft, FieldLook};
use crate::formfieldkind::FormFieldKind;
use crate::formlookband::STYLES;

const WIDTH: &str = "form_style_border_width";
const STYLE: &str = "form_style_border_style";
const FILL: &str = "form_style_fill";
const OUTLINE: &str = "form_style_outline";
const FONT: &str = "form_style_text_font";
const SIZE: &str = "form_style_text_size";
const INK: &str = "form_style_text_colour";

/// The remembered Style look; `None` until the operator sets one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FormStylePrefs {
    /// What the band last showed.
    pub look: Option<FieldLook>,
}

/// The look a first key edits: a text field's defaults, as the band shows them.
fn base() -> FieldLook {
    FieldLook::of(&Draft::fresh(FormFieldKind::Text))
}

/// Claim `key = value` if it is this group's.
pub(super) fn parse_key(prefs: &mut FormStylePrefs, key: &str, value: &str) -> KeyOutcome {
    if !key.starts_with("form_style_") {
        return KeyOutcome::NotMine;
    }
    let value = value.trim();
    let mut look = prefs.look.unwrap_or_else(base);
    let ok = match key {
        WIDTH => number(value).map(|w| look.border_width = w).is_some(),
        STYLE => STYLES
            .into_iter()
            .find(|s| s.token() == value)
            .map(|s| look.border_style = s)
            .is_some(),
        FILL => mk(value).map(|c| look.background = Some(c)).is_some(),
        OUTLINE => mk(value).map(|c| look.border_color = Some(c)).is_some(),
        FONT => Std14::ALL
            .into_iter()
            .find(|f| format!("{f:?}") == value)
            .map(|f| look.text.font = f)
            .is_some(),
        SIZE => number(value).map(|s| look.text.size = s).is_some(),
        INK => ink(value).map(|c| look.text.colour = c).is_some(),
        _ => return KeyOutcome::NotMine,
    };
    if !ok {
        return KeyOutcome::BadValue;
    }
    prefs.look = Some(look);
    KeyOutcome::Accepted
}

/// A finite, non-negative number.
fn number(value: &str) -> Option<f64> {
    value
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
}

/// `gray v` / `rgb r g b` / `cmyk c m y k`, every component in 0–1.
fn components(value: &str) -> Option<(&str, Vec<f64>)> {
    let mut words = value.split_whitespace();
    let space = words.next()?;
    let parts: Option<Vec<f64>> = words
        .map(|w| w.parse::<f64>().ok().filter(|v| (0.0..=1.0).contains(v)))
        .collect();
    Some((space, parts?))
}

/// A widget colour value, including the stated no-colour.
fn mk(value: &str) -> Option<MkColor> {
    if value == "none" {
        return Some(MkColor::None);
    }
    match components(value)? {
        ("gray", v) if v.len() == 1 => Some(MkColor::Gray(v[0] as f32)),
        ("rgb", v) if v.len() == 3 => Some(MkColor::Rgb(v[0] as f32, v[1] as f32, v[2] as f32)),
        ("cmyk", v) if v.len() == 4 => Some(MkColor::Cmyk(
            v[0] as f32,
            v[1] as f32,
            v[2] as f32,
            v[3] as f32,
        )),
        _ => None,
    }
}

/// A text colour value.
fn ink(value: &str) -> Option<TextColor> {
    match components(value)? {
        ("gray", v) if v.len() == 1 => Some(TextColor::Gray(v[0])),
        ("rgb", v) if v.len() == 3 => Some(TextColor::Rgb(v[0], v[1], v[2])),
        ("cmyk", v) if v.len() == 4 => Some(TextColor::Cmyk(v[0], v[1], v[2], v[3])),
        _ => None,
    }
}

fn mk_text(colour: MkColor) -> String {
    match colour {
        MkColor::None => "none".to_owned(),
        MkColor::Gray(g) => format!("gray {g}"), // ui-text-exempt: a preferences-file value, never displayed
        MkColor::Rgb(r, g, b) => format!("rgb {r} {g} {b}"), // ui-text-exempt: a preferences-file value, never displayed
        MkColor::Cmyk(c, m, y, k) => format!("cmyk {c} {m} {y} {k}"), // ui-text-exempt: a preferences-file value, never displayed
    }
}

fn ink_text(colour: TextColor) -> String {
    match colour {
        TextColor::Gray(g) => format!("gray {g}"), // ui-text-exempt: a preferences-file value, never displayed
        TextColor::Rgb(r, g, b) => format!("rgb {r} {g} {b}"), // ui-text-exempt: a preferences-file value, never displayed
        TextColor::Cmyk(c, m, y, k) => format!("cmyk {c} {m} {y} {k}"), // ui-text-exempt: a preferences-file value, never displayed
    }
}

/// Append this group's block to a preferences file.
pub(super) fn write_block(prefs: &FormStylePrefs, out: &mut String) {
    out.push_str(
        "\n\
         # form_style_*: Forms > Style, the look the next placed form field\n\
         # opens with. Absent means not set. Colours: none, gray V, rgb R G B or\n\
         # cmyk C M Y K (0 to 1). Fonts: Helvetica, TimesRoman, Courier and the\n\
         # other standard faces as written here. A text size of 0 is automatic.\n",
    );
    let Some(look) = prefs.look else { return };
    let _ = writeln!(out, "{WIDTH} = {}", look.border_width); // ui-text-exempt: a preferences-file value, never displayed
    let _ = writeln!(out, "{STYLE} = {}", look.border_style.token()); // ui-text-exempt: a preferences-file value, never displayed
    if let Some(c) = look.background {
        let _ = writeln!(out, "{FILL} = {}", mk_text(c)); // ui-text-exempt: a preferences-file value, never displayed
    }
    if let Some(c) = look.border_color {
        let _ = writeln!(out, "{OUTLINE} = {}", mk_text(c)); // ui-text-exempt: a preferences-file value, never displayed
    }
    let _ = writeln!(out, "{FONT} = {:?}", look.text.font); // ui-text-exempt: a preferences-file value, never displayed
    let _ = writeln!(out, "{SIZE} = {}", look.text.size); // ui-text-exempt: a preferences-file value, never displayed
    let _ = writeln!(out, "{INK} = {}", ink_text(look.text.colour)); // ui-text-exempt: a preferences-file value, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formdraft::TextLook;
    use pdfcer_core::edit::BorderStyle;

    fn read(text: &str) -> FormStylePrefs {
        let mut prefs = FormStylePrefs::default();
        for line in text
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
        {
            let (k, v) = line.split_once(" = ").unwrap();
            assert!(
                matches!(parse_key(&mut prefs, k, v), KeyOutcome::Accepted),
                "{line}"
            );
        }
        prefs
    }

    #[test]
    fn a_set_look_round_trips() {
        let look = FieldLook {
            border_width: 2.5,
            border_style: BorderStyle::Dashed,
            background: Some(MkColor::None),
            border_color: Some(MkColor::Rgb(1.0, 0.25, 0.0)),
            text: TextLook {
                font: Std14::TimesRoman,
                size: 17.0,
                colour: TextColor::Cmyk(0.0, 1.0, 0.5, 0.125),
            },
        };
        let mut out = String::new();
        write_block(&FormStylePrefs { look: Some(look) }, &mut out);
        assert_eq!(read(&out).look, Some(look));
    }

    #[test]
    fn unset_writes_no_keys_and_reads_back_unset() {
        let mut out = String::new();
        write_block(&FormStylePrefs::default(), &mut out);
        assert_eq!(read(&out), FormStylePrefs::default());
    }

    #[test]
    fn bad_values_are_refused_and_other_keys_are_not_claimed() {
        let mut prefs = FormStylePrefs::default();
        for (k, v) in [
            (WIDTH, "-1"),
            (STYLE, "wavy"),
            (FILL, "rgb 2 0 0"),
            (FONT, "Arial"),
            (INK, "none"),
        ] {
            assert!(
                matches!(parse_key(&mut prefs, k, v), KeyOutcome::BadValue),
                "{k}"
            );
        }
        assert_eq!(prefs.look, None);
        assert!(matches!(
            parse_key(&mut prefs, "smooth_enlarged_pictures", "true"),
            KeyOutcome::NotMine
        ));
    }
}
