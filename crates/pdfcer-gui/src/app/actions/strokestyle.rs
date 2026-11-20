//! # `app::actions::strokestyle` — line width, dash and opacity of page objects
//!
//! [`Action::SetObjectStrokeStyle`](crate::app::actions::Action) states width
//! and dash in **points**. The style goes to `EditSession::set_object_stroke_style`
//! with `StrokeUnits::Page`, so the engine divides by each path's own stroke
//! scale and one call is one undo step. Leaves of a placed drawing (`leaves`)
//! go through the `_in_form` twin, which takes the units the same way.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/strokestyle.md`.

use pdfcer_core::edit::EditError;
use pdfcer_core::vector::{PageObjects, StrokeStyle, StrokeUnits, VectorObject};

use crate::app::state::OpenDoc;
use crate::text::strokestyle as t;

/// Apply `style` (width and dash in points) to `objects` on `page`: page
/// objects, or `PageObjects::leaves` indices when `leaves`.
pub(super) fn apply(
    doc: &mut OpenDoc,
    page: usize,
    objects: &[usize],
    leaves: bool,
    style: &StrokeStyle,
) {
    let style = in_points(style);
    super::apply::vector_edit(doc, "set-object-stroke-style", page, objects.len(), |s| {
        let (outcome, reach) = if leaves {
            let o = s.set_object_stroke_style_in_form(page, objects, &style)?;
            (o.paint, o.reach.map(|r| r.disclosures).unwrap_or_default())
        } else {
            (s.set_object_stroke_style(page, objects, &style)?, Vec::new())
        };
        let (changed, refused) = (outcome.changed.len(), outcome.refused.len());
        let mut notes = vec![if refused == 0 {
            t::restyled(changed)
        } else {
            t::restyled_partly(changed, refused)
        }];
        notes.extend(outcome.disclosures);
        notes.extend(reach);
        crate::diag::trace(|| {
            let width = style
                .width
                .map_or_else(|| "-".to_owned(), |w| format!("{w:.3}"));
            let units = if style.units == StrokeUnits::Page {
                "page" // ui-text-exempt: trace token
            } else {
                "user" // ui-text-exempt: trace token
            };
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "stroke-style-calls calls=1 units={units} width_pt={width} changed={changed} \
                 refused={refused}"
            )
        });
        Ok::<_, EditError>(notes)
    });
}

/// `style` with its width and dash stated in page points.
fn in_points(style: &StrokeStyle) -> StrokeStyle {
    StrokeStyle {
        units: StrokeUnits::Page,
        ..style.clone()
    }
}

/// Object `index` of `model`'s page list, or of its leaves when `leaves`.
pub(crate) fn object_of(model: &PageObjects, index: usize, leaves: bool) -> Option<&VectorObject> {
    if leaves {
        model.leaves.get(index).map(|leaf| &leaf.object)
    } else {
        model.objects.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::vector::Dash;

    /// The values typed reach the engine unchanged, marked as points.
    #[test]
    fn the_style_is_sent_in_points_as_typed() {
        let style = StrokeStyle {
            width: Some(2.0),
            dash: Some(Dash::new(vec![3.0, 1.0], 0.5)),
            fill_alpha: Some(0.5),
            ..StrokeStyle::default()
        };
        let sent = in_points(&style);
        assert_eq!(sent.units, StrokeUnits::Page);
        assert_eq!(sent.width, Some(2.0));
        assert_eq!(sent.dash, Some(Dash::new(vec![3.0, 1.0], 0.5)));
        assert_eq!(sent.fill_alpha, Some(0.5));
    }
}
