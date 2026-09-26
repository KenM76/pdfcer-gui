//! # `taborderstated` — what the file STATES about tab order
//!
//! Split out of [`super::model`] on 2026-09-02 under R2, when the drag work
//! carried that file past the 1,500-line ceiling. The seam is not arbitrary:
//! this module answers *"what does the document say the tab order is"* and
//! [`super::model`] answers *"what is actually on the page"*. They are two
//! different questions of two different parts of the file — `/Tabs` lives in
//! the page dictionary (and, by inheritance, in its ancestors), `/Annots` and
//! the widgets live below it — and the tab-order panel is precisely the place
//! those two answers are put beside each other and compared.
//!
//! ## Why the distinction is load bearing and not merely tidy
//!
//! Because they can disagree, and the disagreement is the panel's whole
//! subject. A page whose `/Tabs` is absent has **no stated order at all** —
//! the engine's sourced reply on this is blunt: *"when `/Tabs` is absent the
//! standard states no fallback"*, and `/Annots`-order tabbing is implementation
//! practice rather than spec. So a reorder of `/Annots` changes what is on the
//! page without changing what the file states, and an operator has to be told
//! that. [`Sequence`] is the type that carries the difference.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/taborderstated.md`.

use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{ObjId, Object};
use pdfcer_core::page_tree::PageSlot;

/// What the file's `/Tabs` entry says about this page, and where it was found.
///
/// Three states rather than two, and the third is the whole of this module's
/// §4. See that section for the primary-source reading behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsEntry {
    /// **No `/Tabs` on the page dictionary, and none on any ancestor.**
    ///
    /// Reported as absent and given no mode name — not "manual", not
    /// "unspecified". See §4.
    Absent,
    /// `/Tabs` on the page's **own** dictionary. This is the page's tab order.
    OnPage(TabsMode),
    /// No `/Tabs` on the page; the **nearest** ancestor page-tree node that
    /// carries one carries this.
    ///
    /// **Not applied.** `/Tabs` is not among the four inheritable page
    /// attributes (§7.7.3.4), so per the standard this page has no `/Tabs`. It
    /// is reported because it is a fact about the file and because a viewer
    /// that did inherit it would behave differently. See §4.
    OnAncestor(TabsMode),
}

impl TabsEntry {
    /// What this entry implies about whether the `/Annots` sequence on screen
    /// **is** the tab order.
    ///
    #[must_use]
    pub fn sequence(&self) -> Sequence {
        match self {
            Self::Absent | Self::OnAncestor(_) => Sequence::AnnotsOrder,
            Self::OnPage(mode) => mode.sequence(),
        }
    }
}

/// A `/Tabs` name, decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsMode {
    /// `/R` — row order. Derived from where the fields sit.
    Row,
    /// `/C` — column order. Derived from where the fields sit.
    Column,
    /// `/S` — structure order. Derived from the document's tag tree.
    Structure,
    /// `/A` — annotations array order (PDF 2.0). **This list.**
    AnnotsArray,
    /// `/W` — widget order (PDF 2.0): widgets first, in `/Annots` order, then
    /// everything else. **This list**, for the widgets.
    Widgets,
    /// A `/Tabs` name this build does not recognise, carried verbatim.
    Unrecognised(String),
}

impl TabsMode {
    /// Decode a `/Tabs` name.
    #[must_use]
    pub fn from_name(name: &[u8]) -> Self {
        match name {
            b"R" => Self::Row,
            b"C" => Self::Column,
            b"S" => Self::Structure,
            b"A" => Self::AnnotsArray,
            b"W" => Self::Widgets,
            other => Self::Unrecognised(String::from_utf8_lossy(other).into_owned()),
        }
    }

    /// What this mode implies about the sequence on screen. See the table in
    /// this module's §4.
    #[must_use]
    pub fn sequence(&self) -> Sequence {
        match self {
            Self::AnnotsArray | Self::Widgets => Sequence::AnnotsOrder,
            Self::Row | Self::Column | Self::Structure => Sequence::Derived,
            Self::Unrecognised(_) => Sequence::Unknown,
        }
    }
}

/// Whether the `/Annots` sequence this view shows is the tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sequence {
    /// The sequence shown is the tab order (or, with no `/Tabs`, is what
    /// viewers use in practice).
    AnnotsOrder,
    /// The tab order is **derived** — from geometry, or from the structure tree
    /// — rather than stored, so the sequence shown is not it.
    Derived,
    /// The file names an order this build cannot interpret.
    Unknown,
}

/// Read one page's `/Tabs`, and say where it came from.
#[must_use]
pub fn page_tabs<G: ObjectGraph + ?Sized>(graph: &G, slot: &PageSlot) -> TabsEntry {
    if let Some(mode) = tabs_name(graph, slot.id) {
        return TabsEntry::OnPage(mode);
    }
    // Nearest ancestor first. `ancestors` is root-first, so this is a reverse
    // iteration and NOT a `.first()`: an intermediate `Pages` node's `/Tabs`
    // must win over the root's, exactly as it would under a `/Parent` walk.
    for ancestor in slot.ancestors.iter().rev() {
        if let Some(mode) = tabs_name(graph, *ancestor) {
            return TabsEntry::OnAncestor(mode);
        }
    }
    TabsEntry::Absent
}

/// The `/Tabs` name on one page-tree node's dictionary, if it has one.
fn tabs_name<G: ObjectGraph + ?Sized>(graph: &G, id: ObjId) -> Option<TabsMode> {
    let name = graph
        .resolved(id)
        .as_dict()?
        .get(b"Tabs")
        .map(|o| graph.resolve(o))
        .and_then(Object::as_name)?;
    Some(TabsMode::from_name(name.as_bytes()))
}
