//! # `app::actions::handsign::adjust` — move or resize a placed hand signature
//!
//! Contract: [`adjust`] maps the objects inside `field`'s hand-signature
//! sequence on `page` from one page-space rectangle onto another with one
//! `EditSession::transform_objects` call through the funnel: one undo step,
//! the `BDC … EMC` tag kept around what it wraps.
//!
//! The engine reports a mark's field and bounds but not the objects it holds
//! (`pdfcer_core::hand_sig::HandSignatureMark`), so [`mark_objects`] finds
//! them the way `hand_sig::marks_in` does: the tagged sequences' byte ranges
//! in the session's content stream, and the session model's objects whose
//! spans lie inside one. Requested from the engine as G127.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/handsign.md`.

use pdfcer_core::content::{ContentStream, ContentToken, ContentTokenKind};
use pdfcer_core::hand_sig::{HAND_SIGNATURE_FIELD_KEY, HAND_SIGNATURE_TAG};
use pdfcer_core::object::Object;
use pdfcer_core::page_tree::Rect as PageRect;
use pdfcer_core::vector::{Matrix, TransformOptions};

use super::super::funnel::vector_edit_on_page;
use crate::app::state::OpenDoc;

/// The smallest side, in points, either rectangle may have.
const MIN_SIDE: f64 = 0.5;

/// Move and scale `field`'s hand signature on `page` so what spans `from`
/// spans `to`; both in PDF user space.
pub(in crate::app::actions) fn adjust(
    doc: &mut OpenDoc,
    field: &str,
    page: usize,
    from: PageRect,
    to: PageRect,
) {
    let Some(matrix) = onto(from, to) else {
        refuse(page, "degenerate");
        return;
    };
    let objects = match mark_objects(doc, field, page) {
        Some(o) if !o.is_empty() => o,
        _ => {
            refuse(page, "not-found");
            return;
        }
    };
    let before = doc.edit_epoch;
    vector_edit_on_page(
        doc,
        "adjust-hand-signature",
        page,
        objects.len(),
        |session| {
            session
                .transform_objects(page, &objects, matrix, TransformOptions::default())
                .map(|outcome| outcome.disclosures)
        },
    );
    if doc.edit_epoch == before {
        return;
    }
    let depth = doc.session.undo_depth();
    let landed = landed(doc, field, page);
    crate::diag::trace(|| {
        let (tagged, at) = match landed {
            Some(r) => (
                1,
                format!("{:.2} {:.2} {:.2} {:.2}", r.llx, r.lly, r.urx, r.ury),
            ),
            None => (0, "-".to_owned()),
        };
        // ui-text-exempt: diagnostic trace, never displayed. No field name.
        format!(
            "hand-sign-adjusted page={page} objects={} undo_depth={depth} tagged={tagged} \
             m=[{:.4} {:.4} {:.2} {:.2}] now=[{at}]",
            objects.len(),
            matrix.a,
            matrix.d,
            matrix.e,
            matrix.f
        )
    });
}

/// The scale-and-translate matrix taking `from` onto `to`, or `None` when
/// either is too small to scale.
fn onto(from: PageRect, to: PageRect) -> Option<Matrix> {
    let (fw, fh) = (from.urx - from.llx, from.ury - from.lly);
    let (tw, th) = (to.urx - to.llx, to.ury - to.lly);
    if fw.min(fh).min(tw).min(th) < MIN_SIDE {
        return None;
    }
    let (a, d) = (tw / fw, th / fh);
    Some(Matrix {
        a,
        b: 0.0,
        c: 0.0,
        d,
        e: to.llx - a * from.llx,
        f: to.lly - d * from.lly,
    })
}

/// The session-model indices of the objects inside `field`'s hand-signature
/// sequences on `page`, or `None` when the page cannot be read.
fn mark_objects(doc: &mut OpenDoc, field: &str, page: usize) -> Option<Vec<usize>> {
    let session = std::sync::Arc::get_mut(&mut doc.session)?;
    let model = session.page_objects(page).ok()?;
    let sheet = session.pages().ok()?.get(page)?.clone();
    let stream = ContentStream::from_page(&session.view(), &sheet).ok()?;
    let ranges = sequences_of(&stream, field);
    Some(
        model
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                let span = o.bytes();
                ranges
                    .iter()
                    .any(|&(start, end)| span.start >= start && span.start + span.len <= end)
            })
            .map(|(i, _)| i)
            .collect(),
    )
}

/// Every balanced hand-signature sequence for `field` in `cs`, as the byte
/// range from its `BDC`'s first operand to the end of its `EMC`.
fn sequences_of(cs: &ContentStream, field: &str) -> Vec<(usize, usize)> {
    let buf = cs.buf.as_slice();
    let mut open: Vec<Option<usize>> = Vec::new();
    let mut found = Vec::new();
    for op in cs.operations() {
        match op.operator_name(buf) {
            Some(b"BDC") => {
                let start = op
                    .operands
                    .first()
                    .map_or(op.operator.span.start, |t| t.span.start);
                open.push(names(op.operands, field).then_some(start));
            }
            Some(b"BMC") => open.push(None),
            Some(b"EMC") => {
                if let Some(Some(start)) = open.pop() {
                    found.push((start, op.operator.span.start + op.operator.span.len));
                }
            }
            _ => {}
        }
    }
    found
}

/// Whether a `BDC`'s operands are the hand-signature tag naming `field`.
fn names(operands: &[ContentToken], field: &str) -> bool {
    let operand = |i: usize| match operands.get(i).map(|t| &t.kind) {
        Some(ContentTokenKind::Operand(o)) => Some(o),
        _ => None,
    };
    let tagged = operand(0)
        .and_then(Object::as_name)
        .is_some_and(|n| n.as_bytes() == HAND_SIGNATURE_TAG);
    tagged
        && operand(1)
            .and_then(Object::as_dict)
            .and_then(|d| d.get(HAND_SIGNATURE_FIELD_KEY))
            .is_some_and(|v| match v {
                Object::String(s) => pdfcer_core::textstring::decode_text_string(s).text == field,
                _ => false,
            })
}

/// Where `field`'s mark on `page` now reads back, if it is still tagged.
fn landed(doc: &OpenDoc, field: &str, page: usize) -> Option<PageRect> {
    let sheet = doc.pages.get(page)?;
    pdfcer_core::hand_sig::hand_signatures(&doc.session.view(), sheet)
        .ok()?
        .into_iter()
        .find(|m| m.field == field)
        .map(|m| m.bounds)
}

fn refuse(page: usize, reason: &str) {
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| format!("hand-sign-adjust-refused page={page} reason={reason}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(llx: f64, lly: f64, urx: f64, ury: f64) -> PageRect {
        PageRect { llx, lly, urx, ury }
    }

    #[test]
    fn the_matrix_takes_one_rectangle_onto_the_other() {
        let m = onto(
            rect(10.0, 20.0, 50.0, 40.0),
            rect(100.0, 200.0, 120.0, 210.0),
        )
        .unwrap();
        let map = |x: f64, y: f64| (m.a * x + m.e, m.d * y + m.f);
        assert_eq!(map(10.0, 20.0), (100.0, 200.0));
        assert_eq!(map(50.0, 40.0), (120.0, 210.0));
    }

    #[test]
    fn a_collapsed_rectangle_is_refused() {
        assert!(onto(rect(0.0, 0.0, 10.0, 0.1), rect(0.0, 0.0, 10.0, 10.0)).is_none());
        assert!(onto(rect(0.0, 0.0, 10.0, 10.0), rect(5.0, 5.0, 5.0, 9.0)).is_none());
    }

    #[test]
    fn only_the_named_fields_sequences_are_found() {
        let content = b"/pdfc_HandSig <</Field (a)>> BDC 0 0 m 1 1 l S EMC \
                        /pdfc_HandSig <</Field (b)>> BDC 2 2 m 3 3 l S EMC \
                        /OC /L1 BDC 4 4 m 5 5 l S EMC"
            .to_vec();
        let cs = ContentStream::parse(content).unwrap();
        let a = sequences_of(&cs, "a");
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].0, 0);
        assert_eq!(sequences_of(&cs, "b").len(), 1);
        assert!(sequences_of(&cs, "c").is_empty());
    }
}
