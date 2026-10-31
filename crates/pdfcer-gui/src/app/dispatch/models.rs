//! Insert ▸ 3D model: pick the file, then open the options window on the
//! page in view.

use crate::app::state::Status;
use crate::dialogs::DialogsState;

/// The whole of `edit.insert_3d` after its capability guard.
pub(super) fn insert(dialogs: &mut DialogsState, status: &Status) {
    let Status::Open(doc) = status else {
        return;
    };
    if cfg!(target_os = "android") {
        request_later();
        return;
    }
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

/// Android's picker cannot block, so the model file arrives on a later frame
/// through [`answered`].
fn request_later() {
    #[cfg(target_os = "android")]
    {
        let (name, extensions) = crate::app::files::model_filter();
        let started = crate::filedialog::FileDialog::new()
            .add_filter(name, extensions)
            .pick_file_later(crate::filedialog::Purpose::Model);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("model-picked source=android started={started}")
        });
    }
}

/// Opens the options window for each model file Android's picker returned,
/// on the page in view when it arrived.
#[cfg(target_os = "android")]
pub(crate) fn answered(dialogs: &mut DialogsState, status: &Status) {
    for (purpose, source) in crate::filedialog::take_answered() {
        let Status::Open(doc) = status else { continue };
        match purpose {
            crate::filedialog::Purpose::Model => {
                dialogs.open_insert_model(status, source, doc.view.page_index);
            }
        }
    }
}
