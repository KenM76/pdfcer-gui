//! `layerorder` — the Layers panel's tree, and where a move puts an entry.
//!
//! The panel tree is `pdfcer_core::layers::Layers::order` (Table 101's
//! `/D /Order`), reduced to what the panel draws: a **folder** (a label row,
//! no checkbox), a **layer** (perhaps with sublayers), or an unlabelled
//! **grouping**. A position is a path of child indices, root level first.
//!
//! Every move the panel raises names its destination in the tree **as it is
//! now** — "before the child at `index` of `parent`" — and [`engine_move`]
//! turns that into the engine's coordinates, which `EditSession::move_layer_node`
//! reads after the entry is taken out (and after any grouping the removal
//! empties is dropped, as the engine drops it).
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/layerorder.md`.

use pdfcer_core::layers::OrderNode;
use pdfcer_core::object::ObjId;

/// What one panel entry is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// A labelled array or a bare label: a heading with no visibility.
    Folder(String),
    /// An optional-content group; its children are sublayers.
    Layer(ObjId),
    /// An unlabelled array that does not follow a layer.
    Grouping,
}

/// One panel entry and what it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// What it is.
    pub kind: Kind,
    /// What it holds, in panel order.
    pub children: Vec<Node>,
}

impl Node {
    /// The engine's node, reduced.
    #[must_use]
    pub fn of(n: &OrderNode) -> Self {
        let kind = match (&n.label, n.group) {
            (_, Some(id)) => Kind::Layer(id),
            (Some(label), None) => Kind::Folder(label.clone()),
            (None, None) => Kind::Grouping,
        };
        Self {
            kind,
            children: n.children.iter().map(Self::of).collect(),
        }
    }

    /// The folder's label, if this is a folder.
    #[must_use]
    pub fn folder_label(&self) -> Option<&str> {
        match &self.kind {
            Kind::Folder(label) => Some(label),
            _ => None,
        }
    }
}

/// The engine's `/Order` tree, reduced.
#[must_use]
pub fn tree(order: &[OrderNode]) -> Vec<Node> {
    order.iter().map(Node::of).collect()
}

/// The entry at `path`.
#[must_use]
pub fn node_at<'a>(tree: &'a [Node], path: &[usize]) -> Option<&'a Node> {
    let (&first, rest) = path.split_first()?;
    rest.iter()
        .try_fold(tree.get(first)?, |n, &i| n.children.get(i))
}

/// The children of the entry at `parent` (`&[]` for the top level).
#[must_use]
pub fn children_at<'a>(tree: &'a [Node], parent: &[usize]) -> Option<&'a [Node]> {
    if parent.is_empty() {
        Some(tree)
    } else {
        node_at(tree, parent).map(|n| n.children.as_slice())
    }
}

/// Where layer `id` sits, if the tree lists it.
#[must_use]
pub fn path_of_layer(tree: &[Node], id: ObjId) -> Option<Vec<usize>> {
    tree.iter().enumerate().find_map(|(i, n)| {
        if n.kind == Kind::Layer(id) {
            return Some(vec![i]);
        }
        let mut path = path_of_layer(&n.children, id)?;
        path.insert(0, i);
        Some(path)
    })
}

/// Every folder, in panel order, with its path and label.
#[must_use]
pub fn folders(tree: &[Node]) -> Vec<(Vec<usize>, String)> {
    let mut out = Vec::new();
    collect_folders(tree, &mut Vec::new(), &mut out);
    out
}

fn collect_folders(nodes: &[Node], at: &mut Vec<usize>, out: &mut Vec<(Vec<usize>, String)>) {
    for (i, n) in nodes.iter().enumerate() {
        at.push(i);
        if let Some(label) = n.folder_label() {
            out.push((at.clone(), label.to_owned()));
        }
        collect_folders(&n.children, at, out);
        at.pop();
    }
}

/// A move in the tree as it is now: the entry at `from` goes before the child
/// currently at `index` of `parent` (`index` = child count for the end).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    /// The entry moved.
    pub from: Vec<usize>,
    /// Its new parent.
    pub parent: Vec<usize>,
    /// The slot, counted among the parent's children before the move.
    pub index: usize,
}

/// What [`engine_move`] makes of a [`Move`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    /// `EditSession::move_layer_node(from, parent, index)` with these.
    Engine {
        /// The engine's parent path, read after removal.
        parent: Vec<usize>,
        /// The engine's index, read after removal.
        index: usize,
    },
    /// The entry is already there.
    Unchanged,
    /// The destination is the entry itself or inside it.
    IntoItself,
    /// `from` or `parent` names no entry.
    NotFound,
}

/// A node of the tree with its path before the move.
struct Tagged {
    was: Vec<usize>,
    grouping: bool,
    children: Vec<Tagged>,
}

fn tag(nodes: &[Node], at: &mut Vec<usize>) -> Vec<Tagged> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            at.push(i);
            let t = Tagged {
                was: at.clone(),
                grouping: n.kind == Kind::Grouping,
                children: tag(&n.children, at),
            };
            at.pop();
            t
        })
        .collect()
}

fn kids_mut<'a>(tree: &'a mut Vec<Tagged>, path: &[usize]) -> Option<&'a mut Vec<Tagged>> {
    path.iter()
        .try_fold(tree, |kids, &i| kids.get_mut(i).map(|n| &mut n.children))
}

/// `EditSession::move_layer_node`'s removal: take the entry out, then drop
/// each grouping the removal left empty, innermost first.
fn remove(tree: &mut Vec<Tagged>, from: &[usize]) -> Option<()> {
    let (&last, parent) = from.split_last()?;
    let kids = kids_mut(tree, parent)?;
    if last >= kids.len() {
        return None;
    }
    kids.remove(last);
    let mut p = parent.to_vec();
    while let Some((&i, up)) = p.split_last() {
        let empty = kids_mut(tree, &p).is_some_and(|k| k.is_empty());
        let grouping = kids_mut(tree, up)
            .and_then(|k| k.get(i))
            .is_some_and(|n| n.grouping);
        if !(empty && grouping) {
            break;
        }
        if let Some(k) = kids_mut(tree, up) {
            k.remove(i);
        }
        p.pop();
    }
    Some(())
}

/// Where the node that was at `was` now is.
fn find(tree: &[Tagged], was: &[usize]) -> Option<Vec<usize>> {
    tree.iter().enumerate().find_map(|(i, n)| {
        if n.was == was {
            return Some(vec![i]);
        }
        let mut p = find(&n.children, was)?;
        p.insert(0, i);
        Some(p)
    })
}

/// Turn a move in today's tree into the engine's after-removal coordinates.
#[must_use]
pub fn engine_move(tree: &[Node], m: &Move) -> Resolved {
    if m.parent.starts_with(&m.from) {
        return Resolved::IntoItself;
    }
    let (Some(_), Some(siblings)) = (node_at(tree, &m.from), children_at(tree, &m.parent)) else {
        return Resolved::NotFound;
    };
    if m.index > siblings.len() {
        return Resolved::NotFound;
    }
    let Some((&last, from_parent)) = m.from.split_last() else {
        return Resolved::NotFound;
    };
    if m.parent == from_parent && (m.index == last || m.index == last + 1) {
        return Resolved::Unchanged;
    }
    let mut work = tag(tree, &mut Vec::new());
    if remove(&mut work, &m.from).is_none() {
        return Resolved::NotFound;
    }
    let parent = if m.parent.is_empty() {
        Vec::new()
    } else {
        match find(&work, &m.parent) {
            Some(p) => p,
            // The parent was a grouping the removal emptied: nothing moves.
            None => return Resolved::Unchanged,
        }
    };
    let index = kids_mut(&mut work, &parent).map_or(0, |kids| {
        kids.iter()
            .filter(|k| k.was.last().is_some_and(|&w| w < m.index))
            .count()
    });
    Resolved::Engine { parent, index }
}

/// The moves a row's menu offers, each with what it is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Offer {
    /// One place earlier among its siblings.
    Up(Move),
    /// One place later among its siblings.
    Down(Move),
    /// Out of the folder or layer holding it, to just after that entry.
    Out(Move, OutOf),
    /// To the end of a folder.
    Into(Move, String),
}

/// What [`Offer::Out`] leaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutOf {
    /// A folder, by label.
    Folder(String),
    /// A layer, by id.
    Layer(ObjId),
    /// An unlabelled grouping.
    Grouping,
}

/// Every move the entry at `at` can make from its menu.
#[must_use]
pub fn offers(tree: &[Node], at: &[usize]) -> Vec<Offer> {
    let Some((&last, parent)) = at.split_last() else {
        return Vec::new();
    };
    let Some(siblings) = children_at(tree, parent) else {
        return Vec::new();
    };
    let mv = |parent: &[usize], index| Move {
        from: at.to_vec(),
        parent: parent.to_vec(),
        index,
    };
    let mut out = Vec::new();
    if last > 0 {
        out.push(Offer::Up(mv(parent, last - 1)));
    }
    if last + 1 < siblings.len() {
        out.push(Offer::Down(mv(parent, last + 2)));
    }
    if let Some((&p_last, grand)) = parent.split_last()
        && let Some(holder) = node_at(tree, parent)
    {
        let of = match &holder.kind {
            Kind::Folder(label) => OutOf::Folder(label.clone()),
            Kind::Layer(id) => OutOf::Layer(*id),
            Kind::Grouping => OutOf::Grouping,
        };
        out.push(Offer::Out(mv(grand, p_last + 1), of));
    }
    for (path, label) in folders(tree) {
        if path.starts_with(at) || path == parent {
            continue;
        }
        let len = node_at(tree, &path).map_or(0, |n| n.children.len());
        out.push(Offer::Into(mv(&path, len), label));
    }
    out
}

/// A path as the trace and the driven check spell it: `0.1.2`.
#[must_use]
pub fn dotted(path: &[usize]) -> String {
    path.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(n: u32, children: Vec<Node>) -> Node {
        Node {
            kind: Kind::Layer(ObjId::new(n, 0)),
            children,
        }
    }

    fn folder(label: &str, children: Vec<Node>) -> Node {
        Node {
            kind: Kind::Folder(label.to_owned()),
            children,
        }
    }

    fn grouping(children: Vec<Node>) -> Node {
        Node {
            kind: Kind::Grouping,
            children,
        }
    }

    /// The fixture's tree: Sheet{6, 7}, 8{9}, Services{10}, 11.
    fn sample() -> Vec<Node> {
        vec![
            folder("Sheet", vec![layer(6, vec![]), layer(7, vec![])]),
            layer(8, vec![layer(9, vec![])]),
            folder("Services", vec![layer(10, vec![])]),
            layer(11, vec![]),
        ]
    }

    fn mv(from: &[usize], parent: &[usize], index: usize) -> Move {
        Move {
            from: from.to_vec(),
            parent: parent.to_vec(),
            index,
        }
    }

    fn engine(parent: &[usize], index: usize) -> Resolved {
        Resolved::Engine {
            parent: parent.to_vec(),
            index,
        }
    }

    #[test]
    fn a_later_parent_shifts_up_when_an_earlier_sibling_leaves() {
        // Sheet (0) into Services (2), at its end: Services is 1 after removal.
        assert_eq!(engine_move(&sample(), &mv(&[0], &[2], 1)), engine(&[1], 1));
    }

    #[test]
    fn an_earlier_parent_keeps_its_path() {
        // Notes (3) into Sheet (0), at its end.
        assert_eq!(engine_move(&sample(), &mv(&[3], &[0], 2)), engine(&[0], 2));
    }

    #[test]
    fn moving_down_among_siblings_counts_the_gap_it_leaves() {
        // Sheet (0) before Notes (3): index 2 once Sheet is out.
        assert_eq!(engine_move(&sample(), &mv(&[0], &[], 3)), engine(&[], 2));
        // To the end.
        assert_eq!(engine_move(&sample(), &mv(&[0], &[], 4)), engine(&[], 3));
    }

    #[test]
    fn both_slots_beside_an_entry_are_where_it_already_is() {
        assert_eq!(
            engine_move(&sample(), &mv(&[1], &[], 1)),
            Resolved::Unchanged
        );
        assert_eq!(
            engine_move(&sample(), &mv(&[1], &[], 2)),
            Resolved::Unchanged
        );
        assert_ne!(
            engine_move(&sample(), &mv(&[1], &[], 3)),
            Resolved::Unchanged
        );
    }

    #[test]
    fn an_entry_cannot_go_inside_itself() {
        assert_eq!(
            engine_move(&sample(), &mv(&[0], &[0], 0)),
            Resolved::IntoItself
        );
        let deep = vec![folder("A", vec![folder("B", vec![])])];
        assert_eq!(
            engine_move(&deep, &mv(&[0], &[0, 0], 0)),
            Resolved::IntoItself
        );
    }

    #[test]
    fn a_grouping_the_removal_empties_is_dropped_before_the_target_is_read() {
        // [grouping{5}] [Folder X]: moving 5 into X empties the grouping, which
        // the engine drops, so X is at [0] when the insert is read.
        let tree = vec![grouping(vec![layer(5, vec![])]), folder("X", vec![])];
        assert_eq!(engine_move(&tree, &mv(&[0, 0], &[1], 0)), engine(&[0], 0));
    }

    #[test]
    fn a_layer_keeps_its_place_when_its_last_sublayer_leaves() {
        // Dimensions Reference (1.0) to the top level before Notes (3).
        assert_eq!(engine_move(&sample(), &mv(&[1, 0], &[], 3)), engine(&[], 3));
    }

    #[test]
    fn a_missing_entry_or_slot_is_not_found() {
        assert_eq!(
            engine_move(&sample(), &mv(&[9], &[], 0)),
            Resolved::NotFound
        );
        assert_eq!(
            engine_move(&sample(), &mv(&[0], &[], 9)),
            Resolved::NotFound
        );
    }

    #[test]
    fn the_menu_offers_up_down_out_and_every_other_folder() {
        let offers = offers(&sample(), &[0, 1]);
        assert!(offers.contains(&Offer::Up(mv(&[0, 1], &[0], 0))));
        assert!(
            !offers.iter().any(|o| matches!(o, Offer::Down(_))),
            "last child"
        );
        assert!(offers.contains(&Offer::Out(
            mv(&[0, 1], &[], 1),
            OutOf::Folder("Sheet".into())
        )));
        assert!(offers.contains(&Offer::Into(mv(&[0, 1], &[2], 1), "Services".into())));
        assert!(
            !offers
                .iter()
                .any(|o| matches!(o, Offer::Into(_, l) if l == "Sheet")),
            "not into the folder it is already in"
        );
    }

    #[test]
    fn a_folder_is_not_offered_itself() {
        let offers = offers(&sample(), &[2]);
        assert!(
            !offers
                .iter()
                .any(|o| matches!(o, Offer::Into(_, l) if l == "Services"))
        );
        assert!(
            !offers.iter().any(|o| matches!(o, Offer::Out(..))),
            "top level"
        );
        assert!(offers.contains(&Offer::Down(mv(&[2], &[], 4))));
    }

    #[test]
    fn every_menu_move_resolves_to_a_real_engine_move() {
        let tree = sample();
        for at in [
            vec![0],
            vec![0, 0],
            vec![0, 1],
            vec![1],
            vec![1, 0],
            vec![2],
            vec![3],
        ] {
            for offer in offers(&tree, &at) {
                let m = match &offer {
                    Offer::Up(m) | Offer::Down(m) | Offer::Out(m, _) | Offer::Into(m, _) => m,
                };
                assert!(
                    matches!(engine_move(&tree, m), Resolved::Engine { .. }),
                    "{offer:?} from {at:?}"
                );
            }
        }
    }

    #[test]
    fn a_layer_is_found_at_any_depth() {
        assert_eq!(path_of_layer(&sample(), ObjId::new(9, 0)), Some(vec![1, 0]));
        assert_eq!(path_of_layer(&sample(), ObjId::new(11, 0)), Some(vec![3]));
        assert_eq!(path_of_layer(&sample(), ObjId::new(99, 0)), None);
    }

    #[test]
    fn a_path_is_dotted() {
        assert_eq!(dotted(&[2, 0]), "2.0");
        assert_eq!(dotted(&[]), "");
    }
}
