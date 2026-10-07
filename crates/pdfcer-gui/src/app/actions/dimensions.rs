//! Everything the ce-dimension feature asks the document to do.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/dimensions.md`.

#[cfg(test)]
use pdfcer_core::dimension::{
    DimStandard, DimensionId, GroupStyle, ScaleState, StyleOverrides, Unit,
};
use pdfcer_core::edit::GroupDeletion;

use crate::app::state::OpenDoc;

pub use pdfcer_gui_base::editactions::DimensionAction;

/// **Set or clear a ce dimension's caption.**
fn set_label(
    doc: &mut OpenDoc,
    dimension: pdfcer_core::dimension::DimensionId,
    label: Option<&str>,
) {
    let page = doc.view.page_index;
    super::apply::vector_edit(doc, "dimension-label", page, 1, |session| {
        session.set_dimension_label(dimension, label).map(|report| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "dimension-label-applied id={} changed={} measured={:?} printed={:?}",
                    dimension.0, report.changed, report.measured, report.printed
                )
            });
            if !report.changed {
                return Vec::new();
            }
            vec![match report.applied {
                Some(_) => crate::text::panels::dimension::label_set(&report.measured),
                None => crate::text::panels::dimension::label_restored(&report.printed),
            }]
        })
    });
}

/// **Add a group**, optionally starting from another group's calibration and
/// optionally becoming the authoring group.
fn add_group(
    doc: &mut OpenDoc,
    name: &str,
    unit: pdfcer_core::dimension::Unit,
    scale_from: Option<pdfcer_core::dimension::GroupId>,
    author_into: bool,
) {
    let source = scale_from.and_then(|g| {
        doc.session
            .dimension_model()
            .group(g)
            .map(|g| (g.scale, g.format.unit))
    });
    let mut created = None;
    super::apply::vector_edit(doc, "add-dimension-group", 0, 1, |session| {
        let id = session.add_dimension_group(name, unit)?;
        created = Some(id);
        let Some((scale, from)) = source else {
            return Ok::<_, pdfcer_core::edit::EditError>(Vec::new());
        };
        let format = session
            .dimension_model()
            .group(id)
            .map_or_else(|| unit.default_format(), |g| g.format);
        let scale = crate::units::scale_in_unit(scale, from, unit);
        session.set_group_scale(id, scale, format)?;
        let folded = session.coalesce_last(2, pdfcer_core::edit::CommandKind::AddDimension);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "dimension-group-scale-copied group={} unit={unit:?} scale={scale:?} folded={}",
                id.0,
                u8::from(folded)
            )
        });
        Ok(if folded {
            Vec::new()
        } else {
            vec![crate::text::dimension_groups::new_group_two_undos().to_owned()]
        })
    });
    if let (true, Some(id)) = (author_into, created) {
        crate::canvas::measure::queue_active_group(id);
    }
}

/// Apply one ce-dimension verb to the open document.
pub(super) fn apply(doc: &mut OpenDoc, action: DimensionAction) {
    if action.regenerates_the_whole_group() {
        doc.strip_rasters.clear();
    }
    match action {
        // One `add_dimension`, one undo entry — the same contract
        // `CommitMarkup` holds, through the same funnel, so the protocol is
        // not written a second time.
        DimensionAction::Commit {
            page,
            group,
            kind,
            disclosures,
        } => {
            let constraint = match &kind {
                pdfcer_core::dimension::DimensionKind::Linear { constraint, .. } => {
                    constraint_token(*constraint)
                }
                _ => "-",
            };
            super::apply::vector_edit(doc, "add-dimension", page, 1, |session| {
                // The gesture's disclosures are returned as the edit's, which
                // is what puts them on the status bar stamped with the epoch
                // this commit produced.
                session.add_dimension(page, group, kind).map(|(annot, dim)| {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed
                        format!(
                            "dimension-added dim={} annot={}_{} group={} constraint={constraint}",
                            dim.0, annot.num, annot.generation, group.0
                        )
                    });
                    disclosures
                })
            });
            trace_members_shown(doc, group);
        }
        DimensionAction::SetGroupScale {
            group,
            scale,
            format,
        } => {
            super::apply::vector_edit(doc, "set-group-scale", 0, 1, |session| {
                session
                    .set_group_scale(group, scale, format)
                    .map(|_| Vec::new())
            });
            trace_members_shown(doc, group);
        }
        // Creating a group regenerates nothing — it has no members yet — so
        // it is not in `regenerates_the_whole_group` and clears no rasters.
        // It is still a document edit: the sidecar gains a record, and a save
        // taken afterwards carries the group.
        DimensionAction::AddGroup {
            name,
            unit,
            scale_from,
            author_into,
        } => add_group(doc, &name, unit, scale_from, author_into),
        // The one group verb that regenerates NOTHING. No member's
        // appearance depends on what its group is called, so no raster is
        // dropped and `regenerates_the_whole_group` says so.
        DimensionAction::RenameGroup { group, name } => {
            super::apply::vector_edit(doc, "rename-dimension-group", 0, 1, |session| {
                session
                    .rename_dimension_group(group, &name)
                    .map(|()| Vec::new())
            });
        }
        // One line: `set_dimension_label` owns the whole contract — the
        // whitespace-only refusal, keeping the measurement, and regenerating
        // the appearance at the new caption.
        DimensionAction::SetLabel { dimension, label } => {
            set_label(doc, dimension, label.as_deref());
        }
        // Routed through `delete_dimension_group_with` for BOTH policies,
        // including `Refuse` — which is exactly what the no-argument
        // `delete_dimension_group` does.
        //
        // One call site rather than two, because the difference between them is
        // a value this variant already carries, and a `match` here would be a
        // second place for the default policy to be decided. The engine's own
        // pair exists for callers who have no policy to express; this one
        // always does.
        //
        // The returned count is the number REASSIGNED, and it is deliberately
        // dropped: the dialog computed the member count before pressing, from
        // the model it was already holding, and that is the number it showed.
        // Reporting a second count afterwards would be two answers to one
        // question — the shape `set_group_style`'s return value already taught
        // this module to refuse.
        DimensionAction::DeleteGroup { group, policy } => {
            // A reassignment moves members between groups, which re-measures
            // and redraws each of them wherever it is. A refusal moves nothing.
            // Deciding here rather than in `regenerates_the_whole_group` because
            // it is a property of the POLICY, not of the verb — the predicate
            // takes the variant and cannot see inside it.
            if matches!(policy, GroupDeletion::Reassign(_)) {
                doc.strip_rasters.clear();
            }
            super::apply::vector_edit(doc, "delete-dimension-group", 0, 1, |session| {
                session
                    .delete_dimension_group_with(group, policy)
                    .map(|_| Vec::new())
            });
        }
        // One annotation redrawn, and its printed NUMBER changes — see the
        // variant. The page it is on is not known here (a `DimensionId` names a
        // sidecar record, not a page), so page `0` is passed with the note every
        // document-scoped verb in this file passes it with, and the strip is
        // deliberately NOT cleared: exactly one annotation moved.
        DimensionAction::SetDimensionGroup { dimension, group } => {
            super::apply::vector_edit(doc, "set-dimension-group", 0, 1, |session| {
                session
                    .set_dimension_group(dimension, group)
                    .map(|()| Vec::new())
            });
        }
        // Re-measures, so it owes a disclosure. `VertexOutcome` carries the
        // label before and after, because the old value cannot be
        // reconstructed once the geometry it came from is gone — and
        // "12.40 m -> 13.85 m" is a disclosure where "13.85 m" is just the
        // number already on the page.
        DimensionAction::MoveVertex {
            dimension,
            index,
            dx,
            dy,
        } => {
            super::apply::vector_edit(doc, "move-dimension-vertex", 0, 1, |session| {
                session
                    .move_dimension_vertex(dimension, index, dx, dy)
                    .map(|outcome| {
                        // Nothing to say when the number did not move - a
                        // corner dragged along its own segment changes the
                        // shape and not the length, and reporting "13.85 m ->
                        // 13.85 m" would train the operator to ignore the line
                        // that matters.
                        if outcome.label == outcome.previous_label {
                            Vec::new()
                        } else {
                            vec![crate::text::measure::vertex_remeasured(
                                &outcome.previous_label,
                                &outcome.label,
                            )]
                        }
                    })
            });
        }
        // The one arm in this file that touches no document. See the variant:
        // it exists because the canvas cannot reach `app::status::decline`, and
        // a corner drag that is refused with no sentence is the founding defect
        // of this project wearing a new grip.
        DimensionAction::DeclineVertexEdit { why } => {
            crate::app::status::decline::record_vertex_edit_refused(why);
        }
        // The two verbs that change how many corners a measured shape has.
        //
        // Both re-measure, so both owe the disclosure `MoveVertex` owes, plus
        // the corner count: that is the fact the gesture exists to change, and
        // an insert that landed on the wrong segment looks exactly like one
        // that landed on the right one until you read the number.
        //
        // Unlike `MoveVertex` above, the disclosure is NOT suppressed when
        // the label is unchanged, and the asymmetry is deliberate. A corner
        // dragged along its own segment changes the shape and not the length,
        // so "13.85 m -> 13.85 m" there is noise. Here the operator changed the
        // corner COUNT, which always changed even when the length happens not
        // to have — a corner added exactly on a segment adds no length — and a
        // gesture that altered the shape and said nothing is the silence this
        // whole piece of work is about.
        //
        // Page `0` with the note every document-scoped verb in this file passes
        // it with (a `DimensionId` names a sidecar record, not a page), and the
        // strip is deliberately NOT cleared: exactly one annotation moved.
        DimensionAction::InsertVertex {
            dimension,
            after,
            at,
        } => {
            super::apply::vector_edit(doc, "insert-dimension-vertex", 0, 1, |session| {
                session
                    .insert_dimension_vertex(dimension, after, at)
                    .map(|outcome| {
                        vec![crate::text::measure::vertex_inserted(
                            outcome.vertices,
                            &outcome.previous_label,
                            &outcome.label,
                        )]
                    })
            });
        }
        DimensionAction::RemoveVertex { dimension, index } => {
            super::apply::vector_edit(doc, "remove-dimension-vertex", 0, 1, |session| {
                session
                    .remove_dimension_vertex(dimension, index)
                    .map(|outcome| {
                        vec![crate::text::measure::vertex_removed(
                            outcome.vertices,
                            &outcome.previous_label,
                            &outcome.label,
                        )]
                    })
            });
        }
        // One annotation redrawn, in place. No value is re-measured —
        // `place_dimension` writes two fields the value function does not read
        // — so unlike `SetDimensionGroup` there is not even a number to
        // disclose. The page is not known here (a `DimensionId` names a sidecar
        // record, not a page), so page `0` is passed with the note every
        // document-scoped verb in this file passes it with.
        DimensionAction::Place {
            dimension,
            offset,
            text_along,
        } => {
            super::apply::vector_edit(doc, "place-dimension", 0, 1, |session| {
                session
                    .place_dimension(dimension, offset, text_along)
                    .map(|()| Vec::new())
            });
        }
        DimensionAction::SetGroupStandard { group, standard } => {
            super::apply::vector_edit(doc, "set-group-standard", 0, 1, |session| {
                session
                    .set_group_standard(group, standard)
                    .map(|_| Vec::new())
            });
        }
        DimensionAction::SetGroupStyle { group, style } => {
            super::apply::vector_edit(doc, "set-group-style", 0, 1, |session| {
                session.set_group_style(group, style).map(|_| Vec::new())
            });
        }
        // The returned `bool` is the RESULTING visibility, and it is
        // deliberately dropped. The dialog re-reads the model next frame, so
        // carrying the answer back would give the surface two sources for one
        // fact — and the one carried here would be the older of the two by the
        // time anything drew it.
        DimensionAction::ToggleLayer { group, visible } => {
            super::apply::vector_edit(doc, "toggle-dimension-layer", 0, 1, |session| {
                session
                    .toggle_dimension_layer(group, visible)
                    .map(|_| Vec::new())
            });
        }
        DimensionAction::SetStyle { dimension, style } => {
            super::apply::vector_edit(doc, "set-dimension-style", 0, 1, |session| {
                session
                    .set_dimension_style(dimension, style)
                    .map(|_| Vec::new())
            });
        }
        DimensionAction::SetDisplay {
            dimension,
            show_diameter,
        } => {
            super::apply::vector_edit(doc, "set-dimension-display", 0, 1, |session| {
                session
                    .set_dimension_display(dimension, show_diameter)
                    .map(|_| Vec::new())
            });
        }
        DimensionAction::SetArea { dimension, area } => {
            super::apply::vector_edit(doc, "set-dimension-area", 0, 1, |session| {
                session
                    .set_dimension_area(dimension, area)
                    .map(|()| Vec::new())
            });
            // The engine's own label after the switch: what a driven check
            // compares, since perimeter and area differ only in the printed text.
            let shown = doc
                .session
                .dimension_model()
                .display(dimension)
                .map(|d| d.text)
                .unwrap_or_default();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "dimension-area-applied id={} area={} text=\"{shown}\"",
                    dimension.0,
                    u8::from(area)
                )
            });
        }
        DimensionAction::SetExtensionGap {
            dimension,
            end,
            gap,
        } => {
            super::apply::vector_edit(doc, "set-dimension-extension-gap", 0, 1, |session| {
                session
                    .set_dimension_extension_gap(dimension, end, gap)
                    .map(|()| Vec::new())
            });
        }
    }
}

/// The trace token for a linear ce dimension's direction.
const fn constraint_token(c: pdfcer_core::vector::AxisConstraint) -> &'static str {
    use pdfcer_core::vector::AxisConstraint as A;
    match c {
        A::Aligned => "aligned",
        A::Horizontal => "horizontal",
        A::Vertical => "vertical",
    }
}

/// One `dimension-member-shown` line per member of `group`: the text each
/// one reads now, as the engine derives it from the group's scale and format.
fn trace_members_shown(doc: &OpenDoc, group: pdfcer_core::dimension::GroupId) {
    if !crate::diag::enabled() {
        return;
    }
    let model = doc.session.dimension_model();
    for member in model.members(group) {
        if let Some(shown) = model.display(member.id) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "dimension-member-shown group={} dim={} text=\"{}\"",
                    group.0, member.id.0, shown.text
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::dimension::DEFAULT_GROUP_ID;

    /// The blast-radius predicate agrees with the module header's table.
    #[test]
    fn every_group_verb_is_document_wide_and_every_other_is_not() {
        let g = DEFAULT_GROUP_ID;
        let d = DimensionId(0);

        // Group-scoped: every member, on every page.
        for a in [
            DimensionAction::SetGroupScale {
                group: g,
                scale: ScaleState::NeverSet,
                format: Unit::Millimeter.default_format(),
            },
            DimensionAction::SetGroupStandard {
                group: g,
                standard: DimStandard::default(),
            },
            DimensionAction::SetGroupStyle {
                group: g,
                style: GroupStyle::default(),
            },
            DimensionAction::ToggleLayer {
                group: g,
                visible: false,
            },
        ] {
            assert!(
                a.regenerates_the_whole_group(),
                "{a:?} touches every member and must clear the whole strip"
            );
        }

        // Annotation-scoped, plus `AddGroup`, which creates rather than changes
        // and so has no members to regenerate.
        for a in [
            DimensionAction::AddGroup {
                name: "Detail".to_owned(),
                unit: Unit::Millimeter,
                scale_from: None,
                author_into: false,
            },
            DimensionAction::SetStyle {
                dimension: d,
                style: StyleOverrides::default(),
            },
            DimensionAction::SetDisplay {
                dimension: d,
                show_diameter: true,
            },
        ] {
            assert!(
                !a.regenerates_the_whole_group(),
                "{a:?} touches one record and must not clear the whole strip"
            );
        }
    }
}
