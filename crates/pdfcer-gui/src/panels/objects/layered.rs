//! The Objects tree's layer headings.
//!
//! A page whose objects carry `/OC` lists them under the layer each is on.
//! Layers come in the order of their front-most object, objects front-most
//! first within a layer, and content on no layer last. A page with no `/OC`
//! has no headings, and its tree is the flat paint-order list. The layer is
//! the innermost `/OC` the engine reports for the object
//! (`VectorObject::oc`).

use std::collections::BTreeSet;

use pdfcer_core::object::ObjId;

use super::ObjectTreeRow;
use super::provider::ObjectModelProvider;
use crate::text::panels::objects as t;

/// One heading's layer and its objects, front-most first; `None` is the
/// content on no layer.
pub(super) type Group = (Option<ObjId>, Vec<usize>);

/// The page's objects by layer, or `None` when no object is on one.
pub(super) fn groups(provider: &ObjectModelProvider) -> Option<Vec<Group>> {
    let objects = &provider.page_objects().objects;
    let mut out: Vec<Group> = Vec::new();
    let mut loose = Vec::new();
    for index in (0..objects.len()).rev() {
        match objects[index].oc() {
            Some(oc) => match out.iter_mut().find(|(g, _)| *g == Some(oc)) {
                Some((_, members)) => members.push(index),
                None => out.push((Some(oc), vec![index])),
            },
            None => loose.push(index),
        }
    }
    if out.is_empty() {
        return None;
    }
    if !loose.is_empty() {
        out.push((None, loose));
    }
    Some(out)
}

/// The rows for a layered page: each heading, then its objects unless the
/// operator folded it away. `object_rows` adds one object and what is
/// expanded under it.
pub(super) fn rows(
    groups: Vec<Group>,
    collapsed: &BTreeSet<Option<ObjId>>,
    mut object_rows: impl FnMut(&mut Vec<ObjectTreeRow>, usize),
) -> Vec<ObjectTreeRow> {
    let mut out = Vec::new();
    for (group, members) in groups {
        out.push(ObjectTreeRow::Layer {
            group,
            count: members.len(),
        });
        if collapsed.contains(&group) {
            continue;
        }
        for index in members {
            object_rows(&mut out, index);
        }
    }
    out
}

/// A heading's text: the layer's name, whether it is hidden now, and how
/// many objects it holds on this page.
pub(super) fn label(
    read: &pdfcer_core::layers::Layers,
    hidden: impl Fn(ObjId) -> bool,
    group: Option<ObjId>,
    count: usize,
) -> String {
    let Some(id) = group else {
        return t::object_tree_no_layer_row(count);
    };
    let name = crate::panels::layers::layer_name_for(read, id)
        .unwrap_or_else(|| t::object_tree_unlisted_layer().to_owned());
    t::object_tree_layer_row(&name, count, hidden(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;

    fn provider(rel: &str) -> ObjectModelProvider {
        let doc = pdfcer_core::document::Document::load(&engine_fixture(rel)).expect("loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        ObjectModelProvider::build(&doc.view(), &pages[0], 0).expect("it decomposes")
    }

    /// Every object is listed once, under the layer its own `/OC` names,
    /// front-most first.
    #[test]
    fn a_layered_page_lists_each_object_once_under_its_layer() {
        let p = provider("layers/painted-layers.pdf");
        let groups = groups(&p).expect("the fixture's objects are on layers");
        let objects = &p.page_objects().objects;
        let mut seen: Vec<usize> = groups.iter().flat_map(|(_, m)| m.clone()).collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..objects.len()).collect::<Vec<_>>());
        for (group, members) in &groups {
            assert!(members.windows(2).all(|w| w[0] > w[1]), "front-most first");
            for &i in members {
                assert_eq!(
                    objects[i].oc(),
                    *group,
                    "object {i} is under the wrong heading"
                );
            }
        }
        assert!(
            groups.len() > 1,
            "the fixture paints on more than one layer"
        );
    }

    /// A folded heading keeps its row and drops its objects.
    #[test]
    fn a_folded_layer_keeps_its_heading_only() {
        let p = provider("layers/painted-layers.pdf");
        let groups = groups(&p).expect("layered");
        let first = groups[0].0;
        let total: usize = groups.iter().map(|(_, m)| m.len()).sum();
        let heads = groups.len();
        let open = rows(groups.clone(), &BTreeSet::new(), |r, i| {
            r.push(ObjectTreeRow::Object { index: i });
        });
        assert_eq!(open.len(), heads + total);
        let folded = rows(groups.clone(), &[first].into_iter().collect(), |r, i| {
            r.push(ObjectTreeRow::Object { index: i });
        });
        assert_eq!(folded.len(), heads + total - groups[0].1.len());
    }

    /// A page with no `/OC` keeps the flat list.
    #[test]
    fn a_page_with_no_layers_has_no_headings() {
        assert!(groups(&provider("pageops/four-pages.pdf")).is_none());
    }
}
