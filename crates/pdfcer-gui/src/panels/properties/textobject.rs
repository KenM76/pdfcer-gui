//! # `panels::properties::textobject` — the colour of the text you CLICKED
//!
//! `OPERATOR_REQUESTS.md` **O89**, piece 1:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/textobject.md`.

use egui::Ui;

use super::opacityrow::{self, Alpha};
use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::canvas::textedit::pin::{ObjectText, RunFill};
use crate::text::panels::textobject as t;
use pdfcer_gui_base::colourpick::{Ink, Pick, Shown, Spaces};

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
/// The opacity field's region; `{OPACITY_REGION}.unknown` where the file's
/// opacity cannot be read.
// ui-text-exempt: trace region name, never displayed
pub const OPACITY_REGION: &str = "properties.textobject.opacity";

/// What this object's text is painted in, as the control has to draw it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Colour {
    /// Every run that carries glyphs is this colour, in this space.
    Agreed(Ink),
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
    /// What the opacity row shows; `None` for an object with no show run.
    alpha: Option<Alpha>,
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
                    alpha: Alpha::of(found.alphas.iter().copied()),
                });
        }
        self.read.is_some()
    }
}

/// **How the object's per-run fills become one control state.**
fn classify(found: &ObjectText) -> Colour {
    let mut total = 0_usize;
    let mut affected = 0_usize;
    let mut agreed: Option<Ink> = None;
    let mut mixed = false;
    for fill in &found.fills {
        // An absent colour operator is the initial DeviceGray black (§8.6.8).
        let ink = match fill {
            RunFill::NoGlyphs => continue,
            RunFill::DefaultBlack => Some(Ink::Gray(0.0)),
            RunFill::Painted(colour) => Ink::from_text(*colour),
        };
        total += 1;
        match ink {
            None => affected += 1,
            Some(ink) => match agreed {
                None => agreed = Some(ink),
                Some(seen) if seen != ink => mixed = true,
                Some(_) => {}
            },
        }
    }
    if affected > 0 {
        return Colour::Ink { affected, total };
    }
    match agreed {
        Some(ink) if !mixed => Colour::Agreed(ink),
        // No glyph-bearing run at all reads as **mixed**, not as black. A
        // text object whose every string failed to decode has no colour this
        // shell may claim to have read, and `Agreed(black)` would be a claim.
        // The control still works: `format_text` acts on whatever operators are
        // really there, and the engine refuses by name if there are none.
        _ => Colour::Mixed,
    }
}

/// Draw the section, or nothing.
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

    let mut chosen: Option<Pick> = None;
    ui.horizontal(|ui| {
        ui.label(t::colour_label());
        let shown = match read.colour {
            // NO SWATCH. See the header: one opened over a `/Separation`
            // is a click away from a destroyed plate, and it would look
            // entirely normal while it happened.
            Colour::Ink { affected, total } => {
                let said = ui.label(egui::RichText::new(t::ink_present(affected, total)).weak());
                crate::diag::ui_rect_visible(INK_REGION, said.rect, ui.clip_rect());
                return;
            }
            // Opacity has its own row below: a picker carrying it would
            // commit ink and opacity together, so moving one would rewrite
            // the other across runs that disagree on it.
            Colour::Agreed(ink) => Shown::Agreed(Pick { ink, opacity: None }),
            Colour::Mixed => Shown::Mixed {
                with_opacity: false,
            },
        };
        chosen = pdfcer_gui_base::colourpick::show(
            ui,
            // ui-text-exempt: an egui id salt, never displayed
            "properties-textobject-colour",
            shown,
            Spaces::ALL,
            SWATCH_REGION,
            t::mixed_hint(),
        );
    });

    // The ink is written in the space it was picked in; `format_text` stores
    // the space it is given rather than converting.
    if let Some(fill) = chosen.and_then(|pick| pick.ink.fill()) {
        actions.push(Action::TextStyle {
            page,
            runs: read.runs.clone(),
            change: StyleChange::Fill(fill),
        });
    }
    if let Some(alpha) = read.alpha {
        crate::diag::trace_changed(OPACITY_REGION, || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "text-opacity-shown page={page} object={object} alpha={}",
                alpha.token()
            )
        });
        if let Some(a) = opacityrow::row(
            ui,
            crate::text::strokestyle::text_opacity_label(),
            alpha,
            OPACITY_REGION,
        ) {
            actions.push(Action::TextStyle {
                page,
                runs: read.runs.clone(),
                change: StyleChange::Opacity(a),
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
            alphas: Vec::new(),
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

    /// **CMYK is read in CMYK**, so the picker opens on the process values
    /// and writes them back unconverted.
    #[test]
    fn cmyk_is_offered_in_its_own_space() {
        let colour = classify(&reading(vec![RunFill::Painted(TextColor::Cmyk(
            0.0, 0.5, 0.0, 1.0,
        ))]));
        assert_eq!(colour, Colour::Agreed(Ink::Cmyk([0.0, 0.5, 0.0, 1.0])));
    }

    /// The same black in two spaces disagrees: one pick writes one space.
    #[test]
    fn one_colour_in_two_spaces_is_mixed() {
        let colour = classify(&reading(vec![
            RunFill::Painted(TextColor::Cmyk(0.0, 0.0, 0.0, 1.0)),
            RunFill::DefaultBlack,
        ]));
        assert_eq!(colour, Colour::Mixed);
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
        assert_eq!(colour, Colour::Agreed(Ink::Gray(0.0)));
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

    /// Grey is read as grey, not widened to RGB.
    #[test]
    fn gray_is_offered_in_its_own_space() {
        let colour = classify(&reading(vec![RunFill::Painted(TextColor::Gray(1.0))]));
        assert_eq!(colour, Colour::Agreed(Ink::Gray(1.0)));
    }
}
