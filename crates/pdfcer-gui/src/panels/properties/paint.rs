//! # `panels::properties::paint` — **the colour of a selected path**
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

/// Draw the colour section. `true` if anything was drawn.
///
/// Returns `false` for a selection this section has nothing to say about —
/// several objects, an annotation, a form field — rather than drawing an empty
/// heading. `geometry::section` states the same rule and for the same reason: a
/// heading with nothing under it reads as a control that failed to load.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    if doc.selection.annot().is_some() {
        return false;
    }
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    let [object] = objects.as_slice() else {
        // ★ One object only, for now. A multi-object selection can be
        // recoloured — the verb takes a slice — but the CONTROL has no honest
        // starting value when the objects disagree, and opening a swatch on the
        // first one's colour would silently propose flattening the rest to it.
        // That wants a "mixed" state and its own decision.
        return false;
    };
    let object = *object;

    let Some(provider) = doc.page_objects() else {
        return false;
    };
    let Some(model) = provider.page_objects_model(page) else {
        return false;
    };
    let Some(pdfcer_core::vector::VectorObject::Path(path)) = model.objects.get(object) else {
        // Not a path. Text has its own colour control in the Text section, and
        // an image has no paint at all.
        return false;
    };
    let fill = path.fill_paint.clone();
    let stroke = path.stroke_paint.clone();
    drop(provider);

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    // ★ Plain, not `.strong()`. `check-strong-text` forbids it here and its
    // remedy is the right one: the emphasis is invisible against this panel's
    // background anyway, and the hierarchy is carried by this being the one
    // line in the group that is NOT `.small().weak()`.
    ui.label(t::heading());

    let mut change: Option<Recolour> = None;
    if let Some(rgb) = swatch(ui, &fill, t::fill_label(), REGION_FILL) {
        change = Some(Recolour {
            fill: Some(rgb),
            stroke: None,
        });
    }
    if let Some(rgb) = swatch(ui, &stroke, t::stroke_label(), REGION_STROKE) {
        change = Some(Recolour {
            fill: None,
            stroke: Some(rgb),
        });
    }

    if let Some(to) = change {
        actions.push(Action::SetObjectPaint {
            page,
            objects: vec![object],
            fill: to.fill,
            stroke: to.stroke,
        });
    }
    true
}

/// One channel: a swatch when the colour is knowable, its ink's name when not.
///
/// Returns the newly chosen colour, or `None` when nothing was changed this
/// frame.
fn swatch(ui: &mut Ui, paint: &PathPaint, label: String, region: &str) -> Option<[u8; 3]> {
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.label(label);
        match paint.rgb() {
            Some(rgb) => {
                // ★★ `Rgb` is three `f32` in 0..1, which is PDF's own unit;
                // `egui`'s swatch speaks 8-bit sRGB. The conversion happens
                // HERE, at the one boundary, rather than being carried through
                // the action — so the operand that reaches the engine is in the
                // engine's units and cannot be misread as bytes.
                let was = to_bytes(rgb);
                let mut current = was;
                let response = ui.color_edit_button_srgb(&mut current);
                crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
                if response.changed() && current != was {
                    chosen = Some(current);
                }
            }
            // ★★★ NO SWATCH. See the header: one opening on black over a spot
            // ink is one click from destroying a plate, and it would look right
            // while it happened.
            None => {
                let said = ui.label(egui::RichText::new(t::undecoded(ink_name(paint))).weak());
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
    use pdfcer_core::vector::Rgb;

    /// ★★★ **An undecodable paint must never yield a colour to open a swatch
    /// on.**
    ///
    /// The one assertion this module exists for. If `rgb()` ever answered
    /// `Some` for `Other`, this section would draw a swatch over a spot ink and
    /// the first click would convert it — invisibly, permanently, and looking
    /// entirely normal.
    #[test]
    fn a_spot_ink_offers_no_colour_to_edit() {
        let spot = PathPaint::Other {
            space: Some(b"PANTONE 300".to_vec()),
            comps: vec![1.0],
            pattern: false,
        };
        assert!(
            spot.rgb().is_none(),
            "an undecoded space has no RGB to show"
        );
        assert_eq!(ink_name(&spot).as_deref(), Some("PANTONE 300"));
    }

    /// ★★ `Default` and a chosen black both draw a swatch — they are the same
    /// PICTURE and different facts, and only the type keeps them apart.
    #[test]
    fn nobody_chose_and_somebody_chose_black_both_show_black() {
        assert_eq!(PathPaint::Default.rgb(), Some(Rgb::BLACK));
        let chosen = PathPaint::Device {
            space: pdfcer_core::vector::DevicePaintSpace::Gray,
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
}
