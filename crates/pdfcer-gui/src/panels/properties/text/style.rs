//! The style forecast: **what pressing Bold or Italic would do to this run**,
//! and the seven sentences that say it.
//!
//! # Why this is a module and not a section of [`super`]
//!
//! `panels/properties/text.rs` reached 1,473 lines on 2026-09-11 against R2's
//! 1,500-line ceiling, and the rule's own instruction for that moment is to
//! find the seam rather than raise the limit. The seam is clean: everything
//! here is downstream of one engine call, `EditSession::preview_style_ladder`,
//! and nothing in it is touched by the size field, the colour swatch, the face
//! chooser or the section's layout.
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

use super::{TextStyleDraft, shorten};
use crate::app::state::OpenDoc;
use crate::text::panels::properties as t;

/// ★★★ **Which RUNG of the engine's style ladder one of the two weight buttons
/// would take**, said before the press.
///
/// # ★★★ It previewed the GATE until 2026-09-11, and the gate is a different
/// question
///
/// Until that afternoon this came from `preview_style_resolution` joined
/// against `preview_font_resources`. `preview_style_resolution` previews the
/// **R90 synthesis gate**, whose answer `StyleOutcome::WouldSynthesize` means
/// exactly *"no real face on THIS PAGE claims that style and covers this
/// run"*. That stopped being the same question as *"what will pressing Bold
/// do?"* the moment `Pass 179.0` added rung 2, because **the standard-14
/// sibling is by construction not on the page** — `Helvetica-Bold` needs no
/// font file at all — so the gate cannot see it and neither could this.
///
/// The visible effect was two instruments disagreeing by construction: the
/// tooltip promised thickened letters and the status line afterwards reported a
/// real `Helvetica-Bold`, on a CAD title block, which is the commonest page in
/// the operator's working set.
///
/// `EditSession::preview_style_ladder` (`Pass 295.0`) runs **the same planner
/// `format_text` runs**, walks the page's content once, and stages nothing. So
/// these variants are the engine's own rungs rather than a shell's inference
/// from an adjacent answer, and the preview and the commit are two readings of
/// one computation instead of two answers kept in agreement by hand.
///
/// ⇒ No rule is re-implemented here and none ever was: engine invariant R74
/// forbids `pdfcer-gui` re-deriving `family_stem`, `name_claims_bold` or
/// `name_claims_italic`, and this asks a question rather than answering one.
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
    /// ★★ Its own variant and not a flag, because to a draughtsman these are
    /// different events. Taking the same family's bold face is invisible on a
    /// plot; taking another family's changes the shape of a title block. Only
    /// one of the two is worth warning him about before the press, and
    /// `StyleLadder::same_family` is the engine's own answer to which is which.
    OtherFamilyOnPage(String),
    /// **Rung 2**: nothing on the page can do it, so pdfcer will bind the
    /// standard-14 sibling of this text's own family — a real face, needing no
    /// font file, costing about sixty bytes.
    ///
    /// ★★★ **The outcome this preview could not see at all until 2026-09-11**,
    /// and the one that fires most often on the operator's drawings. See the
    /// type's header.
    StandardSibling(String),
    /// **Rung 4**: no real face is available by any route, so the letters will
    /// be thickened or slanted and the engine will say so afterwards.
    Synthesized,
    /// **Rung 0**: the run is already that way, so the press will change
    /// nothing.
    ///
    /// ★ Worth a sentence rather than silence. "I pressed it and nothing
    /// happened" is indistinguishable from a broken button, and this is the
    /// difference between a control that did nothing and one that had nothing
    /// to do.
    AlreadyStyled,
    /// The ladder would reach rung 4 and the operator's own posture forbids it,
    /// so the press will be **declined** —
    /// `FormatError::SynthesisRefusedByPosture`.
    ///
    /// ★★ The button still does not grey. R9 reserves greying for the
    /// *temporarily* unavailable, and this is neither temporary nor a
    /// malfunction: it is *Settings ▸ Fonts ▸ never fake it* being obeyed
    /// exactly as set. The hover says so in those terms, because an operator
    /// who reads "pdfcer could not change that text" about their own
    /// instruction turns the setting back off.
    Declined,
}

/// One real face the ladder will try and pass over, in the operator's words.
///
/// # ★★★ The feature `passed_over` being prose cost outright
///
/// `StyleLadder::passed_over` was a `Vec<String>` of `"BaseFont (reason)"`
/// until `Pass 295.0`. Saying *"pdfcer will try `Times-Bold` and it has no
/// `o`"* in this shell's own voice would have meant splitting on `" ("` and
/// stripping a `")"` — a locator for `pdfcer-core`'s message format, living in
/// a GUI, breaking silently the first time a reason sentence gained a
/// parenthesis. A shell disciplined about not re-deriving engine facts keeps
/// quiet instead, so **the sentence was never written at all**: the operator
/// was told which rung bound and never which faces were tried and rejected,
/// which is the half he asks about.
///
/// ★★ `PassedOver` is now `{ base_font, reason, refusal }`, and
/// `Refusal::character` gives **the offending character**. That is what makes
/// this worth a type of its own here: the engine's `reason` is accurate and
/// technical (*"R-INV-1: character U+006F 'o' has no code in font
/// 'Times-Bold'"*), and the hover wants *"no 'o'"*. The character is carried;
/// the prose is not.
///
/// ★ `character` is `Option` because a face can be passed over for a reason
/// that is not one character — it could not be planned at all, say. The
/// sentence degrades to naming the face, which is still more than nothing was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TriedFace {
    /// The `/BaseFont` that claimed the style, shortened for reading.
    pub(crate) face: String,
    /// The character with no glyph, when one character is the whole reason.
    pub(crate) character: Option<char>,
}

/// What one weight button would do, and what it will step over on the way.
///
/// ★ Two fields rather than a sixth [`StyleOutlook`] variant, because they
/// answer different questions and are independently present: a rung-2 bind can
/// pass over two page faces on the way, and a rung-1 bind can pass over none.
/// Folding them together would make the passed-over list a property of the
/// outcome, which it is not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StyleForecast {
    /// The rung the ladder will take.
    pub(crate) outlook: StyleOutlook,
    /// The real faces it will try and reject first, in the order tried.
    pub(crate) tried: Vec<TriedFace>,
}

impl TextStyleDraft {
    /// One axis's [`StyleForecast`], or `None` when the probe did not answer.
    ///
    /// # ★★★ One engine call, and it is the SAME computation the press will run
    ///
    /// `EditSession::preview_style_ladder` runs `plan_style_ladder` — the
    /// function `format_text` runs — against the session's *staged* content, so
    /// the preview and the commit that follows it are two readings of one
    /// answer rather than two answers that have to be kept in agreement by
    /// hand. It walks the page's content once, plans, and stages, commits and
    /// caches nothing.
    ///
    /// ⇒ Nothing is re-derived here. Engine invariant R74 forbids `pdfcer-gui`
    /// re-deriving `family_stem`, `name_claims_bold` or `name_claims_italic`,
    /// and this function asks a question and maps the reply onto sentences.
    /// The previous shape — joining `preview_style_resolution`'s `selector`
    /// against `preview_font_resources`' accepted list to reconstruct an answer
    /// neither call gave — is deleted; see [`TextStyleDraft::sync`].
    ///
    /// # ★★ `options` is not optional, and passing the wrong one is a lie
    ///
    /// `StylePolicy::Refuse` changes the answer: under it a ladder that reaches
    /// rung 4 is a **refusal**, not a synthesis. The engine says so at the
    /// method — *"pass the same `FormatOptions` the commit will use"* — and
    /// this passes the operator's own posture out of the settings store, the
    /// same value `crate::app::actions::textstyle` puts on the commit. A
    /// preview run under a default posture would promise thickened letters to
    /// an operator who had ticked *never fake it*, which is the exact defect
    /// the deleted `Refuse`-pinned probe used to cause from the other side.
    ///
    /// ★ A single axis per call, because the two buttons issue two separate
    /// single-axis requests; see [`TextStyleDraft::italic_outlook`] for why
    /// neither may borrow the other's answer.
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
                // ★★★ The operator's own setting, previewed as their setting. This
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

        // ★★ The faces the ladder will step over, carried alongside the rung
        // rather than folded into it. `Refusal::character` is the whole reason
        // this is worth carrying: the engine's `reason` is accurate and
        // technical, and the hover wants *"no 'o'"*. See [`TriedFace`].
        //
        // ★ The closure's parameter is ANNOTATED, and that is a fix rather than
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
            // ★ `same_family` is `Option<bool>` and `None` means NOTHING WAS
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
            // ★ A named catch-all rather than a fall-through, because
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
///
/// # ★★★ Seven sentences, and the last is the one that was there before
///
/// | outlook | what the operator reads |
/// |---|---|
/// | [`StyleOutlook::AlreadyStyled`] | *this text is already bold* |
/// | [`StyleOutlook::SiblingOnPage`] | *this page already carries **Arial-Bold**, the bold form of this text's own typeface* |
/// | [`StyleOutlook::OtherFamilyOnPage`] | *pdfcer will use **Arial-Bold**, a real bold face from a different typeface* |
/// | [`StyleOutlook::StandardSibling`] | *pdfcer will add **Helvetica-Bold**, a standard PDF face, without embedding a font file* |
/// | [`StyleOutlook::Synthesized`] | *no real bold face can show this text, so pdfcer will thicken the letters* |
/// | [`StyleOutlook::Declined`] | *your settings say never fake it, so Bold will be refused here* |
/// | `None` — the probe did not answer | the conditional hint, unchanged |
///
/// Plus, appended to any of the six, the **passed-over addendum** when the
/// ladder stepped over a face on the way: *"It will pass over Times-Bold (no
/// 'o'), which cannot show this text."* That clause is the reason
/// [`StyleForecast`] carries `tried` alongside the outlook — the rung says what
/// will happen, and `tried` says what was rejected to get there, and an
/// operator staring at a title block full of `Times-Bold` wants the second.
///
/// The last row is not a fallback that should have been designed away. A probe
/// returns `None` for a page whose content cannot be planned, for an
/// `#[non_exhaustive]` rung this build has never seen — rung 3, the
/// `--font-dir` donor, is exactly that — and for an encrypted document; and in
/// every one of those the honest thing to say is the mechanism rather than a
/// prediction. That is what
/// [`crate::text::panels::properties::text_bold_hint`] already said, which is
/// why it stays.
///
/// # ★★ None of the seven greys the button, and that is still the engine's ruling
///
/// *"Do not grey out a bold button. Offer it, and surface the disclosure when
/// synthesis fires."* [`StyleOutlook::Declined`] is the row where greying could
/// now be argued — it is a **measured** prediction of a refusal, not a guess —
/// and it is still a sentence, because the thing that produces it is a setting
/// the operator can change. R9 reserves greying for the *temporarily*
/// unavailable and demands the reason on hover; this is the reason on hover,
/// and pressing it produces a refusal that names the same setting.
pub(super) fn bold_hint(draft: &TextStyleDraft) -> String {
    hint(draft.bold_outlook(), true)
}

/// The Italic button's hover text. See [`bold_hint`] for the whole argument;
/// this is the same seven rows with *slant* in place of *thicken*, from the
/// draft's separately-probed italic axis.
///
/// ★ It reads [`TextStyleDraft::italic_outlook`] and never the bold one. The
/// two are genuinely different answers on an ordinary page — one holding a real
/// `Arial-Bold` and no `Arial-Italic` gives `SiblingOnPage` for one button and
/// `Synthesized` for the other — and a shared sentence would be wrong on
/// exactly the pages an operator is most likely to be working on.
pub(super) fn italic_hint(draft: &TextStyleDraft) -> String {
    hint(draft.italic_outlook(), false)
}

/// Both buttons' hover text, from one forecast and one axis flag.
///
/// # ★★ One function for two axes, which is the opposite of what was here
///
/// The two hints used to be two `match`es over the same variants, differing
/// only in which catalog function each arm called. That shape survived three
/// variants; at seven it is fourteen arms that have to be kept in step by hand,
/// and the recorded defect of this project is a copy-paste that reads the wrong
/// axis' field. Taking `bold: bool` and selecting the catalog function at the
/// leaf makes the divergence impossible: there is one arm per outlook, and the
/// axis is a parameter rather than a duplicated body.
///
/// ★ The addendum is appended here rather than folded into each sentence,
/// because it is **orthogonal** — a ladder can pass over faces on its way to any
/// rung, including the one that ends in a refusal, and writing it into seven
/// sentences twice over is how a clause goes stale in six of fourteen places.
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
    ///
    /// A constructor rather than three field assignments after
    /// `Default::default()`, which is what clippy's `field_reassign_with_default`
    /// asks for and is better here anyway: what these tests vary is the pair of
    /// forecasts, and a helper that takes exactly the pair says so.
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

    /// ★★★ **Seven outcomes, seven different sentences**, per axis.
    ///
    /// If any two collapsed, the probe would be decoration: an operator whose
    /// page carries the bold form of their own typeface and one whose page
    /// carries a bold face from some other family would read the same words and
    /// learn nothing either way — and those two results look quite different on
    /// the page.
    ///
    /// ★★ The three face-bearing rows all carry **the same face name** on
    /// purpose. A test that varied the name as well as the variant would pass
    /// on a build whose sentences were identical apart from the interpolated
    /// string, which is exactly the bug this is here to catch: the distinction
    /// that matters is *what pdfcer will do*, not *which face it names*.
    ///
    /// ★ The `None` row is included deliberately — it is the sentence that was
    /// there before any preview existed, and it must remain distinguishable
    /// from the six predictions rather than being absorbed into one of them.
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

    /// ★★ **The two axes read their own probes**, and never each other's.
    ///
    /// The state this pins is the ordinary one, not an exotic one: a page
    /// carrying a real `Arial-Bold` and no `Arial-Italic` gives `SiblingOnPage`
    /// for one button and `Synthesized` for the other. A shared sentence — or a
    /// copy-paste that read `bold_outlook` in both helpers — would be wrong on
    /// exactly the pages an operator is most likely to be working on, and would
    /// be invisible on every page where the two answers happen to agree.
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

    /// ★ **Bold thickens and italic slants**, and neither sentence borrows the
    /// other's verb.
    ///
    /// They are different synthetic operations — a weight is the regular face
    /// stroked, a slant is the upright face sheared — and an operator who has
    /// read one should not have to guess that the other means something else.
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

    /// ★★★ **The two rung-1 sentences disagree about the letterforms**, which
    /// is the only thing the operator can act on.
    ///
    /// Both bind a real face already on the page and embed nothing, so a
    /// sentence that stopped at *"pdfcer will use a real bold face"* would be
    /// true of both and useful for neither. The difference is that one keeps
    /// the drawing's typeface and the other changes it visibly — the engine's
    /// own words for the second are *"a bigger change than a weight swap"*, and
    /// rule 4 makes the visible half the half that must be disclosed.
    ///
    /// ★ Asserted on the *shape* claim rather than on the whole sentence, so
    /// rewording the hover does not break the test while removing the
    /// distinction silently would.
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

    /// ★★★ **The refusal is predicted, names the cause, and does not prescribe
    /// a font.**
    ///
    /// `StylePolicy::Refuse` is the operator's own setting, so the honest
    /// sentence says which setting and stops. Naming the face chooser as a
    /// remedy would be this shell second-guessing pdfcer's font selection —
    /// decision 058's exact case — and telling them to pick a different font
    /// would be advice about a page the shell has not surveyed.
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

    /// ★★★ **A face the ladder will step over is named before the press**, with
    /// the character that defeated it.
    ///
    /// The engine discloses `passed_over` **after** the commit and this shell
    /// already surfaces that verbatim, so the value here is entirely in the
    /// timing: the same fact one gesture earlier, where it can still change what
    /// the operator does.
    ///
    /// ★ The addendum is checked on a rung that is **not** a refusal, because
    /// that is the case a naive design gets wrong — passing over a face and then
    /// succeeding is the ordinary path, not an error path.
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

    /// ★★ **A face passed over for no one character gets no empty parenthesis.**
    ///
    /// `Refusal::character` is an `Option` and a refusal about the whole run is
    /// a real case; *"Times-Bold ()"* would be this shell rendering an absence
    /// as a presence, which is the same defect class as a `{:?}` in an operator
    /// string.
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

    /// ★★ **Nothing passed over means no addendum at all.**
    ///
    /// The commonest case by far, and the one where an unconditional clause
    /// would read as a warning about nothing. Falsifies the previous two tests:
    /// without this, a build that appended the clause always would still pass
    /// them.
    #[test]
    fn an_empty_passed_over_list_says_nothing() {
        let line = bold_hint(&drafted(
            plain(StyleOutlook::StandardSibling("Helvetica-Bold".to_owned())),
            None,
        ));
        assert!(!line.contains("pass over"), "{line}");
    }

    /// ★★ **A fresh draft says the conditional**, not a prediction.
    ///
    /// `TextStyleDraft::default()` has never been synced, so both forecasts are
    /// `None` — and the honest thing to say about a run nothing has been read
    /// from is the mechanism, which is exactly what the hint said before any of
    /// this landed. A build that guessed `Synthesized` here would tell an
    /// operator their letters are about to be thickened on a page that carries
    /// a real bold face.
    #[test]
    fn an_unsynced_draft_promises_nothing() {
        let draft = drafted(None, None);
        assert_eq!(bold_hint(&draft), t::text_bold_hint());
        assert_eq!(italic_hint(&draft), t::text_italic_hint());
    }
}
