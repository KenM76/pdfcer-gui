//! # `canvas::formfield` — placing a new form field on the page
//!
//!
//! > *"get all the form buttons on the ribbon working next along with adding
//! > all the form feature buttons. when I click one I should be able to click
//! > on the canvas to place the position or drag a box for size then a pop up
//! > lets me set the details for the feature."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/formfield.md`.

pub use pdfcer_gui_base::pushbutton as action;
pub mod draft;
pub mod ghost;

pub use draft::{Draft, Remembered};

use crate::canvas::markup::MarkupKind;

/// The five kinds of form control pdfcer can author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormFieldKind {
    /// A box the operator types into.
    Text,
    /// A single on/off box.
    CheckBox,
    /// One of a group, where choosing one clears the others.
    ///
    /// The only kind whose meaning depends on a *group* rather than on the
    /// field alone: radio buttons that share a name are one control. The dialog
    /// therefore asks for the group name, and two radios placed with the same
    /// name become alternatives rather than two independent buttons.
    Radio,
    /// A drop-down or list of options.
    Choice,
    /// A button that performs an action.
    ///
    ///
    /// That is R9-correct rather than an exception to it. R9 reserves greying
    /// for a *temporarily* unavailable capability that is explained on hover,
    /// and this is precisely that: `add_push_button` works, so the control is
    /// authorable — what pdfcer cannot yet do is **run** what a button would do,
    /// because it executes no PDF actions. A button placed today would look
    /// right and do nothing, and a control that silently does nothing is worse
    /// than one that says why it is unavailable.
    PushButton,
}

impl FormFieldKind {
    /// Every kind, in the order they appear on the ribbon.
    pub const ALL: [Self; 5] = [
        Self::Text,
        Self::CheckBox,
        Self::Radio,
        Self::Choice,
        Self::PushButton,
    ];

    /// Whether pdfcer can do anything useful with this kind **once placed**.
    #[must_use]
    pub const fn is_useful_once_placed(self) -> bool {
        match self {
            Self::Text | Self::CheckBox | Self::Radio | Self::Choice | Self::PushButton => true,
        }
    }

    /// The command id that arms this kind.
    #[must_use]
    pub const fn command_id(self) -> &'static str {
        match self {
            Self::Text => "edit.form_text_field",
            Self::CheckBox => "edit.form_check_box",
            Self::Radio => "edit.form_radio_button",
            Self::Choice => "edit.form_choice",
            Self::PushButton => "edit.form_push_button",
        }
    }

    /// The default size, in points, for a field placed by a single **click**
    /// rather than by a drag.
    #[must_use]
    pub const fn default_size_pt(self) -> (f64, f64) {
        match self {
            Self::Text | Self::Choice => (160.0, 20.0),
            Self::CheckBox | Self::Radio => (14.0, 14.0),
            Self::PushButton => (80.0, 22.0),
        }
    }

    /// What to call this kind in a sentence to the operator.
    #[must_use]
    pub fn noun(self) -> String {
        match self {
            Self::Text => crate::text::forms::form_noun_text(),
            Self::CheckBox => crate::text::forms::form_noun_check_box(),
            Self::Radio => crate::text::forms::form_noun_radio(),
            Self::Choice => crate::text::forms::form_noun_choice(),
            Self::PushButton => crate::text::forms::form_noun_push_button(),
        }
    }

    /// The stem an auto-generated field name is built from.
    #[must_use]
    pub const fn name_prefix(self) -> &'static str {
        match self {
            Self::Text => "Text", // ui-text-exempt: a PDF /T field-name stem written into the file
            Self::CheckBox => "Check Box", // ui-text-exempt: a PDF /T field-name stem written into the file
            Self::Radio => "Group",
            Self::Choice => "Dropdown",
            Self::PushButton => "Button",
        }
    }

    /// The markup kind whose drag machinery this borrows.
    #[must_use]
    pub const fn drag_shape(self) -> MarkupKind {
        MarkupKind::Rectangle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every kind has a distinct command id**, because R8 makes the id the
    /// only route by which the ribbon learns the capability exists — two kinds
    /// sharing one would make a build that strips one strip both.
    #[test]
    fn each_kind_has_its_own_command() {
        for (i, a) in FormFieldKind::ALL.iter().enumerate() {
            for b in FormFieldKind::ALL.iter().skip(i + 1) {
                assert_ne!(a.command_id(), b.command_id(), "{a:?} and {b:?}");
            }
        }
    }

    /// **A click places something with real area**, for every kind.
    #[test]
    fn a_click_places_something_with_area() {
        for k in FormFieldKind::ALL {
            let (w, h) = k.default_size_pt();
            assert!(w > 1.0 && h > 1.0, "{k:?} would place a {w}x{h} pt field");
        }
    }

    /// **NO KIND IS AUTHORABLE-BUT-INERT ANY MORE**, and the test that
    /// used to say otherwise did its job.
    #[test]
    fn no_kind_is_authorable_but_inert() {
        let inert: Vec<_> = FormFieldKind::ALL
            .into_iter()
            .filter(|k| !k.is_useful_once_placed())
            .collect();
        assert!(
            inert.is_empty(),
            "{inert:?} can be authored and does nothing once placed. Either wire the verb that \
             makes it useful, or grey its command with a condition `app::conditions` does not \
             set and give `app::dispatch::forms` a worded refusal for it — greying alone is a \
             drawing, not a rule, and every other route into the dispatcher ignores it."
        );
    }

    /// **The push button in particular**, named rather than left to the
    /// blanket above.
    ///
    #[test]
    fn a_push_button_can_be_given_something_to_do() {
        assert!(
            FormFieldKind::PushButton.is_useful_once_placed(),
            "the placement dialog offers seven actions and the ribbon item is live; a `false` \
             here would grey the control while the dialog behind it still worked"
        );
    }
}
