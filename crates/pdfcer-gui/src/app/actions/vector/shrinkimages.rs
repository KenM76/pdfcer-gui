//! Commit `VectorAction::ShrinkImages`: rewrite the chosen image XObjects
//! with one `ImagePatch`, as one undoable optimize.
//!
//! The engine's in-document optimize: an `OptimizeSettings::new()` that
//! leaves every stream, plus one `Rule` selecting the pictures by id, planned
//! then applied (`CommandKind::Optimize`, one undo entry). Rewriting an
//! XObject changes every place it is drawn, which the dialog says before
//! Apply. What the plan did and why any picture was left is reported
//! off-canvas; the trace is `images-shrunk`.

use pdfcer_core::object::ObjId;
use pdfcer_core::optimize::{
    ImagePatch, OptimizeError, OptimizeSettings, Rule, Selector, StreamAction,
};

use crate::app::actions::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::dialogs::compact_images::Tally;

/// Rewrite the pictures `ids` with `patch`; `page` is where the first was
/// selected and is used for the trace and the edit's label only.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, ids: &[ObjId], patch: ImagePatch) {
    let mut done = None;
    vector_edit(doc, "shrink-images", page, ids.len(), |session| {
        let mut select = Selector::new();
        select.ids = ids.to_vec();
        let mut settings = OptimizeSettings::new();
        settings.rules.push(Rule {
            select,
            action: StreamAction::Image(patch),
        });
        let plan = session.plan_optimize(&settings)?;
        let tally = Tally::of(&plan);
        let mut notes = tally.lines();
        if tally.lossy {
            notes.insert(0, crate::text::compact_images::shrink_lossy().to_owned());
        }
        notes.extend(left(&plan));
        notes.extend(plan.notes.iter().cloned());
        let changed = plan.changes.len();
        if changed > 0 {
            session.apply_optimize(&plan)?;
        } else {
            notes.insert(0, crate::text::compact_images::shrink_nothing().to_owned());
        }
        done = Some((tally, changed, plan.skipped.len()));
        Ok::<_, OptimizeError>(notes)
    });
    let Some((tally, changed, skipped)) = done else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "images-shrunk page={page} asked={} changed={changed} resized={} jpeg={} \
             converted={} lossy={} ccitt={} jbig2={} lzw={} runlength={} skipped={skipped}",
            ids.len(),
            tally.resized,
            tally.jpeg,
            tally.converted,
            tally.lossy,
            tally.exact[0],
            tally.exact[1],
            tally.exact[2],
            tally.exact[3]
        )
    });
}

/// Why pictures were left as stored, one line per reason with its count.
fn left(plan: &pdfcer_core::optimize::OptimizePlan) -> Vec<String> {
    let mut by_reason = std::collections::BTreeMap::<String, usize>::new();
    for skipped in &plan.skipped {
        *by_reason
            .entry(crate::text::compact::left_because(&skipped.reason))
            .or_insert(0) += 1;
    }
    by_reason
        .into_iter()
        .map(|(reason, n)| crate::text::compact_images::shrink_left(&reason, n))
        .collect()
}
