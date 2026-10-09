//! Commit `VectorAction::TransformLeavesInForm`: scale or rotate objects drawn
//! inside a placed drawing, as one undo entry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/vector/leaftransform.md`.

use pdfcer_core::vector::{Matrix, TransformOptions};

use crate::app::actions::apply::vector_edit_on_page;
use crate::app::state::OpenDoc;

/// Transform `leaves` on `page` by the page-space `matrix`.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, leaves: &[usize], matrix: Matrix) {
    if leaves.is_empty() {
        return;
    }
    let mut reach = None;
    vector_edit_on_page(
        doc,
        "transform-leaves-in-form",
        page,
        leaves.len(),
        |session| {
            session
                .transform_objects_in_form(page, leaves, matrix, TransformOptions::default())
                .map(|outcome| {
                    reach = Some((outcome.invocations, outcome.pages));
                    outcome.disclosures
                })
        },
    );
    let Some((invocations, pages)) = reach else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "transform-leaves-in-form-applied page={page} leaves={} invocations={invocations} \
             pages={pages} m=[{:.4} {:.4} {:.4} {:.4} {:.2} {:.2}]",
            crate::diag::index_list(leaves),
            matrix.a,
            matrix.b,
            matrix.c,
            matrix.d,
            matrix.e,
            matrix.f,
        )
    });
}
