//! # `linkread` — a link's target and border, read from the file
//!
//! The values `set_link_target` and `set_link_border` write, which neither
//! `Annotation` nor `LinkDestination` carries in an editable form (G178).
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/linkread.md`.

use pdfcer_core::edit::EditSession;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{Dict, ObjId, Object};

/// Where a link goes, as the file spells it.
#[derive(Debug, Clone, PartialEq)]
pub enum Goes {
    /// A page of this document, 0-based.
    Page(usize),
    /// A named destination, by its key bytes.
    Named(Vec<u8>),
    /// A web address.
    Uri(String),
    /// An action this shell does not author, by its `/S` (`JavaScript`,
    /// `GoToR`, `Launch`, …), or a destination naming no page of this file.
    Other(String),
    /// No `/Dest` and no `/A`.
    Nowhere,
}

/// A link's visible border, or `None` for none.
#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    /// Stroke width in points, above zero.
    pub width: f64,
    /// `/C` as 0–255 RGB, `None` when absent or empty (no stroke colour).
    pub rgb: Option<[u8; 3]>,
    /// Whether it is dashed.
    pub dashed: bool,
}

/// Everything the Properties link section shows.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkReading {
    /// Where it goes.
    pub goes: Goes,
    /// Its border, or `None` when it draws none.
    pub border: Option<Border>,
}

/// **Read the link `id`**, or `None` when it is not a dictionary.
///
/// `pages` is the document's page ids in order, to number a `/Dest` array's
/// page. The border follows Table 164 and Table 166: `/BS /W` wins over
/// `/Border`'s third element, both absent is a 1-point border, a zero width
/// or an empty `/C` draws nothing.
#[must_use]
pub fn read(session: &EditSession, id: ObjId, pages: &[ObjId]) -> Option<LinkReading> {
    let graph = session.graph();
    let Some(Object::Dict(dict)) = session.value(id) else {
        return None;
    };
    Some(LinkReading {
        goes: goes(&graph, dict, pages),
        border: border(&graph, dict),
    })
}

fn goes(graph: &impl ObjectGraph, dict: &Dict, pages: &[ObjId]) -> Goes {
    if let Some(dest) = dict.get(b"Dest") {
        return destination(graph, dest, pages);
    }
    let Some(Object::Dict(action)) = dict.get(b"A").map(|a| graph.resolve(a)) else {
        return Goes::Nowhere;
    };
    let kind = action
        .get(b"S")
        .and_then(|s| graph.resolve(s).as_name())
        .map(|n| String::from_utf8_lossy(n.as_bytes()).into_owned())
        .unwrap_or_default();
    match kind.as_str() {
        "URI" => match action.get(b"URI").map(|u| graph.resolve(u)) {
            Some(Object::String(bytes)) => Goes::Uri(String::from_utf8_lossy(bytes).into_owned()),
            _ => Goes::Other(kind),
        },
        "GoTo" => action
            .get(b"D")
            .map_or(Goes::Other(kind), |d| destination(graph, d, pages)),
        _ => Goes::Other(kind),
    }
}

/// A `/Dest` or `/D`: a name or string is a named destination, an array
/// names a page.
fn destination(graph: &impl ObjectGraph, dest: &Object, pages: &[ObjId]) -> Goes {
    match graph.resolve(dest) {
        Object::Name(n) => Goes::Named(n.as_bytes().to_vec()),
        Object::String(s) => Goes::Named(s.clone()),
        Object::Array(items) => match items.first() {
            Some(Object::Reference(page)) => pages
                .iter()
                .position(|p| p == page)
                .map_or_else(|| Goes::Other("GoTo".to_owned()), Goes::Page),
            _ => Goes::Other("GoTo".to_owned()),
        },
        _ => Goes::Other("GoTo".to_owned()),
    }
}

fn border(graph: &impl ObjectGraph, dict: &Dict) -> Option<Border> {
    let bs = match dict.get(b"BS").map(|b| graph.resolve(b)) {
        Some(Object::Dict(d)) => Some(d),
        _ => None,
    };
    let legacy = dict
        .get(b"Border")
        .and_then(|b| graph.resolve(b).as_array())
        .map(<[Object]>::to_vec);
    let width = bs
        .and_then(|d| d.get(b"W"))
        .and_then(|w| graph.resolve(w).as_number())
        .or_else(|| {
            legacy
                .as_ref()
                .and_then(|a| a.get(2))
                .and_then(|w| graph.resolve(w).as_number())
        })
        .unwrap_or(1.0);
    let colour = dict
        .get(b"C")
        .and_then(|c| graph.resolve(c).as_array())
        .map(|a| {
            a.iter()
                .filter_map(|o| graph.resolve(o).as_number())
                .collect::<Vec<_>>()
        });
    if width <= 0.0 || colour.as_ref().is_some_and(Vec::is_empty) {
        return None;
    }
    let dashed = bs
        .and_then(|d| d.get(b"S"))
        .and_then(|s| graph.resolve(s).as_name())
        .is_some_and(|n| n.as_bytes() == b"D")
        || legacy.as_ref().is_some_and(|a| a.len() > 3);
    Some(Border {
        width,
        rgb: colour.as_deref().and_then(rgb_of),
        dashed,
    })
}

/// `/C`'s numbers as 0–255 RGB: one is grey, three RGB, four CMYK.
fn rgb_of(c: &[f64]) -> Option<[u8; 3]> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    match *c {
        [g] => Some([byte(g); 3]),
        [r, g, b] => Some([byte(r), byte(g), byte(b)]),
        [c, m, y, k] => Some([
            byte((1.0 - c) * (1.0 - k)),
            byte((1.0 - m) * (1.0 - k)),
            byte((1.0 - y) * (1.0 - k)),
        ]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_arrays_of_each_length_become_rgb() {
        assert_eq!(rgb_of(&[0.5]), Some([128, 128, 128]));
        assert_eq!(rgb_of(&[1.0, 0.0, 0.0]), Some([255, 0, 0]));
        assert_eq!(rgb_of(&[0.0, 0.0, 0.0, 1.0]), Some([0, 0, 0]));
        assert_eq!(rgb_of(&[0.1, 0.2]), None);
    }
}
