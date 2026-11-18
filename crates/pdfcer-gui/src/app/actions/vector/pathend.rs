//! Commit `VectorAction::PathEnd`: continue an open path from an end node, or
//! open a closed subpath at a node, as one undo entry, then select the node
//! the operator would act on next.

use pdfcer_core::edit::{EditError, EditSession};
use pdfcer_core::vector::{Point, Segment, VectorObject};

use crate::app::actions::apply::vector_edit_on_page;
use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;
use crate::text::pathedit::PathEditRefusal;
use pdfcer_gui_base::subactions::{NodeHost, PathEdit, VectorAction};

/// Where a node sits in its subpath, read before the edit renumbers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    /// The subpath's first object-scoped node index.
    first: usize,
    /// Its last.
    last: usize,
}

/// Apply a `VectorAction::PathEnd`; any other action is ignored.
pub(super) fn apply(doc: &mut OpenDoc, action: VectorAction) {
    let VectorAction::PathEnd {
        page,
        host,
        node,
        edit,
    } = action
    else {
        return;
    };
    let edit = &edit;
    if let PathEdit::Extend(points) = edit
        && points.is_empty()
    {
        crate::app::status::decline::record_path_edit(PathEditRefusal::NoClicks);
        return;
    }
    let target = target_of(host);
    let span = span_of(doc, target, node);
    let mut refused = None;
    let mut ok = false;
    vector_edit_on_page(doc, label(edit), page, 1, |session| {
        let result = one(session, page, host, node, edit);
        match &result {
            Ok(_) => ok = true,
            Err(why) => refused = Some(PathEditRefusal::of_edit(why)),
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "path-end-applied page={page} host={} edit={} node={node} points={} ok={ok}",
                super::nodeshape::host_token(host),
                token(edit),
                point_count(edit),
            )
        });
        result
    });
    if let Some(why) = refused {
        crate::app::status::decline::record_path_edit(why);
    }
    if ok && let Some(next) = span.and_then(|span| selected_after(span, node, edit)) {
        let part = doc.selection.part_of_node(page, target, node);
        doc.selection
            .select_nodes(page, target, &[(part, next)], "path-end-applied");
    }
}

/// One verb, in the host's index space.
fn one(
    session: &mut EditSession,
    page: usize,
    host: NodeHost,
    node: usize,
    edit: &PathEdit,
) -> Result<Vec<String>, EditError> {
    match (host, edit) {
        (NodeHost::Page(o), PathEdit::Extend(points)) => {
            session.extend_path(page, o, node, &segments(points))
        }
        (NodeHost::Page(o), PathEdit::Open) => session.open_subpath(page, o, node),
        (NodeHost::Leaf(l), PathEdit::Extend(points)) => session
            .extend_path_in_form(page, l, node, &segments(points))
            .map(|outcome| outcome.disclosures),
        (NodeHost::Leaf(l), PathEdit::Open) => session
            .open_subpath_in_form(page, l, node)
            .map(|outcome| outcome.disclosures),
    }
}

/// Straight segments through `points`, in page space.
fn segments(points: &[(f64, f64)]) -> Vec<Segment> {
    points
        .iter()
        .map(|&(x, y)| Segment::Line {
            to: Point::new(x, y),
        })
        .collect()
}

/// The subpath holding `node`, from the decomposition the canvas drew.
fn span_of(doc: &OpenDoc, target: TargetId, node: usize) -> Option<Span> {
    let provider = doc.page_objects()?;
    let Some(VectorObject::Path(path)) = provider.object_for(target) else {
        return None;
    };
    let mut first = 0usize;
    for subpath in path.page_subpaths() {
        let count = subpath.anchors().count();
        if node < first + count {
            return Some(Span {
                first,
                last: first + count - 1,
            });
        }
        first += count;
    }
    None
}

/// The node to select once the edit has landed: the new free end of a
/// continued path, or the start of an opened one.
fn selected_after(span: Span, node: usize, edit: &PathEdit) -> Option<usize> {
    match edit {
        // A one-node subpath is continued from its end, like a last node.
        PathEdit::Extend(points) if node == span.last => Some(node + points.len()),
        // From the first node the far end becomes the subpath's first node.
        PathEdit::Extend(_) if node == span.first => Some(span.first),
        PathEdit::Extend(_) => None,
        PathEdit::Open => Some(span.first),
    }
}

const fn target_of(host: NodeHost) -> TargetId {
    match host {
        NodeHost::Page(o) => TargetId::Object(o as u64),
        NodeHost::Leaf(l) => TargetId::Leaf(l as u64),
    }
}

/// The funnel's label for each verb.
const fn label(edit: &PathEdit) -> &'static str {
    // ui-text-exempt: funnel labels, never displayed.
    match edit {
        PathEdit::Extend(_) => "extend-path",
        PathEdit::Open => "open-subpath",
    }
}

const fn token(edit: &PathEdit) -> &'static str {
    // ui-text-exempt: trace tokens, never displayed.
    match edit {
        PathEdit::Extend(_) => "extend",
        PathEdit::Open => "open",
    }
}

fn point_count(edit: &PathEdit) -> usize {
    match edit {
        PathEdit::Extend(points) => points.len(),
        PathEdit::Open => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPAN: Span = Span { first: 3, last: 5 };

    #[test]
    fn the_selection_follows_the_new_free_end() {
        let two = PathEdit::Extend(vec![(0.0, 0.0), (1.0, 1.0)]);
        assert_eq!(selected_after(SPAN, 5, &two), Some(7));
        assert_eq!(selected_after(SPAN, 3, &two), Some(3));
        assert_eq!(selected_after(SPAN, 4, &two), None);
        assert_eq!(selected_after(SPAN, 4, &PathEdit::Open), Some(3));
    }

    #[test]
    fn a_one_node_subpath_continues_from_its_end() {
        let one = PathEdit::Extend(vec![(0.0, 0.0)]);
        assert_eq!(selected_after(Span { first: 2, last: 2 }, 2, &one), Some(3));
    }
}
