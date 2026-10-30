//! Insert ▸ 3D model: pick the file, then open the options window on the
//! page in view.

use crate::app::state::Status;
use crate::dialogs::DialogsState;

/// The whole of `edit.insert_3d` after its capability guard.
pub(super) fn insert(dialogs: &mut DialogsState, status: &Status) {
    let Status::Open(doc) = status else {
        return;
    };
    let page = doc.view.page_index;
    let crate::app::files::Picked::Path(source) = crate::app::files::pick_model_source() else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "model-insert-cancelled".to_owned()
        });
        return;
    };
    dialogs.open_insert_model(status, source, page);
}
