//! # `pastechords` — which chord means which paste
//!
//! One preference, two values, and the whole of its subject is that **neither
//! order is obviously right**.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pastechords.md`.

/// **Which chord pastes a form field as a NEW field, and which as a DUPLICATE.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PasteChords {
    /// `Ctrl+V` pastes a **new** field; `Ctrl+Shift+V` pastes a **duplicate**.
    ///
    /// The default, and the operator's own ruling.
    #[default]
    PdfcerOrder,
    /// `Ctrl+V` pastes a **duplicate**; `Ctrl+Shift+V` pastes a **new** field.
    ///
    /// Acrobat's own assignment.
    AcrobatOrder,
}

impl PasteChords {
    /// Both, in the order the settings pane offers them.
    pub const ALL: &'static [Self] = &[Self::PdfcerOrder, Self::AcrobatOrder];

    /// **A harness override, read from the environment.**
    #[must_use]
    pub fn from_environment() -> Option<Self> {
        // ui-text-exempt: an environment variable name, never displayed.
        std::env::var("PDFCER_DIAG_PASTE_CHORDS")
            .ok()
            .and_then(|v| Self::from_key(v.trim()))
    }

    /// The token this is written under in the preferences file.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::PdfcerOrder => "new_field_first",
            // ui-text-exempt: a file token, never displayed.
            Self::AcrobatOrder => "acrobat",
        }
    }

    /// Read a token back, or `None` if it names nothing.
    ///
    /// Derived from [`Self::ALL`] and [`Self::key`] so it cannot drift from the
    /// writer — the same shape `WheelPaging::from_key` uses.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|o| o.key() == key)
    }

    /// The chord that reaches `edit.paste` — a **new** field — under this order.
    #[must_use]
    pub const fn new_field_chord(self) -> &'static str {
        match self {
            // ui-text-exempt: keymap chord spellings, matched against the manifest.
            Self::PdfcerOrder => "Ctrl+V",
            Self::AcrobatOrder => "Ctrl+Shift+V",
        }
    }

    /// The chord that reaches `edit.paste_duplicate` under this order.
    #[must_use]
    pub const fn duplicate_chord(self) -> &'static str {
        match self {
            // ui-text-exempt: keymap chord spellings, matched against the manifest.
            Self::PdfcerOrder => "Ctrl+Shift+V",
            Self::AcrobatOrder => "Ctrl+V",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two orders must be an EXCHANGE, not two independent choices.
    #[test]
    fn every_order_binds_the_two_commands_to_two_different_chords() {
        for order in PasteChords::ALL {
            assert_ne!(
                order.new_field_chord(),
                order.duplicate_chord(),
                "{order:?} puts both pastes on one chord, so one of them is unreachable"
            );
        }
    }

    /// The two orders are each other's mirror, and nothing else.
    #[test]
    fn the_acrobat_order_is_exactly_the_pdfcer_order_reversed() {
        assert_eq!(
            PasteChords::PdfcerOrder.new_field_chord(),
            PasteChords::AcrobatOrder.duplicate_chord()
        );
        assert_eq!(
            PasteChords::PdfcerOrder.duplicate_chord(),
            PasteChords::AcrobatOrder.new_field_chord()
        );
    }

    /// Every value survives the preferences file.
    ///
    /// Over `ALL`, not over two literals: a third order added later is
    /// covered without anybody remembering to extend this.
    #[test]
    fn every_order_round_trips_through_its_file_token() {
        for order in PasteChords::ALL {
            assert_eq!(
                PasteChords::from_key(order.key()),
                Some(*order),
                "{order:?} does not survive a save and reload"
            );
        }
        assert_eq!(PasteChords::from_key("nonsense"), None);
    }

    /// The default is the operator's own ruling, not Acrobat's.
    #[test]
    fn the_default_is_the_operators_ruling() {
        assert_eq!(PasteChords::default(), PasteChords::PdfcerOrder);
        assert_eq!(PasteChords::default().new_field_chord(), "Ctrl+V");
    }
}
