//! Reading the current page's model from the edit session's memo, so the
//! window and the editing verbs share one decomposition per revision.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/opendoc/cache.md`.

use std::time::Instant;

use crate::doccache::SessionModel;
use crate::opendoc::OpenDoc;

impl OpenDoc {
    /// Ask the session for the current page's model when the window's own is
    /// stale, on a frame that holds the session exclusively. Silent when it
    /// is shared (a render in flight) or refuses; the window then decomposes
    /// the page itself.
    pub(super) fn read_session_model(&mut self) {
        let key = (self.view.page_index, self.page_objects_revision());
        let stale = self.page_objects.built_for.get() != Some(key);
        let held = self
            .page_objects
            .from_session
            .borrow()
            .as_ref()
            .is_some_and(|m| m.key == key);
        if !stale || held {
            return;
        }
        let Some(session) = std::sync::Arc::get_mut(&mut self.session) else {
            return;
        };
        let started = Instant::now();
        if let Ok(objects) = session.page_objects(key.0) {
            *self.page_objects.from_session.borrow_mut() = Some(SessionModel {
                key,
                objects,
                took: started.elapsed(),
            });
        }
    }

    /// The session's model for `key`, if one was read for it.
    pub(super) fn take_session_model(&self, key: (usize, u64)) -> Option<SessionModel> {
        let mut slot = self.page_objects.from_session.borrow_mut();
        if slot.as_ref().is_some_and(|m| m.key == key) {
            slot.take()
        } else {
            None
        }
    }
}
