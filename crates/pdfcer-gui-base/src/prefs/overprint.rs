//! # `prefs::overprint` — whether a document opens with overprint preview on
//!
//! Key: `overprint_preview_default` (`pdfx` | `always` | `never` |
//! `automatic`, default `always`). Acrobat's *Use Overprint Preview* choice,
//! with its four answers. It decides `view.overprint_preview` once, as a
//! document opens; the View ▸ Display button still flips it afterwards.
//! Screen only, like the button: printing and exports are not affected.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/overprint.md`.

use std::fmt::Write as _;

use super::printing::KeyOutcome;

/// What a document says about how it prints, as far as the overprint
/// default reads it. Read by [`crate::overprintseed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrintClaims {
    /// The file claims PDF/X.
    pub pdfx: bool,
    /// Whether anything drawn asks for overprint; `None` when that is not
    /// known, because the walk has not answered or could not finish.
    pub overprint: Option<bool>,
}

/// The file key.
const KEY: &str = "overprint_preview_default";

/// When a newly opened document shows overprint preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverprintDefault {
    /// Only when the file claims PDF/X (Acrobat's own default).
    PdfxOnly,
    /// Every document (pdfcer's default: the page as it prints).
    #[default]
    Always,
    /// No document; the View button turns it on.
    Never,
    /// When the document asks for overprint anywhere.
    Automatic,
}

impl OverprintDefault {
    /// Every choice, in Acrobat's order.
    pub const ALL: &'static [Self] = &[Self::PdfxOnly, Self::Always, Self::Never, Self::Automatic];

    /// The file and trace token.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::PdfxOnly => "pdfx",
            Self::Always => "always",
            Self::Never => "never",
            Self::Automatic => "automatic",
        }
    }

    /// The choice a token names.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.id() == id)
    }

    /// Whether a document making `claims` opens with the preview on. An
    /// unknown overprint answer is on: the page as it prints.
    #[must_use]
    pub const fn resolve(self, claims: PrintClaims) -> bool {
        match self {
            Self::PdfxOnly => claims.pdfx,
            Self::Always => true,
            Self::Never => false,
            Self::Automatic => match claims.overprint {
                Some(asks) => asks,
                None => true,
            },
        }
    }

    /// Whether answering needs the document read at all.
    #[must_use]
    pub const fn reads_document(self) -> bool {
        matches!(self, Self::PdfxOnly | Self::Automatic)
    }
}

/// Claim `key = value` if it is this group's.
pub(super) fn parse_key(pref: &mut OverprintDefault, key: &str, value: &str) -> KeyOutcome {
    if key != KEY {
        return KeyOutcome::NotMine;
    }
    match OverprintDefault::from_id(value.trim()) {
        Some(choice) => {
            *pref = choice;
            KeyOutcome::Accepted
        }
        None => KeyOutcome::BadValue,
    }
}

/// Append this group's block to a preferences file.
pub(super) fn write_block(pref: OverprintDefault, out: &mut String) {
    out.push_str(
        "\n\
         # overprint_preview_default: pdfx, always, never or automatic\n\
         # (default always). Whether a document opens showing overprinting\n\
         # ink as it prints: only PDF/X files, every file, none, or files\n\
         # that ask for overprint. Screen only.\n",
    );
    let _ = writeln!(out, "{KEY} = {}", pref.id()); // ui-text-exempt: a preferences-file line, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: PrintClaims = PrintClaims {
        pdfx: false,
        overprint: Some(false),
    };

    #[test]
    fn each_choice_reads_its_own_claim() {
        let pdfx = PrintClaims {
            pdfx: true,
            ..PLAIN
        };
        let op = PrintClaims {
            overprint: Some(true),
            ..PLAIN
        };
        use OverprintDefault as D;
        assert!(D::PdfxOnly.resolve(pdfx) && !D::PdfxOnly.resolve(op));
        assert!(D::Automatic.resolve(op) && !D::Automatic.resolve(pdfx));
        assert!(D::Always.resolve(PLAIN) && !D::Never.resolve(pdfx));
    }

    #[test]
    fn an_unknown_overprint_answer_shows_the_preview() {
        let unknown = PrintClaims {
            overprint: None,
            ..PLAIN
        };
        assert!(OverprintDefault::Automatic.resolve(unknown));
        assert!(!OverprintDefault::PdfxOnly.resolve(unknown));
    }

    #[test]
    fn every_choice_round_trips_and_a_bad_value_is_refused() {
        for choice in OverprintDefault::ALL {
            let mut out = String::new();
            write_block(*choice, &mut out);
            let (key, value) = out.lines().last().unwrap().split_once(" = ").unwrap();
            let mut back = OverprintDefault::default();
            assert!(matches!(
                parse_key(&mut back, key, value),
                KeyOutcome::Accepted
            ));
            assert_eq!(back, *choice);
        }
        let mut pref = OverprintDefault::default();
        assert!(matches!(
            parse_key(&mut pref, KEY, "sometimes"),
            KeyOutcome::BadValue
        ));
        assert!(matches!(
            parse_key(&mut pref, "other", "never"),
            KeyOutcome::NotMine
        ));
    }
}
