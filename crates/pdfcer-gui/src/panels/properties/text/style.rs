//! The style forecast: **what pressing Bold or Italic would do to this run**,
//! and the seven sentences that say it.
//!
//! # Why this is a module and not a section of [`super`]
//!
//!
//! ⇒ What stayed behind is the *draft* — the mutable state of the section —
//! and what came here is the *prediction*. [`TextStyleDraft::forecast`] is
//! still a method on that draft, in a second `impl` block, because a child
//! module can see its parent's private fields and splitting a type from its
//! own probe would have meant widening the fields to `pub(crate)` for no
//! reason but file length.
//!
//! # What this module holds
//!
//! * [`StyleOutlook`] — the six outcomes the engine's ladder can reach, plus
//!   the operator's own refusal posture;
//! * [`TriedFace`] and [`StyleForecast`] — an outcome together with the real
//!   faces the ladder will step over on the way to it;
//! * [`TextStyleDraft::forecast`] — the one engine call, per axis;
//! * [`bold_hint`], [`italic_hint`] and the shared [`hint`] — the mapping from
//!   a forecast to the sentence the operator reads on hover.
//!
//! The sentences themselves live in [`crate::text::panels::properties`], per
//! the `ui_text` gate; this module chooses between them and never writes one.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/text/style.md`.

use super::{TextStyleDraft, shorten};
use crate::app::state::OpenDoc;
use crate::text::panels::properties as t;

/// **Which RUNG of the engine's style ladder one of the two weight buttons
/// would take**, said before the press.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StyleOutlook {
    /// **Rung 1**, same family: the page already carries the bold or italic
    /// form of this text's own typeface, and pdfcer will bind it.
    ///
    /// The string is the human `/BaseFont`, shortened the way the face chooser
    /// shortens it, because it is going into a sentence a person reads.
    SiblingOnPage(String),
    /// **Rung 1**, different family: the only real face on the page that can
    /// show this run belongs to another typeface.
    ///
    /// Its own variant and not a flag, because to a draughtsman these are
    /// different events. Taking the same family's bold face is invisible on a
    /// plot; taking another family's changes the shape of a title block. Only
    /// one of the two is worth warning him about before the press, and
    /// `StyleLadder::same_family` is the engine's own answer to which is which.
    OtherFamilyOnPage(String),
    /// **Rung 2**: nothing on the page can do it, so pdfcer will bind the
    /// standard-14 sibling of this text's own family — a real face, needing no
    /// font file, costing about sixty bytes.
    ///
    StandardSibling(String),
    /// **Rung 4**: no real face is available by any route, so the letters will
    /// be thickened or slanted and the engine will say so afterwards.
    Synthesized,
    /// **Rung 0**: the run is already that way, so the press will change
    /// nothing.
    ///
    /// Worth a sentence rather than silence. "I pressed it and nothing
    /// happened" is indistinguishable from a broken button, and this is the
    /// difference between a control that did nothing and one that had nothing
    /// to do.
    AlreadyStyled,
    /// The ladder would reach rung 4 and the operator's own posture forbids it,
    /// so the press will be **declined** —
    /// `FormatError::SynthesisRefusedByPosture`.
    ///
    /// The button still does not grey. R9 reserves greying for the
    /// *temporarily* unavailable, and this is neither temporary nor a
    /// malfunction: it is *Settings ▸ Fonts ▸ never fake it* being obeyed
    /// exactly as set. The hover says so in those terms, because an operator
    /// who reads "pdfcer could not change that text" about their own
    /// instruction turns the setting back off.
    Declined,
}

/// One real face the ladder will try and pass over, in the operator's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TriedFace {
    /// The `/BaseFont` that claimed the style, shortened for reading.
    pub(crate) face: String,
    /// The character with no glyph, when one character is the whole reason.
    pub(crate) character: Option<char>,
}

/// What one weight button would do, and what it will step over on the way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StyleForecast {
    /// The rung the ladder will take.
    pub(crate) outlook: StyleOutlook,
    /// The real faces it will try and reject first, in the order tried.
    pub(crate) tried: Vec<TriedFace>,
}

impl TextStyleDraft {
    /// One axis's [`StyleForecast`], or `None` when the probe did not answer.
    pub(super) fn forecast(
        &self,
        doc: &OpenDoc,
        page: usize,
        read: &crate::canvas::textedit::pin::Inspected,
        bold: bool,
    ) -> Option<StyleForecast> {
        use pdfcer_core::text_edit::{
            FormatError, FormatOptions, PassedOver, StyleRung, StyleSynthesis,
        };
        let want = StyleSynthesis::new(bold, !bold);
        let options = FormatOptions::default().with_style_policy(doc.settings.style_policy);
        let ladder =
            match doc
                .session
                .preview_style_ladder(page, "", Some(read.pin.span), want, &options)
            {
                Ok(ladder) => ladder,
                // The operator's own setting, previewed as their setting. This
                // is the ONE error worth a sentence: every other failure here is
                // "the probe could not run", and this one is "the probe ran and the
                // answer is that pdfcer will decline, because you told it to".
                Err(FormatError::SynthesisRefusedByPosture { .. }) => {
                    return Some(StyleForecast {
                        outlook: StyleOutlook::Declined,
                        tried: Vec::new(),
                    });
                }
                Err(_) => return None,
            };

        // The faces the ladder will step over, carried alongside the rung
        // rather than folded into it. `Refusal::character` is the whole reason
        // this is worth carrying: the engine's `reason` is accurate and
        // technical, and the hover wants *"no 'o'"*. See [`TriedFace`].
        //
        // The closure's parameter is ANNOTATED, and that is a fix rather than
        // decoration. `Pass 295.0` re-exported `StyleLadder` but not
        // `PassedOver`, so `pdfcer_core::text_edit::PassedOver` did not resolve
        // and the shape this code walks was invisible — inference typed it and
        // no reader could tell what `passed` was without opening the engine.
        // `Pass 295.1` (`e360e11`) re-exported it within the hour, and gated the
        // whole class: a re-exported type whose public field names an
        // un-re-exported type is now a build failure over there. Two more were
        // live at the time, `NewTextFace` and `FormLeaf`.
        let tried = ladder
            .passed_over
            .iter()
            .map(|passed: &PassedOver| TriedFace {
                face: shorten(&passed.base_font).to_owned(),
                character: passed.refusal.as_ref().and_then(|r| r.character),
            })
            .collect();

        let outlook = match ladder.rung {
            StyleRung::AlreadyStyled => StyleOutlook::AlreadyStyled,
            StyleRung::Synthetic => StyleOutlook::Synthesized,
            // `same_family` is `Option<bool>` and `None` means NOTHING WAS
            // BOUND — the engine says at the field that it must not be
            // flattened into `Some(false)`. On a rung that bound a face that
            // would be an engine invariant breaking, so it falls to the
            // catch-all below and the hover says the honest conditional thing
            // rather than claiming a family relationship nobody measured.
            StyleRung::RealFaceOnPage => match (ladder.bound.as_deref(), ladder.same_family) {
                (Some(face), Some(true)) => StyleOutlook::SiblingOnPage(shorten(face).to_owned()),
                (Some(face), Some(false)) => {
                    StyleOutlook::OtherFamilyOnPage(shorten(face).to_owned())
                }
                _ => return None,
            },
            StyleRung::StandardFourteenSibling => {
                StyleOutlook::StandardSibling(shorten(ladder.bound.as_deref()?).to_owned())
            }
            // A named catch-all rather than a fall-through, because
            // `StyleRung` is `#[non_exhaustive]`: rung 3 (`--font-dir` donors,
            // `Pass 142.0`) is not built in this shell and a fifth rung is
            // possible. A variant this build has never seen must land somewhere
            // honest, and "nothing is known" is the one answer that is true of
            // it. It renders the conditional hint, which is what was said
            // before any of this existed.
            _ => return None,
        };
        Some(StyleForecast { outlook, tried })
    }
}

/// The Bold button's hover text, given what the engine says would happen.
pub(super) fn bold_hint(draft: &TextStyleDraft) -> String {
    hint(draft.bold_outlook(), true)
}

/// The Italic button's hover text. See [`bold_hint`] for the whole argument;
/// this is the same seven rows with *slant* in place of *thicken*, from the
/// draft's separately-probed italic axis.
pub(super) fn italic_hint(draft: &TextStyleDraft) -> String {
    hint(draft.italic_outlook(), false)
}

/// Both buttons' hover text, from one forecast and one axis flag.
fn hint(forecast: Option<&StyleForecast>, bold: bool) -> String {
    let Some(forecast) = forecast else {
        return if bold {
            t::text_bold_hint().to_owned()
        } else {
            t::text_italic_hint().to_owned()
        };
    };
    let mut line = match &forecast.outlook {
        StyleOutlook::AlreadyStyled => {
            if bold {
                t::text_bold_hint_already().to_owned()
            } else {
                t::text_italic_hint_already().to_owned()
            }
        }
        StyleOutlook::SiblingOnPage(face) => {
            if bold {
                t::text_bold_hint_sibling_face(face)
            } else {
                t::text_italic_hint_sibling_face(face)
            }
        }
        StyleOutlook::OtherFamilyOnPage(face) => {
            if bold {
                t::text_bold_hint_other_family(face)
            } else {
                t::text_italic_hint_other_family(face)
            }
        }
        StyleOutlook::StandardSibling(face) => {
            if bold {
                t::text_bold_hint_standard_sibling(face)
            } else {
                t::text_italic_hint_standard_sibling(face)
            }
        }
        StyleOutlook::Synthesized => {
            if bold {
                t::text_bold_hint_synthetic().to_owned()
            } else {
                t::text_italic_hint_synthetic().to_owned()
            }
        }
        StyleOutlook::Declined => {
            if bold {
                t::text_bold_hint_declined().to_owned()
            } else {
                t::text_italic_hint_declined().to_owned()
            }
        }
    };
    if !forecast.tried.is_empty() {
        let tried: Vec<(&str, Option<char>)> = forecast
            .tried
            .iter()
            .map(|t| (t.face.as_str(), t.character))
            .collect();
        line.push_str(&t::text_hint_faces_tried(&tried));
    }
    line
}

#[cfg(test)]
mod outlook_tests {
    use super::*;

    /// A draft carrying the two forecasts and nothing else.
    fn drafted(bold: Option<StyleForecast>, italic: Option<StyleForecast>) -> TextStyleDraft {
        TextStyleDraft {
            bold_outlook: bold,
            italic_outlook: italic,
            ..Default::default()
        }
    }

    /// A forecast of one outlook with nothing passed over — the ordinary case,
    /// and the one where the addendum must not appear.
    fn plain(outlook: StyleOutlook) -> Option<StyleForecast> {
        Some(StyleForecast {
            outlook,
            tried: Vec::new(),
        })
    }

    /// **Seven outcomes, seven different sentences**, per axis.
    #[test]
    fn every_outlook_earns_its_own_sentence() {
        let mut seen: Vec<String> = Vec::new();
        for forecast in [
            None,
            plain(StyleOutlook::AlreadyStyled),
            plain(StyleOutlook::SiblingOnPage("Arial-Bold".to_owned())),
            plain(StyleOutlook::OtherFamilyOnPage("Arial-Bold".to_owned())),
            plain(StyleOutlook::StandardSibling("Arial-Bold".to_owned())),
            plain(StyleOutlook::Synthesized),
            plain(StyleOutlook::Declined),
        ] {
            let draft = drafted(forecast, None);
            let line = bold_hint(&draft);
            assert!(
                !seen.contains(&line),
                "two outlooks produced the same hover text: {line}"
            );
            seen.push(line);
        }
        assert_eq!(seen.len(), 7, "a row was dropped from the sweep");
    }

    /// **The two axes read their own probes**, and never each other's.
    #[test]
    fn the_two_buttons_do_not_borrow_each_others_answer() {
        let draft = drafted(
            plain(StyleOutlook::SiblingOnPage("Arial-Bold".to_owned())),
            plain(StyleOutlook::Synthesized),
        );
        assert!(bold_hint(&draft).contains("Arial-Bold"));
        assert!(!italic_hint(&draft).contains("Arial-Bold"));
        assert!(italic_hint(&draft).contains("slant"));
    }

    /// **Bold thickens and italic slants**, and neither sentence borrows the
    /// other's verb.
    #[test]
    fn the_synthetic_sentences_name_the_right_operation() {
        let draft = drafted(
            plain(StyleOutlook::Synthesized),
            plain(StyleOutlook::Synthesized),
        );
        let bold = bold_hint(&draft);
        let italic = italic_hint(&draft);
        assert!(bold.contains("thicken"), "{bold}");
        assert!(!bold.contains("slant"), "{bold}");
        assert!(italic.contains("slant"), "{italic}");
        assert!(!italic.contains("thicken"), "{italic}");
    }

    /// **The two rung-1 sentences disagree about the letterforms**, which
    /// is the only thing the operator can act on.
    #[test]
    fn the_family_verdict_changes_what_the_hover_promises() {
        let same = bold_hint(&drafted(
            plain(StyleOutlook::SiblingOnPage("Arial-Bold".to_owned())),
            None,
        ));
        let other = bold_hint(&drafted(
            plain(StyleOutlook::OtherFamilyOnPage("Arial-Bold".to_owned())),
            None,
        ));
        assert!(same.contains("own typeface"), "{same}");
        assert!(!same.contains("shaped differently"), "{same}");
        assert!(other.contains("different typeface"), "{other}");
        assert!(other.contains("shaped differently"), "{other}");
    }

    /// **The refusal is predicted, names the cause, and does not prescribe
    /// a font.**
    #[test]
    fn the_declined_case_names_the_setting_and_not_a_remedy() {
        let line = bold_hint(&drafted(plain(StyleOutlook::Declined), None));
        assert!(line.contains("refused"), "{line}");
        assert!(line.contains("never to fake"), "{line}");
        assert!(
            !line.to_lowercase().contains("choose another"),
            "the sentence must not prescribe a font: {line}"
        );
    }

    /// **A face the ladder will step over is named before the press**, with
    /// the character that defeated it.
    #[test]
    fn a_passed_over_face_is_named_with_its_missing_character() {
        let draft = drafted(
            Some(StyleForecast {
                outlook: StyleOutlook::StandardSibling("Helvetica-Bold".to_owned()),
                tried: vec![TriedFace {
                    face: "Times-Bold".to_owned(),
                    character: Some('o'),
                }],
            }),
            None,
        );
        let line = bold_hint(&draft);
        assert!(line.contains("Helvetica-Bold"), "{line}");
        assert!(line.contains("Times-Bold"), "{line}");
        assert!(line.contains("no 'o'"), "{line}");
    }

    /// **A face passed over for no one character gets no empty parenthesis.**
    #[test]
    fn a_passed_over_face_with_no_character_is_named_bare() {
        let draft = drafted(
            Some(StyleForecast {
                outlook: StyleOutlook::Synthesized,
                tried: vec![TriedFace {
                    face: "Times-Bold".to_owned(),
                    character: None,
                }],
            }),
            None,
        );
        let line = bold_hint(&draft);
        assert!(line.contains("Times-Bold"), "{line}");
        assert!(!line.contains("()"), "{line}");
        assert!(!line.contains("(no"), "{line}");
    }

    /// **Nothing passed over means no addendum at all.**
    #[test]
    fn an_empty_passed_over_list_says_nothing() {
        let line = bold_hint(&drafted(
            plain(StyleOutlook::StandardSibling("Helvetica-Bold".to_owned())),
            None,
        ));
        assert!(!line.contains("pass over"), "{line}");
    }

    /// **A fresh draft says the conditional**, not a prediction.
    #[test]
    fn an_unsynced_draft_promises_nothing() {
        let draft = drafted(None, None);
        assert_eq!(bold_hint(&draft), t::text_bold_hint());
        assert_eq!(italic_hint(&draft), t::text_italic_hint());
    }
}
