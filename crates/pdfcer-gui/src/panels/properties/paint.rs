//! # `panels::properties::paint` — **the colour of the selected path(s)**
//!
//! `OPERATOR_REQUESTS.md` **O89**, and the half that did not exist:
//!
//! > *"I don't see where I am able to edit the color of text, vectors, etc."*
//!
//! Text had a control and he could not find it. Vectors had none, because
//! `pdfcer-core` had no verb — every colour verb it owned worked on an
//! annotation or on text. `Pass 218.0`/`219.0` shipped
//! `EditSession::set_object_paint`, and this is the control.
//!
//! ## ★★★ The swatch shows the object's OWN colour, or refuses to show one
//!
//! `PathPaint` has three states and the difference between them is the whole
//! design of this section:
//!
//! | state | what it means | what is drawn |
//! |---|---|---|
//! | `Default` | §8.6.8 black — **nobody chose a colour** | a swatch, black |
//! | `Device { rgb, … }` | somebody chose this | a swatch, that colour |
//! | `Other { space, … }` | a space pdfcer does not decode — a spot ink, a pattern | **the ink's NAME, and no swatch** |
//!
//! ⇒ The request made the argument and the engine wrote it into the verb's
//! docs: *"a colour control with no current value is a control that silently
//! discards what was there the moment it is touched."* A swatch opening on
//! black over a `/Separation` stroke is exactly that — one click and a named
//! spot ink is screen colour, permanently, and it looked right while it
//! happened.
//!
//! ★★ `Default` and `Device`-holding-black are drawn the same and are **not**
//! the same fact. Only the first may be replaced without comment; the
//! distinction is kept because it costs nothing to keep and cannot be
//! recovered once collapsed.
//!
//! ## ★★ Why a spot ink is NAMED rather than converted
//!
//! Evaluating a tint transform lives in `pdfcer-render`, which `pdfcer-core`
//! cannot depend on. The engine declined to duplicate it — a second colour-space
//! implementation is the class of defect this whole Pass removed — and it is
//! also the better answer for this panel: *"this stroke is spot ink PANTONE
//! 300"* tells a drawing office more than a square of approximate blue.
//!
//! ## Fill and stroke are separate, and a refusal is per channel
//!
//! `None` leaves a channel alone. Recolouring the fill of an object whose
//! stroke is a spot ink is **not** blocked by the channel nobody touched — the
//! engine has a test for it, and this section relies on it rather than
//! pre-emptively greying a control that would have worked.
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! # ★★★ MORE THAN ONE OBJECT — built 2026-09-05, O89 piece 2
//! ═══════════════════════════════════════════════════════════════════════════
//!
//! This section drew for **exactly one** object until 2026-09-05, and said so:
//!
//! > *"One object at a time, for now. The engine will recolour a whole
//! > selection; the control does not offer it yet, because when the objects
//! > disagree there is no honest colour to open on and picking the first one's
//! > would quietly propose flattening the rest to it."*
//!
//! The danger in that sentence is real and the conclusion was wrong.
//! **Every editor in this product class has already solved it** — Illustrator,
//! Inkscape, Figma and Word all show an *indeterminate* control over a
//! disagreeing selection, and applying a value sets every member. This
//! project's standing rule is that the convergence of the product class **is**
//! the specification and an invented interaction is a defect even when it
//! works, so the mixed state is not one option among several; it is the answer.
//!
//! [`super::swatch`] is that control. It opens on nothing in particular (PDF
//! §8.6.8's default, never applied unless the operator moves the picker), it
//! reads as *no single value* using the marker this shell already writes for
//! one, and picking a colour applies it to the whole selection.
//!
//! ★ **One undo step for the whole gesture** — and that took the swatch being
//! hand-built rather than `ui.color_edit_button_srgb`. `egui`'s own colour
//! button marks itself changed on *every frame of a drag inside the picker*, so
//! acting on `.changed()` authors an edit per frame; [`super::swatch`]'s header
//! carries the measurement and the fix (commit when the picker closes). ★★ That
//! defect was present in the single-object control this section shipped with —
//! it is fixed by the same change, for the same reason, and nothing about it is
//! new to the multi-object path.
//!
//! ## ★★★ A MIXED SELECTION CONTAINING ONE SPOT INK — the decision, and why
//!
//! The choice was between refusing the whole apply by name and applying to the
//! process-colour members while reporting the spot ones off-canvas. **This
//! section applies and reports**, and the argument is four things rather than a
//! preference:
//!
//! 1. **The guard is held by the engine, structurally, not by this control.**
//!    `EditSession::set_object_paint` tests every object's paint on every
//!    channel it is asked to change and returns
//!    `PaintOutcome { changed, refused }` — a `/Separation` member is refused
//!    *by the verb*, whatever this panel draws. A shell-side blanket refusal
//!    would add nothing to the plate's safety and would remove a capability.
//!    ⇒ There is **no hole**: the single-select path's guard is *"do not offer
//!    a swatch whose only possible effect is destruction"*, and it still holds,
//!    because where every member is a named ink no swatch is drawn at all.
//! 2. **The operator ruled on this exact case**, and it is quoted in
//!    `crate::text::paint::recoloured_partly`'s own doc comment: *"a selection
//!    of twelve strokes where three are in a colour space pdfcer will not
//!    rewrite needs to say 'nine changed', not 'done'."* That is
//!    apply-and-report, in his words, about this shape.
//! 3. **Refusing would be unusable on the documents this program is for.** One
//!    spot-inked line inside a marquee of two hundred would block the gesture,
//!    and the remedy — find it and deselect it — is being asked of an operator
//!    who cannot see which line it is. A safe operation would have been traded
//!    for an impossible one.
//! 4. **The disclosure arrives BEFORE the gesture, not only after it.**
//!    `crate::text::paint::mixed_named_inks` names how many members carry an
//!    ink and what those inks are called, on the row, above the swatch. The
//!    status line's *"nine changed, three left alone"* is the confirmation, not
//!    the first news.
//!
//! ## Rule 4 — nothing here marks the canvas
//!
//! The recoloured objects render exactly as the saved file will render them.
//! Which members were skipped, which are named inks and whether they disagreed
//! is disclosed **off-canvas**: on this row, and in the status bar through
//! `crate::app::actions::disclosure`. No badge, no tint, no outline on the page.

use egui::Ui;
use pdfcer_core::vector::PathPaint;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::target::CanvasTargetProvider as _;
use crate::text::paint as t;

/// The fill swatch's rectangle, for a driven check.
const REGION_FILL: &str = "properties.paint.fill"; // ui-text-exempt: a trace region name
/// The stroke swatch's rectangle.
const REGION_STROKE: &str = "properties.paint.stroke"; // ui-text-exempt: a trace region name
/// The sentence drawn where a swatch cannot be.
const REGION_UNDECODED: &str = "properties.paint.undecoded"; // ui-text-exempt: a trace region name
/// The line naming how many objects the controls act on.
///
/// ★ Its own region since the multi-object state shipped, because *"the section
/// drew"* and *"the section told the operator how many things it is about to
/// change"* are two different claims and a driven check has to be able to
/// assert the second.
const REGION_SUBJECT: &str = "properties.paint.subject"; // ui-text-exempt: a trace region name
/// The line naming the members that carry an ink this control will not
/// overwrite, drawn **above** a swatch that will still apply to the rest.
const REGION_PARTIAL_INK: &str = "properties.paint.partial-ink"; // ui-text-exempt: a trace region name

/// **What one frame's interaction asked for**, per channel.
///
/// ★ A named pair rather than a tuple of two options, because the two positions
/// are not interchangeable and a tuple invites reading them the wrong way round
/// exactly once — after which the fill control recolours the line. Clippy asked
/// for the type; the naming is why it was worth asking.
struct Recolour {
    /// The new fill, or `None` to leave it alone.
    fill: Option<[u8; 3]>,
    /// The new stroke, or `None` to leave it alone.
    stroke: Option<[u8; 3]>,
}

/// **What one channel looks like across the whole selection.**
///
/// The value the control needs and the disclosure the row owes, computed
/// together because they come from one walk.
struct Channel {
    /// What the swatch shows, or `None` when **every** member of the selection
    /// carries an ink this control will not overwrite — in which case no
    /// control is drawn at all and the sentence stands in its place.
    value: Option<super::swatch::Value>,
    /// The names of the members' inks this control will not overwrite, in
    /// selection order, `None` where the space carries no name.
    ///
    /// Empty when every member can be recoloured. Non-empty *with* a
    /// [`Self::value`] is the partial case the module header argues.
    inks: Vec<Option<String>>,
    /// How many objects contributed to this channel at all.
    total: usize,
}

/// Draw the colour section. `true` if anything was drawn.
///
/// Returns `false` for a selection this section has nothing to say about — an
/// annotation, a form field, a selection with no path in it — rather than
/// drawing an empty heading. `geometry::section` states the same rule and for
/// the same reason: a heading with nothing under it reads as a control that
/// failed to load.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    if doc.selection.annot().is_some() {
        return false;
    }
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    if objects.is_empty() {
        return false;
    }

    // ★ Read every selected object's two paints in ONE borrow of the provider,
    // and drop it before anything is drawn. Holding a `Ref` across a `Ui`
    // closure is how a panel comes to panic on a re-entrant borrow, and the
    // single-object version of this function already took care to drop it.
    let mut paints: Vec<(PathPaint, PathPaint)> = Vec::new();
    let mut not_paths = 0_usize;
    {
        let Some(provider) = doc.page_objects() else {
            return false;
        };
        let Some(model) = provider.page_objects_model(page) else {
            return false;
        };
        for &object in &objects {
            match model.objects.get(object) {
                Some(pdfcer_core::vector::VectorObject::Path(path)) => {
                    paints.push((path.fill_paint.clone(), path.stroke_paint.clone()));
                }
                // Text has its own colour controls (`super::text` for a swept
                // range, `super::textobject` for a clicked shape) and an image
                // has no paint at all. Counted rather than ignored: a marquee
                // over a table catches lines AND labels, and an operator who
                // recolours it needs to be told the labels were not included.
                _ => not_paths += 1,
            }
        }
    }
    if paints.is_empty() {
        return false;
    }

    let fill = channel(paints.iter().map(|(f, _)| f));
    let stroke = channel(paints.iter().map(|(_, s)| s));

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    // ★ Plain, not `.strong()`. `check-strong-text` forbids it here and its
    // remedy is the right one: the emphasis is invisible against this panel's
    // background anyway, and the hierarchy is carried by this being the one
    // line in the group that is NOT `.small().weak()`.
    ui.label(t::heading());

    // ★★★ **What the controls are about to act on, before they are used.**
    // Drawn only for a real multi-selection: with one object the answer is on
    // screen already (it is outlined on the page and named by the Objects
    // section), and a line saying "1 shape" is noise that trains the eye to
    // skip the place the count lives.
    if paints.len() > 1 || not_paths > 0 {
        let said = ui.label(
            egui::RichText::new(t::subject(paints.len(), not_paths))
                .small()
                .weak(),
        );
        crate::diag::ui_rect_visible(REGION_SUBJECT, said.rect, ui.clip_rect());
    }

    let mut change: Option<Recolour> = None;
    if let Some(rgb) = row(ui, &fill, t::fill_label(), REGION_FILL, "paint-fill") {
        change = Some(Recolour {
            fill: Some(rgb),
            stroke: None,
        });
    }
    if let Some(rgb) = row(
        ui,
        &stroke,
        t::stroke_label(),
        REGION_STROKE,
        "paint-stroke",
    ) {
        change = Some(Recolour {
            fill: None,
            stroke: Some(rgb),
        });
    }

    if let Some(to) = change {
        actions.push(Action::SetObjectPaint {
            page,
            // ★ Every selected PATH, in the selection's own order. The
            // non-paths are not sent: the engine would refuse them by name
            // (`PaintRefusalReason::NotAPath`) and the refusal count in the
            // status line would then mix "not a shape" with "a named ink",
            // which are two different pieces of news and only one of them is
            // about the operator's plates.
            objects: path_indices(doc, page, &objects),
            fill: to.fill,
            stroke: to.stroke,
        });
    }
    true
}

/// The selected indices that are paths, in selection order.
///
/// ★ Re-derived rather than collected in the walk above, because the walk's
/// output is *paints* and pairing them with indices would make one `Vec` whose
/// two halves have to be kept in step by hand. The provider read is cheap (a
/// slice index per entry) and the alternative is the class of bug where a
/// filter and its operand drift.
fn path_indices(doc: &OpenDoc, page: usize, objects: &[usize]) -> Vec<usize> {
    let Some(provider) = doc.page_objects() else {
        return Vec::new();
    };
    let Some(model) = provider.page_objects_model(page) else {
        return Vec::new();
    };
    objects
        .iter()
        .copied()
        .filter(|&o| {
            matches!(
                model.objects.get(o),
                Some(pdfcer_core::vector::VectorObject::Path(_))
            )
        })
        .collect()
}

/// **Fold one channel of a whole selection into a control state.**
///
/// ★★★ The ink check comes **before** agreement, not after, and that ordering
/// is the guard. *"They all agree and one of them is a spot ink"* must never
/// draw a swatch: agreement between two members of a named-ink selection is not
/// permission to overwrite them.
///
/// ★★ A member whose paint cannot be shown is excluded from the agreement
/// question entirely rather than counted as a disagreement. It is not a colour
/// this control can compare, and folding it in would report *"mixed"* for a
/// selection of one red line and one PANTONE line — implying a value would
/// unify them, which is exactly what will not happen.
fn channel<'a>(paints: impl Iterator<Item = &'a PathPaint>) -> Channel {
    let mut inks: Vec<Option<String>> = Vec::new();
    let mut agreed: Option<[u8; 3]> = None;
    let mut mixed = false;
    let mut total = 0_usize;
    for paint in paints {
        total += 1;
        match paint.rgb() {
            None => inks.push(ink_name(paint)),
            Some(rgb) => {
                let rgb = to_bytes(rgb);
                match agreed {
                    None => agreed = Some(rgb),
                    Some(seen) if seen != rgb => mixed = true,
                    Some(_) => {}
                }
            }
        }
    }
    let value = match agreed {
        // Every member is an ink this control will not overwrite. No swatch —
        // the whole of O89's vector ruling, unchanged by the selection size.
        None => None,
        Some(_) if mixed => Some(super::swatch::Value::Mixed),
        Some(rgb) => Some(super::swatch::Value::Agreed(rgb)),
    };
    Channel { value, inks, total }
}

/// One channel's row: its label, its disclosure, and its control — or its
/// refusal, where no control may be drawn.
///
/// Returns the newly chosen colour, or `None` when nothing was committed this
/// frame. ★ *Committed*, not *changed*: [`super::swatch::show`] answers only on
/// the frame the picker closes, so one drag through a colour wheel is one
/// action and one undo entry.
fn row(
    ui: &mut Ui,
    channel: &Channel,
    label: String,
    region: &str,
    id_salt: &str,
) -> Option<[u8; 3]> {
    let mut chosen = None;
    // ★★★ The partial-ink disclosure sits ABOVE the control, on this project's
    // standing rule that *"a caveat below a list arrives after the operator has
    // already drawn a conclusion."* It is the sentence that makes pressing the
    // swatch an informed act rather than a surprise reported afterwards.
    if !channel.inks.is_empty() && channel.value.is_some() {
        let said = ui.label(
            egui::RichText::new(t::mixed_named_inks(&channel.inks, channel.total))
                .small()
                .weak(),
        );
        crate::diag::ui_rect_visible(REGION_PARTIAL_INK, said.rect, ui.clip_rect());
    }
    ui.horizontal(|ui| {
        ui.label(label);
        match channel.value {
            Some(value) => {
                // ★ The hint names THIS row's subject — shapes, not words. See
                // `super::swatch::show`'s note on why the widget takes it as a
                // parameter rather than reaching for one.
                chosen = super::swatch::show(ui, id_salt, value, region, &t::mixed_hint());
            }
            // ★★★ NO SWATCH. See the header: one opening on black over a spot
            // ink is one click from destroying a plate, and it would look right
            // while it happened.
            None => {
                let first = channel.inks.first().cloned().flatten();
                let said =
                    ui.label(egui::RichText::new(t::undecoded_across(first, channel.total)).weak());
                crate::diag::ui_rect_visible(REGION_UNDECODED, said.rect, ui.clip_rect());
            }
        }
    });
    chosen
}

/// The engine's 0..1 components as the swatch's 8-bit sRGB.
///
/// ★ Rounded rather than truncated. Truncation makes 1.0 into 255 correctly and
/// 0.5 into 127 — half a step dark on every mid-tone, which over a round trip
/// through the swatch would walk a colour steadily darker every time it was
/// opened and closed without being changed.
fn to_bytes(rgb: pdfcer_core::vector::Rgb) -> [u8; 3] {
    let f = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [f(rgb.r), f(rgb.g), f(rgb.b)]
}

/// The ink's name as the file states it, when there is one.
///
/// ★ Raw bytes, decoded loosely. A colour-space resource name is a PDF name
/// object and carries no declared encoding; showing it as it is beats showing
/// nothing, and beats a repaired version that no longer matches what the
/// operator would find in the file.
fn ink_name(paint: &PathPaint) -> Option<String> {
    match paint {
        PathPaint::Other { space, .. } => space
            .as_ref()
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::vector::{DevicePaintSpace, Rgb};

    fn spot(name: &str) -> PathPaint {
        PathPaint::Other {
            space: Some(name.as_bytes().to_vec()),
            comps: vec![1.0],
            pattern: false,
        }
    }

    fn rgb(r: f32, g: f32, b: f32) -> PathPaint {
        PathPaint::Device {
            space: DevicePaintSpace::Rgb,
            comps: vec![f64::from(r), f64::from(g), f64::from(b)],
            rgb: Rgb { r, g, b },
        }
    }

    /// ★★★ **An undecodable paint must never yield a colour to open a swatch
    /// on.**
    ///
    /// The one assertion this module exists for. If `rgb()` ever answered
    /// `Some` for `Other`, this section would draw a swatch over a spot ink and
    /// the first click would convert it — invisibly, permanently, and looking
    /// entirely normal.
    #[test]
    fn a_spot_ink_offers_no_colour_to_edit() {
        let ink = spot("PANTONE 300");
        assert!(ink.rgb().is_none(), "an undecoded space has no RGB to show");
        assert_eq!(ink_name(&ink).as_deref(), Some("PANTONE 300"));
    }

    /// ★★ `Default` and a chosen black both draw a swatch — they are the same
    /// PICTURE and different facts, and only the type keeps them apart.
    #[test]
    fn nobody_chose_and_somebody_chose_black_both_show_black() {
        assert_eq!(PathPaint::Default.rgb(), Some(Rgb::BLACK));
        let chosen = PathPaint::Device {
            space: DevicePaintSpace::Gray,
            comps: vec![0.0],
            rgb: Rgb::BLACK,
        };
        assert_eq!(chosen.rgb(), Some(Rgb::BLACK));
    }

    /// A pattern has no colour at all (§8.7.3) and must not be given one.
    #[test]
    fn a_pattern_offers_no_colour_either() {
        let pattern = PathPaint::Other {
            space: None,
            comps: Vec::new(),
            pattern: true,
        };
        assert!(pattern.rgb().is_none());
        assert!(
            ink_name(&pattern).is_none(),
            "an unnamed space names nothing"
        );
    }

    /// ★★★ **A selection that disagrees reads as MIXED and still offers a
    /// control.**
    ///
    /// The whole of O89 piece 2. ★ The fixture genuinely disagrees — red and
    /// green — because a fixture whose members all share one colour would pass
    /// against an implementation that simply showed the first one's.
    #[test]
    fn two_different_colours_read_as_mixed() {
        let ch = channel([rgb(1.0, 0.0, 0.0), rgb(0.0, 1.0, 0.0)].iter());
        assert_eq!(ch.value, Some(super::super::swatch::Value::Mixed));
        assert!(ch.inks.is_empty());
        assert_eq!(ch.total, 2);
    }

    /// Agreement across several members opens on the agreed colour, not on
    /// mixed. The counterpart of the test above: if `mixed` were set
    /// unconditionally the control would be indeterminate for every selection
    /// and the mixed test would still pass.
    #[test]
    fn a_selection_that_agrees_opens_on_that_colour() {
        let ch = channel([rgb(1.0, 0.0, 0.0), rgb(1.0, 0.0, 0.0)].iter());
        assert_eq!(
            ch.value,
            Some(super::super::swatch::Value::Agreed([255, 0, 0]))
        );
    }

    /// ★★★ **One spot ink among process colours keeps the control AND names
    /// the ink.**
    ///
    /// The decision the module header argues, asserted rather than left to the
    /// prose: the swatch survives (so the nine reachable strokes can be
    /// recoloured), and the ink is listed (so the operator knows before
    /// pressing that one of them will be left alone).
    #[test]
    fn one_spot_ink_among_process_colours_keeps_the_swatch_and_names_the_ink() {
        let ch = channel([rgb(1.0, 0.0, 0.0), rgb(1.0, 0.0, 0.0), spot("PANTONE 300")].iter());
        assert_eq!(
            ch.value,
            Some(super::super::swatch::Value::Agreed([255, 0, 0])),
            "the process-colour members still have a colour to open on"
        );
        assert_eq!(ch.inks, vec![Some("PANTONE 300".to_owned())]);
        assert_eq!(ch.total, 3);
    }

    /// ★★★ **Every member a named ink: no control at all.**
    ///
    /// The single-object guard, unchanged by the selection size. This is the
    /// case where a swatch's only possible effect is destruction, and it is the
    /// reason the partial case above is safe to allow: the two are different
    /// states and this test is what keeps them different.
    #[test]
    fn a_selection_of_named_inks_offers_no_swatch() {
        let ch = channel([spot("PANTONE 300"), spot("PANTONE 485")].iter());
        assert!(
            ch.value.is_none(),
            "a swatch over a selection of named inks is one click from a destroyed plate"
        );
        assert_eq!(ch.inks.len(), 2);
    }

    /// ★★ A spot ink must not be counted as a *disagreement*.
    ///
    /// If it were, one red line plus one PANTONE line would read as "mixed" —
    /// which tells the operator that picking a colour will unify them, and it
    /// will not. The honest reading is "red, and one ink I will leave alone".
    #[test]
    fn a_spot_ink_does_not_make_the_process_colours_look_mixed() {
        let ch = channel([rgb(1.0, 0.0, 0.0), spot("PANTONE 300")].iter());
        assert_eq!(
            ch.value,
            Some(super::super::swatch::Value::Agreed([255, 0, 0]))
        );
    }
}
