//! # `panels::properties::textobject` — the colour of the text you CLICKED
//!
//! `OPERATOR_REQUESTS.md` **O89**, piece 1:
//!
//! > *"I don't see where I am able to edit the color of text, vectors, etc."*
//!
//! ## What was wrong, in one paragraph
//!
//! Text colour shipped in two places — Format ▸ Font ▸ Colour, and Properties ▸
//! Text — and **both were gated on a swept text range**. Clicking a piece of
//! text with the Select tool selects the *object*, so both controls stayed
//! greyed, and the way to un-grey them (press `T`, sweep across the words) is
//! not guessable from anything on screen. The capability shipped; the way in
//! did not. This project's standing reading of that shape is that a request for
//! something already shipped is a **discoverability report**.
//!
//! O89 listed three candidates and picked none. This module is the first —
//! *"a colour control on a selected text object that sweeps it for you"*, which
//! O89 itself called *"closest to what you tried"*.
//!
//! ★★★ **The other two were already built**, which was measured rather than
//! assumed and is recorded in O89 in place of the sentences that said
//! otherwise. The Properties panel's *"press T and sweep"* sentence has existed
//! since 2026-08-29 (`super::text`'s `route`, whose sentence is
//! `crate::text::panels::properties::text_object_route` and now lives on this
//! section), and every one of the five Font commands' tooltips has ended
//! *"Sweeping text with the Text tool (T) chooses what it applies to"* since
//! the group shipped. What was NOT true was O89's third row — *"the greyed
//! button saying so on hover"* — for exactly **one** of the five controls: the
//! ribbon's Colour swatch answered a greyed hover with the CMYK-and-spot-ink
//! sentence, a claim about text it had not read. Fixed in `app::fontband`.
//!
//! ## ★★★ THE OPERAND IS THE OBJECT'S OWN BYTE SPAN, NOT A GUESS AT GEOMETRY
//!
//! [`super::text`]'s header states, correctly, that the object selection and
//! the text selection are unrelated index spaces and that an inference between
//! them *"restyles text the operator did not select, silently, in a file they
//! then send to somebody."* That paragraph was written about a **bounding-box
//! overlap**, and it is still right about one.
//!
//! This is not one. `pdfcer_core::vector::TextObject` carries the `BT`…`ET`
//! **byte span** in the decoded content buffer, and every glyph's
//! `GlyphProvenance` carries its show operator's byte span *in the same
//! buffer*, with the buffer named beside it. Membership is byte-range
//! containment: exact, total, and the same kind of fact the pinned edit path
//! already stakes every restyle on. `crate::canvas::textedit::pin::object_text`
//! is the join and its header carries the argument.
//!
//! ⇒ **No new selection unit was invented.** The operand handed to the engine
//! is a list of extraction run ordinals — the same operand a hand sweep
//! produces, through the same `Action::TextStyle`, into the same
//! `EditSession::format_text` calls. This module chooses *which* runs; it
//! changes nothing about what a restyle is.
//!
//! ## ★★ Why the range and not the exact set — "sweeps it for you", literally
//!
//! The operand is `first_run..=last_run`, which is precisely what
//! `TextSelection::runs` produces for a hand sweep (`(start.run..=end.run)`).
//! Taking the inclusive range rather than the exact membership set makes the
//! two routes **the same gesture with the same operand**, rather than two
//! things that usually agree and diverge on a document nobody tested. They can
//! differ only where another object's show operators interleave inside this
//! one's `BT`…`ET`, which §9.4's grammar forbids.
//!
//! ## ★★★ Why COLOUR and not the other four Font controls
//!
//! Face, size, bold and italic are **not** offered on an object selection, and
//! that is a decision rather than an omission. Each of the four needs a
//! *reading of one run* to be honest — which typeface, what size, whether a
//! real bold face covers *these characters* — and a whole text object has no
//! single answer to any of them. `EditSession::preview_style_resolution` and
//! `preview_font_resources` are both per-run by construction, and their own
//! invariants forbid a shell from re-deriving a page-level answer.
//!
//! Colour is the one property where *"they disagree"* is itself a displayable
//! answer, because the whole product class already has a control that says it:
//! the indeterminate swatch. So colour gets a working control here and the
//! other four get `crate::text::panels::properties::text_object_route`, which
//! was re-aimed in place to name the four things it is the route *to* rather
//! than claiming nothing about these words can be changed.
//!
//! ## ★★★ The spot-ink guard survives the object route
//!
//! `pdfcer_core::text_extract::TextColor::Other` means *"set in a colour space
//! this extraction does not decode"* — a `/Separation`, a `/DeviceN`, an
//! `/ICCBased`. O89's ruling for paths applies here word for word: *"a colour
//! picker that opened on black over a spot ink would be one click from
//! destroying a plate, and it would look completely normal while it
//! happened."*
//!
//! So an object **any** of whose runs is painted in a space
//! [`super::text::rgb_of`] will not round-trip gets **no swatch at all** — the
//! sentence [`t::ink_present`] stands where it would have been, and it names
//! how many of how many runs are affected. Two properties of that rule matter:
//!
//! * It is **all-or-nothing for the object**, because the operand is the
//!   object. A partial apply would need the operator to have asked for a
//!   partial thing, and they asked for *this shape*.
//! * The refusal is decided by the **same function** the swept-text swatch uses
//!   to decide whether to show a colour at all, so the two surfaces cannot
//!   disagree about which spaces are safe. That is why `rgb_of` was widened to
//!   `pub(super)` rather than copied.
//!
//! ★ Note what this does NOT do: it does not name the ink.
//! `TextColor::Other` is a fieldless variant and carries no `/Separation` name,
//! unlike `PathPaint::Other`. [`t::ink_present`]'s doc comment carries that
//! distinction; a sentence naming a spot colour here would be invented.
//!
//! ## ★★ The canvas selection is NOT changed by a press, and that is on purpose
//!
//! An early design had this control set `doc.text_selection` to the object's
//! runs after applying, so the operator would end the gesture with the words
//! visibly swept and the whole Font group live — teaching the route by doing
//! it. It is not built, for a reason found by reading the existing code rather
//! than by taste: **a restyle bumps `edit_epoch`, and a `TextSelection` records
//! the epoch it was resolved against**, so the selection would be stale on the
//! very next frame and the group would grey itself immediately. That is already
//! what happens after a swept-text restyle, and `app::conditions`' note on
//! `selection.text` argues it is the honest behaviour. Producing a selection
//! that is dead on arrival would have looked like a bug in the feature.
//!
//! ## ★★ The cost, and where it is paid
//!
//! [`TextObjectDraft::sync`] runs one page extraction with provenance capture
//! on — **392 ms on the operator's benchmark sheet** — behind a
//! `(page, object, edit epoch)` stamp. It is therefore paid **once per object
//! the operator clicks**, and only while this section is actually drawn: a
//! docked pane behind another tab draws nothing, so a panel the operator is not
//! looking at costs nothing at all.
//!
//! That is the same trade [`super::text::TextStyleDraft`] already makes on
//! every text-selection change, for the same measured reason, and it is stated
//! here rather than discovered: the alternative — matching the object's box
//! against the runs' boxes, which is free — is the geometric inference the
//! header above refuses.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas. The recoloured text renders exactly as the
//! saved file will render it; what was skipped, what disagreed and what is a
//! named ink is disclosed **off-canvas**, in this panel and in the status bar
//! through `app::actions::disclosure`.

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
/// ★ Published separately from [`REGION`] for the reason
/// [`super::text::BOLD_REGION`] gives: a check that computed a control's
/// position from a section's bounds is a check that passes on a build where the
/// controls moved.
// ui-text-exempt: trace region name, never displayed
pub const SWATCH_REGION: &str = "properties.textobject.swatch";
/// The region of the sentence drawn **instead of** a swatch over an ink pdfcer
/// will not overwrite.
///
/// ★ Its own name, because *"the section said something about this text"* must
/// not pass in the state where what it said is *"there is no control here"*.
/// The same argument [`super::text::ROUTE_REGION`] makes about its own state.
// ui-text-exempt: trace region name, never displayed
pub const INK_REGION: &str = "properties.textobject.ink";
/// The region of the route sentence — the way to the other four Font controls.
///
/// ★★★ **Spelled `properties.text.route`, which is `super::text`'s old name,
/// and that is deliberate.** The sentence moved from that module to this one
/// when the object state gained a working control; the *surface* did not move,
/// and `tools/ui-verify/src/checks/font_group.rs` finds it by this name. A
/// rename would have been a harness break dressed as tidiness — and a driven
/// check that stops finding a region reports the feature as missing, which is
/// the exact wrong story to tell about the thing O89 asked for.
// ui-text-exempt: trace region name, never displayed
pub const ROUTE_REGION: &str = "properties.text.route";

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
    ///
    /// ★ The expensive call is behind the stamp comparison and nothing else, so
    /// the ordinary frame — the operator looking at a selection they made three
    /// seconds ago — costs one tuple comparison.
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
///
/// Its own function, and every branch is a decision O89 argued:
///
/// 1. **A glyphless run is not a colour.** It has no show operator, so
///    `textstyle::apply` skips it; counting it would let a derived word space
///    make an object "mixed".
/// 2. **An ink pdfcer will not convert wins over everything.** It is checked
///    before agreement, not after, because *"they all agree and one of them is a
///    spot ink"* must not draw a swatch. The check is
///    [`super::text::rgb_of`] — the same predicate the swept-text swatch uses —
///    so the two surfaces cannot disagree about which spaces are safe.
/// 3. **`DefaultBlack` is black**, not "no opinion". §8.6.8 says an absent
///    colour operator paints black, so an object of one red run and one
///    default-black run is genuinely **mixed** and must say so. See
///    [`RunFill`]'s own docs for the flattening that collapsing this would
///    cause.
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
        // ★ No glyph-bearing run at all reads as **mixed**, not as black. A
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
/// # ★ The four gates, in the order they are cheapest
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
    // A swept range takes precedence: `super::text::section` draws the full
    // five-control editor for it, and drawing both would put two Colour
    // controls with different operands one above the other.
    if doc
        .text_selection
        .as_ref()
        .is_some_and(|s| s.live(doc.edit_epoch))
    {
        return false;
    }
    if doc.selection.annot().is_some() {
        return false;
    }
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    let [object] = objects.as_slice() else {
        return false;
    };
    let object = *object;
    if !is_text(doc, page, object) {
        return false;
    }
    if !draft.sync(doc, page, object) {
        // Text by kind, and nothing could be read from it — a page whose fonts
        // will not decode. Saying nothing here would be the silently-missing
        // control defect this section exists to end, so the route sentence
        // still stands and the colour row does not.
        ui.heading(t::heading());
        route(ui);
        crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
        ui.separator();
        return true;
    }
    let Some(read) = draft.read.as_ref() else {
        return false;
    };

    ui.heading(t::heading());
    ui.label(t::covers(read.runs.len()));

    let mut chosen: Option<[u8; 3]> = None;
    ui.horizontal(|ui| {
        ui.label(t::colour_label());
        match read.colour {
            // ★★★ NO SWATCH. See the header: one opened over a `/Separation`
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

    route(ui);
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    ui.separator();
    true
}

/// The route to the four controls this section does not offer.
///
/// ★ Drawn in **both** states — with a working swatch and without one — because
/// it is not the colour control's excuse. It is the answer to *"how do I change
/// the font"*, which is a live question the moment an operator has found the
/// colour and wants the rest.
fn route(ui: &mut Ui) {
    let said = ui.label(
        egui::RichText::new(crate::text::panels::properties::text_object_route())
            .small()
            .weak(),
    );
    crate::diag::ui_rect_visible(ROUTE_REGION, said.rect, ui.clip_rect());
}

/// Is the selected object text?
///
/// ★ Asked through `panels::objects::summary::object_kind`, which is the same
/// classification the Objects panel row and the read-only object section use,
/// so what this section calls text and what the panel beside it calls text
/// cannot disagree. [`super::text::route`] asked it the same way, and this is
/// that function's body: the gate moved here with the section it gates.
fn is_text(doc: &OpenDoc, page: usize, object: usize) -> bool {
    use crate::canvas::target::CanvasTargetProvider as _;
    doc.page_objects().is_some_and(|provider| {
        provider
            .page_objects_model(page)
            .and_then(|model| model.objects.get(object))
            .is_some_and(|o| {
                crate::panels::objects::summary::object_kind(o)
                    == crate::panels::objects::summary::ObjectKind::Text
            })
    })
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

    /// ★★★ **A spot ink anywhere in the object removes the swatch**, even when
    /// every other run agrees.
    ///
    /// The assertion this module exists for. If the ink check ran *after* the
    /// agreement check, an object of nine black runs and one `/Separation` run
    /// would draw a black swatch, and one click would convert the plate colour
    /// — invisibly, permanently, looking entirely normal.
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

    /// ★★★ **CMYK counts as an ink that must not be flattened**, not as a
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

    /// ★★★ **An absent colour operator is BLACK and therefore disagrees with
    /// red.**
    ///
    /// The [`RunFill`] distinction, asserted where it is consumed. Written as
    /// its own test because the failure it guards has no symptom: the control
    /// would open on red, and pressing nothing would change nothing, so only a
    /// deliberate check can see it.
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
