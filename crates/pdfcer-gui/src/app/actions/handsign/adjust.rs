//! # `app::actions::handsign::adjust` — move or resize a placed hand signature
//!
//! Contract: [`adjust`] maps the objects inside `field`'s hand-signature
//! sequence on `page` from one page-space rectangle onto another with one
//! `EditSession::transform_objects` call through the funnel: one undo step,
//! the `BDC … EMC` tag kept around what it wraps.
//!
//! The objects come from `EditSession::hand_signatures`, whose
//! `HandSignatureMark::objects` are `page_objects` indices — the numbering
//! `transform_objects` takes — valid for the revision they were read from,
//! so they are read immediately before the transform.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/handsign.md`.

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

/// The `page_objects` indices of every object in `field`'s hand-signature
/// marks on `page`, ascending, or `None` when the page cannot be read.
fn mark_objects(doc: &mut OpenDoc, field: &str, page: usize) -> Option<Vec<usize>> {
    let session = std::sync::Arc::get_mut(&mut doc.session)?;
    let mut objects: Vec<usize> = session
        .hand_signatures(page)
        .ok()?
        .into_iter()
        .filter(|m| m.field == field)
        .flat_map(|m| m.objects)
        .collect();
    objects.sort_unstable();
    objects.dedup();
    Some(objects)
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
}
