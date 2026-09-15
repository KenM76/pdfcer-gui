//! # `app::spinnerdraft` — keeping a spinner's value alive between frames
//!
//! Two functions that **every** `DragValue` or `Slider` in this crate whose
//! value is a local re-seeded from a getter each frame must route through.
//! That shape, not any one control, is the defect [`drafted`] describes.
//!
//! `pub(crate)` rather than scoped to the module that needs it first: a helper
//! reachable only from there leaves every sibling control free to reinvent the
//! same dead drag.
/// **Hold a spinner's value across frames while it is being dragged.**
///
/// # The defect this exists to stop
///
/// A control written like this commits nothing on a drag, ever:
///
/// ```ignore
/// let was = /* read from the document, every frame */;
/// let mut value = was;
/// let response = ui.add(egui::DragValue::new(&mut value) …);
/// if response.drag_stopped() && value != was { commit(value) }
/// ```
///
/// `egui::DragValue` accumulates the pointer's motion into the borrowed value
/// *within a frame*; the next frame re-seeds it from the document, which has
/// not changed because nothing was committed. So the value never travels. On
/// the release frame the pointer has not moved at all, `value - was` is exactly
/// zero, and the guard declines. Typing into the same field works, because a
/// typed edit lands in egui's own text buffer and arrives complete on
/// `lost_focus` — and that contrast is the diagnostic when a control is
/// suspected.
///
/// ⇒ **A control whose backing value is re-read every frame cannot be dragged.**
/// The value has to survive between frames somewhere, and the document is the
/// one place it must not survive — an uncommitted drag is not a document
/// change.
///
/// ## No unit test can see it
///
/// The failure is a property of a **sequence** of frames — seed, drag, re-seed,
/// release. A test that calls the commit path directly, or asserts that
/// `MarkupEdit::Width` carries what it was handed, passes on a build whose
/// control is dead. Only `tools/ui-verify` dragging the real pointer can
/// falsify it, which is rule R1 (`DEVELOPING.md` §4) in one control.
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
/// silently, and a control showing a value the file does not have is a worse
/// symptom than the dead drag this fixes.
pub(crate) fn drafted<T>(ui: &egui::Ui, id: egui::Id, from_document: T) -> T
where
    T: Copy + Send + Sync + 'static,
{
    ui.data(|d| d.get_temp::<T>(id)).unwrap_or(from_document)
}

/// Store or drop [`drafted`]'s value according to the widget's own state.
///
/// Returns whether the interaction just ended, which is the frame a commit is
/// allowed on. Keeping that decision here is what stops two controls drifting
/// apart about what "the operator has finished" means.
pub(crate) fn keep_draft<T>(
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
