//! # `redrawoffer` — a restyle the engine refused because pdfcer did not draw
//! the mark's current look, held so the Properties panel can offer to redraw
//! it.
//!
//! `set_text_annot_style` and `set_marker_style` refuse a mark whose
//! appearance another program drew (or, for a text box, whose words carry
//! rich text), and accept the same call with `redraw_as_plain` set. The
//! refusal is recorded here with the call that was refused; the panel shows
//! the reason and one button, and the button raises [`RedrawOffer::retry`].
//! An offer belongs to one annotation: the panel shows it only while that
//! annotation is selected, and a later successful restyle of it clears it.

use pdfcer_core::annot_author::Color;
use pdfcer_core::edit::TextAnnotStyle;
use pdfcer_core::object::ObjId;

use crate::annotaction::AnnotAction;

/// Why the engine refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// `EditError::FreeTextAppearanceForeign`.
    TextBoxForeign,
    /// `EditError::FreeTextIsRichText`.
    TextBoxRich,
    /// `EditError::MarkerAppearanceForeign`.
    MarkerForeign,
}

impl Why {
    /// The trace token, stable for `ui-verify`.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::TextBoxForeign => "text-box-foreign",
            Self::TextBoxRich => "text-box-rich",
            Self::MarkerForeign => "marker-foreign",
        }
    }
}

/// The refused call, to send again with the redraw flag set.
#[derive(Debug, Clone, PartialEq)]
pub enum Refused {
    /// A `set_text_annot_style` call.
    TextStyle(Box<TextAnnotStyle>),
    /// A `set_marker_style` call's colour.
    Marker(Color),
}

/// One refused restyle, offered for a redraw.
#[derive(Debug, Clone, PartialEq)]
pub struct RedrawOffer {
    /// The annotation.
    pub id: ObjId,
    /// Why it was refused.
    pub why: Why,
    /// What was asked.
    pub refused: Refused,
}

impl RedrawOffer {
    /// The offer for an engine refusal, or `None` when the refusal is not one
    /// a redraw answers.
    #[must_use]
    pub fn of(error: &pdfcer_core::edit::EditError, id: ObjId, refused: Refused) -> Option<Self> {
        use pdfcer_core::edit::EditError as E;
        let why = match error {
            E::FreeTextAppearanceForeign { .. } => Why::TextBoxForeign,
            E::FreeTextIsRichText { .. } => Why::TextBoxRich,
            E::MarkerAppearanceForeign { .. } => Why::MarkerForeign,
            _ => return None,
        };
        Some(Self { id, why, refused })
    }

    /// The same call, with the redraw flag set.
    #[must_use]
    pub fn retry(&self) -> AnnotAction {
        match &self.refused {
            Refused::TextStyle(style) => AnnotAction::SetTextAnnotStyle {
                id: self.id,
                style: Box::new(TextAnnotStyle {
                    redraw_as_plain: true,
                    ..(**style).clone()
                }),
            },
            Refused::Marker(color) => AnnotAction::SetMarkerColour {
                id: self.id,
                color: *color,
                redraw_as_plain: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::edit::EditError;

    const ID: ObjId = ObjId {
        num: 7,
        generation: 0,
    };

    #[test]
    fn only_the_three_redrawable_refusals_make_an_offer() {
        let style = || Refused::TextStyle(Box::default());
        let rich = RedrawOffer::of(&EditError::FreeTextIsRichText { id: ID }, ID, style());
        assert_eq!(rich.map(|o| o.why), Some(Why::TextBoxRich));
        let marker = RedrawOffer::of(
            &EditError::MarkerAppearanceForeign { id: ID },
            ID,
            Refused::Marker(Color::Gray(0.0)),
        );
        assert_eq!(marker.map(|o| o.why), Some(Why::MarkerForeign));
        assert!(
            RedrawOffer::of(
                &EditError::AnnotationLocked {
                    id: ID,
                    subtype: "Text".to_owned()
                },
                ID,
                style()
            )
            .is_none()
        );
    }

    #[test]
    fn the_retry_is_the_refused_call_with_the_redraw_flag_set() {
        let asked = TextAnnotStyle {
            font_size: Some(14.0),
            ..TextAnnotStyle::default()
        };
        let offer = RedrawOffer {
            id: ID,
            why: Why::TextBoxForeign,
            refused: Refused::TextStyle(Box::new(asked)),
        };
        let AnnotAction::SetTextAnnotStyle { id, style } = offer.retry() else {
            panic!("a text-style offer retries the text-style verb");
        };
        assert_eq!(id, ID);
        assert!(style.redraw_as_plain);
        assert_eq!(style.font_size, Some(14.0));
    }
}
