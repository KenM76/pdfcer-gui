//! # `app::actions::group` — commit Group and Ungroup
//!
//! Page objects become one form XObject (`EditSession::group_objects`) and a
//! form is inlined again (`ungroup_form`); markups are linked under a primary
//! (`group_annotations`) and released (`ungroup_annotations`). Each is one undo
//! entry. A page-object edit renumbers the page, so the selection is moved onto
//! the indices the engine answers; a markup keeps its id, so it stays selected.

use pdfcer_core::edit::{AnnotationGroupError, EditError};
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::{GroupError, RestackLimitReason};

use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;
use crate::diag::index_list as list;
use crate::text::grouping::{self as words, GroupRefusal};

pub use pdfcer_gui_base::groupaction::GroupAction;

/// Apply one grouping verb.
pub(super) fn apply(doc: &mut OpenDoc, action: GroupAction) {
    match action {
        GroupAction::Objects { page, objects } => group_objects(doc, page, &objects),
        GroupAction::UngroupForm { page, object } => ungroup_form(doc, page, object),
        GroupAction::Markups {
            page,
            primary,
            members,
        } => group_markups(doc, page, primary, &members),
        GroupAction::UngroupMarkups { page, markup } => ungroup_markups(doc, page, markup),
    }
}

/// Record the sentence a refused verb owes.
fn refuse(error: &EditError) {
    crate::app::status::decline::record_group(refusal_for(error));
}

/// The shell's reading of an engine refusal.
fn refusal_for(error: &EditError) -> GroupRefusal {
    match error {
        EditError::Group(why) => match why {
            GroupError::NoObjects => GroupRefusal::TooFew,
            GroupError::MemberCannotJoin {
                reason: RestackLimitReason::Scope,
                ..
            } => GroupRefusal::DifferentScope,
            GroupError::MemberCannotJoin { .. } => GroupRefusal::Entangled,
            GroupError::NotInvertible => GroupRefusal::NoBox,
            GroupError::NotAForm { .. } => GroupRefusal::NotADrawing,
            GroupError::TransparencyGroup => GroupRefusal::BlendsAsOne,
            GroupError::OptionalContent => GroupRefusal::OnALayer,
            GroupError::Unportable { .. } => GroupRefusal::CannotInline,
            _ => GroupRefusal::Other,
        },
        EditError::AnnotationGroup(why) => match why {
            AnnotationGroupError::NoMembers { .. }
            | AnnotationGroupError::MemberIsPrimary { .. } => GroupRefusal::TooFew,
            AnnotationGroupError::NotMarkup { .. } => GroupRefusal::NotMarkup,
            AnnotationGroupError::CeDimension { .. } => GroupRefusal::CeDimension,
            AnnotationGroupError::DifferentPage { .. } => GroupRefusal::DifferentPages,
            AnnotationGroupError::AlreadyLinked { .. } => GroupRefusal::AlreadyLinked,
            AnnotationGroupError::MemberHasGroup { .. } => GroupRefusal::LeadsAGroup,
            AnnotationGroupError::NotGrouped { .. } => GroupRefusal::NotGrouped,
            _ => GroupRefusal::Other,
        },
        _ => GroupRefusal::Other,
    }
}

/// Whether `objects` are next to each other in paint order, so gathering them
/// passes nothing.
fn contiguous(objects: &[usize]) -> bool {
    let mut sorted = objects.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    match (sorted.first(), sorted.last()) {
        (Some(first), Some(last)) => last - first + 1 == sorted.len(),
        _ => true,
    }
}

fn group_objects(doc: &mut OpenDoc, page: usize, objects: &[usize]) {
    let mut made = None;
    super::apply::vector_edit_on_page(doc, "group-objects", page, objects.len(), |session| {
        let outcome = session.group_objects(page, objects).inspect_err(refuse)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "group-applied page={page} asked={} members={} index={} form={}",
                list(objects),
                outcome.members,
                outcome.index,
                outcome.form.num,
            )
        });
        made = Some(outcome.index);
        let mut said = Vec::new();
        if !contiguous(objects) {
            said.push(words::group_gathered_at_the_front().to_owned());
        }
        Ok::<_, EditError>(said)
    });
    if let Some(index) = made {
        doc.selection
            .select_only(page, TargetId::Object(index as u64), "group");
    }
}

fn ungroup_form(doc: &mut OpenDoc, page: usize, object: usize) {
    let mut freed = None;
    super::apply::vector_edit_on_page(doc, "ungroup-form", page, 1, |session| {
        let outcome = session.ungroup_form(page, object).inspect_err(refuse)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "ungroup-applied page={page} object={object} form={} first={} count={} \
                 clipped={} renamed={}",
                outcome.form.num,
                outcome.indices.start,
                outcome.indices.len(),
                outcome.clipped,
                outcome.renamed,
            )
        });
        let mut said = Vec::new();
        if outcome.clipped {
            said.push(words::ungrouped_keeps_its_frame().to_owned());
        }
        if outcome.renamed > 0 {
            said.push(words::ungrouped_renamed(outcome.renamed));
        }
        freed = Some(outcome.indices);
        Ok::<_, EditError>(said)
    });
    if let Some(indices) = freed {
        let hits: Vec<TargetId> = indices.map(|i| TargetId::Object(i as u64)).collect();
        doc.selection.marquee(page, &hits, false);
    }
}

fn group_markups(doc: &mut OpenDoc, page: usize, primary: ObjId, members: &[ObjId]) {
    super::apply::vector_edit(doc, "group-markups", page, members.len() + 1, |session| {
        let change = session
            .group_annotations(primary, members)
            .inspect_err(refuse)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "markup-group-applied page={page} primary={} members={}",
                change.primary.num,
                change.members.len(),
            )
        });
        Ok::<_, EditError>(vec![words::markups_grouped(change.members.len() + 1)])
    });
}

fn ungroup_markups(doc: &mut OpenDoc, page: usize, markup: ObjId) {
    super::apply::vector_edit(doc, "ungroup-markups", page, 1, |session| {
        let change = session.ungroup_annotations(markup).inspect_err(refuse)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "markup-ungroup-applied page={page} primary={} members={}",
                change.primary.num,
                change.members.len(),
            )
        });
        Ok::<_, EditError>(vec![words::markups_ungrouped(change.members.len())])
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Neighbours in paint order owe no depth sentence; a gap does.**
    #[test]
    fn a_gap_between_members_is_what_the_sentence_is_about() {
        assert!(contiguous(&[4, 3, 5]));
        assert!(contiguous(&[7]));
        assert!(contiguous(&[2, 2, 3]));
        assert!(!contiguous(&[1, 3]));
    }

    /// **Each engine refusal reaches its own sentence.**
    #[test]
    fn engine_refusals_are_worded_apart() {
        let scope = EditError::Group(GroupError::MemberCannotJoin {
            object: 3,
            reason: RestackLimitReason::Scope,
        });
        let tangled = EditError::Group(GroupError::MemberCannotJoin {
            object: 3,
            reason: RestackLimitReason::Entangled,
        });
        assert_eq!(refusal_for(&scope), GroupRefusal::DifferentScope);
        assert_eq!(refusal_for(&tangled), GroupRefusal::Entangled);
        assert_eq!(
            refusal_for(&EditError::Group(GroupError::TransparencyGroup)),
            GroupRefusal::BlendsAsOne
        );
    }
}
