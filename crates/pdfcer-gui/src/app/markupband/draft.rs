//! # `app::markupband::draft` — keeping a spinner's value alive between frames
//!
//! Two functions, one defect, and the defect is the reason this file has a
//! header longer than its code.
/// **Hold a spinner's value across frames while it is being dragged.**
///
/// # ★★★ The defect this exists to stop, found by DRIVING and by nothing else
///
/// `width` and `opacity` both began as:
///
/// ```ignore
/// let was = /* read from the document, every frame */;
/// let mut value = was;
/// let response = ui.add(egui::DragValue::new(&mut value) …);
/// if response.drag_stopped() && value != was { commit(value) }
/// ```
///
/// **A drag on that control commits nothing, ever.** `egui::DragValue`
/// accumulates the pointer's motion into the borrowed value *within a frame*;
/// the next frame re-seeds it from the document, which has not changed because
/// nothing was committed. So the value never travels. On the release frame the
/// pointer has not moved at all, `value - was` is exactly zero, and the guard
/// declines. Typing into the same field works, because a typed edit lands in
/// egui's own text buffer and arrives complete on `lost_focus`.
///
/// ⇒ **A control whose backing value is re-read every frame cannot be dragged.**
/// The value has to survive between frames somewhere, and the document is the
/// one place it must not survive — an uncommitted drag is not a document
/// change.
///
/// ## Why no test could see it, and this project has the receipts
///
/// Every unit test over these controls calls the commit path directly, or
/// asserts that `MarkupEdit::Width` carries what it was handed. All of them
/// pass. The failure is a **property of a sequence of frames** — seed, drag,
/// re-seed, release — and a harness that builds one frame cannot express it.
/// It was found by `tools/ui-verify` dragging the real control 200 px with the
/// real pointer and watching nothing happen, then typing `12` into the same
/// field and watching that work. That contrast is what named the cause.
///
/// This is R1 in one paragraph, and it is the second time this project has
/// been handed the same lesson: *the tests pass* is not a report of working
/// software.
///
/// # What it does
///
/// Keeps the in-progress value in `egui`'s per-frame data store, under an id
/// derived from the widget's own, for exactly as long as the operator is
/// interacting with it. The draft is **dropped** the moment the interaction
/// ends, so the very next frame reads the document again — which is what makes
/// a committed change, an undo, or an edit from another surface show up here
/// rather than being masked by a stale draft.
///
/// ⚠ Dropped on `drag_stopped` and `lost_focus` **whether or not the value
/// changed**. A draft that outlived a no-op drag would shadow the document
/// silently, and the symptom — a control showing a value the file does not
/// have — is worse than the defect it came from.
pub(super) fn drafted<T>(ui: &egui::Ui, id: egui::Id, from_document: T) -> T
where
    T: Copy + Send + Sync + 'static,
{
    ui.data(|d| d.get_temp::<T>(id)).unwrap_or(from_document)
}

/// Store or drop [`drafted`]'s value according to the widget's own state.
///
/// Returns whether the interaction just ended, which is the frame a commit is
/// allowed on. Keeping that decision here means `width` and `opacity` cannot
/// drift apart about what "the operator has finished" means — they did not
/// share this once, and they shared the bug instead.
pub(super) fn keep_draft<T>(
    ui: &egui::Ui,
    id: egui::Id,
    response: &egui::Response,
    value: T,
) -> bool
where
    T: Copy + Send + Sync + 'static,
{
    let ended = response.drag_stopped() || response.lost_focus();
    if ended {
        ui.data_mut(|d| d.remove::<T>(id));
    } else if response.dragged() || response.has_focus() {
        ui.data_mut(|d| d.insert_temp::<T>(id, value));
    }
    ended
}
