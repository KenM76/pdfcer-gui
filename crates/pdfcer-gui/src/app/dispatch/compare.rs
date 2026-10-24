//! # `app::dispatch::compare` — `tools.compare`
//!
//! Shows the Compare panel, asks for the older version, and starts the
//! comparison with the panel's current options. A cancelled pick leaves the
//! panel showing whatever it had.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/compare.md`.

use crate::app::PdfcerApp;
use crate::app::files::Picked;
use crate::app::state::Status;
use crate::panels::Panel;
use crate::panels::compare::COMMAND_ID;

/// Whether this module owns `id`.
pub(crate) fn handles(id: &str) -> bool {
    id == COMMAND_ID
}

/// `tools.compare`.
pub(crate) fn dispatch(app: &mut PdfcerApp, ctx: &egui::Context) {
    if !matches!(app.status, Status::Open(_)) {
        return;
    }
    app.show_panel(Panel::Compare);
    let Picked::Path(older) = crate::app::files::pick_compare_source() else {
        return;
    };
    let Status::Open(doc) = &app.status else {
        return;
    };
    let state = app.panels.compare_mut();
    state.older = Some(older);
    state.start(doc, ctx);
}
