//! The annotations a saved file finally puts on its pages, read from the
//! bytes so a check judges what another viewer will see.
//!
//! An appended save keeps every earlier body of an object, including a
//! deleted annotation's, so being defined is not being on the page: only the
//! last definition of each object counts, and only when the last definition
//! of a page lists it in `/Annots`. Objects inside object streams are not
//! seen; pdfcer's save writes its appended objects plainly.

use std::collections::BTreeMap;

/// One annotation as the saved file finally states it.
#[derive(Debug, Clone)]
pub(super) struct SavedAnnot {
    pub num: u32,
    /// The `/Subtype` name without its slash, e.g. `Square`.
    pub subtype: String,
    pub rect: [f64; 4],
    /// The object's last body, for keys a caller reads itself.
    pub body: String,
}

/// Every annotation listed on a page, in `/Annots` order (front-most last),
/// pages taken in object-number order.
pub(super) fn listed_annots(bytes: &[u8]) -> Vec<SavedAnnot> {
    let text = String::from_utf8_lossy(bytes);
    let last = last_definitions(&text);
    last.values()
        .filter(|body| is_page(body))
        .filter_map(|body| references_after(body, "/Annots"))
        .flatten()
        .filter_map(|num| {
            let body = last.get(&num)?;
            let rect: [f64; 4] = numbers_after(body, "/Rect")?.try_into().ok()?;
            Some(SavedAnnot {
                num,
                subtype: subtype_of(body)?,
                rect,
                body: (*body).to_owned(),
            })
        })
        .collect()
}

/// The numbers in the array following `key` in `body`.
pub(super) fn numbers_after(body: &str, key: &str) -> Option<Vec<f64>> {
    let (open, close) = array_after(body, key)?;
    body[open..close]
        .split_whitespace()
        .map(|n| n.parse::<f64>().ok())
        .collect()
}

/// Object number to its last body, as an appended save leaves them.
pub(super) fn last_definitions(text: &str) -> BTreeMap<u32, &str> {
    let mut last = BTreeMap::new();
    let mut rest = text;
    while let Some(at) = rest.find(" 0 obj") {
        let head = &rest[..at];
        let digits = head.len() - head.trim_end_matches(|c: char| c.is_ascii_digit()).len();
        let body_start = at + " 0 obj".len();
        let Some(end) = rest[body_start..].find("endobj") else {
            break;
        };
        if digits > 0
            && let Ok(num) = head[head.len() - digits..].parse::<u32>()
        {
            last.insert(num, &rest[body_start..body_start + end]);
        }
        rest = &rest[body_start + end..];
    }
    last
}

/// The `/Subtype` name of `body`, without its slash.
fn subtype_of(body: &str) -> Option<String> {
    let from = body.find("/Subtype")? + "/Subtype".len();
    let name = body[from..].trim_start().strip_prefix('/')?;
    let end = name
        .find(|c: char| c.is_whitespace() || "/[]<>()".contains(c))
        .unwrap_or(name.len());
    Some(name[..end].to_owned())
}

/// Whether `body` is a `/Page` dictionary (not `/Pages`).
fn is_page(body: &str) -> bool {
    ["/Type /Page", "/Type/Page"].iter().any(|t| {
        body.match_indices(t)
            .any(|(i, _)| !body[i + t.len()..].starts_with('s'))
    })
}

/// The object numbers referenced (`n g R`) in the array following `key`.
pub(super) fn references_after(body: &str, key: &str) -> Option<Vec<u32>> {
    let (open, close) = array_after(body, key)?;
    let words: Vec<&str> = body[open..close].split_whitespace().collect();
    Some(
        words
            .windows(3)
            .filter(|w| w[2] == "R")
            .filter_map(|w| w[0].parse().ok())
            .collect(),
    )
}

/// The byte range inside the `[…]` directly following `key`; `None` when the
/// value is not an inline array (an indirect `/Annots n 0 R` is not read).
fn array_after(body: &str, key: &str) -> Option<(usize, usize)> {
    let from = body.find(key)? + key.len();
    let rest = body[from..].trim_start();
    if !rest.starts_with('[') {
        return None;
    }
    let open = body.len() - rest.len() + 1;
    let close = body[open..].find(']')? + open;
    Some((open, close))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_definition_of_an_object_wins() {
        let bytes = b"3 0 obj << /Type /Page /Annots [5 0 R] >> endobj\n\
                      5 0 obj << /Subtype /Link /Rect [0 0 1 1] >> endobj\n\
                      5 0 obj << /Subtype /Link /Rect [0 0 2 2] >> endobj\n";
        let annots = listed_annots(bytes);
        assert_eq!(annots.len(), 1);
        assert_eq!(annots[0].rect, [0.0, 0.0, 2.0, 2.0]);
        assert_eq!(annots[0].subtype, "Link");
    }

    #[test]
    fn an_annotation_the_page_no_longer_lists_is_gone() {
        let bytes = b"3 0 obj << /Type /Page /Annots [5 0 R] >> endobj\n\
                      5 0 obj << /Subtype /Link /Rect [0 0 1 1] >> endobj\n\
                      6 0 obj << /Subtype /Link /Rect [0 0 2 2] >> endobj\n\
                      3 0 obj << /Type /Page /Annots [6 0 R] >> endobj\n";
        let annots = listed_annots(bytes);
        assert_eq!(annots.len(), 1);
        assert_eq!(annots[0].num, 6);
    }

    #[test]
    fn annotations_come_in_the_pages_order() {
        let bytes = b"3 0 obj << /Type /Page /Annots [6 0 R 5 0 R] >> endobj\n\
                      5 0 obj << /Subtype/Square /Rect [0 0 1 1] >> endobj\n\
                      6 0 obj << /Subtype /Text /Rect [0 0 2 2] >> endobj\n";
        let order: Vec<u32> = listed_annots(bytes).iter().map(|a| a.num).collect();
        assert_eq!(order, [6, 5]);
    }

    #[test]
    fn a_pages_tree_node_is_not_a_page() {
        assert!(!is_page("<< /Type /Pages /Annots [5 0 R] >>"));
        assert!(is_page("<< /Type/Page /Annots [5 0 R] >>"));
    }
}
