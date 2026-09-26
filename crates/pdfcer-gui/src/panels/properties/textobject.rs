//! # `panels::properties::textobject` — the colour of the text you CLICKED
//!
//! `OPERATOR_REQUESTS.md` **O89**, piece 1:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/textobject.md`.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::canvas::textedit::pin::{ObjectText, RunFill};
use crate::text::panels::textobject as t;

/// The section's own trace region, so a driven check can find it on screen.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.textobject";
/// The colour swatch's own region.
///
/// Published separately from [`REGION`] for the reason
/// [`super::text::BOLD_REGION`] gives: a check that computed a control's
/// position from a section's bounds is a check that passes on a build where the
/// controls moved.
// ui-text-exempt: trace region name, never displayed
pub const SWATCH_REGION: &str = "properties.textobject.swatch";
/// The region of the sentence drawn **instead of** a swatch over an ink pdfcer
/// will not overwrite.
///
/// Its own name, because *"the section said something about this text"* must
/// not pass in the state where what it said is *"there is no control here"*.
/// The same argument every other region here makes about its own state.
// ui-text-exempt: trace region name, never displayed
pub const INK_REGION: &str = "properties.textobject.ink";
//
// It read *"To change the font, size, bold or italic of these words, press T
// for the Text tool and sweep across them"*, and `OPERATOR_REQUESTS.md` O198
// made every clause of it false: face, size, bold and italic now act on the
// clicked object through `app::textoperand`, from this panel and from the
// ribbon band alike. A disclosure has a subject, and when the fix removes the
// subject the disclosure goes with it — leaving it in place would have been
// the application telling the operator to go and do something it had just
// done for him.
//
// ⚠ `tools/ui-verify/src/checks/font_group.rs` found the surface by the
// string `properties.text.route` and was updated in the same commit. A driven
// check left aiming at a deleted region reports the feature as missing.

/// What this object's text is painted in, as the control has to draw it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Colour {
    /// Every run that carries glyphs is this colour.
    Agreed([u8; 3]),
    /// They disagree. A swatch is still offered and applies to all of them.
    Mixed,
    /// `affected` of `total` glyph-bearing runs are painted in a space this
    /// shell will not overwrite with a screen colour, so **no swatch is drawn**.
    Ink {
        /// How many runs carry an ink pdfcer declines to convert.
        affected: usize,
        /// How many runs carry glyphs at all.
        total: usize,
    },
}

/// The object's text, read once and re-read only when it can have changed.
///
/// The stamp is three parts and every one is load-bearing, exactly as
/// [`super::text::TextStyleDraft`]'s is:
///
/// * **page** — an object index means nothing without one;
/// * **object** — the operator clicked a different shape;
/// * **edit epoch** — the same shape, restyled, and the swatch must show the
///   new colour. Without this term the panel would show the pre-edit colour for
///   ever after the first change.
#[derive(Default)]
pub struct TextObjectDraft {
    /// `(page, object, edit epoch)` the reading below was taken at.
    stamp: Option<(usize, usize, u64)>,
    /// The reading, or `None` when this object could not be read as text.
    read: Option<Reading>,
}

/// One object's reading: what to act on, and what to draw.
#[derive(Debug, Clone, PartialEq)]
struct Reading {
    /// The runs, ascending — the operand handed to `Action::TextStyle`.
    runs: Vec<usize>,
    /// What the swatch shows.
    colour: Colour,
}

impl TextObjectDraft {
    /// Re-read if the stamp moved. `true` when a reading is available.
    fn sync(&mut self, doc: &OpenDoc, page: usize, object: usize) -> bool {
        let stamp = (page, object, doc.edit_epoch);
        if self.stamp != Some(stamp) {
            self.stamp = Some(stamp);
            self.read =
                crate::canvas::textedit::pin::object_text(doc, page, object).map(|found| Reading {
                    runs: (found.first_run..=found.last_run).collect(),
                    colour: classify(&found),
                });
        }
        self.read.is_some()
    }
}

/// **How the object's per-run fills become one control state.**
fn classify(found: &ObjectText) -> Colour {
    let mut total = 0_usize;
    let mut affected = 0_usize;
    let mut agreed: Option<[u8; 3]> = None;
    let mut mixed = false;
    for fill in &found.fills {
        let rgb = match fill {
            RunFill::NoGlyphs => continue,
            RunFill::DefaultBlack => Some([0, 0, 0]),
            RunFill::Painted(colour) => super::text::rgb_of(*colour),
        };
        total += 1;
        match rgb {
            None => affected += 1,
            Some(rgb) => match agreed {
                None => agreed = Some(rgb),
                Some(seen) if seen != rgb => mixed = true,
                Some(_) => {}
            },
        }
    }
    if affected > 0 {
        return Colour::Ink { affected, total };
    }
    match agreed {
        Some(rgb) if !mixed => Colour::Agreed(rgb),
        // No glyph-bearing run at all reads as **mixed**, not as black. A
        // text object whose every string failed to decode has no colour this
        // shell may claim to have read, and `Agreed(black)` would be a claim.
        // The control still works: `format_text` acts on whatever operators are
        // really there, and the engine refuses by name if there are none.
        _ => Colour::Mixed,
    }
}

/// Draw the section, or nothing.
///
/// Returns whether it drew, so [`super::body_sections`] knows the panel has
/// said something about the selection.
///
/// # The four gates, in the order they are cheapest
///
/// An annotation is not page text; more than one object has no single subject
/// (the rule [`super::geometry::section`] states and this shares); an object
/// that is not text has nothing to say here; and only then is the expensive
/// reading attempted. Every one of the first three is free.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &mut TextObjectDraft,
    actions: &mut Vec<Action>,
) -> bool {
    //
    // A **swept** operand means the section above owns the whole editor
    // including its Colour row, so this one stands down. Drawing both would put
    // two Colour controls with different operands one above the other, which is
    // a way to recolour the wrong thing while looking straight at it.
    let Some((page, object)) = crate::app::textoperand::selected_text_object(doc) else {
        return false;
    };
    if doc
        .text_selection
        .as_ref()
        .is_some_and(|s| s.live(doc.edit_epoch))
    {
        return false;
    }
    if !draft.sync(doc, page, object) {
        // Text by kind, and nothing could be read from it — a page whose fonts
        // will not decode. `super::text::section` above has already drawn the
        // heading and the one sentence that says so, because its own read of
        // the same object failed for the same reason; a second heading here
        // would be the panel saying it twice.
        return false;
    }
    let Some(read) = draft.read.as_ref() else {
        return false;
    };

    let mut chosen: Option<[u8; 3]> = None;
    ui.horizontal(|ui| {
        ui.label(t::colour_label());
        match read.colour {
            // NO SWATCH. See the header: one opened over a `/Separation`
            // is a click away from a destroyed plate, and it would look
            // entirely normal while it happened.
            Colour::Ink { affected, total } => {
                let said = ui.label(egui::RichText::new(t::ink_present(affected, total)).weak());
                crate::diag::ui_rect_visible(INK_REGION, said.rect, ui.clip_rect());
            }
            Colour::Agreed(rgb) => {
                chosen = super::swatch::show(
                    ui,
                    // ui-text-exempt: an egui id salt, never displayed
                    "properties-textobject-colour",
                    super::swatch::Value::Agreed(rgb),
                    SWATCH_REGION,
                    t::mixed_hint(),
                );
            }
            Colour::Mixed => {
                chosen = super::swatch::show(
                    ui,
                    // ui-text-exempt: an egui id salt, never displayed
                    "properties-textobject-colour",
                    super::swatch::Value::Mixed,
                    SWATCH_REGION,
                    t::mixed_hint(),
                );
            }
        }
    });

    if let Some(rgb) = chosen {
        // The same `NewFill` the swept-text control builds, so the two routes
        // reach `format_text` with an identical operand shape. `FillModel::Rgb`
        // because the operator picked in sRGB; the engine stores the space it
        // is given rather than force-converting, which is the care this control
        // must not undo.
        let components = vec![
            f64::from(rgb[0]) / 255.0,
            f64::from(rgb[1]) / 255.0,
            f64::from(rgb[2]) / 255.0,
        ];
        if let Ok(fill) =
            pdfcer_core::text_edit::NewFill::new(pdfcer_core::text_edit::FillModel::Rgb, components)
        {
            actions.push(Action::TextStyle {
                page,
                runs: read.runs.clone(),
                change: StyleChange::Fill(fill),
            });
        }
    }

    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    // The separator for the WHOLE text block, this section and the editor
    // above it. `super::text::section` draws its own only when it owns the
    // Colour row, which is exactly when this section did not draw at all.
    ui.separator();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::text_extract::TextColor;

    fn reading(fills: Vec<RunFill>) -> ObjectText {
        ObjectText {
            first_run: 0,
            last_run: fills.len().saturating_sub(1),
            fills,
        }
    }

    /// **A spot ink anywhere in the object removes the swatch**, even when
    /// every other run agrees.
    #[test]
    fn one_undecodable_run_removes_the_swatch_for_the_whole_object() {
        let colour = classify(&reading(vec![
            RunFill::DefaultBlack,
            RunFill::DefaultBlack,
            RunFill::Painted(TextColor::Other),
        ]));
        assert_eq!(
            colour,
            Colour::Ink {
                affected: 1,
                total: 3
            }
        );
    }

    /// **CMYK counts as an ink that must not be flattened**, not as a
    /// colour that disagrees.
    ///
    /// A mixed swatch over CMYK would still apply sRGB to it. The whole point
    /// of the refusal is that pdfcer stores the space it was given.
    #[test]
    fn cmyk_is_a_refusal_and_not_a_disagreement() {
        let colour = classify(&reading(vec![
            RunFill::Painted(TextColor::Cmyk(0.0, 0.0, 0.0, 1.0)),
            RunFill::Painted(TextColor::Rgb(1.0, 0.0, 0.0)),
        ]));
        assert!(matches!(
            colour,
            Colour::Ink {
                affected: 1,
                total: 2
            }
        ));
    }

    /// **An absent colour operator is BLACK and therefore disagrees with
    /// red.**
    #[test]
    fn a_default_black_run_disagrees_with_a_coloured_one() {
        let colour = classify(&reading(vec![
            RunFill::Painted(TextColor::Rgb(1.0, 0.0, 0.0)),
            RunFill::DefaultBlack,
        ]));
        assert_eq!(colour, Colour::Mixed);
    }

    /// A glyphless run — a derived word space — must not make an object mixed.
    #[test]
    fn a_derived_space_between_two_black_runs_is_not_a_disagreement() {
        let colour = classify(&reading(vec![
            RunFill::DefaultBlack,
            RunFill::NoGlyphs,
            RunFill::DefaultBlack,
        ]));
        assert_eq!(colour, Colour::Agreed([0, 0, 0]));
    }

    /// Two different explicit colours are mixed, and mixed carries no colour.
    #[test]
    fn two_colours_read_as_mixed() {
        let colour = classify(&reading(vec![
            RunFill::Painted(TextColor::Rgb(1.0, 0.0, 0.0)),
            RunFill::Painted(TextColor::Rgb(0.0, 1.0, 0.0)),
        ]));
        assert_eq!(colour, Colour::Mixed);
    }

    /// Gray round-trips, so a gray object gets a swatch — the same reading
    /// `super::text::rgb_of` gives the swept-text control, asserted here so the
    /// two surfaces cannot drift apart silently.
    #[test]
    fn gray_is_offered_because_it_round_trips() {
        let colour = classify(&reading(vec![RunFill::Painted(TextColor::Gray(1.0))]));
        assert_eq!(colour, Colour::Agreed([255, 255, 255]));
    }
}
