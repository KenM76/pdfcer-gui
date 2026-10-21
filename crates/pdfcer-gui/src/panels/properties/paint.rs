//! # `panels::properties::paint` — **the colour of the selected path(s)**
//!
//! `OPERATOR_REQUESTS.md` **O89**, and the half that did not exist:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/paint.md`.

use egui::Ui;
use pdfcer_core::annot_author::Color;
use pdfcer_core::vector::{DevicePaintSpace, PathPaint};
use pdfcer_gui_base::colourpick::{self, Ink, Pick, Shown, Spaces};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::target::CanvasTargetProvider as _;
use crate::text::paint as t;

/// The trace slot of the section's `paint-shown` line.
const REGION_PAINT: &str = "properties.paint"; // ui-text-exempt: a trace region name
/// The fill swatch's rectangle, for a driven check.
const REGION_FILL: &str = "properties.paint.fill"; // ui-text-exempt: a trace region name
/// The stroke swatch's rectangle.
const REGION_STROKE: &str = "properties.paint.stroke"; // ui-text-exempt: a trace region name
/// The sentence drawn where a swatch cannot be.
const REGION_UNDECODED: &str = "properties.paint.undecoded"; // ui-text-exempt: a trace region name
/// The line naming how many objects the controls act on.
const REGION_SUBJECT: &str = "properties.paint.subject"; // ui-text-exempt: a trace region name
/// The line naming the members that carry an ink this control will not
/// overwrite, drawn **above** a swatch that will still apply to the rest.
const REGION_PARTIAL_INK: &str = "properties.paint.partial-ink"; // ui-text-exempt: a trace region name

/// **What one frame's interaction asked for**, per channel.
struct Recolour {
    /// The new fill, or `None` to leave it alone.
    fill: Option<Color>,
    /// The new stroke, or `None` to leave it alone.
    stroke: Option<Color>,
}

/// **What one channel looks like across the whole selection.**
///
/// The value the control needs and the disclosure the row owes, computed
/// together because they come from one walk.
struct Channel {
    /// What the swatch shows, or `None` when **every** member of the selection
    /// carries an ink this control will not overwrite — in which case no
    /// control is drawn at all and the sentence stands in its place.
    value: Option<Shown>,
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
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    if doc.selection.annot().is_some() {
        return false;
    }
    let page = doc.view.page_index;
    let Some((objects, leaves)) = super::strokestyle::selected(doc, page) else {
        return false;
    };

    // Read every selected object's two paints in ONE borrow of the provider,
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
            match crate::app::actions::styled_object(model, object, leaves) {
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
    trace(leaves, paints.len(), &fill, &stroke);

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    // Plain, not `.strong()`. `check-strong-text` forbids it here and its
    // remedy is the right one: the emphasis is invisible against this panel's
    // background anyway, and the hierarchy is carried by this being the one
    // line in the group that is NOT `.small().weak()`.
    ui.label(t::heading());

    // **What the controls are about to act on, before they are used.**
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
    if let Some(ink) = row(ui, &fill, t::fill_label(), REGION_FILL, "paint-fill") {
        change = Some(Recolour {
            fill: Some(ink.into()),
            stroke: None,
        });
    }
    if let Some(ink) = row(
        ui,
        &stroke,
        t::stroke_label(),
        REGION_STROKE,
        "paint-stroke",
    ) {
        change = Some(Recolour {
            fill: None,
            stroke: Some(ink.into()),
        });
    }

    if let Some(to) = change {
        actions.push(Action::SetObjectPaint {
            page,
            // Every selected PATH, in the selection's own order. The
            // non-paths are not sent: the engine would refuse them by name
            // (`PaintRefusalReason::NotAPath`) and the refusal count in the
            // status line would then mix "not a shape" with "a named ink",
            // which are two different pieces of news and only one of them is
            // about the operator's plates.
            objects: path_indices(doc, page, &objects, leaves),
            leaves,
            fill: to.fill,
            stroke: to.stroke,
        });
    }
    true
}

/// One line per change of what the section shows, for a driven check:
/// `fill=`/`stroke=` are the screen `r,g,b`, `mixed` or `ink`;
/// `fill_space=`/`stroke_space=` the agreed colour's stored space, else `-`.
fn trace(leaves: bool, paths: usize, fill: &Channel, stroke: &Channel) {
    let token = |c: &Channel| match c.value {
        Some(Shown::Agreed(pick)) => {
            let rgb = pick.ink.screen();
            format!("{},{},{}", rgb.r(), rgb.g(), rgb.b())
        }
        Some(Shown::Mixed { .. }) => "mixed".to_owned(), // ui-text-exempt: trace token
        None => "ink".to_owned(),                        // ui-text-exempt: trace token
    };
    let space = |c: &Channel| match c.value {
        Some(Shown::Agreed(pick)) => colourpick::key(pick.ink.space()),
        _ => "-",
    };
    crate::diag::trace_changed(REGION_PAINT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "paint-shown leaves={leaves} paths={paths} fill={} stroke={} fill_space={} \
             stroke_space={}",
            token(fill),
            token(stroke),
            space(fill),
            space(stroke)
        )
    });
}

/// The selected indices that are paths, in selection order.
fn path_indices(doc: &OpenDoc, page: usize, objects: &[usize], leaves: bool) -> Vec<usize> {
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
                crate::app::actions::styled_object(model, o, leaves),
                Some(pdfcer_core::vector::VectorObject::Path(_))
            )
        })
        .collect()
}

/// **Fold one channel of a whole selection into a control state.**
fn channel<'a>(paints: impl Iterator<Item = &'a PathPaint>) -> Channel {
    let mut inks: Vec<Option<String>> = Vec::new();
    let mut agreed: Option<Ink> = None;
    let mut mixed = false;
    let mut total = 0_usize;
    for paint in paints {
        total += 1;
        match ink_of(paint) {
            None => inks.push(ink_name(paint)),
            Some(ink) => match agreed {
                None => agreed = Some(ink),
                Some(seen) if seen != ink => mixed = true,
                Some(_) => {}
            },
        }
    }
    let value = match agreed {
        // Every member is an ink this control will not overwrite. No swatch —
        // the whole of O89's vector ruling, unchanged by the selection size.
        None => None,
        Some(_) if mixed => Some(Shown::Mixed {
            with_opacity: false,
        }),
        Some(ink) => Some(Shown::Agreed(Pick { ink, opacity: None })),
    };
    Channel { value, inks, total }
}

/// One channel's row: its label, its disclosure, and its control — or its
/// refusal, where no control may be drawn.
fn row(ui: &mut Ui, channel: &Channel, label: String, region: &str, id_salt: &str) -> Option<Ink> {
    let mut chosen = None;
    // The partial-ink disclosure sits ABOVE the control, on this project's
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
                // The hint names THIS row's subject — shapes, not words. See
                // `colourpick::show`'s note on why the widget takes it as a
                // parameter rather than reaching for one. No opacity here:
                // a path's opacity is the Line section's.
                chosen =
                    colourpick::show(ui, id_salt, value, Spaces::ALL, region, &t::mixed_hint())
                        .map(|pick| pick.ink);
            }
            // NO SWATCH. See the header: one opening on black over a spot
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

/// The paint as the picker edits it, in the device space the file wrote it
/// in. `Default` is the initial DeviceGray black (ISO 32000-2 §8.6.8);
/// `None` is a space pdfcer does not decode, which gets no picker.
#[allow(clippy::cast_possible_truncation)] // colour components are 0..1
fn ink_of(paint: &PathPaint) -> Option<Ink> {
    let f = |v: &f64| *v as f32;
    match paint {
        PathPaint::Default => Some(Ink::Gray(0.0)),
        PathPaint::Device { space, comps, rgb } => Some(match (space, comps.as_slice()) {
            (DevicePaintSpace::Gray, [g]) => Ink::Gray(f(g)),
            (DevicePaintSpace::Rgb, [r, g, b]) => Ink::Rgb([f(r), f(g), f(b)]),
            (DevicePaintSpace::Cmyk, [c, m, y, k]) => Ink::Cmyk([f(c), f(m), f(y), f(k)]),
            // A wrong operand count: edit what pdfcer drew.
            _ => Ink::Rgb([rgb.r, rgb.g, rgb.b]),
        }),
        PathPaint::Other { .. } => None,
    }
}

/// The ink's name as the file states it, when there is one.
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

    fn red() -> Option<Shown> {
        Some(Shown::Agreed(Pick {
            ink: Ink::Rgb([1.0, 0.0, 0.0]),
            opacity: None,
        }))
    }

    /// A CMYK path opens the picker on CMYK, not on its RGB preview.
    #[test]
    fn a_cmyk_path_opens_on_cmyk() {
        let cmyk = PathPaint::Device {
            space: DevicePaintSpace::Cmyk,
            comps: vec![0.0, 1.0, 1.0, 0.0],
            rgb: Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
        };
        assert_eq!(ink_of(&cmyk), Some(Ink::Cmyk([0.0, 1.0, 1.0, 0.0])));
        assert_eq!(ink_of(&PathPaint::Default), Some(Ink::Gray(0.0)));
    }

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

    /// **An undecodable paint must never yield a colour to open a swatch
    /// on.**
    #[test]
    fn a_spot_ink_offers_no_colour_to_edit() {
        let ink = spot("PANTONE 300");
        assert!(ink.rgb().is_none(), "an undecoded space has no RGB to show");
        assert_eq!(ink_name(&ink).as_deref(), Some("PANTONE 300"));
    }

    /// `Default` and a chosen black both draw a swatch — they are the same
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

    /// **A selection that disagrees reads as MIXED and still offers a
    /// control.**
    #[test]
    fn two_different_colours_read_as_mixed() {
        let ch = channel([rgb(1.0, 0.0, 0.0), rgb(0.0, 1.0, 0.0)].iter());
        assert_eq!(
            ch.value,
            Some(Shown::Mixed {
                with_opacity: false
            })
        );
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
        assert_eq!(ch.value, red());
    }

    /// **One spot ink among process colours keeps the control AND names
    /// the ink.**
    #[test]
    fn one_spot_ink_among_process_colours_keeps_the_swatch_and_names_the_ink() {
        let ch = channel([rgb(1.0, 0.0, 0.0), rgb(1.0, 0.0, 0.0), spot("PANTONE 300")].iter());
        assert_eq!(
            ch.value,
            red(),
            "the process-colour members still have a colour to open on"
        );
        assert_eq!(ch.inks, vec![Some("PANTONE 300".to_owned())]);
        assert_eq!(ch.total, 3);
    }

    /// **Every member a named ink: no control at all.**
    #[test]
    fn a_selection_of_named_inks_offers_no_swatch() {
        let ch = channel([spot("PANTONE 300"), spot("PANTONE 485")].iter());
        assert!(
            ch.value.is_none(),
            "a swatch over a selection of named inks is one click from a destroyed plate"
        );
        assert_eq!(ch.inks.len(), 2);
    }

    /// A spot ink must not be counted as a *disagreement*.
    #[test]
    fn a_spot_ink_does_not_make_the_process_colours_look_mixed() {
        let ch = channel([rgb(1.0, 0.0, 0.0), spot("PANTONE 300")].iter());
        assert_eq!(ch.value, red());
    }
}
