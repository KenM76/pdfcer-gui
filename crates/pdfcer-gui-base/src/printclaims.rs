//! # `printclaims` — what a document says about how it prints
//!
//! Two answers the overprint-preview default needs as a document opens
//! ([`crate::prefs::overprint`]): does the file claim PDF/X, and does any
//! page ask for overprint. Read here because the engine answers neither; both
//! are filed as requests (`request_detect_pdfx_claim.md`,
//! `request_document_uses_overprint.md`). Delete this module when they land.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printclaims.md`.

use std::collections::HashSet;

use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{Dict, ObjId, Object};

/// Dictionaries visited before the overprint search gives up and answers
/// "no". Bounds the open on a file with a vast resource tree; a page that
/// asks for overprint past this still draws it — only the default is decided.
pub const VISIT_BUDGET: usize = 20_000;

/// Inherited-attribute depth followed up `/Parent` (ISO 32000-1 §7.7.3.4).
const PARENT_DEPTH: usize = 32;

/// The two claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrintClaims {
    /// The catalog's `/OutputIntents` has an `/S /GTS_PDFX` intent, or the
    /// Info dictionary carries `/GTS_PDFXVersion` (ISO 15930).
    pub pdfx: bool,
    /// An `/ExtGState` reachable from a page sets `/OP` or `/op` true.
    pub overprint: bool,
}

/// Read both claims. `pages` are the page objects, in any order.
#[must_use]
pub fn read<G: ObjectGraph + ?Sized>(graph: &G, pages: &[ObjId]) -> PrintClaims {
    PrintClaims {
        pdfx: claims_pdfx(graph),
        overprint: asks_overprint(graph, pages),
    }
}

/// Resolve `obj` to a dictionary, following references.
fn dict_of<'a, G: ObjectGraph + ?Sized>(graph: &'a G, obj: &'a Object) -> Option<&'a Dict> {
    graph.resolve(obj).as_dict()
}

/// The PDF/X claim: an output intent of subtype `GTS_PDFX`, or the version key.
pub fn claims_pdfx<G: ObjectGraph + ?Sized>(graph: &G) -> bool {
    let intent = graph
        .trailer_entry(b"Root")
        .and_then(|root| dict_of(graph, root))
        .and_then(|cat| cat.get(b"OutputIntents"))
        .and_then(|list| graph.resolve(list).as_array())
        .is_some_and(|items| {
            items.iter().any(|item| {
                dict_of(graph, item)
                    .and_then(|d| d.get(b"S"))
                    .and_then(|s| graph.resolve(s).as_name())
                    .is_some_and(|n| n.as_bytes() == b"GTS_PDFX")
            })
        });
    intent
        || graph
            .trailer_entry(b"Info")
            .and_then(|info| dict_of(graph, info))
            .is_some_and(|info| info.get(b"GTS_PDFXVersion").is_some())
}

/// Whether a graphics state on any page, or in any form it draws, sets
/// overprint on.
pub fn asks_overprint<G: ObjectGraph + ?Sized>(graph: &G, pages: &[ObjId]) -> bool {
    let mut search = Search {
        graph,
        seen: HashSet::new(),
        visits: 0,
    };
    pages.iter().any(|&page| {
        let page = Object::Reference(page);
        search.page(&page)
    })
}

/// One walk's state: visited objects and the budget spent.
struct Search<'g, G: ?Sized> {
    graph: &'g G,
    seen: HashSet<ObjId>,
    visits: usize,
}

impl<G: ObjectGraph + ?Sized> Search<'_, G> {
    /// A page's resources, inherited from the nearest ancestor that has them.
    fn page(&mut self, page: &Object) -> bool {
        let graph = self.graph;
        let mut node = dict_of(graph, page);
        for _ in 0..PARENT_DEPTH {
            let Some(dict) = node else { return false };
            if let Some(res) = dict.get(b"Resources") {
                return self.resources(res);
            }
            node = dict.get(b"Parent").and_then(|p| dict_of(graph, p));
        }
        false
    }

    /// A first visit to `obj` within budget; a direct object always is one.
    fn first_visit(&mut self, obj: &Object) -> bool {
        if self.visits >= VISIT_BUDGET {
            return false;
        }
        self.visits += 1;
        match obj {
            Object::Reference(id) => self.seen.insert(*id),
            _ => true,
        }
    }

    /// A resource dictionary: its graphics states, then its forms.
    fn resources(&mut self, res: &Object) -> bool {
        if !self.first_visit(res) {
            return false;
        }
        let graph = self.graph;
        let Some(res) = dict_of(graph, res) else {
            return false;
        };
        let states = res.get(b"ExtGState").and_then(|s| dict_of(graph, s));
        if states.is_some_and(|s| s.iter().any(|(_, gs)| sets_overprint(graph, gs))) {
            return true;
        }
        let Some(xobjects) = res.get(b"XObject").and_then(|x| dict_of(graph, x)) else {
            return false;
        };
        xobjects.iter().any(|(_, xobj)| self.form(xobj))
    }

    /// A form XObject's own resources.
    fn form(&mut self, xobj: &Object) -> bool {
        if !self.first_visit(xobj) {
            return false;
        }
        let graph = self.graph;
        let Some(dict) = dict_of(graph, xobj) else {
            return false;
        };
        let is_form = dict
            .get(b"Subtype")
            .and_then(|s| graph.resolve(s).as_name())
            .is_some_and(|n| n.as_bytes() == b"Form");
        is_form && dict.get(b"Resources").is_some_and(|r| self.resources(r))
    }
}

/// `/OP true` or `/op true` on one graphics-state dictionary (§8.6.7).
fn sets_overprint<G: ObjectGraph + ?Sized>(graph: &G, gs: &Object) -> bool {
    dict_of(graph, gs).is_some_and(|d| {
        [b"OP".as_slice(), b"op"].iter().any(|key| {
            d.get(key)
                .is_some_and(|v| matches!(graph.resolve(v), Object::Boolean(true)))
        })
    })
}

#[cfg(test)]
mod tests;
