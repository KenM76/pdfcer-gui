//! # `declined::remedy` — the one command a decline offers as a button
//!
//! Contract: [`Declined::remedy`] names the registered command that removes the
//! cause of a decline, or `None`. The status bar draws it beside the sentence
//! only when that command is registered in this build, so a decline never
//! offers a button for a capability the build lacks.

use super::Declined;
use crate::editmodel::refusal::Refusal;
use crate::text::textedit::{EditRefusal, ReflowRefusal};

impl Declined {
    /// The command id whose dialog removes this decline's cause.
    #[must_use]
    pub fn remedy(&self) -> Option<&'static str> {
        match self {
            Self::TextClick(Refusal::PictureOfText | Refusal::NoText) => {
                Some("file.ocr") // ui-text-exempt: command id, never displayed
            }
            Self::EditText(EditRefusal::DocumentProtected)
            | Self::Reflow(ReflowRefusal::Encrypted) => {
                Some("file.encrypt") // ui-text-exempt: command id, never displayed
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture of text offers Recognise text; a protected document offers
    /// Encrypt…; a click on blank paper offers nothing.
    #[test]
    fn each_remedy_names_the_command_that_removes_the_cause() {
        assert_eq!(
            Declined::TextClick(Refusal::PictureOfText).remedy(),
            Some("file.ocr")
        );
        assert_eq!(
            Declined::EditText(EditRefusal::DocumentProtected).remedy(),
            Some("file.encrypt")
        );
        assert_eq!(
            Declined::Reflow(ReflowRefusal::Encrypted).remedy(),
            Some("file.encrypt")
        );
        assert_eq!(Declined::TextClick(Refusal::NoRun).remedy(), None);
    }
}
