//! # `app::actions::groupform` — Group and Ungroup inside a placed drawing
//!
//! Parts selected inside one placement of a form become a group within that
//! form (`EditSession::group_objects_in_form`), and the form one part is
//! directly inside is inlined into its container
//! (`EditSession::ungroup_form_in_form`). Both edit the form in place, so
//! every other place it is drawn changes too and the status line says how
//! many. Leaf indices are renumbered by either edit: after Group the members
//! are found again by the group they are now inside; after Ungroup the
//! selection is cleared.

use pdfcer_core::edit::EditError;

use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;
use crate::diag::index_list as list;
use crate::text::grouping as words;

/// Group `leaves` on `page` inside the form that holds them.
pub(super) fn group(doc: &mut OpenDoc, page: usize, leaves: &[usize]) {
    let mut members: Option<Vec<usize>> = None;
    super::apply::vector_edit_on_page(doc, "group-in-form", page, leaves.len(), |session| {
        let outcome = session
            .group_objects_in_form(page, leaves)
            .inspect_err(super::group::refuse)?;
        let found: Vec<usize> = session
            .page_objects(page)
            .map(|model| {
                model
                    .leaves
                    .iter()
                    .enumerate()
                    .filter(|(_, leaf)| leaf.containment.last() == Some(&outcome.group))
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "group-in-form-applied page={page} asked={} members={} group={} container={} \
                 invocations={} pages={} leaves={}",
                list(leaves),
                outcome.members,
                outcome.group.num,
                outcome.container.num,
                outcome.invocations,
                outcome.pages,
                list(&found),
            )
        });
        members = Some(found);
        let mut said = vec![words::grouped_in_drawing(outcome.members)];
        said.extend(crate::text::unshare::remedy_if_shared(
            outcome.invocations,
            outcome.pages,
        ));
        said.extend(outcome.disclosures);
        Ok::<_, EditError>(said)
    });
    match members {
        Some(found) if !found.is_empty() => {
            let hits: Vec<TargetId> = found.iter().map(|&i| TargetId::Leaf(i as u64)).collect();
            doc.selection.marquee(page, &hits, false);
        }
        Some(_) => {
            doc.selection.clear();
        }
        None => {}
    }
}

/// Inline the form `leaf` on `page` is directly inside into its container.
pub(super) fn ungroup(doc: &mut OpenDoc, page: usize, leaf: usize) {
    let mut done = false;
    super::apply::vector_edit_on_page(doc, "ungroup-in-form", page, 1, |session| {
        let outcome = session
            .ungroup_form_in_form(page, leaf)
            .inspect_err(super::group::refuse)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "ungroup-in-form-applied page={page} leaf={leaf} form={} container={} \
                 invocations={} pages={} clipped={} renamed={}",
                outcome.form.num,
                outcome.container.map_or(0, |c| c.num),
                outcome.invocations,
                outcome.pages,
                outcome.clipped,
                outcome.renamed,
            )
        });
        done = true;
        let mut said = vec![if outcome.container.is_some() {
            words::ungrouped_in_drawing().to_owned()
        } else {
            words::ungrouped_out_of_drawing().to_owned()
        }];
        if outcome.clipped {
            said.push(words::ungrouped_keeps_its_frame().to_owned());
        }
        if outcome.renamed > 0 {
            said.push(words::ungrouped_renamed(outcome.renamed));
        }
        said.extend(crate::text::unshare::remedy_if_shared(
            outcome.invocations,
            outcome.pages,
        ));
        said.extend(outcome.disclosures);
        Ok::<_, EditError>(said)
    });
    if done {
        doc.selection.clear();
    }
}
