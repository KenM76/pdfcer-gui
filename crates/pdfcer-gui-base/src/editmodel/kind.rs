//! # `editmodel::kind` — which of the two text verbs is armed
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/editmodel/kind.md`.

/// **Which of the two text verbs is armed.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEditKind {
    /// `edit.text` — replace the words in a run that is already on the page.
    Edit,
    /// `edit.add_text` — place new page content where the operator clicks.
    Add,
}

impl TextEditKind {
    /// The command id that arms this kind.
    #[must_use]
    pub const fn command_id(self) -> &'static str {
        match self {
            // ui-text-exempt: command ids, never displayed.
            Self::Edit => "edit.text",
            Self::Add => "edit.add_text",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TextEditKind;

    /// **The two kinds name the two registered commands, and they are
    /// different.** A copy-paste that gave both the same id would arm one tool
    /// from two buttons and nothing would notice.
    #[test]
    fn each_kind_names_its_own_registered_command() {
        assert_eq!(TextEditKind::Edit.command_id(), "edit.text");
        assert_eq!(TextEditKind::Add.command_id(), "edit.add_text");
    }
}
