//! # `app::dispatch::region` — the snapshot region's commands
//!
//! Each command reads the region (the snapshot box), resolves it and raises an
//! [`Action::Region`] or opens a window; none reads the canvas selection, so
//! adjusting the selection with Shift or Ctrl after drawing the box does not
//! change what these act on. All four are greyed without a box
//! (`doc.snapshot`).
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/region.md`.

use pdfcer_gui_base::regionaction::RegionAction;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};

/// `edit.delete_in_region`.
pub const DELETE: &str = "edit.delete_in_region"; // ui-text-exempt: a command id, never displayed
/// `edit.redact_region`.
pub const REDACT: &str = "edit.redact_region"; // ui-text-exempt: a command id, never displayed
/// `edit.redact_outside_region`.
pub const REDACT_OUTSIDE: &str = "edit.redact_outside_region"; // ui-text-exempt: a command id, never displayed
/// `pages.crop_to_region`.
pub const CROP: &str = "pages.crop_to_region"; // ui-text-exempt: a command id, never displayed

/// Whether `id` is one of this module's commands.
#[must_use]
pub fn handles(id: &str) -> bool {
    [DELETE, REDACT, REDACT_OUTSIDE, CROP].contains(&id)
}

/// Run one of this module's commands.
pub fn dispatch(app: &mut PdfcerApp, ctx: &egui::Context, id: &str, actions: &mut Vec<Action>) {
    // The panel's own chosen look, as every other marking route uses.
    let appearance = app.panels.redact_mut().appearance.to_core();
    let Status::Open(doc) = &app.status else {
        return;
    };
    let Some(region) = doc.snapshot else {
        return;
    };
    match id {
        REDACT => actions.push(Action::Region(RegionAction::Redact {
            page: region.page,
            rect: region.rect,
            appearance,
        })),
        REDACT_OUTSIDE => app.dialogs.open_redact_outside(doc, region),
        CROP => app
            .dialogs
            .open_crop_to_region(doc, region.page, region.rect),
        DELETE => delete(doc, crate::canvas::region::touching(ctx), actions),
        // ui-text-exempt: a panic message, never rendered.
        other => unreachable!("dispatch::region was handed `{other}`, which it does not claim"),
    }
}

/// Delete in region: what the region takes under the Inside only / Touching
/// choice, through the same delete ladder the Delete key uses.
fn delete(doc: &OpenDoc, touching: bool, actions: &mut Vec<Action>) {
    let epoch = doc.edit_epoch;
    let Some(region) = doc.snapshot else {
        return;
    };
    if region.page != doc.view.page_index {
        crate::app::actions::record_note(epoch, crate::text::region::other_page(region.page));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "region-delete-declined page={} reason=other-page",
                region.page
            )
        });
        return;
    }
    let Some(taken) = crate::canvas::region::take(doc, touching) else {
        crate::diag::trace(|| "region-delete-declined reason=no-objects".to_owned());
        return;
    };
    let (deleted, crossing_kept) = (taken.selection.len(), taken.crossing_kept);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "region-delete-requested page={} touching={touching} objects={deleted} \
             crossing_kept={crossing_kept}",
            region.page
        )
    });
    if deleted == 0 {
        crate::app::actions::record_note(
            epoch,
            crate::text::region::nothing_to_delete(crossing_kept),
        );
        return;
    }
    let outcome = {
        let targets = doc.page_objects();
        crate::canvas::deleting::subject(&taken.selection, region.page, targets.as_deref())
    };
    match outcome {
        Ok(subject) => actions.push(Action::Region(RegionAction::Delete {
            delete: crate::canvas::deleting::action(subject),
            deleted,
            crossing_kept,
        })),
        Err(reason) => crate::canvas::deleting::decline(&taken.selection, reason, epoch, true),
    }
}
