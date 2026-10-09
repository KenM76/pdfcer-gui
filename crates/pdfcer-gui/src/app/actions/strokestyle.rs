//! # `app::actions::strokestyle` — line width, dash and opacity of page objects
//!
//! [`Action::SetObjectStrokeStyle`](crate::app::actions::Action) states width
//! and dash in **points**; `EditSession::set_object_stroke_style` takes them in
//! each path's own user space. Paths are grouped by their CTM's scale, one verb
//! call per group, and the calls are folded into one undo step with
//! `coalesce_last`. Opacity is unitless and needs one call. Leaves of a placed
//! drawing (`leaves`) go through the `_in_form` twin; a leaf's `ctm` is
//! already page space, so the conversion is the same.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/strokestyle.md`.

use pdfcer_core::edit::{CommandKind, EditError};
use pdfcer_core::vector::{Dash, PageObjects, StrokeStyle, VectorObject};

use crate::app::state::OpenDoc;
use crate::canvas::shapes::average_scale;
use crate::text::strokestyle as t;

/// Two scales closer than this, relative, are one group.
const SAME_SCALE: f64 = 1e-6;

/// Apply `style` (width and dash in points) to `objects` on `page`: page
/// objects, or `PageObjects::leaves` indices when `leaves`.
pub(super) fn apply(
    doc: &mut OpenDoc,
    page: usize,
    objects: &[usize],
    leaves: bool,
    style: &StrokeStyle,
) {
    super::apply::vector_edit(doc, "set-object-stroke-style", page, objects.len(), |s| {
        let scales: Vec<(usize, f64)> = {
            let model = s.page_objects(page)?;
            objects
                .iter()
                .map(|&i| (i, object_of(&model, i, leaves).map_or(1.0, scale_of)))
                .collect()
        };
        let calls: Vec<(Vec<usize>, StrokeStyle)> = if style.width.is_none() && style.dash.is_none()
        {
            vec![(objects.to_vec(), style.clone())]
        } else {
            group_by_scale(&scales)
                .into_iter()
                .map(|(k, members)| (members, in_user_space(style, k)))
                .collect()
        };
        // Every call's style is checked before the first one commits, so a
        // value one group cannot take leaves the page untouched.
        for (_, user) in &calls {
            user.validate()?;
        }
        let (mut changed, mut refused, mut commands) = (0, 0, 0);
        let mut reach: Vec<String> = Vec::new();
        for (members, user) in &calls {
            let outcome = if leaves {
                let o = s.set_object_stroke_style_in_form(page, members, user)?;
                if let Some(r) = o.reach {
                    reach = r.disclosures;
                }
                o.paint
            } else {
                s.set_object_stroke_style(page, members, user)?
            };
            commands += usize::from(!outcome.changed.is_empty());
            changed += outcome.changed.len();
            refused += outcome.refused.len();
        }
        let mut notes = vec![if refused == 0 {
            t::restyled(changed)
        } else {
            t::restyled_partly(changed, refused)
        }];
        notes.extend(reach);
        let folded = commands < 2 || s.coalesce_last(commands, CommandKind::SetObjectStrokeStyle);
        if !folded {
            notes.push(t::several_undos(commands));
        }
        crate::diag::trace(|| {
            let widths: Vec<String> = calls
                .iter()
                .map(|(_, u)| {
                    u.width
                        .map_or_else(|| "-".to_owned(), |w| format!("{w:.3}"))
                })
                .collect();
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "stroke-style-calls calls={} commands={commands} folded={folded} changed={changed} \
                 refused={refused} user_widths={}",
                calls.len(),
                widths.join(",")
            )
        });
        Ok::<_, EditError>(notes)
    });
}

/// Object `index` of `model`'s page list, or of its leaves when `leaves`.
pub(crate) fn object_of(model: &PageObjects, index: usize, leaves: bool) -> Option<&VectorObject> {
    if leaves {
        model.leaves.get(index).map(|leaf| &leaf.object)
    } else {
        model.objects.get(index)
    }
}

/// User space to points for one object. Only a path's matters: width and dash
/// never reach an image or a form.
pub(crate) fn scale_of(object: &VectorObject) -> f64 {
    match object {
        VectorObject::Path(path) => average_scale(path.ctm),
        _ => 1.0,
    }
}

/// `(index, scale)` pairs grouped by scale, first-seen order kept.
fn group_by_scale(scales: &[(usize, f64)]) -> Vec<(f64, Vec<usize>)> {
    let mut groups: Vec<(f64, Vec<usize>)> = Vec::new();
    for &(index, k) in scales {
        match groups
            .iter_mut()
            .find(|(g, _)| (g - k).abs() <= SAME_SCALE * g.abs().max(k.abs()))
        {
            Some((_, members)) => members.push(index),
            None => groups.push((k, vec![index])),
        }
    }
    groups
}

/// `style` with its width and dash lengths divided by `k`.
fn in_user_space(style: &StrokeStyle, k: f64) -> StrokeStyle {
    StrokeStyle {
        width: style.width.map(|w| w / k),
        dash: style
            .dash
            .as_ref()
            .map(|d| Dash::new(d.array.iter().map(|v| v / k).collect(), d.phase / k)),
        ..style.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objects_at_one_scale_make_one_call() {
        let groups = group_by_scale(&[(3, 0.24), (7, 0.24), (9, 0.240_000_000_1)]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].1, vec![3, 7, 9]);
    }

    #[test]
    fn objects_at_two_scales_make_two_calls_in_selection_order() {
        let groups = group_by_scale(&[(3, 0.24), (7, 1.0), (9, 0.24)]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].1, vec![3, 9]);
        assert_eq!(groups[1].1, vec![7]);
    }

    /// Two points on a path drawn at a quarter scale is eight user units.
    #[test]
    fn points_become_user_units() {
        let style = StrokeStyle {
            width: Some(2.0),
            dash: Some(Dash::new(vec![3.0, 1.0], 0.5)),
            fill_alpha: Some(0.5),
            ..StrokeStyle::default()
        };
        let user = in_user_space(&style, 0.25);
        assert_eq!(user.width, Some(8.0));
        assert_eq!(user.dash, Some(Dash::new(vec![12.0, 4.0], 2.0)));
        assert_eq!(user.fill_alpha, Some(0.5), "an alpha has no unit");
    }

    /// A solid line is the empty array at every scale.
    #[test]
    fn solid_stays_solid() {
        let style = StrokeStyle {
            dash: Some(Dash::default()),
            ..StrokeStyle::default()
        };
        assert!(
            in_user_space(&style, 0.1)
                .dash
                .is_some_and(|d| d.is_solid())
        );
    }
}
