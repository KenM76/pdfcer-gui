//! # `app::documents` — more than one document open at once
//!
//! The operator's request, verbatim:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/documents.md`.

use crate::app::PdfcerApp;
use crate::app::state::{Origin, Status};

impl PdfcerApp {
    /// **How many documents are open**, which is how many tabs are drawn.
    ///
    /// `0` and only `0` means [`Status::Empty`] with nothing parked — see this
    /// module's §2 for why that is the one state without a tab.
    #[must_use]
    pub fn document_count(&self) -> usize {
        if self.parked.is_empty() && matches!(self.status, Status::Empty) {
            0
        } else {
            self.parked.len() + 1
        }
    }

    /// The document in tab position `slot`, or `None` past the end.
    ///
    /// The read half of the encoding described in §1. Written out rather than
    /// routed through [`Self::take_slots`] because it must not move anything:
    /// the tab strip calls it once per tab per frame.
    #[must_use]
    pub fn slot(&self, slot: usize) -> Option<&Status> {
        if self.document_count() == 0 {
            return None;
        }
        match slot.cmp(&self.active_slot) {
            std::cmp::Ordering::Equal => Some(&self.status),
            std::cmp::Ordering::Less => self.parked.get(slot),
            std::cmp::Ordering::Greater => self.parked.get(slot - 1),
        }
    }

    /// **Flatten the two fields into one tab-ordered vector**, leaving the
    /// application document-less.
    fn take_slots(&mut self) -> Vec<Status> {
        if self.document_count() == 0 {
            return Vec::new();
        }
        let mut all = std::mem::take(&mut self.parked);
        let active = std::mem::replace(&mut self.status, Status::Empty);
        let at = self.active_slot.min(all.len());
        all.insert(at, active);
        all
    }

    /// **Put a tab-ordered vector back**, with `active` as the one on screen.
    fn put_slots(&mut self, mut all: Vec<Status>, active: usize) {
        if all.is_empty() {
            self.status = Status::Empty;
            self.parked = Vec::new();
            self.active_slot = 0;
            return;
        }
        let active = active.min(all.len() - 1);
        self.status = all.remove(active);
        self.parked = all;
        self.active_slot = active;
    }

    /// **The tab this path is already open in**, if it is.
    ///
    /// §3's rule. Only [`Origin::Opened`] documents can match: a created
    /// document's path is a name, not a location.
    #[must_use]
    pub fn slot_of_path(&self, path: &std::path::Path) -> Option<usize> {
        (0..self.document_count()).find(|slot| match self.slot(*slot) {
            Some(Status::Open(doc)) => doc.origin == Origin::Opened && doc.path == path,
            Some(Status::Failed { path: p, .. })
            | Some(Status::Unsupported { path: p, .. })
            | Some(Status::NeedsPassword { path: p, .. }) => p == path,
            Some(Status::Empty) | None => false,
        })
    }

    /// **Park the active document and make `incoming` the active one**, as a
    /// new tab at the end of the strip.
    ///
    /// The one entry point for "a document has just been produced" — an open,
    /// a create, a failed open. It does **not** run `PdfcerApp::adopt`; the
    /// caller does, because `adopt` is also what seeds a new document's view
    /// from the opening preferences and only the caller knows whether this is
    /// a new document or a returning one.
    ///
    /// A new tab goes at the **end**, which is where every tabbed application
    /// puts one. Inserting beside the active tab was considered and rejected:
    /// browsers that do that do it for tabs *spawned by* the current page, and
    /// an Open is not that.
    pub fn park_and_adopt(&mut self, incoming: Status) {
        if self.document_count() == 0 {
            self.status = incoming;
            self.parked = Vec::new();
            self.active_slot = 0;
            return;
        }
        let mut all = self.take_slots();
        all.push(incoming);
        let last = all.len() - 1;
        self.put_slots(all, last);
        self.forget_previous_documents_view();
    }

    /// **Show the document in tab position `slot`.**
    ///
    /// A no-op if it is already active or the slot does not exist, which is
    /// what lets the tab strip call it unconditionally on a click.
    ///
    /// Forgets what §4 says it must and nothing more. In particular it does
    /// not touch the incoming document's view, its rasters or its selection —
    /// those are the state that makes coming back to a tab worth doing.
    pub fn activate_slot(&mut self, slot: usize) {
        if slot >= self.document_count() || slot == self.active_slot {
            return;
        }
        let all = self.take_slots();
        self.put_slots(all, slot);
        self.forget_previous_documents_view();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "document-activate slot={slot} of={} path={:?}",
                self.document_count(),
                self.active_path(),
            )
        });
    }

    /// **Close the document in tab position `slot`.**
    ///
    /// §5's rule for what becomes active afterwards. Closing the last one
    /// leaves [`Status::Empty`].
    ///
    /// The unsaved-edits question belongs to the caller. This is reached
    /// from [`PdfcerApp::close_document`] (which is behind both guards) and
    /// from the tab strip's ✕ (which raises an action that goes through the
    /// same guards). Nothing may call it directly from a click.
    pub fn close_slot(&mut self, slot: usize) {
        if slot >= self.document_count() {
            return;
        }
        crate::diag::trace(|| match self.slot(slot) {
            Some(Status::Open(doc)) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "close slot={slot} path={:?} pages={}",
                doc.path,
                doc.pages.len()
            ),
            Some(Status::Failed { path, .. })
            | Some(Status::Unsupported { path, .. })
            | Some(Status::NeedsPassword { path, .. }) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "close slot={slot} unopened path={path:?}"
            ),
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Some(Status::Empty) | None => format!("close slot={slot} nothing-open"),
        });

        let was_active = self.active_slot;
        let mut all = self.take_slots();
        all.remove(slot);
        // The browser rule, stated once. Closing a tab left of the active one
        // shifts the active document down; closing the active one hands the
        // position to its right-hand neighbour, which is the same index once
        // the removal has happened.
        let next = if slot < was_active {
            was_active.saturating_sub(1)
        } else {
            was_active
        };
        self.put_slots(all, next);
        self.forget_previous_documents_view();
    }

    /// **Move the tab at `from` to the boundary `gap`**, keeping the same
    /// document on screen.
    ///
    /// `gap` is a **boundary**, not a destination index: `0` is before the
    /// first tab and `document_count()` is after the last, which is the same
    /// vocabulary the insertion caret is drawn in and the same one a page drop
    /// uses. `egui_shell::tabstrip::TabIntent::Reorder` carries the argument
    /// for why it is not "the index it ends up at" — the two differ by one
    /// whenever a tab moves rightward, because it is removed before it is
    /// re-inserted, and a caller with the wrong convention is off by one in one
    /// direction only.
    ///
    /// # The document on screen does not change, and that is arithmetic
    ///
    /// Reordering tabs is not navigation. An operator dragging tab 5 to the
    /// front has not asked to *look* at it, so the active document has to
    /// follow its own tab through the permutation rather than staying at an
    /// index. Getting that wrong would switch document as a side effect of
    /// tidying the strip, which no application does.
    ///
    /// Three cases, and the third is the one that needs the `+1`:
    ///
    /// | the active tab | where it goes |
    /// |---|---|
    /// | **is** the one being moved | wherever it lands |
    /// | was to the **right** of `from` | one place left, because a tab was removed in front of it |
    /// | ends up at or after the insertion point | one place right, because a tab was inserted in front of it |
    ///
    /// The two adjustments compose — a tab can be both — which is why they are
    /// applied in sequence rather than as a `match`.
    pub fn move_slot(&mut self, from: usize, gap: usize) {
        let count = self.document_count();
        if from >= count {
            return;
        }
        // Where it actually lands. Moving rightward, the removal has already
        // shifted every later tab down by one, so the boundary `gap` is one
        // place further along than the index to insert at.
        let insert_at = if gap > from { gap - 1 } else { gap }.min(count - 1);
        if insert_at == from {
            // Dropped where it already is. Not traced and not applied: a
            // reorder that changes nothing is a gesture the operator abandoned,
            // and treating it as an event would put a line in the log for every
            // tab they thought better of moving.
            return;
        }

        let was_active = self.active_slot;
        let mut all = self.take_slots();
        let moved = all.remove(from);
        all.insert(insert_at, moved);

        let active = if was_active == from {
            insert_at
        } else {
            let shifted = if was_active > from {
                was_active - 1
            } else {
                was_active
            };
            if shifted >= insert_at {
                shifted + 1
            } else {
                shifted
            }
        };
        self.put_slots(all, active);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "document-reorder from={from} gap={gap} to={insert_at} active={}",
                self.active_slot
            )
        });
    }

    /// **Move to the next or previous tab**, wrapping.
    ///
    /// Wrapping because Ctrl+Tab wraps in every application that has it, and
    /// an operator with two documents open would otherwise find the chord dead
    /// half the time.
    pub fn cycle_document(&mut self, forward: bool) {
        let count = self.document_count();
        if count < 2 {
            return;
        }
        let next = if forward {
            (self.active_slot + 1) % count
        } else {
            (self.active_slot + count - 1) % count
        };
        self.activate_slot(next);
    }

    /// The active document's path, for a trace line and the window title.
    /// `None` when nothing is open.
    #[must_use]
    pub fn active_path(&self) -> Option<&std::path::Path> {
        match &self.status {
            Status::Open(doc) => Some(doc.path.as_path()),
            Status::Failed { path, .. }
            | Status::Unsupported { path, .. }
            | Status::NeedsPassword { path, .. } => Some(path.as_path()),
            Status::Empty => None,
        }
    }

    /// Everything that a **different** document being on screen makes stale.
    fn forget_previous_documents_view(&mut self) {
        self.panels.forget_document();
        self.find.forget_document();
        crate::diag::reset_change_gates();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A `Status` that is a tab but is not a whole document, so the encoding
    /// can be exercised without building four `EditSession`s.
    fn tab(name: &str) -> Status {
        Status::Failed {
            path: PathBuf::from(name),
            // ui-text-exempt: test fixture, never displayed
            message: String::from("fixture"),
        }
    }

    /// The paths of every tab, left to right — the operator's strip as a
    /// string, which is what makes the assertions below readable.
    fn strip(app: &PdfcerApp) -> Vec<String> {
        (0..app.document_count())
            .map(|slot| match app.slot(slot) {
                Some(Status::Failed { path, .. }) => path.display().to_string(),
                other => format!("{}", other.is_some()),
            })
            .collect()
    }

    fn app_with(names: &[&str]) -> PdfcerApp {
        let mut app = PdfcerApp::new();
        for name in names {
            app.park_and_adopt(tab(name));
        }
        app
    }

    /// **Nothing open is zero tabs, not one empty one.**
    ///
    /// The invariant §2 states. Every other function here asks
    /// `document_count`, so this is the assertion the rest rest on.
    #[test]
    fn an_empty_application_has_no_tabs() {
        let app = PdfcerApp::new();
        assert_eq!(app.document_count(), 0);
        assert!(app.slot(0).is_none());
    }

    /// **A new document goes at the end of the strip and becomes active.**
    #[test]
    fn opening_appends_a_tab_and_shows_it() {
        let app = app_with(&["a", "b", "c"]);
        assert_eq!(app.document_count(), 3);
        assert_eq!(strip(&app), ["a", "b", "c"]);
        assert_eq!(app.active_slot, 2, "the newest document is the one shown");
    }

    /// **The encoding survives a round trip through every active position.**
    #[test]
    fn the_strip_order_is_independent_of_which_tab_is_active() {
        for active in 0..4 {
            let mut app = app_with(&["a", "b", "c", "d"]);
            app.activate_slot(active);
            assert_eq!(
                strip(&app),
                ["a", "b", "c", "d"],
                "activating slot {active} reordered the strip"
            );
            assert_eq!(app.active_slot, active);
        }
    }

    /// **Closing a tab left of the active one keeps the same document on
    /// screen.**
    #[test]
    fn closing_a_tab_to_the_left_keeps_the_active_document() {
        let mut app = app_with(&["a", "b", "c"]);
        app.activate_slot(2);
        app.close_slot(0);
        assert_eq!(strip(&app), ["b", "c"]);
        assert_eq!(app.active_slot, 1, "still looking at c");
    }

    /// **Closing the active tab shows its right-hand neighbour**, and the
    /// rightmost falls back to the new last tab. The browser rule, §5.
    #[test]
    fn closing_the_active_tab_moves_right_then_clamps() {
        let mut app = app_with(&["a", "b", "c"]);
        app.activate_slot(1);
        app.close_slot(1);
        assert_eq!(strip(&app), ["a", "c"]);
        assert_eq!(app.active_slot, 1, "c took b's position");

        app.close_slot(1);
        assert_eq!(strip(&app), ["a"]);
        assert_eq!(app.active_slot, 0, "the rightmost close clamps");
    }

    /// **Closing the last tab is the empty state the application starts in**,
    /// which is what keeps every downstream surface free of a second one.
    #[test]
    fn closing_the_last_tab_is_the_start_up_state() {
        let mut app = app_with(&["only"]);
        app.close_slot(0);
        assert_eq!(app.document_count(), 0);
        assert!(matches!(app.status, Status::Empty));
        assert!(app.parked.is_empty());
        assert_eq!(app.active_slot, 0);
    }

    /// **Ctrl+Tab wraps in both directions**, and does nothing at all with one
    /// document — the state a non-wrapping implementation gets right by
    /// accident and a broken one gets wrong by panicking.
    #[test]
    fn cycling_wraps_both_ways_and_is_inert_below_two_documents() {
        let mut app = app_with(&["a"]);
        app.cycle_document(true);
        assert_eq!(app.active_slot, 0);

        let mut app = app_with(&["a", "b", "c"]);
        app.activate_slot(2);
        app.cycle_document(true);
        assert_eq!(
            app.active_slot, 0,
            "forward from the last wraps to the first"
        );
        app.cycle_document(false);
        assert_eq!(app.active_slot, 2, "back from the first wraps to the last");
    }

    /// **Reordering tabs keeps the same document on screen.**
    #[test]
    fn reordering_tabs_never_changes_which_document_is_on_screen() {
        for active in 0..4 {
            for from in 0..4 {
                for gap in 0..=4 {
                    let mut app = app_with(&["a", "b", "c", "d"]);
                    app.activate_slot(active);
                    let looking_at = strip(&app)[active].clone();

                    app.move_slot(from, gap);

                    assert_eq!(
                        app.document_count(),
                        4,
                        "a reorder lost or gained a tab: from={from} gap={gap}"
                    );
                    assert_eq!(
                        strip(&app)[app.active_slot],
                        looking_at,
                        "moving tab {from} to gap {gap} with {looking_at} on screen \
                         switched document"
                    );
                    let mut sorted = strip(&app);
                    sorted.sort();
                    assert_eq!(
                        sorted,
                        ["a", "b", "c", "d"],
                        "a reorder duplicated or dropped a document: from={from} gap={gap}"
                    );
                }
            }
        }
    }

    /// **A tab dropped where it already is changes nothing**, and the two gaps
    /// that mean that are both of them.
    #[test]
    fn dropping_a_tab_where_it_already_is_does_nothing() {
        for from in 0..4 {
            for gap in [from, from + 1] {
                let mut app = app_with(&["a", "b", "c", "d"]);
                app.move_slot(from, gap);
                assert_eq!(
                    strip(&app),
                    ["a", "b", "c", "d"],
                    "from={from} gap={gap} moved a tab that was already there"
                );
            }
        }
    }

    /// **A path that is already open is found**, so the caller can activate it
    /// rather than opening a second session over the same file (§3).
    #[test]
    fn a_path_that_is_already_open_is_found() {
        let app = app_with(&["a", "b", "c"]);
        assert_eq!(app.slot_of_path(std::path::Path::new("b")), Some(1));
        assert_eq!(app.slot_of_path(std::path::Path::new("z")), None);
    }
}
