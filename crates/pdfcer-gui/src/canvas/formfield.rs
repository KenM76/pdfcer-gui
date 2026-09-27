//! # `canvas::formfield` — placing a new form field on the page
//!
//!
//! > *"get all the form buttons on the ribbon working next along with adding
//! > all the form feature buttons. when I click one I should be able to click
//! > on the canvas to place the position or drag a box for size then a pop up
//! > lets me set the details for the feature."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/formfield.md`.

pub use pdfcer_gui_base::formdraft as draft;
pub use pdfcer_gui_base::pushbutton as action;
pub mod ghost;

pub use draft::{Draft, Remembered};

pub use pdfcer_gui_base::formfieldkind::FormFieldKind;

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
